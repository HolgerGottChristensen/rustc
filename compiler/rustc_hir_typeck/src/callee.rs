use super::method::probe::{IsSuggestion, Mode, ProbeScope};
use super::method::MethodCallee;
use super::{Expectation, FnCtxt, TupleArgumentsFlag};

use crate::type_error_struct;
use rustc_ast::util::parser::PREC_POSTFIX;
use rustc_errors::{struct_span_err, Applicability, Diagnostic, ErrorGuaranteed, StashKey};
use rustc_hir as hir;
use rustc_hir::def::{self, CtorKind, Namespace, Res};
use rustc_hir::def_id::DefId;
use rustc_infer::{
    infer,
    traits::{self, Obligation},
};
use rustc_infer::{
    infer::type_variable::{TypeVariableOrigin, TypeVariableOriginKind},
    traits::ObligationCause,
};
use rustc_middle::ty::adjustment::{
    Adjust, Adjustment, AllowTwoPhase, AutoBorrow, AutoBorrowMutability,
};
use rustc_middle::ty::{HKTSubstType, SubstsRef, ty_slice_as_generic_args};
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitable};
use rustc_span::def_id::LocalDefId;
use rustc_span::symbol::{sym, Ident};
use rustc_span::Span;
use rustc_target::spec::abi;
use rustc_trait_selection::autoderef::Autoderef;
use rustc_trait_selection::infer::InferCtxtExt as _;
use rustc_trait_selection::traits::error_reporting::DefIdOrName;
use rustc_trait_selection::traits::query::evaluate_obligation::InferCtxtExt as _;

use std::{iter, slice};

/// Checks that it is legal to call methods of the trait corresponding
/// to `trait_id` (this only cares about the trait, not the specific
/// method that is called).
pub fn check_legal_trait_for_method_call(
    tcx: TyCtxt<'_>,
    span: Span,
    receiver: Option<Span>,
    expr_span: Span,
    trait_id: DefId,
) {
    if tcx.lang_items().drop_trait() == Some(trait_id) {
        let mut err = struct_span_err!(tcx.sess, span, E0040, "explicit use of destructor method");
        err.span_label(span, "explicit destructor calls not allowed");

        let (sp, suggestion) = receiver
            .and_then(|s| tcx.sess.source_map().span_to_snippet(s).ok())
            .filter(|snippet| !snippet.is_empty())
            .map(|snippet| (expr_span, format!("drop({snippet})")))
            .unwrap_or_else(|| (span, "drop".to_string()));

        err.span_suggestion(
            sp,
            "consider using `drop` function",
            suggestion,
            Applicability::MaybeIncorrect,
        );

        err.emit();
    }
}

#[derive(Debug)]
enum CallStep<'tcx> {
    Builtin(Ty<'tcx>),
    DeferredClosure(LocalDefId, ty::FnSig<'tcx>),
    /// E.g., enum variant constructors.
    Overloaded(MethodCallee<'tcx>),
}

impl<'a, 'tcx> FnCtxt<'a, 'tcx> {

    /// Type check that a call function has the expected type
    ///
    /// # Arguments
    ///
    /// * `call_expr`: The expression containing the function being called and the arguments being called with
    /// * `callee_expr`: An expression pointing to the function being called
    /// * `arg_exprs`: The list of expressions which are the arguments of the function call
    /// * `expected`: The expected type of the expression. This could for example be when you explicitly
    /// specify a type in a let, then the expected for the rhs of the let would be expected to return the
    /// specified type.
    ///
    /// returns: The return type of the function call.
    #[instrument(level = "info", skip(self, call_expr, callee_expr, arg_exprs), fields(id = ?call_expr.hir_id)ret)]
    pub fn check_call(
        &self,
        call_expr: &'tcx hir::Expr<'tcx>,
        callee_expr: &'tcx hir::Expr<'tcx>,
        arg_exprs: &'tcx [hir::Expr<'tcx>],
        expected: Expectation<'tcx>,
    ) -> Ty<'tcx> {

        info!("Pending obligations1: {:#?}", self.fulfillment_cx.borrow().pending_obligations());

        // Check if we have a simple function call as the callee or some expression that should
        // be evaluated to get the type of the callee.
        let original_callee_ty = match &callee_expr.kind {
            hir::ExprKind::Path(hir::QPath::Resolved(..) | hir::QPath::TypeRelative(..)) => self
                .check_expr_with_expectation_and_args(
                    callee_expr,
                    Expectation::NoExpectation,
                    arg_exprs,
                ),
            _ => self.check_expr(callee_expr),
        };

        info!("Pending obligations2: {:#?}", self.fulfillment_cx.borrow().pending_obligations());

        let expr_ty = self.structurally_resolved_type(call_expr.span, original_callee_ty);

        // Try to automatically deref the expression type until either it cant anymore or we
        // reach some valid call
        let mut autoderef = self.autoderef(callee_expr.span, expr_ty);
        let mut result = None;
        while result.is_none() && autoderef.next().is_some() {
            result = self.try_overloaded_call_step(call_expr, callee_expr, arg_exprs, &autoderef);
        }

        // The auto deref will result in a set of obligations, for example that something should
        // implement deref. We register these obligations.
        let obligations = autoderef.into_obligations();
        self.register_predicates(obligations);

        info!("Pending obligations: {:#?}", self.fulfillment_cx.borrow().pending_obligations());

        let output = match result {
            None => {
                // This will report an error since original_callee_ty is not a fn
                self.confirm_builtin_call(
                    call_expr,
                    callee_expr,
                    original_callee_ty,
                    arg_exprs,
                    expected,
                )
            }

            Some(CallStep::Builtin(callee_ty)) => {
                self.confirm_builtin_call(call_expr, callee_expr, callee_ty, arg_exprs, expected)
            }

            Some(CallStep::DeferredClosure(def_id, fn_sig)) => {
                self.confirm_deferred_closure_call(call_expr, arg_exprs, expected, def_id, fn_sig)
            }

            Some(CallStep::Overloaded(method_callee)) => {
                self.confirm_overloaded_call(call_expr, arg_exprs, expected, method_callee)
            }
        };

        // We must check that return type of called functions is well-formed:
        self.register_wf_obligation(output.into(), call_expr.span, traits::WellFormed(None));

        output
    }

    #[instrument(level = "debug", skip(self, call_expr, callee_expr, arg_exprs, autoderef), ret)]
    fn try_overloaded_call_step(
        &self,
        call_expr: &'tcx hir::Expr<'tcx>,
        callee_expr: &'tcx hir::Expr<'tcx>,
        arg_exprs: &'tcx [hir::Expr<'tcx>],
        autoderef: &Autoderef<'a, 'tcx>,
    ) -> Option<CallStep<'tcx>> {
        let adjusted_ty =
            self.structurally_resolved_type(autoderef.span(), autoderef.final_ty(false));

        // If the callee is a bare function or a closure, then we're all set.
        match *adjusted_ty.kind() {
            ty::FnDef(..) | ty::FnPtr(_) => {
                let adjustments = self.adjust_steps(autoderef);
                self.apply_adjustments(callee_expr, adjustments);
                return Some(CallStep::Builtin(adjusted_ty));
            }

            ty::Closure(def_id, substs) => {
                let def_id = def_id.expect_local();

                // Check whether this is a call to a closure where we
                // haven't yet decided on whether the closure is fn vs
                // fnmut vs fnonce. If so, we have to defer further processing.
                if self.closure_kind(substs).is_none() {
                    let closure_sig = substs.as_closure().sig();
                    let closure_sig = self.replace_bound_vars_with_fresh_vars(
                        call_expr.span,
                        infer::FnCall,
                        closure_sig,
                    );
                    let adjustments = self.adjust_steps(autoderef);
                    self.record_deferred_call_resolution(
                        def_id,
                        DeferredCallResolution {
                            call_expr,
                            callee_expr,
                            adjusted_ty,
                            adjustments,
                            fn_sig: closure_sig,
                            closure_substs: substs,
                        },
                    );
                    return Some(CallStep::DeferredClosure(def_id, closure_sig));
                }
            }

            // Hack: we know that there are traits implementing Fn for &F
            // where F:Fn and so forth. In the particular case of types
            // like `f: &mut FnMut()`, if there is a call `f()`, we would
            // normally translate to `FnMut::call_mut(&mut f, ())`, but
            // that winds up potentially requiring the user to mark their
            // variable as `mut` which feels unnecessary and unexpected.
            //
            //     fn foo(f: &mut impl FnMut()) { f() }
            //            ^ without this hack `f` would have to be declared as mutable
            //
            // The simplest fix by far is to just ignore this case and deref again,
            // so we wind up with `FnMut::call_mut(&mut *f, ())`.
            ty::Ref(..) if autoderef.step_count() == 0 => {
                return None;
            }

            ty::Error(_) => {
                return None;
            }

            _ => {}
        }

        // Now, we look for the implementation of a Fn trait on the object's type.
        // We first do it with the explicit instruction to look for an impl of
        // `Fn<Tuple>`, with the tuple `Tuple` having an arity corresponding
        // to the number of call parameters.
        // If that fails (or_else branch), we try again without specifying the
        // shape of the tuple (hence the None). This allows to detect an Fn trait
        // is implemented, and use this information for diagnostic.
        self.try_overloaded_call_traits(call_expr, adjusted_ty, Some(arg_exprs))
            .or_else(|| self.try_overloaded_call_traits(call_expr, adjusted_ty, None))
            .map(|(autoref, method)| {
                let mut adjustments = self.adjust_steps(autoderef);
                adjustments.extend(autoref);
                self.apply_adjustments(callee_expr, adjustments);
                CallStep::Overloaded(method)
            })
    }

    fn try_overloaded_call_traits(
        &self,
        call_expr: &hir::Expr<'_>,
        adjusted_ty: Ty<'tcx>,
        opt_arg_exprs: Option<&'tcx [hir::Expr<'tcx>]>,
    ) -> Option<(Option<Adjustment<'tcx>>, MethodCallee<'tcx>)> {
        // Try the options that are least restrictive on the caller first.
        for (opt_trait_def_id, method_name, borrow) in [
            (self.tcx.lang_items().fn_trait(), Ident::with_dummy_span(sym::call), true),
            (self.tcx.lang_items().fn_mut_trait(), Ident::with_dummy_span(sym::call_mut), true),
            (self.tcx.lang_items().fn_once_trait(), Ident::with_dummy_span(sym::call_once), false),
        ] {
            let Some(trait_def_id) = opt_trait_def_id else { continue };

            let opt_input_type = opt_arg_exprs.map(|arg_exprs| {
                self.tcx.mk_tup(arg_exprs.iter().map(|e| {
                    self.next_ty_var(TypeVariableOrigin {
                        kind: TypeVariableOriginKind::TypeInference,
                        span: e.span,
                    })
                }))
            });

            if let Some(ok) = self.lookup_method_in_trait(
                self.misc(call_expr.span),
                method_name,
                trait_def_id,
                adjusted_ty,
                opt_input_type.as_ref().map(slice::from_ref),
            ) {
                let method = self.register_infer_ok_obligations(ok);
                let mut autoref = None;
                if borrow {
                    // Check for &self vs &mut self in the method signature. Since this is either
                    // the Fn or FnMut trait, it should be one of those.
                    let ty::Ref(region, _, mutbl) = method.sig.inputs()[0].kind() else {
                        // The `fn`/`fn_mut` lang item is ill-formed, which should have
                        // caused an error elsewhere.
                        self.tcx
                            .sess
                            .delay_span_bug(call_expr.span, "input to call/call_mut is not a ref?");
                        return None;
                    };

                    // For initial two-phase borrow
                    // deployment, conservatively omit
                    // overloaded function call ops.
                    let mutbl = AutoBorrowMutability::new(*mutbl, AllowTwoPhase::No);

                    autoref = Some(Adjustment {
                        kind: Adjust::Borrow(AutoBorrow::Ref(*region, mutbl)),
                        target: method.sig.inputs()[0],
                    });
                }
                return Some((autoref, method));
            }
        }

        None
    }

    /// Give appropriate suggestion when encountering `||{/* not callable */}()`, where the
    /// likely intention is to call the closure, suggest `(||{})()`. (#55851)
    fn identify_bad_closure_def_and_call(
        &self,
        err: &mut Diagnostic,
        hir_id: hir::HirId,
        callee_node: &hir::ExprKind<'_>,
        callee_span: Span,
    ) {
        let hir = self.tcx.hir();
        let parent_hir_id = hir.parent_id(hir_id);
        let parent_node = hir.get(parent_hir_id);
        if let (
            hir::Node::Expr(hir::Expr {
                kind: hir::ExprKind::Closure(&hir::Closure { fn_decl_span, body, .. }),
                ..
            }),
            hir::ExprKind::Block(..),
        ) = (parent_node, callee_node)
        {
            let fn_decl_span = if hir.body(body).generator_kind
                == Some(hir::GeneratorKind::Async(hir::AsyncGeneratorKind::Closure))
            {
                // Actually need to unwrap a few more layers of HIR to get to
                // the _real_ closure...
                let async_closure = hir.parent_id(hir.parent_id(parent_hir_id));
                if let hir::Node::Expr(hir::Expr {
                    kind: hir::ExprKind::Closure(&hir::Closure { fn_decl_span, .. }),
                    ..
                }) = hir.get(async_closure)
                {
                    fn_decl_span
                } else {
                    return;
                }
            } else {
                fn_decl_span
            };

            let start = fn_decl_span.shrink_to_lo();
            let end = callee_span.shrink_to_hi();
            err.multipart_suggestion(
                "if you meant to create this closure and immediately call it, surround the \
                closure with parentheses",
                vec![(start, "(".to_string()), (end, ")".to_string())],
                Applicability::MaybeIncorrect,
            );
        }
    }

    /// Give appropriate suggestion when encountering `[("a", 0) ("b", 1)]`, where the
    /// likely intention is to create an array containing tuples.
    fn maybe_suggest_bad_array_definition(
        &self,
        err: &mut Diagnostic,
        call_expr: &'tcx hir::Expr<'tcx>,
        callee_expr: &'tcx hir::Expr<'tcx>,
    ) -> bool {
        let hir_id = self.tcx.hir().parent_id(call_expr.hir_id);
        let parent_node = self.tcx.hir().get(hir_id);
        if let (
            hir::Node::Expr(hir::Expr { kind: hir::ExprKind::Array(_), .. }),
            hir::ExprKind::Tup(exp),
            hir::ExprKind::Call(_, args),
        ) = (parent_node, &callee_expr.kind, &call_expr.kind)
            && args.len() == exp.len()
        {
            let start = callee_expr.span.shrink_to_hi();
            err.span_suggestion(
                start,
                "consider separating array elements with a comma",
                ",",
                Applicability::MaybeIncorrect,
            );
            return true;
        }
        false
    }

    /// Check that the function call is the same type as the expectation
    ///
    /// # Arguments
    ///
    /// * `call_expr`: The expression containing a function being called and the arguments being called with
    /// * `callee_expr`: The expression being called `function` in `function(arg1, arg2, ...)`
    /// * `callee_ty`: The type of the calle expression
    /// * `arg_exprs`: The argument expressions being called with. `arg1, arg2, ...` in `function(arg1, arg2, ...)`
    /// * `expected`: The expected type of the function call if any
    ///
    /// returns: The type of the function call expression
    #[instrument(level = "info", skip(self, call_expr, callee_expr, callee_ty, arg_exprs, expected))]
    fn confirm_builtin_call(
        &self,
        call_expr: &'tcx hir::Expr<'tcx>,
        callee_expr: &'tcx hir::Expr<'tcx>,
        callee_ty: Ty<'tcx>,
        arg_exprs: &'tcx [hir::Expr<'tcx>],
        expected: Expectation<'tcx>,
    ) -> Ty<'tcx> {
        // TODO WE FAIL HERE SOMEWHERE
        //warn!("Call expr: {:#?}", call_expr);
        //warn!("Callee expr: {:#?}", callee_expr);
        //warn!("Callee type kind: {:#?}", callee_ty.kind());
        //warn!("Arg expr: {:#?}", arg_exprs);


        // Try to get the signature of the function being called and its definition ID.
        let (fn_sig, def_id) = match *callee_ty.kind() {
            // We are calling a function that is defines somewhere
            ty::FnDef(def_id, subst) => {
                let new_ty = self.tcx.mk_ty(ty::Bool);
                let new_gen_arg = ty_slice_as_generic_args(self.tcx.arena.alloc_from_iter([new_ty]));
                let _new_subst = self.tcx.intern_substs(new_gen_arg);

                // Retrieve the function signature
                let fn_sig = self.tcx.bound_fn_sig(def_id);
                info!("Sig before substs: {:?}, substs: {:#?}", fn_sig, subst);
                //let fn_sig = fn_sig.subst(self.tcx, new_subst, HKTSubstType::SubstHKTParamWithType);
                let fn_sig = fn_sig.subst(self.tcx, subst, HKTSubstType::SubstHKTParamWithType);
                info!("Sig after substs: {:?}", fn_sig);

                /*
                struct InfFolder<'tcx> { tcx: TyCtxt<'tcx>, subst_ty: Ty<'tcx> }

                impl TypeFolder for InfFolder {
                    fn tcx<'a>(&'a self) -> TyCtxt<'tcx> {
                        self.tcx
                    }

                    fn fold_ty(&mut self, t: Ty<'tcx>) -> Ty<'tcx> {
                        if self.subst_ty == t {
                            self.subst_ty
                        } else {
                            t.super_fold_with(self)
                        }
                    }
                }



                let obligations = self.fulfillment_cx.borrow_mut().pending_obligations();

                let new_obligations = obligations
                    .iter()
                    .map(|obl| {
                        let subst_ty = self.tcx.mk_ty(ty::Bool);
                        obl.fold_with(&mut InfFolder { tcx: self.tcx, subst_ty })
                    })
                    .collect::<Vec<_>>();
                */
                /*
                let mut indices = Vec::new();
                for x in 0..obligations.len() {
                    if let Some(obligation) = obligations.get(x) {
                        /*
                        let wf = match obligation.predicate.kind().skip_binder() {
                            PredicateKind::WellFormed(t) => Some(t),
                            _ => None
                        };

                        if let Some(t) = wf {
                            if let Some(_) = subst.iter().find(|s| {s.expect_ty() == t.expect_ty()}) {
                                indices.push(x);
                            }
                        }
                        */
                        let subst_ty = self.tcx.mk_ty(ty::Bool);
                        let new_obligation = obligation.fold_with(
                            &mut InfFolder {
                                tcx: self.tcx,
                                subst_ty
                            }
                        );
                    }
                }

                for x in 0..indices.len() {
                    if let Some(found_x) = obligations.get(indices[x]) {
                        info!("found infer predicates{:#?}", found_x.predicate.kind().skip_binder());

                        if let Some(obl) = obligations.get(x) {
                            obl.fold_with(&mut InfFolder {tcx})
                        }
                    }
                }

                 */

                // Unit testing: function items annotated with
                // `#[rustc_evaluate_where_clauses]` trigger special output
                // to let us test the trait evaluation system.
                if self.tcx.has_attr(def_id, sym::rustc_evaluate_where_clauses) {
                    let predicates = self.tcx.predicates_of(def_id);
                    let predicates = predicates.instantiate(self.tcx, subst);
                    for (predicate, predicate_span) in
                        predicates.predicates.iter().zip(&predicates.spans)
                    {
                        let obligation = Obligation::new(
                            self.tcx,
                            ObligationCause::dummy_with_span(callee_expr.span),
                            self.param_env,
                            *predicate,
                        );
                        let result = self.evaluate_obligation(&obligation);
                        self.tcx
                            .sess
                            .struct_span_err(
                                callee_expr.span,
                                &format!("evaluate({:?}) = {:?}", predicate, result),
                            )
                            .span_label(*predicate_span, "predicate")
                            .emit();
                    }
                }


                (fn_sig, Some(def_id))
            }
            // If all we have is a function ptr, for example a closure.
            ty::FnPtr(sig) => (sig, None),
            // If we are trying to check a function call, but the expression being called was not a function
            _ => {
                for arg in arg_exprs {
                    self.check_expr(arg);
                }

                if let hir::ExprKind::Path(hir::QPath::Resolved(_, path)) = &callee_expr.kind
                    && let [segment] = path.segments
                    && let Some(mut diag) = self
                        .tcx
                        .sess
                        .diagnostic()
                        .steal_diagnostic(segment.ident.span, StashKey::CallIntoMethod)
                {
                    // Try suggesting `foo(a)` -> `a.foo()` if possible.
                    if let Some(ty) =
                        self.suggest_call_as_method(
                            &mut diag,
                            segment,
                            arg_exprs,
                            call_expr,
                            expected
                        )
                    {
                        diag.emit();
                        return ty;
                    } else {
                        diag.emit();
                    }
                }

                let err = self.report_invalid_callee(call_expr, callee_expr, callee_ty, arg_exprs);

                return self.tcx.ty_error_with_guaranteed(err);
            }
        };

        /*
        let fn_sig_new = if self.is_function_hkt(callee_ty) {
            if let Some(f) = self.get_constraints_if_hkt(callee_ty, arg_exprs, fn_sig, &expected_arg_tys) {
                f
            } else {
                fn_sig
            }
        } else {
            fn_sig
        };
        */

        // Replace any late-bound regions that appear in the function
        // signature with region variables. We also have to
        // re-normalize the associated types at this point, since they
        // previously appeared within a `Binder<>` and hence would not
        // have been normalized before.
        let fn_sig = self.replace_bound_vars_with_fresh_vars(call_expr.span, infer::FnCall, fn_sig);
        let fn_sig = self.normalize(call_expr.span, fn_sig);

        // Call the generic checker.
        let expected_arg_tys = self.expected_inputs_for_expected_output(
            call_expr.span,
            expected,
            fn_sig.output(),
            fn_sig.inputs(),
        );


        info!("fn_sig inputs {:#?}, with fun_id {:#?}", fn_sig.inputs(), def_id);
        self.check_argument_types(
            call_expr.span,
            call_expr,
            fn_sig.inputs(),
            expected_arg_tys,
            arg_exprs,
            fn_sig.c_variadic,
            TupleArgumentsFlag::DontTupleArguments,
            def_id,
        );

        if fn_sig.abi == abi::Abi::RustCall {
            let sp = arg_exprs.last().map_or(call_expr.span, |expr| expr.span);
            if let Some(ty) = fn_sig.inputs().last().copied() {
                self.register_bound(
                    ty,
                    self.tcx.require_lang_item(hir::LangItem::Tuple, Some(sp)),
                    traits::ObligationCause::new(sp, self.body_id, traits::RustCall),
                );
            } else {
                self.tcx.sess.span_err(
                        sp,
                        "functions with the \"rust-call\" ABI must take a single non-self tuple argument",
                    );
            }
        }

        fn_sig.output()
    }


    #[allow(dead_code)]
    fn is_function_hkt(&self, callee_ty: Ty<'tcx>) -> bool {
        match *callee_ty.kind() {
            ty::FnDef(def_id, _) => {
                let fn_signature = self.tcx.bound_fn_sig(def_id);
                let is_hkt = fn_signature.skip_binder().skip_binder().inputs().to_vec().iter().any(|inp| {
                    if let ty::HKT(_, _, _) = inp.kind() {
                        true
                    } else {
                        false
                    }
                });
                is_hkt
            },
            _ => false
        }
    }

    /*
    #[allow(dead_code)]
    fn get_constraints_if_hkt(&self, callee_ty: Ty<'tcx>, _arg_exprs: &[Expr<'tcx>], fn_sig: FnSig<'tcx>, expected_arg_tys: &Option<Vec<Ty<'tcx>>>) -> Option<PolyFnSig<'tcx>> {
        if fn_sig.inputs().len() > 1 {
            return None
        }
        info!("expect before function call: {:#?}", expected_arg_tys);
        info!("function signature: {:#?}", fn_sig);
        info!("callee ty: {:#?}", callee_ty);
        // Figure out whether this function have HKT parameters.
        // If so, infer the type I<%J> of the first argument.
        // When we infer the type of I - Infer_I - we will call 'check_argument_types', however, we have now changed formal_input_tys
        // by substituting the type parameters of the function signature of the called function with substitution containing Infer_I<%J>.
        // This will be analogous to calling the function with type annotation. This method only works when using %J of I

        match *callee_ty.kind() {
            ty::FnDef(def_id, _) => {
                let fn_signature = self.tcx.bound_fn_sig(def_id);

                let mut lefties = Vec::new();
                let mut righties = Vec::new();
                let mut constraints = Vec::new();

                for (l, i) in fn_signature.skip_binder().inputs().skip_binder().iter().zip(0..fn_signature.skip_binder().inputs().skip_binder().len()) {
                    // if l is HKT push to lefties
                    if let ty::HKT(_, _, _) = l.kind() {
                        lefties.push((l.clone(), i));
                    }
                }

                let expected_input_tys = if let Some(expect) = expected_arg_tys.clone() {
                    expect
                } else {
                    (*fn_sig.inputs()).to_vec()
                };

                let args_expect = _arg_exprs.iter().zip(expected_input_tys).collect::<Vec<(&Expr<'tcx>, Ty<'tcx>)>>();

                for (_, i) in &lefties {
                    let (arg, expect) = &args_expect[*i];
                    let r = self.check_expr_with_expectation(&arg, Expectation::rvalue_hint(self, *expect));
                    righties.push((r, i));
                }

                for i in 0..lefties.len() {
                    let (l_ty, _) = lefties[i];
                    let (r_ty, _) = righties[i];
                    //create constraint with l_ty and r_ty
                    constraints.push((l_ty, r_ty));
                }

                let solutions = Self::huets(HashMap::new(), constraints);
                let new_ty = solutions[0];
                Some(fn_signature.subst(self.tcx, &[GenericArg::from(new_ty)]))
            },
            _ => None
        }
    }

    fn huets(_context: HashMap<String, usize>, _problem: Vec<(Ty<'tcx>, Ty<'tcx>)>) -> Vec<Ty<'tcx>> {
        todo!()
    }
     */


    /// Attempts to reinterpret `method(rcvr, args...)` as `rcvr.method(args...)`
    /// and suggesting the fix if the method probe is successful.
    fn suggest_call_as_method(
        &self,
        diag: &mut Diagnostic,
        segment: &'tcx hir::PathSegment<'tcx>,
        arg_exprs: &'tcx [hir::Expr<'tcx>],
        call_expr: &'tcx hir::Expr<'tcx>,
        expected: Expectation<'tcx>,
    ) -> Option<Ty<'tcx>> {
        if let [callee_expr, rest @ ..] = arg_exprs {
            let callee_ty = self.typeck_results.borrow().expr_ty_adjusted_opt(callee_expr)?;

            // First, do a probe with `IsSuggestion(true)` to avoid emitting
            // any strange errors. If it's successful, then we'll do a true
            // method lookup.
            let Ok(pick) = self
            .probe_for_name(
                Mode::MethodCall,
                segment.ident,
                IsSuggestion(true),
                callee_ty,
                call_expr.hir_id,
                // We didn't record the in scope traits during late resolution
                // so we need to probe AllTraits unfortunately
                ProbeScope::AllTraits,
            ) else {
                return None;
            };

            let pick = self.confirm_method(
                call_expr.span,
                callee_expr,
                call_expr,
                callee_ty,
                &pick,
                segment,
            );
            if pick.illegal_sized_bound.is_some() {
                return None;
            }

            let up_to_rcvr_span = segment.ident.span.until(callee_expr.span);
            let rest_span = callee_expr.span.shrink_to_hi().to(call_expr.span.shrink_to_hi());
            let rest_snippet = if let Some(first) = rest.first() {
                self.tcx
                    .sess
                    .source_map()
                    .span_to_snippet(first.span.to(call_expr.span.shrink_to_hi()))
            } else {
                Ok(")".to_string())
            };

            if let Ok(rest_snippet) = rest_snippet {
                let sugg = if callee_expr.precedence().order() >= PREC_POSTFIX {
                    vec![
                        (up_to_rcvr_span, "".to_string()),
                        (rest_span, format!(".{}({rest_snippet}", segment.ident)),
                    ]
                } else {
                    vec![
                        (up_to_rcvr_span, "(".to_string()),
                        (rest_span, format!(").{}({rest_snippet}", segment.ident)),
                    ]
                };
                let self_ty = self.resolve_vars_if_possible(pick.callee.sig.inputs()[0]);
                diag.multipart_suggestion(
                    format!(
                        "use the `.` operator to call the method `{}{}` on `{self_ty}`",
                        self.tcx
                            .associated_item(pick.callee.def_id)
                            .trait_container(self.tcx)
                            .map_or_else(
                                || String::new(),
                                |trait_def_id| self.tcx.def_path_str(trait_def_id) + "::"
                            ),
                        segment.ident
                    ),
                    sugg,
                    Applicability::MaybeIncorrect,
                );

                // Let's check the method fully now
                let return_ty = self.check_method_argument_types(
                    segment.ident.span,
                    call_expr,
                    Ok(pick.callee),
                    rest,
                    TupleArgumentsFlag::DontTupleArguments,
                    expected,
                );

                return Some(return_ty);
            }
        }

        None
    }

    fn report_invalid_callee(
        &self,
        call_expr: &'tcx hir::Expr<'tcx>,
        callee_expr: &'tcx hir::Expr<'tcx>,
        callee_ty: Ty<'tcx>,
        arg_exprs: &'tcx [hir::Expr<'tcx>],
    ) -> ErrorGuaranteed {
        let mut unit_variant = None;
        if let hir::ExprKind::Path(qpath) = &callee_expr.kind
            && let Res::Def(def::DefKind::Ctor(kind, CtorKind::Const), _)
                = self.typeck_results.borrow().qpath_res(qpath, callee_expr.hir_id)
            // Only suggest removing parens if there are no arguments
            && arg_exprs.is_empty()
        {
            let descr = match kind {
                def::CtorOf::Struct => "struct",
                def::CtorOf::Variant => "enum variant",
            };
            let removal_span = callee_expr.span.shrink_to_hi().to(call_expr.span.shrink_to_hi());
            unit_variant = Some((removal_span, descr, rustc_hir_pretty::qpath_to_string(qpath)));
        }

        let callee_ty = self.resolve_vars_if_possible(callee_ty);
        let mut err = type_error_struct!(
            self.tcx.sess,
            callee_expr.span,
            callee_ty,
            E0618,
            "expected function, found {}",
            match &unit_variant {
                Some((_, kind, path)) => format!("{kind} `{path}`"),
                None => format!("`{callee_ty}`"),
            }
        );

        self.identify_bad_closure_def_and_call(
            &mut err,
            call_expr.hir_id,
            &callee_expr.kind,
            callee_expr.span,
        );

        if let Some((removal_span, kind, path)) = &unit_variant {
            err.span_suggestion_verbose(
                *removal_span,
                &format!(
                    "`{path}` is a unit {kind}, and does not take parentheses to be constructed",
                ),
                "",
                Applicability::MachineApplicable,
            );
        }

        let mut inner_callee_path = None;
        let def = match callee_expr.kind {
            hir::ExprKind::Path(ref qpath) => {
                self.typeck_results.borrow().qpath_res(qpath, callee_expr.hir_id)
            }
            hir::ExprKind::Call(ref inner_callee, _) => {
                // If the call spans more than one line and the callee kind is
                // itself another `ExprCall`, that's a clue that we might just be
                // missing a semicolon (Issue #51055)
                let call_is_multiline = self.tcx.sess.source_map().is_multiline(call_expr.span);
                if call_is_multiline {
                    err.span_suggestion(
                        callee_expr.span.shrink_to_hi(),
                        "consider using a semicolon here",
                        ";",
                        Applicability::MaybeIncorrect,
                    );
                }
                if let hir::ExprKind::Path(ref inner_qpath) = inner_callee.kind {
                    inner_callee_path = Some(inner_qpath);
                    self.typeck_results.borrow().qpath_res(inner_qpath, inner_callee.hir_id)
                } else {
                    Res::Err
                }
            }
            _ => Res::Err,
        };

        if !self.maybe_suggest_bad_array_definition(&mut err, call_expr, callee_expr) {
            if let Some((maybe_def, output_ty, _)) =
                self.extract_callable_info(callee_expr, callee_ty)
                && !self.type_is_sized_modulo_regions(self.param_env, output_ty, callee_expr.span)
            {
                let descr = match maybe_def {
                    DefIdOrName::DefId(def_id) => self.tcx.def_kind(def_id).descr(def_id),
                    DefIdOrName::Name(name) => name,
                };
                err.span_label(
                    callee_expr.span,
                    format!("this {descr} returns an unsized value `{output_ty}`, so it cannot be called")
                );
                if let DefIdOrName::DefId(def_id) = maybe_def
                    && let Some(def_span) = self.tcx.hir().span_if_local(def_id)
                {
                    err.span_label(def_span, "the callable type is defined here");
                }
            } else {
                err.span_label(call_expr.span, "call expression requires function");
            }
        }

        if let Some(span) = self.tcx.hir().res_span(def) {
            let callee_ty = callee_ty.to_string();
            let label = match (unit_variant, inner_callee_path) {
                (Some((_, kind, path)), _) => Some(format!("{kind} `{path}` defined here")),
                (_, Some(hir::QPath::Resolved(_, path))) => self
                    .tcx
                    .sess
                    .source_map()
                    .span_to_snippet(path.span)
                    .ok()
                    .map(|p| format!("`{p}` defined here returns `{callee_ty}`")),
                _ => {
                    match def {
                        // Emit a different diagnostic for local variables, as they are not
                        // type definitions themselves, but rather variables *of* that type.
                        Res::Local(hir_id) => Some(format!(
                            "`{}` has type `{}`",
                            self.tcx.hir().name(hir_id),
                            callee_ty
                        )),
                        Res::Def(kind, def_id) if kind.ns() == Some(Namespace::ValueNS) => {
                            Some(format!("`{}` defined here", self.tcx.def_path_str(def_id),))
                        }
                        _ => Some(format!("`{callee_ty}` defined here")),
                    }
                }
            };
            if let Some(label) = label {
                err.span_label(span, label);
            }
        }
        err.emit()
    }

    fn confirm_deferred_closure_call(
        &self,
        call_expr: &'tcx hir::Expr<'tcx>,
        arg_exprs: &'tcx [hir::Expr<'tcx>],
        expected: Expectation<'tcx>,
        closure_def_id: LocalDefId,
        fn_sig: ty::FnSig<'tcx>,
    ) -> Ty<'tcx> {
        // `fn_sig` is the *signature* of the closure being called. We
        // don't know the full details yet (`Fn` vs `FnMut` etc), but we
        // do know the types expected for each argument and the return
        // type.

        let expected_arg_tys = self.expected_inputs_for_expected_output(
            call_expr.span,
            expected,
            fn_sig.output(),
            fn_sig.inputs(),
        );

        self.check_argument_types(
            call_expr.span,
            call_expr,
            fn_sig.inputs(),
            expected_arg_tys,
            arg_exprs,
            fn_sig.c_variadic,
            TupleArgumentsFlag::TupleArguments,
            Some(closure_def_id.to_def_id()),
        );

        fn_sig.output()
    }

    fn confirm_overloaded_call(
        &self,
        call_expr: &'tcx hir::Expr<'tcx>,
        arg_exprs: &'tcx [hir::Expr<'tcx>],
        expected: Expectation<'tcx>,
        method_callee: MethodCallee<'tcx>,
    ) -> Ty<'tcx> {
        let output_type = self.check_method_argument_types(
            call_expr.span,
            call_expr,
            Ok(method_callee),
            arg_exprs,
            TupleArgumentsFlag::TupleArguments,
            expected,
        );

        self.write_method_call(call_expr.hir_id, method_callee);
        output_type
    }
}

#[derive(Debug)]
pub struct DeferredCallResolution<'tcx> {
    call_expr: &'tcx hir::Expr<'tcx>,
    callee_expr: &'tcx hir::Expr<'tcx>,
    adjusted_ty: Ty<'tcx>,
    adjustments: Vec<Adjustment<'tcx>>,
    fn_sig: ty::FnSig<'tcx>,
    closure_substs: SubstsRef<'tcx>,
}

impl<'a, 'tcx> DeferredCallResolution<'tcx> {
    pub fn resolve(self, fcx: &FnCtxt<'a, 'tcx>) {
        debug!("DeferredCallResolution::resolve() {:?}", self);

        // we should not be invoked until the closure kind has been
        // determined by upvar inference
        assert!(fcx.closure_kind(self.closure_substs).is_some());

        // We may now know enough to figure out fn vs fnmut etc.
        match fcx.try_overloaded_call_traits(self.call_expr, self.adjusted_ty, None) {
            Some((autoref, method_callee)) => {
                // One problem is that when we get here, we are going
                // to have a newly instantiated function signature
                // from the call trait. This has to be reconciled with
                // the older function signature we had before. In
                // principle we *should* be able to fn_sigs(), but we
                // can't because of the annoying need for a TypeTrace.
                // (This always bites me, should find a way to
                // refactor it.)
                let method_sig = method_callee.sig;

                debug!("attempt_resolution: method_callee={:?}", method_callee);

                for (method_arg_ty, self_arg_ty) in
                    iter::zip(method_sig.inputs().iter().skip(1), self.fn_sig.inputs())
                {
                    fcx.demand_eqtype(self.call_expr.span, *self_arg_ty, *method_arg_ty);
                }

                fcx.demand_eqtype(self.call_expr.span, method_sig.output(), self.fn_sig.output());

                let mut adjustments = self.adjustments;
                adjustments.extend(autoref);
                fcx.apply_adjustments(self.callee_expr, adjustments);

                fcx.write_method_call(self.call_expr.hir_id, method_callee);
            }
            None => {
                // This can happen if `#![no_core]` is used and the `fn/fn_mut/fn_once`
                // lang items are not defined (issue #86238).
                let mut err = fcx.inh.tcx.sess.struct_span_err(
                    self.call_expr.span,
                    "failed to find an overloaded call trait for closure call",
                );
                err.help(
                    "make sure the `fn`/`fn_mut`/`fn_once` lang items are defined \
                     and have associated `call`/`call_mut`/`call_once` functions",
                );
                err.emit();
            }
        }
    }
}
