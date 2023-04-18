use std::cell::RefCell;
use std::hash::BuildHasherDefault;
use std::rc::Rc;
use rustc_ast::Mutability;
use rustc_data_structures::fx::{FxHasher, FxHashMap};
use rustc_errors::fluent_bundle::types::AnyEq;
use rustc_middle::ty;
use rustc_middle::ty::{AdtDef, ArgumentDef, ClosureSubsts, GenericArg, GenericArgKind, PolyFnSig, TyCtxt, TypeAndMut};
use rustc_span::def_id::DefId;
use rustc_target::spec::abi::Abi;
use crate::huets::datatype::{Constraint, Context, Problem, Solution, Type, Term, SolutionSet};
use crate::huets::datatype::Term::{Abs, App, Meta, Var};
use crate::huets::r#match::match_;
use crate::huets::simpl::simpl;
use crate::huets::substs::problem_substitution;
use crate::rustc_middle::ty::Ty;

mod datatype;
mod substs;
mod r#match;
mod simpl;
mod print;
mod util;


pub fn main_huet(context: &mut Context, problem: Problem) {
    let p_simpl = simpl(context.clone(), problem);

    if p_simpl.is_none() {
        return
    }

    let p_simpl = p_simpl.unwrap();

    if p_simpl.0.is_empty() {
        context.solutions.borrow_mut().push(Solution(context.substitutions.clone()));
        return;
    }

    let constraint = p_simpl.0[0].clone();

    let substitution_set = match_(context.clone(), constraint);

    for substitution in substitution_set {
        let new_problem = problem_substitution(p_simpl.clone(), substitution.clone());
        let mut substs_for_context = context.substitutions.clone();
        substs_for_context.push(substitution.clone());

        let mut new_context = Context {
            typing_context: context.typing_context.clone(),
            substitutions: substs_for_context,
            solutions: context.solutions.clone(),
            name_map: context.name_map.clone(),
        };

        main_huet(&mut new_context, new_problem);
    }
}

pub fn create_constraints_from_rust_tys<'tcx>(tcx: TyCtxt<'tcx>, ty_map: &mut FxHashMap<String, Ty<'tcx>>, left: Vec<Ty<'tcx>>, right: Vec<Ty<'tcx>>) -> (Context, Problem) {
    let mut context = new_context();
    let (lh, rh) = map_list_of_rust_ty_to_huet_ty(tcx, &mut context, ty_map, left, right);
    (context, Problem(map_list_of_tys_to_constraints(lh, rh)))
}

pub fn solution_as_ty<'tcx>(tcx: TyCtxt<'tcx>, ty_map: &FxHashMap<String, Ty<'tcx>>, solution: Solution) -> Vec<Ty<'tcx>> {
    let mut tys = Vec::new();

    for subst in solution.0 {
        let term = subst.with;
        let name = subst.name;
        if let Some(ty) = map_term_to_ty(tcx, ty_map, name.clone(), term, &mut Vec::new(), &mut vec![]) {
            tys.push(ty);
        }
    }
    tys.reverse();
    tys
}

pub fn get_solution_from_solution_set(solutions: SolutionSet) -> Result<Solution, SolutionSet> {
    let existence_filtered = existence(solutions);
    let generality_filtered = generality(existence_filtered);
    let exhaustiveness_filtered = exhaustiveness(generality_filtered);
    let ordering_filtered = ordering(exhaustiveness_filtered);
    let simplicity_filterered = simplicity(ordering_filtered);

    if simplicity_filterered.0.len() == 1 {
        Ok(simplicity_filterered.0[0].clone())
    } else {
        Err(simplicity_filterered)
    }
}

fn existence(mut solutions: SolutionSet) -> SolutionSet {
    solutions.0.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    let mut new_list = Vec::new();
    if let Some(first) = solutions.0.first() {
        let max = first.0.len();

        while let Some(elem) = solutions.0.pop() {
            if elem.0.len() == max {
                new_list.push(elem);
            }
        }
    }
    SolutionSet(new_list)
}

fn generality(mut solutions: SolutionSet) -> SolutionSet {
    solutions.0.sort_by(|a, b| {
        a.number_of_constants().cmp(&b.number_of_constants())
    });

    let mut new_list = Vec::new();
    if let Some(first) = solutions.0.first() {
        let max = first.number_of_constants();

        while let Some(elem) = solutions.0.pop() {
            if elem.number_of_constants() == max {
                new_list.push(elem);
            }
        }
    }

    SolutionSet(new_list)
}

fn exhaustiveness(mut solutions: SolutionSet) -> SolutionSet {
    solutions.0.sort_by(|a, b| {
        b.number_of_unique_params().cmp(&a.number_of_unique_params())
    });

    let mut new_list = Vec::new();
    if let Some(first) = solutions.0.first() {
        let max = first.number_of_unique_params();

        while let Some(elem) = solutions.0.pop() {
            if elem.number_of_unique_params() == max {
                new_list.push(elem);
            }
        }
    }

    SolutionSet(new_list)
}

fn ordering(mut solutions: SolutionSet) -> SolutionSet {
    solutions.0.sort_by(|a, b| {
        // FIXMIG : what to we sort by?
        a.number_of_swaps().cmp(&b.number_of_swaps())
    });

    let mut new_list = Vec::new();
    if let Some(first) = solutions.0.first() {
        let max = first.number_of_swaps();

        while let Some(elem) = solutions.0.pop() {
            if elem.number_of_swaps() == max {
                new_list.push(elem);
            }
        }
    }

    SolutionSet(new_list)
}

fn simplicity(mut solutions: SolutionSet) -> SolutionSet {
    solutions.0.sort_by(|a, b| {
        // FIXMIG : what to we sort by?
        a.number_of_params().cmp(&b.number_of_params())
    });

    let mut new_list = Vec::new();
    if let Some(first) = solutions.0.first() {
        let max = first.number_of_params();

        while let Some(elem) = solutions.0.pop() {
            if elem.number_of_params() == max {
                new_list.push(elem);
            }
        }
    }

    SolutionSet(new_list)
}

fn map_term_to_ty<'tcx>(
    tcx: TyCtxt<'tcx>,
    ty_map: &FxHashMap<String, Ty<'tcx>>,
    mapping_for: String,
    term: Term,
    type_args: &mut Vec<String>,
    substitutions: &mut Vec<Ty<'tcx>>
) -> Option<Ty<'tcx>> {
    match term {
        Var(s) => {
            if type_args.contains(&s) {
                // FIXMIG: create ty::Argument
                let ty_opt: Option<Ty<'tcx>> = ty_map.get(mapping_for.as_str()).cloned();
                if let None = ty_opt {
                    info!("cannot map following type from term 1");
                    return None
                }
                let type_arg_index_opt = type_args.iter().position(|arg| arg.equals(&s));
                if let None = type_arg_index_opt {
                    info!("cannot map following type from term 2");
                    return None
                }

                match ty_opt.unwrap().kind() {
                    ty::HKT(did, _, _) => {
                        let adid : DefId = *did;
                        let generics: &ty::Generics = tcx.generics_of(did);
                        let index = type_arg_index_opt.unwrap();

                        if index < generics.params.len() {
                            return Some(tcx.mk_ty(ty::Argument(ArgumentDef {
                                def_id: adid,
                                index: generics.params[index].index,
                                name: generics.params[index].name,
                            })))
                        }
                        info!("cannot map following type from term 3");
                        None
                    }
                    _ => {
                        info!("cannot map following type from term 4");
                        None
                    }
                }

            } else {
                ty_map.get(s.as_str()).cloned()
            }

        },
        Abs(arg, t, inner_term) => {
            assert_eq!(t, Type::Star);
            type_args.push(arg);
            map_term_to_ty(tcx, ty_map, mapping_for, *inner_term, type_args, &mut vec![])
        }
        App(callee, call_arg) => {
            let call_arg_ty_opt = map_term_to_ty(tcx, ty_map, mapping_for.clone(), *call_arg.clone(), type_args, &mut vec![]);
            if let None = call_arg_ty_opt {
                info!("cannot map following type from term 5");
                return None;
            }
            let mapped_opt = map_term_to_ty(tcx, ty_map, mapping_for.clone(), *callee.clone(), type_args, substitutions);
            if let None = mapped_opt {
                info!("cannot map following type from term 6");
                return None
            }
            substitutions.push(call_arg_ty_opt.unwrap());
            match mapped_opt.unwrap().kind() {
                ty::Adt(adt_def, _) => {
                    let subst = tcx.mk_substs(substitutions.clone().into_iter().map(|x| GenericArg::from(x)));
                    //FIXMIG: remove old adt?
                    Some(tcx.mk_adt(*adt_def, subst))
                }
                ty::Tuple(_) => {
                    Some(tcx.mk_tup(substitutions.clone().into_iter()))
                }
                ty::Array(_, size) => {
                    assert_eq!(substitutions.len(), 1);
                    Some(tcx.mk_ty(ty::Array(substitutions[0].clone(), size.clone())))
                }
                ty::Slice(_) => {
                    assert_eq!(substitutions.len(), 1);
                    Some(tcx.mk_ty(ty::Slice(substitutions[0].clone())))
                }
                ty::Ref(l,_,m) => {
                    assert_eq!(substitutions.len(), 1);
                    Some(tcx.mk_ty(ty::Ref(l.clone(), substitutions[0].clone(), m.clone())))
                }
                ty::FnPtr(fnpolysig) => {
                    let new_poly = fnpolysig.map_bound(|ty::FnSig{ c_variadic, unsafety, abi, .. }| {
                        ty::FnSig {
                            inputs_and_output: tcx.mk_type_list(substitutions.iter()),
                            c_variadic,
                            unsafety,
                            abi,
                        }
                    });
                    info!("its a function pointer!");
                    Some(tcx.mk_fn_ptr(new_poly))
                }
                ty::Closure(_,substs) => {
                    let substss : ClosureSubsts<'tcx> = ClosureSubsts { substs };

                    let fn_sig = substss.sig();

                    let new_poly = fn_sig.map_bound(|ty::FnSig{ c_variadic, unsafety, abi, .. }| {
                        info!("fucking abi {:#?}", abi);
                        ty::FnSig {
                            inputs_and_output: tcx.mk_type_list(substitutions.iter()),
                            c_variadic,
                            unsafety,
                            abi: Abi::Rust, // FIXMIG: should we not do this?
                        }
                    });


                    Some(tcx.mk_ty(ty::FnPtr(new_poly)))

                }
                ty::FnDef(def_id, _) => {
                    // FIXMIG: We cannot map to FnPtr every time. If FnDef is a closure then we cannot map it back to a FnPtr
                    /*let car_ty = tcx.mk_ty(ty::Char);
                    let subst = tcx.mk_substs([car_ty].into_iter().map(|x| GenericArg::from(x)));
                    Some(tcx.mk_fn_def(*defid, subst))*/
                    let fn_sig: PolyFnSig<'tcx> = tcx.fn_sig(def_id);

                    let new_poly = fn_sig.map_bound(|ty::FnSig{ c_variadic, unsafety, abi, .. }| {
                        ty::FnSig {
                            inputs_and_output: tcx.mk_type_list(substitutions.iter()),
                            c_variadic,
                            unsafety,
                            abi,
                        }
                    });
                    info!("its a function def!");
                    Some(tcx.mk_fn_ptr(new_poly))
                }
                _ => {
                    info!("cannot map following type from term 7: {:?}", mapped_opt.unwrap().kind());
                    None
                }
            }
        }
        _ => None
    }
}

fn map_rust_ty_to_huet_ty<'tcx>(tcx: TyCtxt<'tcx>, ctxt: &mut Context, ty_map: &mut FxHashMap<String, Ty<'tcx>>, rust_ty: Ty<'tcx>) -> Option<(Term, Type)> {
    match rust_ty.kind() {
        ty::Bool => {
            ctxt.typing_context.insert("bool".to_string(), Type::Star);
            ty_map.insert("bool".to_string(), rust_ty);
            Some((Var("bool".to_string()), Type::Star))
        },
        ty::Char => {
            ctxt.typing_context.insert("char".to_string(), Type::Star);
            ty_map.insert("char".to_string(), rust_ty);
            Some((Var("char".to_string()), Type::Star))
        },
        ty::Int(int_ty) => {
            // FIXMIG: check om IntTy er unknown
            ctxt.typing_context.insert(int_ty.name_str().to_string(), Type::Star);
            ty_map.insert(int_ty.name_str().to_string(), rust_ty);
            Some((Var(int_ty.name_str().to_string()), Type::Star))
        },
        ty::Uint(uint_ty) => {
            ctxt.typing_context.insert(uint_ty.name_str().to_string(), Type::Star);
            ty_map.insert(uint_ty.name_str().to_string(), rust_ty);
            Some((Var(uint_ty.name_str().to_string()), Type::Star))
        },
        ty::Float(float_ty) => {
            ctxt.typing_context.insert(float_ty.name_str().to_string(), Type::Star);
            ty_map.insert(float_ty.name_str().to_string(), rust_ty);
            Some((Var(float_ty.name_str().to_string()), Type::Star))
        },
        ty::Str => {
            ctxt.typing_context.insert("str".to_string(), Type::Star);
            ty_map.insert("str".to_string(), rust_ty);
            Some((Var("str".to_string()), Type::Star))
        },
        ty::Ref(_, t, m) => {
            if let Some((inner_ty, _)) = map_rust_ty_to_huet_ty(tcx, ctxt, ty_map, *t) {
                // FIXMIG: add lifetime to name?
                let ref_name = format!("&{:?}", m);
                ctxt.typing_context.insert(ref_name.clone(), Type::Arrow(Box::new(Type::Star), Box::new(Type::Star)));
                ty_map.insert(ref_name.clone(), rust_ty);
                Some((App(
                    Box::new(Var(ref_name.clone())),
                    Box::new(inner_ty)
                ), Type::Star))
            } else {
                None
            }
        }
        ty::Array(t, size) => {
            if let Some((inner_ty, _)) = map_rust_ty_to_huet_ty(tcx, ctxt, ty_map, *t) {
                let array_name = format!("a-{:?}[]", size);
                ctxt.typing_context.insert(array_name.clone(), Type::Arrow(Box::new(Type::Star), Box::new(Type::Star)));
                ty_map.insert(array_name.clone(), rust_ty);
                Some((App(
                    Box::new(Var(array_name.clone())),
                    Box::new(inner_ty)
                ), Type::Star))
            } else {
                None
            }
        }
        ty::Slice(t) => {
            if let Some((inner_ty, _)) = map_rust_ty_to_huet_ty(tcx, ctxt, ty_map, *t) {
                ctxt.typing_context.insert("s[]".to_string(), Type::Arrow(Box::new(Type::Star), Box::new(Type::Star)));
                ty_map.insert("s[]".to_string(), rust_ty);
                Some((App(
                    Box::new(Var("s[]".to_string())),
                    Box::new(inner_ty)
                ), Type::Star))
            } else {
                None
            }
        }
        ty::Tuple(list) => {
            if list.len() == 0 {
                ctxt.typing_context.insert("()".to_string(),Type::Star);
                ty_map.insert("()".to_string(), rust_ty);
                Some((Var("()".to_string()), Type::Star))
            } else {
                let mut counter = 0;
                let mut term_acc = Var("()".to_string());
                let mut term_kind = Type::Star;
                while counter < list.len() {
                    let i = map_rust_ty_to_huet_ty(tcx, ctxt, ty_map, list[counter]);
                    if let Some((t, _)) = i {
                        term_kind = Type::Arrow(
                            Box::new(Type::Star),
                            Box::new(term_kind)
                        );

                        term_acc = App(
                            Box::new(term_acc),
                            Box::new(t)
                        )
                    }
                    counter += 1;
                }

                ctxt.typing_context.insert("()".to_string(), term_kind.clone());
                ty_map.insert("()".to_string(), rust_ty);
                Some((term_acc, term_kind))
            }
        }
        ty::Argument(_) => {
            ctxt.typing_context.insert("%".to_string(), Type::Star);
            ty_map.insert("%".to_string(), rust_ty);
            Some((Var("%".to_string()), Type::Star))
        },
        ty::Infer(v) => {
            ctxt.typing_context.insert(v.to_string(), Type::Star);
            ty_map.insert(v.to_string(), rust_ty);
            Some((Var(v.to_string()), Type::Star))
        }
        ty::RawPtr(TypeAndMut {ty, mutbl}) => {
            if let Some((inner_ty, _)) = map_rust_ty_to_huet_ty(tcx, ctxt, ty_map, *ty) {
                match mutbl {
                    Mutability::Mut => {
                        ctxt.typing_context.insert("*mut".to_string(), Type::Arrow(Box::new(Type::Star), Box::new(Type::Star)));
                        ty_map.insert("*mut".to_string(), rust_ty);
                        Some((App(
                            Box::new(Var("*mut".to_string())),
                            Box::new(inner_ty)
                        ), Type::Star))
                    },
                    Mutability::Not => {
                        ctxt.typing_context.insert("*const".to_string(), Type::Arrow(Box::new(Type::Star), Box::new(Type::Star)));
                        ty_map.insert("*const".to_string(), rust_ty);
                        Some((App(
                            Box::new(Var("*const".to_string())),
                            Box::new(inner_ty)
                        ), Type::Star))
                    },
                }
            } else {
                None
            }
        }
        ty::Adt(a, substs) => {
            let b: AdtDef<'tcx> = *a;
            let adt_name = format!("{:?}", b.did());
            if substs.len() == 0 {
                ctxt.typing_context.insert(adt_name.clone(), Type::Star);
                ty_map.insert(adt_name.clone(), rust_ty);
                Some((Var(adt_name), Type::Star))
            } else {
                let mut counter = 0;
                let mut term_acc = Var(adt_name.clone());
                let mut term_kind = Type::Star;
                while counter < substs.len() {
                    let i = match substs[counter].unpack() {
                        GenericArgKind::Type(ty) => map_rust_ty_to_huet_ty(tcx, ctxt, ty_map, ty),
                        _ => None
                    };
                    if let Some((t, _)) = i {
                        term_kind = Type::Arrow(
                            Box::new(Type::Star),
                            Box::new(term_kind)
                        );
                        term_acc = App(
                            Box::new(term_acc),
                            Box::new(t)
                        )
                    }
                    counter += 1;
                }
                ctxt.typing_context.insert(adt_name.clone(), term_kind.clone());
                ty_map.insert(adt_name.clone(), rust_ty);
                Some((term_acc, term_kind))
            }
        }
        ty::FnDef(did, _) => {
            // FIXMIG: We should take account for binders and unsafety which are found in fn_sig
            let fn_sig : PolyFnSig<'tcx> = tcx.fn_sig(did);
            let fn_inputs = fn_sig.skip_binder().inputs();
            let fn_output = fn_sig.skip_binder().output();
            let fn_name = format!("fn{}{}", fn_sig.unsafety(), fn_inputs.len());

            ty_map.insert(fn_name.clone(), rust_ty);

            let mut term_acc = Var(fn_name.clone());
            let mut term_kind = Type::Star;

            for input_ty in fn_inputs {
                let term_opt = map_rust_ty_to_huet_ty(tcx, ctxt, ty_map, input_ty.clone());

                if let Some((term, _)) = term_opt {
                    term_kind = Type::Arrow(
                        Box::new(Type::Star),
                        Box::new(term_kind)
                    );
                    term_acc = App(
                        Box::new(term_acc),
                        Box::new(term)
                    );
                }
            }

            let output_term_opt = map_rust_ty_to_huet_ty(tcx, ctxt, ty_map, fn_output.clone());
            if let Some((term, _)) = output_term_opt {
                term_kind = Type::Arrow(
                    Box::new(Type::Star),
                    Box::new(term_kind)
                );
                term_acc = App(
                    Box::new(term_acc),
                    Box::new(term)
                );
            }

            ctxt.typing_context.insert(fn_name.clone(), term_kind.clone());
            ty_map.insert(fn_name.clone(), rust_ty);
            Some((term_acc, term_kind))
        }
        ty::Closure(_,substs) => {
            let fn_sig : PolyFnSig<'tcx> = substs.as_closure().sig();
            let fn_inputs_container = fn_sig.skip_binder().inputs();
            let fn_inputs = match fn_inputs_container.first().unwrap().kind() {
                ty::Tuple(list_ty) => list_ty.clone(),
                _ => panic!("closure should only have tuple as input")
            };
            info!("closure inputs {:#?}", fn_inputs);
            let fn_output = fn_sig.skip_binder().output();
            let fn_name = format!("fn{}{}", fn_sig.unsafety(), fn_inputs.len());

            ty_map.insert(fn_name.clone(), rust_ty);

            let mut term_acc = Var(fn_name.clone());
            let mut term_kind = Type::Star;

            for input_ty in fn_inputs {
                let term_opt = map_rust_ty_to_huet_ty(tcx, ctxt, ty_map, input_ty.clone());

                if let Some((term, _)) = term_opt {
                    term_kind = Type::Arrow(
                        Box::new(Type::Star),
                        Box::new(term_kind)
                    );
                    term_acc = App(
                        Box::new(term_acc),
                        Box::new(term)
                    );
                }
            }

            let output_term_opt = map_rust_ty_to_huet_ty(tcx, ctxt, ty_map, fn_output.clone());
            if let Some((term, _)) = output_term_opt {
                term_kind = Type::Arrow(
                    Box::new(Type::Star),
                    Box::new(term_kind)
                );
                term_acc = App(
                    Box::new(term_acc),
                    Box::new(term)
                );
            }

            ctxt.typing_context.insert(fn_name.clone(), term_kind.clone());
            ty_map.insert(fn_name.clone(), rust_ty);
            Some((term_acc, term_kind))

        }
        ty::HKT(did, _, substs) => {
            let hkt_name = format!("{:?}", did);
            if substs.len() == 0 {
                ty_map.insert(hkt_name.clone(), rust_ty);
                Some((Meta(hkt_name), Type::Star))
            } else {
                let mut counter = 0;
                let mut term_acc = Meta(hkt_name.clone());
                let mut term_kind = Type::Star;
                while counter < substs.len() {
                    let i = match substs[counter].unpack() {
                        GenericArgKind::Type(ty) => map_rust_ty_to_huet_ty(tcx, ctxt, ty_map, ty),
                        _ => None
                    };
                    if let Some((t, _)) = i {
                        term_kind = Type::Arrow(
                            Box::new(Type::Star),
                            Box::new(term_kind)
                        );
                        term_acc = App(
                            Box::new(term_acc),
                            Box::new(t)
                        )
                    }
                    counter += 1;
                }
                ty_map.insert(hkt_name.clone(), rust_ty);
                Some((term_acc, term_kind))
            }
        }
        ty::Param(param_ty) => {
            let param_name = param_ty.name().to_string();
            ctxt.typing_context.insert(param_name.clone(), Type::Star);
            ty_map.insert(param_name.clone(), rust_ty);
            Some((Meta(param_name), Type::Star))
        }
        ty::FnPtr(polysig) => {
            // FIXMIG: We need to take account for unsafety field in PolyFnSig
            let sig: PolyFnSig<'tcx> = *polysig;
            let fn_inputs = sig.skip_binder().inputs();
            let fn_output = sig.skip_binder().output();

            let fn_name = format!("fn{}{}", sig.unsafety(), fn_inputs.len());
            ty_map.insert(fn_name.clone(), rust_ty);

            let mut term_acc = Var(fn_name.clone());
            let mut term_kind = Type::Star;

            for input_ty in fn_inputs {
                let term_opt = map_rust_ty_to_huet_ty(tcx, ctxt, ty_map, input_ty.clone());

                if let Some((term, _)) = term_opt {
                    term_kind = Type::Arrow(
                        Box::new(Type::Star),
                        Box::new(term_kind)
                    );
                    term_acc = App(
                        Box::new(term_acc),
                        Box::new(term)
                    );
                }
            }

            let output_term_opt = map_rust_ty_to_huet_ty(tcx, ctxt, ty_map, fn_output.clone());
            if let Some((term, _)) = output_term_opt {
                term_kind = Type::Arrow(
                    Box::new(Type::Star),
                    Box::new(term_kind)
                );
                term_acc = App(
                    Box::new(term_acc),
                    Box::new(term)
                );
            }

            ctxt.typing_context.insert(fn_name.clone(), term_kind.clone());
            ty_map.insert(fn_name.clone(), rust_ty);
            Some((term_acc, term_kind))
        }
        _ => {
            info!("failing on rust ty: {:#?}", rust_ty.kind());
            None
        }
    }
}

fn map_list_of_rust_ty_to_huet_ty<'tcx>(tcx: TyCtxt<'tcx>, ctxt: &mut Context, ty_map: &mut FxHashMap<String, Ty<'tcx>>, left: Vec<Ty<'tcx>>, right: Vec<Ty<'tcx>>) -> (Vec<Term>, Vec<Term>) {
    assert_eq!(left.len(),right.len());
    let mut left_acc = Vec::new();
    let mut right_acc = Vec::new();

    let mut counter = 0;
    while counter < left.len() {
        let t1 = map_rust_ty_to_huet_ty(tcx, ctxt, ty_map, left[counter].clone());
        let t2 = map_rust_ty_to_huet_ty(tcx, ctxt, ty_map, right[counter].clone());

        if let (Some((l, _)), Some((r, _))) = (t1, t2) {
            left_acc.push(l);
            right_acc.push(r);
        }
        counter += 1;
    }
    assert_eq!(right_acc.len(),left_acc.len());
    assert_eq!(left.len(),left_acc.len());
    (left_acc, right_acc)
}

fn map_list_of_tys_to_constraints(l1: Vec<Term>, l2: Vec<Term>) -> Vec<Constraint> {
    let mut cons_acc = Vec::new();

    assert_eq!(l1.len(),l2.len());

    let mut counter = 0;
    while counter < l1.len() {
        let t1 = l1[counter].clone();
        let t2 = l2[counter].clone();

        let constraint = Constraint {left: t1, right: t2};
        cons_acc.push(constraint);
        counter += 1;
    }
    cons_acc
}

fn new_context() -> Context {
    Context {
        typing_context: FxHashMap::with_hasher(BuildHasherDefault::<FxHasher>::default()),
        substitutions: Vec::new(),
        solutions: Rc::new(RefCell::new(vec![])),
        name_map: FxHashMap::with_hasher(BuildHasherDefault::<FxHasher>::default()),
    }
}
