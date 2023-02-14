use super::{ForceCollect, Parser, TrailingToken};

use rustc_ast::{Generics, token};
use rustc_ast::{
    self as ast, AttrVec, GenericBounds, GenericParam, GenericParamKind, TyKind, WhereClause,
};
use rustc_ast::token::BinOp;
use rustc_errors::{Applicability, PResult};
use rustc_span::symbol::kw;

impl<'a> Parser<'a> {
    /// Parses bounds of a lifetime parameter `BOUND + BOUND + BOUND`, possibly with trailing `+`.
    ///
    /// ```text
    /// BOUND = LT_BOUND (e.g., `'a`)
    /// ```
    fn parse_lt_param_bounds(&mut self) -> GenericBounds {
        let mut lifetimes = Vec::new();
        while self.check_lifetime() {
            lifetimes.push(ast::GenericBound::Outlives(self.expect_lifetime()));

            if !self.eat_plus() {
                break;
            }
        }
        lifetimes
    }

    fn parse_hkt_param_helper1(&mut self) -> PResult<'a, GenericParam> {
        let _ = self.expect(&BinOp(token::Percent))?;

        let ident = self.parse_ident()?;

        if self.eat_lt() {

            let params = self.parse_hkt_param_helper()?;

            self.expect_gt()?;

            Ok(GenericParam {
                ident,
                id: ast::DUMMY_NODE_ID,
                attrs: AttrVec::new(),
                bounds: GenericBounds::new(),
                kind: GenericParamKind::HKT(Generics {
                    params,
                    where_clause: Default::default(),
                    span: Default::default(), // FIXMIG: hoch fix spans
                }),
                is_placeholder: false,
                colon_span: None,
            })
        } else {
            Ok(GenericParam {
                ident,
                id: ast::DUMMY_NODE_ID,
                attrs: AttrVec::new(),
                bounds: GenericBounds::new(),
                kind: GenericParamKind::Type { default: None },
                is_placeholder: false,
                colon_span: None,
            })
        }
    }

    fn parse_hkt_param_helper(&mut self) -> PResult<'a, Vec<GenericParam>> {
        let mut inner = vec![];
        let mut done = false;

        while !done {
            if let Ok(params) = self.parse_hkt_param_helper1() {
                inner.push(params)
            } else {
                break;
            }

            if !self.eat(&token::Comma) {
                done = true;
            }
        }

        return Ok(inner)
    }

    /// Parsing a higher ranked generic, e.g. I<...>
    fn parse_hkt_param(&mut self, preceding_attrs: AttrVec) -> PResult<'a, GenericParam> {
        // Parse the outermost identifier
        let ident = self.parse_ident()?;

        // If we have another < after the ident, parse a list of hkt parameters followed by a
        // > with possible bounds : BOUND

        let mut kind = if self.eat_lt() {
            let p = self.parse_hkt_param_helper()?;

            self.expect_gt()?;

            GenericParamKind::HKT( Generics {
                params: p,
                where_clause: Default::default(),
                span: Default::default(), // FIXMIG: hoch fix spans
            })
        } else {
            GenericParamKind::Type { default: None }
        };

        let mut colon_span = None;

        let bounds = if self.eat(&token::Colon) {
            colon_span = Some(self.prev_token.span);
            // recover from `impl Trait` in type param bound
            if self.token.is_keyword(kw::Impl) {
                let impl_span = self.token.span;
                let snapshot = self.create_snapshot_for_diagnostic();
                match self.parse_ty() {
                    Ok(p) => {
                        if let TyKind::ImplTrait(_, bounds) = &(*p).kind {
                            let span = impl_span.to(self.token.span.shrink_to_lo());
                            let mut err = self.struct_span_err(
                                span,
                                "expected trait bound, found `impl Trait` type",
                            );
                            err.span_label(span, "not a trait");
                            if let [bound, ..] = &bounds[..] {
                                err.span_suggestion_verbose(
                                    impl_span.until(bound.span()),
                                    "use the trait bounds directly",
                                    String::new(),
                                    Applicability::MachineApplicable,
                                );
                            }
                            err.emit();
                            return Err(err);
                        }
                    }
                    Err(err) => {
                        err.cancel();
                    }
                }
                self.restore_snapshot(snapshot);
            }
            self.parse_generic_bounds(colon_span)?
        } else {
            Vec::new()
        };

        if let GenericParamKind::Type{ default } = &mut kind {
            *default = if self.eat(&token::Eq) { Some(self.parse_ty()?) } else { None };
        }

        Ok(GenericParam {
            ident,
            id: ast::DUMMY_NODE_ID,
            attrs: preceding_attrs,
            bounds,
            kind,
            is_placeholder: false,
            colon_span,
        })
    }

    /// Matches `typaram = IDENT (`?` unbound)? optbounds ( EQ ty )?`.
    #[allow(unused)]
    fn parse_ty_param(&mut self, preceding_attrs: AttrVec) -> PResult<'a, GenericParam> {
        let ident = self.parse_ident()?;

        // Parse optional colon and param bounds.
        let mut colon_span = None;
        let bounds = if self.eat(&token::Colon) {
            colon_span = Some(self.prev_token.span);
            // recover from `impl Trait` in type param bound
            if self.token.is_keyword(kw::Impl) {
                let impl_span = self.token.span;
                let snapshot = self.create_snapshot_for_diagnostic();
                match self.parse_ty() {
                    Ok(p) => {
                        if let TyKind::ImplTrait(_, bounds) = &(*p).kind {
                            let span = impl_span.to(self.token.span.shrink_to_lo());
                            let mut err = self.struct_span_err(
                                span,
                                "expected trait bound, found `impl Trait` type",
                            );
                            err.span_label(span, "not a trait");
                            if let [bound, ..] = &bounds[..] {
                                err.span_suggestion_verbose(
                                    impl_span.until(bound.span()),
                                    "use the trait bounds directly",
                                    String::new(),
                                    Applicability::MachineApplicable,
                                );
                            }
                            err.emit();
                            return Err(err);
                        }
                    }
                    Err(err) => {
                        err.cancel();
                    }
                }
                self.restore_snapshot(snapshot);
            }
            self.parse_generic_bounds(colon_span)?
        } else {
            Vec::new()
        };

        let default = if self.eat(&token::Eq) { Some(self.parse_ty()?) } else { None };
        Ok(GenericParam {
            ident,
            id: ast::DUMMY_NODE_ID,
            attrs: preceding_attrs,
            bounds,
            kind: GenericParamKind::Type { default },
            is_placeholder: false,
            colon_span,
        })
    }

    pub(crate) fn parse_const_param(
        &mut self,
        preceding_attrs: AttrVec,
    ) -> PResult<'a, GenericParam> {
        let const_span = self.token.span;

        self.expect_keyword(kw::Const)?;
        let ident = self.parse_ident()?;
        self.expect(&token::Colon)?;
        let ty = self.parse_ty()?;

        // Parse optional const generics default value.
        let default = if self.eat(&token::Eq) { Some(self.parse_const_arg()?) } else { None };

        Ok(GenericParam {
            ident,
            id: ast::DUMMY_NODE_ID,
            attrs: preceding_attrs,
            bounds: Vec::new(),
            kind: GenericParamKind::Const { ty, kw_span: const_span, default },
            is_placeholder: false,
            colon_span: None,
        })
    }

    /// Parses a (possibly empty) list of lifetime and type parameters, possibly including
    /// a trailing comma and erroneous trailing attributes.
    #[instrument(level = "debug", skip(self))]
    pub(super) fn parse_generic_params(&mut self) -> PResult<'a, Vec<GenericParam>> {
        let mut params = Vec::new();
        let mut done = false;

        while !done {
            let attrs = self.parse_outer_attributes()?;
            let param =
                self.collect_tokens_trailing_token(attrs, ForceCollect::No, |this, attrs| {
                    if this.eat_keyword_noexpect(kw::SelfUpper) {
                        // `Self` as a generic param is invalid. Here we emit the diagnostic and continue parsing
                        // as if `Self` never existed.
                        this.struct_span_err(
                            this.prev_token.span,
                            "unexpected keyword `Self` in generic parameters",
                        )
                        .note("you cannot use `Self` as a generic parameter because it is reserved for associated items")
                        .emit();

                        this.eat(&token::Comma);
                    }

                    let param = if this.check_lifetime() {
                        let lifetime = this.expect_lifetime();
                        // Parse lifetime parameter.
                        let (colon_span, bounds) = if this.eat(&token::Colon) {
                            (Some(this.prev_token.span), this.parse_lt_param_bounds())
                        } else {
                            (None, Vec::new())
                        };
                        Some(GenericParam {
                            ident: lifetime.ident,
                            id: lifetime.id,
                            attrs,
                            bounds,
                            kind: GenericParamKind::Lifetime,
                            is_placeholder: false,
                            colon_span,
                        })
                    } else if this.check_keyword(kw::Const) {
                        // Parse const parameter.
                        Some(this.parse_const_param(attrs)?)
                    } else if this.check_ident() {
                        // Parse type parameter.
                        Some(this.parse_hkt_param(attrs)?)
                    } else if this.token.can_begin_type() {
                        // Trying to write an associated type bound? (#26271)
                        let snapshot = this.create_snapshot_for_diagnostic();
                        match this.parse_ty_where_predicate() {
                            Ok(where_predicate) => {
                                this.struct_span_err(
                                    where_predicate.span(),
                                    "bounds on associated types do not belong here",
                                )
                                .span_label(where_predicate.span(), "belongs in `where` clause")
                                .emit();
                                // FIXME - try to continue parsing other generics?
                                return Ok((None, TrailingToken::None));
                            }
                            Err(err) => {
                                err.cancel();
                                // FIXME - maybe we should overwrite 'self' outside of `collect_tokens`?
                                this.restore_snapshot(snapshot);
                                return Ok((None, TrailingToken::None));
                            }
                        }
                    } else {
                        // Check for trailing attributes and stop parsing.
                        if !attrs.is_empty() {
                            if !params.is_empty() {
                                this.struct_span_err(
                                    attrs[0].span,
                                    "trailing attribute after generic parameter",
                                )
                                .span_label(attrs[0].span, "attributes must go before parameters")
                                .emit();
                            } else {
                                this.struct_span_err(
                                    attrs[0].span,
                                    "attribute without generic parameters",
                                )
                                .span_label(
                                    attrs[0].span,
                                    "attributes are only permitted when preceding parameters",
                                )
                                .emit();
                            }
                        }
                        return Ok((None, TrailingToken::None));
                    };

                    if !this.eat(&token::Comma) {
                        done = true;
                    }
                    // We just ate the comma, so no need to use `TrailingToken`
                    Ok((param, TrailingToken::None))
                })?;

            if let Some(param) = param {
                params.push(param);
            } else {
                break;
            }
        }

        use rustc_ast_pretty::pprust::PrintState;
        debug!("{}", rustc_ast_pretty::pprust::State::new().generic_params_to_string(&params));

        Ok(params)
    }

    /// Parses a set of optional generic type parameter declarations. Where
    /// clauses are not parsed here, and must be added later via
    /// `parse_where_clause()`.
    ///
    /// matches generics = ( ) | ( < > ) | ( < typaramseq ( , )? > ) | ( < lifetimes ( , )? > )
    ///                  | ( < lifetimes , typaramseq ( , )? > )
    /// where   typaramseq = ( typaram ) | ( typaram , typaramseq )
    #[instrument(level = "debug", skip(self))]
    pub(super) fn parse_generics(&mut self) -> PResult<'a, ast::Generics> {
        let span_lo = self.token.span;
        let (params, span) = if self.eat_lt() {
            let params = self.parse_generic_params()?;
            self.expect_gt()?;
            (params, span_lo.to(self.prev_token.span))
        } else {
            (vec![], self.prev_token.span.shrink_to_hi())
        };

        use rustc_ast_pretty::pprust::PrintState;
        debug!("Parsed: {}", rustc_ast_pretty::pprust::State::new().generic_params_to_string(&params));


        Ok(ast::Generics {
            params,
            where_clause: WhereClause {
                has_where_token: false,
                predicates: Vec::new(),
                span: self.prev_token.span.shrink_to_hi(),
            },
            span,
        })
    }

    /// Parses an optional where-clause and places it in `generics`.
    ///
    /// ```ignore (only-for-syntax-highlight)
    /// where T : Trait<U, V> + 'b, 'a : 'b
    /// ```
    pub(super) fn parse_where_clause(&mut self) -> PResult<'a, WhereClause> {
        let mut where_clause = WhereClause {
            has_where_token: false,
            predicates: Vec::new(),
            span: self.prev_token.span.shrink_to_hi(),
        };

        if !self.eat_keyword(kw::Where) {
            return Ok(where_clause);
        }
        where_clause.has_where_token = true;
        let lo = self.prev_token.span;

        // We are considering adding generics to the `where` keyword as an alternative higher-rank
        // parameter syntax (as in `where<'a>` or `where<T>`. To avoid that being a breaking
        // change we parse those generics now, but report an error.
        if self.choose_generics_over_qpath(0) {
            let generics = self.parse_generics()?;
            self.struct_span_err(
                generics.span,
                "generic parameters on `where` clauses are reserved for future use",
            )
            .span_label(generics.span, "currently unsupported")
            .emit();
        }

        loop {
            let lo = self.token.span;
            if self.check_lifetime() && self.look_ahead(1, |t| !t.is_like_plus()) {
                let lifetime = self.expect_lifetime();
                // Bounds starting with a colon are mandatory, but possibly empty.
                self.expect(&token::Colon)?;
                let bounds = self.parse_lt_param_bounds();
                where_clause.predicates.push(ast::WherePredicate::RegionPredicate(
                    ast::WhereRegionPredicate {
                        span: lo.to(self.prev_token.span),
                        lifetime,
                        bounds,
                    },
                ));
            } else if self.check_type() {
                where_clause.predicates.push(self.parse_ty_where_predicate()?);
            } else {
                break;
            }

            let prev_token = self.prev_token.span;
            let ate_comma = self.eat(&token::Comma);

            if self.eat_keyword_noexpect(kw::Where) {
                let msg = "cannot define duplicate `where` clauses on an item";
                let mut err = self.struct_span_err(self.token.span, msg);
                err.span_label(lo, "previous `where` clause starts here");
                err.span_suggestion_verbose(
                    prev_token.shrink_to_hi().to(self.prev_token.span),
                    "consider joining the two `where` clauses into one",
                    ",",
                    Applicability::MaybeIncorrect,
                );
                err.emit();
            } else if !ate_comma {
                break;
            }
        }

        where_clause.span = lo.to(self.prev_token.span);
        Ok(where_clause)
    }

    fn parse_ty_where_predicate(&mut self) -> PResult<'a, ast::WherePredicate> {
        let lo = self.token.span;
        // Parse optional `for<'a, 'b>`.
        // This `for` is parsed greedily and applies to the whole predicate,
        // the bounded type can have its own `for` applying only to it.
        // Examples:
        // * `for<'a> Trait1<'a>: Trait2<'a /* ok */>`
        // * `(for<'a> Trait1<'a>): Trait2<'a /* not ok */>`
        // * `for<'a> for<'b> Trait1<'a, 'b>: Trait2<'a /* ok */, 'b /* not ok */>`
        let lifetime_defs = self.parse_late_bound_lifetime_defs()?;

        // Parse type with mandatory colon and (possibly empty) bounds,
        // or with mandatory equality sign and the second type.
        let ty = self.parse_ty_for_where_clause()?;
        if self.eat(&token::Colon) {
            let bounds = self.parse_generic_bounds(Some(self.prev_token.span))?;
            Ok(ast::WherePredicate::BoundPredicate(ast::WhereBoundPredicate {
                span: lo.to(self.prev_token.span),
                bound_generic_params: lifetime_defs,
                bounded_ty: ty,
                bounds,
            }))
        // FIXME: Decide what should be used here, `=` or `==`.
        // FIXME: We are just dropping the binders in lifetime_defs on the floor here.
        } else if self.eat(&token::Eq) || self.eat(&token::EqEq) {
            let rhs_ty = self.parse_ty()?;
            Ok(ast::WherePredicate::EqPredicate(ast::WhereEqPredicate {
                span: lo.to(self.prev_token.span),
                lhs_ty: ty,
                rhs_ty,
            }))
        } else {
            self.maybe_recover_bounds_doubled_colon(&ty)?;
            self.unexpected()
        }
    }

    pub(super) fn choose_generics_over_qpath(&self, start: usize) -> bool {
        // There's an ambiguity between generic parameters and qualified paths in impls.
        // If we see `<` it may start both, so we have to inspect some following tokens.
        // The following combinations can only start generics,
        // but not qualified paths (with one exception):
        //     `<` `>` - empty generic parameters
        //     `<` `#` - generic parameters with attributes
        //     `<` (LIFETIME|IDENT) `>` - single generic parameter
        //     `<` (LIFETIME|IDENT) `,` - first generic parameter in a list
        //     `<` (LIFETIME|IDENT) `:` - generic parameter with bounds
        //     `<` (LIFETIME|IDENT) `=` - generic parameter with a default
        //     `<` const                - generic const parameter
        // The only truly ambiguous case is
        //     `<` IDENT `>` `::` IDENT ...
        // we disambiguate it in favor of generics (`impl<T> ::absolute::Path<T> { ... }`)
        // because this is what almost always expected in practice, qualified paths in impls
        // (`impl <Type>::AssocTy { ... }`) aren't even allowed by type checker at the moment.
        self.look_ahead(start, |t| t == &token::Lt)
            && (self.look_ahead(start + 1, |t| t == &token::Pound || t == &token::Gt)
                || self.look_ahead(start + 1, |t| t.is_lifetime() || t.is_ident())
                    && self.look_ahead(start + 2, |t| {
                        matches!(t.kind, token::Gt | token::Comma | token::Colon | token::Eq)
                    })
                || self.is_keyword_ahead(start + 1, &[kw::Const]))
    }
}
