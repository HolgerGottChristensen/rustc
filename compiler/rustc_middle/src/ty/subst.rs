// Type substitutions.

use crate::ty::codec::{TyDecoder, TyEncoder};
use crate::ty::fold::{FallibleTypeFolder, TypeFoldable, TypeFolder, TypeSuperFoldable};
use crate::ty::sty::{ClosureSubsts, GeneratorSubsts, InlineConstSubsts};
use crate::ty::visit::{TypeVisitable, TypeVisitor};
use crate::ty::{self, ArgumentDef, Const, ConstKind, Generics, Lift, List, ParamConst, ParamTy, Region, RegionKind, Ty, TyCtxt, TyKind};

use rustc_data_structures::intern::Interned;
use rustc_hir::def_id::DefId;
use rustc_macros::HashStable;
use rustc_serialize::{self, Decodable, Encodable};
use rustc_type_ir::WithCachedTypeInfo;
use smallvec::SmallVec;

use core::intrinsics;
use std::cmp::Ordering;
use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::num::NonZeroUsize;
use std::ops::{ControlFlow, Deref};
use std::slice;
use rustc_data_structures::fx::FxHashMap;
use rustc_middle::ty::PolyFnSig;
use rustc_span::Symbol;

/// An entity in the Rust type system, which can be one of
/// several kinds (types, lifetimes, and consts).
/// To reduce memory usage, a `GenericArg` is an interned pointer,
/// with the lowest 2 bits being reserved for a tag to
/// indicate the type (`Ty`, `Region`, or `Const`) it points to.
///
/// Note: the `PartialEq`, `Eq` and `Hash` derives are only valid because `Ty`,
/// `Region` and `Const` are all interned.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct GenericArg<'tcx> {
    ptr: NonZeroUsize,
    marker: PhantomData<(Ty<'tcx>, ty::Region<'tcx>, ty::Const<'tcx>)>,
}

const TAG_MASK: usize = 0b11;
const TYPE_TAG: usize = 0b00;
const REGION_TAG: usize = 0b01;
const CONST_TAG: usize = 0b10;

#[derive(Debug, TyEncodable, TyDecodable, PartialEq, Eq, PartialOrd, Ord)]
pub enum GenericArgKind<'tcx> {
    Lifetime(ty::Region<'tcx>),
    Type(Ty<'tcx>),
    Const(ty::Const<'tcx>),
}

/// This function goes from `&'a [Ty<'tcx>]` to `&'a [GenericArg<'tcx>]`
///
/// This is sound as, for types, `GenericArg` is just
/// `NonZeroUsize::new_unchecked(ty as *const _ as usize)` as
/// long as we use `0` for the `TYPE_TAG`.
pub fn ty_slice_as_generic_args<'a, 'tcx>(ts: &'a [Ty<'tcx>]) -> &'a [GenericArg<'tcx>] {
    assert_eq!(TYPE_TAG, 0);
    // SAFETY: the whole slice is valid and immutable.
    // `Ty` and `GenericArg` is explained above.
    unsafe { slice::from_raw_parts(ts.as_ptr().cast(), ts.len()) }
}

impl<'tcx> List<Ty<'tcx>> {
    /// Allows to freely switch between `List<Ty<'tcx>>` and `List<GenericArg<'tcx>>`.
    ///
    /// As lists are interned, `List<Ty<'tcx>>` and `List<GenericArg<'tcx>>` have
    /// be interned together, see `intern_type_list` for more details.
    #[inline]
    pub fn as_substs(&'tcx self) -> SubstsRef<'tcx> {
        assert_eq!(TYPE_TAG, 0);
        // SAFETY: `List<T>` is `#[repr(C)]`. `Ty` and `GenericArg` is explained above.
        unsafe { &*(self as *const List<Ty<'tcx>> as *const List<GenericArg<'tcx>>) }
    }
}

impl<'tcx> GenericArgKind<'tcx> {
    #[inline]
    fn pack(self) -> GenericArg<'tcx> {
        let (tag, ptr) = match self {
            GenericArgKind::Lifetime(lt) => {
                // Ensure we can use the tag bits.
                assert_eq!(mem::align_of_val(&*lt.0.0) & TAG_MASK, 0);
                (REGION_TAG, lt.0.0 as *const ty::RegionKind<'tcx> as usize)
            }
            GenericArgKind::Type(ty) => {
                // Ensure we can use the tag bits.
                assert_eq!(mem::align_of_val(&*ty.0.0) & TAG_MASK, 0);
                (TYPE_TAG, ty.0.0 as *const WithCachedTypeInfo<ty::TyKind<'tcx>> as usize)
            }
            GenericArgKind::Const(ct) => {
                // Ensure we can use the tag bits.
                assert_eq!(mem::align_of_val(&*ct.0.0) & TAG_MASK, 0);
                (CONST_TAG, ct.0.0 as *const ty::ConstData<'tcx> as usize)
            }
        };

        GenericArg { ptr: unsafe { NonZeroUsize::new_unchecked(ptr | tag) }, marker: PhantomData }
    }
}

impl<'tcx> fmt::Debug for GenericArg<'tcx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.unpack() {
            GenericArgKind::Lifetime(lt) => lt.fmt(f),
            GenericArgKind::Type(ty) => ty.kind().fmt(f),
            GenericArgKind::Const(ct) => ct.fmt(f),
        }
    }
}

impl<'tcx> Ord for GenericArg<'tcx> {
    fn cmp(&self, other: &GenericArg<'tcx>) -> Ordering {
        self.unpack().cmp(&other.unpack())
    }
}

impl<'tcx> PartialOrd for GenericArg<'tcx> {
    fn partial_cmp(&self, other: &GenericArg<'tcx>) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl<'tcx> From<ty::Region<'tcx>> for GenericArg<'tcx> {
    #[inline]
    fn from(r: ty::Region<'tcx>) -> GenericArg<'tcx> {
        GenericArgKind::Lifetime(r).pack()
    }
}

impl<'tcx> From<Ty<'tcx>> for GenericArg<'tcx> {
    #[inline]
    fn from(ty: Ty<'tcx>) -> GenericArg<'tcx> {
        GenericArgKind::Type(ty).pack()
    }
}

impl<'tcx> From<ty::Const<'tcx>> for GenericArg<'tcx> {
    #[inline]
    fn from(c: ty::Const<'tcx>) -> GenericArg<'tcx> {
        GenericArgKind::Const(c).pack()
    }
}

impl<'tcx> From<ty::Term<'tcx>> for GenericArg<'tcx> {
    fn from(value: ty::Term<'tcx>) -> Self {
        match value.unpack() {
            ty::TermKind::Ty(t) => t.into(),
            ty::TermKind::Const(c) => c.into(),
        }
    }
}

impl<'tcx> GenericArg<'tcx> {
    #[inline]
    pub fn unpack(self) -> GenericArgKind<'tcx> {
        let ptr = self.ptr.get();
        // SAFETY: use of `Interned::new_unchecked` here is ok because these
        // pointers were originally created from `Interned` types in `pack()`,
        // and this is just going in the other direction.
        unsafe {
            match ptr & TAG_MASK {
                REGION_TAG => GenericArgKind::Lifetime(ty::Region(Interned::new_unchecked(
                    &*((ptr & !TAG_MASK) as *const ty::RegionKind<'tcx>),
                ))),
                TYPE_TAG => GenericArgKind::Type(Ty(Interned::new_unchecked(
                    &*((ptr & !TAG_MASK) as *const WithCachedTypeInfo<ty::TyKind<'tcx>>),
                ))),
                CONST_TAG => GenericArgKind::Const(ty::Const(Interned::new_unchecked(
                    &*((ptr & !TAG_MASK) as *const ty::ConstData<'tcx>),
                ))),
                _ => intrinsics::unreachable(),
            }
        }
    }

    /// Unpack the `GenericArg` as a region when it is known certainly to be a region.
    pub fn expect_region(self) -> ty::Region<'tcx> {
        match self.unpack() {
            GenericArgKind::Lifetime(lt) => lt,
            _ => bug!("expected a region, but found another kind"),
        }
    }

    /// Unpack the `GenericArg` as a type when it is known certainly to be a type.
    /// This is true in cases where `Substs` is used in places where the kinds are known
    /// to be limited (e.g. in tuples, where the only parameters are type parameters).
    pub fn expect_ty(self) -> Ty<'tcx> {
        match self.unpack() {
            GenericArgKind::Type(ty) => ty,
            v => bug!("expected a type, but found another kind ({:?})", v),
        }
    }

    /// Unpack the `GenericArg` as a const when it is known certainly to be a const.
    pub fn expect_const(self) -> ty::Const<'tcx> {
        match self.unpack() {
            GenericArgKind::Const(c) => c,
            _ => bug!("expected a const, but found another kind"),
        }
    }

    pub fn is_non_region_infer(self) -> bool {
        match self.unpack() {
            GenericArgKind::Lifetime(_) => false,
            GenericArgKind::Type(ty) => ty.is_ty_infer(),
            GenericArgKind::Const(ct) => ct.is_ct_infer(),
        }
    }
}

impl<'a, 'tcx> Lift<'tcx> for GenericArg<'a> {
    type Lifted = GenericArg<'tcx>;

    fn lift_to_tcx(self, tcx: TyCtxt<'tcx>) -> Option<Self::Lifted> {
        match self.unpack() {
            GenericArgKind::Lifetime(lt) => tcx.lift(lt).map(|lt| lt.into()),
            GenericArgKind::Type(ty) => tcx.lift(ty).map(|ty| ty.into()),
            GenericArgKind::Const(ct) => tcx.lift(ct).map(|ct| ct.into()),
        }
    }
}

impl<'tcx> TypeFoldable<'tcx> for GenericArg<'tcx> {
    fn try_fold_with<F: FallibleTypeFolder<'tcx>>(self, folder: &mut F) -> Result<Self, F::Error> {
        match self.unpack() {
            GenericArgKind::Lifetime(lt) => lt.try_fold_with(folder).map(Into::into),
            GenericArgKind::Type(ty) => ty.try_fold_with(folder).map(Into::into),
            GenericArgKind::Const(ct) => ct.try_fold_with(folder).map(Into::into),
        }
    }
}

impl<'tcx> TypeVisitable<'tcx> for GenericArg<'tcx> {
    fn visit_with<V: TypeVisitor<'tcx>>(&self, visitor: &mut V) -> ControlFlow<V::BreakTy> {
        match self.unpack() {
            GenericArgKind::Lifetime(lt) => lt.visit_with(visitor),
            GenericArgKind::Type(ty) => ty.visit_with(visitor),
            GenericArgKind::Const(ct) => ct.visit_with(visitor),
        }
    }
}

impl<'tcx, E: TyEncoder<I = TyCtxt<'tcx>>> Encodable<E> for GenericArg<'tcx> {
    fn encode(&self, e: &mut E) {
        self.unpack().encode(e)
    }
}

impl<'tcx, D: TyDecoder<I = TyCtxt<'tcx>>> Decodable<D> for GenericArg<'tcx> {
    fn decode(d: &mut D) -> GenericArg<'tcx> {
        GenericArgKind::decode(d).pack()
    }
}

/// List of generic arguments that are gonna be used to substitute generic parameters.
pub type InternalSubsts<'tcx> = List<GenericArg<'tcx>>;
pub type InternalHKTParameters<'tcx> = List<Symbol>;

pub type SubstsRef<'tcx> = &'tcx InternalSubsts<'tcx>;
pub type HKTParametersRef<'tcx> = &'tcx InternalHKTParameters<'tcx>;

impl<'tcx> InternalSubsts<'tcx> {
    /// Checks whether all elements of this list are types, if so, transmute.
    pub fn try_as_type_list(&'tcx self) -> Option<&'tcx List<Ty<'tcx>>> {
        if self.iter().all(|arg| matches!(arg.unpack(), GenericArgKind::Type(_))) {
            assert_eq!(TYPE_TAG, 0);
            // SAFETY: All elements are types, see `List<Ty<'tcx>>::as_substs`.
            Some(unsafe { &*(self as *const List<GenericArg<'tcx>> as *const List<Ty<'tcx>>) })
        } else {
            None
        }
    }

    /// Interpret these substitutions as the substitutions of a closure type.
    /// Closure substitutions have a particular structure controlled by the
    /// compiler that encodes information like the signature and closure kind;
    /// see `ty::ClosureSubsts` struct for more comments.
    pub fn as_closure(&'tcx self) -> ClosureSubsts<'tcx> {
        ClosureSubsts { substs: self }
    }

    /// Interpret these substitutions as the substitutions of a generator type.
    /// Generator substitutions have a particular structure controlled by the
    /// compiler that encodes information like the signature and generator kind;
    /// see `ty::GeneratorSubsts` struct for more comments.
    pub fn as_generator(&'tcx self) -> GeneratorSubsts<'tcx> {
        GeneratorSubsts { substs: self }
    }

    /// Interpret these substitutions as the substitutions of an inline const.
    /// Inline const substitutions have a particular structure controlled by the
    /// compiler that encodes information like the inferred type;
    /// see `ty::InlineConstSubsts` struct for more comments.
    pub fn as_inline_const(&'tcx self) -> InlineConstSubsts<'tcx> {
        InlineConstSubsts { substs: self }
    }

    /// Creates an `InternalSubsts` that maps each generic parameter to itself.
    pub fn identity_for_item(tcx: TyCtxt<'tcx>, def_id: DefId) -> SubstsRef<'tcx> {
        Self::for_item(tcx, def_id, |param, _| tcx.mk_param_from_def(param))
    }

    /// Creates an `InternalSubsts` for generic parameter definitions,
    /// by calling closures to obtain each kind.
    /// The closures get to observe the `InternalSubsts` as they're
    /// being built, which can be used to correctly
    /// substitute defaults of generic parameters.
    pub fn for_item<F>(tcx: TyCtxt<'tcx>, def_id: DefId, mut mk_kind: F) -> SubstsRef<'tcx>
    where
        F: FnMut(&ty::GenericParamDef, &[GenericArg<'tcx>]) -> GenericArg<'tcx>,
    {
        let defs = tcx.generics_of(def_id);
        let count = defs.count();
        let mut substs = SmallVec::with_capacity(count);
        Self::fill_item(&mut substs, tcx, defs, &mut mk_kind);
        tcx.intern_substs(&substs)
    }

    pub fn extend_to<F>(&self, tcx: TyCtxt<'tcx>, def_id: DefId, mut mk_kind: F) -> SubstsRef<'tcx>
    where
        F: FnMut(&ty::GenericParamDef, &[GenericArg<'tcx>]) -> GenericArg<'tcx>,
    {
        Self::for_item(tcx, def_id, |param, substs| {
            self.get(param.index as usize).cloned().unwrap_or_else(|| mk_kind(param, substs))
        })
    }

    pub fn fill_item<F>(
        substs: &mut SmallVec<[GenericArg<'tcx>; 8]>,
        tcx: TyCtxt<'tcx>,
        defs: &ty::Generics,
        mk_kind: &mut F,
    ) where
        F: FnMut(&ty::GenericParamDef, &[GenericArg<'tcx>]) -> GenericArg<'tcx>,
    {
        if let Some(def_id) = defs.parent {
            let parent_defs = tcx.generics_of(def_id);
            Self::fill_item(substs, tcx, parent_defs, mk_kind);
        }
        Self::fill_single(substs, defs, mk_kind)
    }

    pub fn fill_single<F>(
        substs: &mut SmallVec<[GenericArg<'tcx>; 8]>,
        defs: &ty::Generics,
        mk_kind: &mut F,
    ) where
        F: FnMut(&ty::GenericParamDef, &[GenericArg<'tcx>]) -> GenericArg<'tcx>,
    {
        substs.reserve(defs.params.len());
        for param in &defs.params {
            let kind = mk_kind(param, substs);
            assert_eq!(param.index as usize, substs.len(), "{substs:#?}, {defs:#?}");
            substs.push(kind);
        }
    }

    // Extend an `original_substs` list to the full number of substs expected by `def_id`,
    // filling in the missing parameters with error ty/ct or 'static regions.
    pub fn extend_with_error(
        tcx: TyCtxt<'tcx>,
        def_id: DefId,
        original_substs: &[GenericArg<'tcx>],
    ) -> SubstsRef<'tcx> {
        ty::InternalSubsts::for_item(tcx, def_id, |def, substs| {
            if let Some(subst) = original_substs.get(def.index as usize) {
                *subst
            } else {
                def.to_error(tcx, substs)
            }
        })
    }

    #[inline]
    pub fn types(&'tcx self) -> impl DoubleEndedIterator<Item = Ty<'tcx>> + 'tcx {
        self.iter()
            .filter_map(|k| if let GenericArgKind::Type(ty) = k.unpack() { Some(ty) } else { None })
    }

    #[inline]
    pub fn regions(&'tcx self) -> impl DoubleEndedIterator<Item = ty::Region<'tcx>> + 'tcx {
        self.iter().filter_map(|k| {
            if let GenericArgKind::Lifetime(lt) = k.unpack() { Some(lt) } else { None }
        })
    }

    #[inline]
    pub fn consts(&'tcx self) -> impl DoubleEndedIterator<Item = ty::Const<'tcx>> + 'tcx {
        self.iter().filter_map(|k| {
            if let GenericArgKind::Const(ct) = k.unpack() { Some(ct) } else { None }
        })
    }

    #[inline]
    pub fn non_erasable_generics(
        &'tcx self,
    ) -> impl DoubleEndedIterator<Item = GenericArgKind<'tcx>> + 'tcx {
        self.iter().filter_map(|k| match k.unpack() {
            GenericArgKind::Lifetime(_) => None,
            generic => Some(generic),
        })
    }

    #[inline]
    #[track_caller]
    pub fn type_at(&self, i: usize) -> Ty<'tcx> {
        if let GenericArgKind::Type(ty) = self[i].unpack() {
            ty
        } else {
            bug!("expected type for param #{} in {:?}", i, self);
        }
    }

    #[inline]
    #[track_caller]
    pub fn region_at(&self, i: usize) -> ty::Region<'tcx> {
        if let GenericArgKind::Lifetime(lt) = self[i].unpack() {
            lt
        } else {
            bug!("expected region for param #{} in {:?}", i, self);
        }
    }

    #[inline]
    #[track_caller]
    pub fn const_at(&self, i: usize) -> ty::Const<'tcx> {
        if let GenericArgKind::Const(ct) = self[i].unpack() {
            ct
        } else {
            bug!("expected const for param #{} in {:?}", i, self);
        }
    }

    #[inline]
    #[track_caller]
    pub fn type_for_def(&self, def: &ty::GenericParamDef) -> GenericArg<'tcx> {
        self.type_at(def.index as usize).into()
    }

    /// Transform from substitutions for a child of `source_ancestor`
    /// (e.g., a trait or impl) to substitutions for the same child
    /// in a different item, with `target_substs` as the base for
    /// the target impl/trait, with the source child-specific
    /// parameters (e.g., method parameters) on top of that base.
    ///
    /// For example given:
    ///
    /// ```no_run
    /// trait X<S> { fn f<T>(); }
    /// impl<U> X<U> for U { fn f<V>() {} }
    /// ```
    ///
    /// * If `self` is `[Self, S, T]`: the identity substs of `f` in the trait.
    /// * If `source_ancestor` is the def_id of the trait.
    /// * If `target_substs` is `[U]`, the substs for the impl.
    /// * Then we will return `[U, T]`, the subst for `f` in the impl that
    ///   are needed for it to match the trait.
    pub fn rebase_onto(
        &self,
        tcx: TyCtxt<'tcx>,
        source_ancestor: DefId,
        target_substs: SubstsRef<'tcx>,
    ) -> SubstsRef<'tcx> {
        let defs = tcx.generics_of(source_ancestor);
        tcx.mk_substs(target_substs.iter().chain(self.iter().skip(defs.params.len())))
    }

    pub fn truncate_to(&self, tcx: TyCtxt<'tcx>, generics: &ty::Generics) -> SubstsRef<'tcx> {
        tcx.mk_substs(self.iter().take(generics.count()))
    }
}

impl<'tcx> TypeFoldable<'tcx> for SubstsRef<'tcx> {
    fn try_fold_with<F: FallibleTypeFolder<'tcx>>(self, folder: &mut F) -> Result<Self, F::Error> {
        // This code is hot enough that it's worth specializing for the most
        // common length lists, to avoid the overhead of `SmallVec` creation.
        // The match arms are in order of frequency. The 1, 2, and 0 cases are
        // typically hit in 90--99.99% of cases. When folding doesn't change
        // the substs, it's faster to reuse the existing substs rather than
        // calling `intern_substs`.
        match self.len() {
            1 => {
                let param0 = self[0].try_fold_with(folder)?;
                if param0 == self[0] { Ok(self) } else { Ok(folder.tcx().intern_substs(&[param0])) }
            }
            2 => {
                let param0 = self[0].try_fold_with(folder)?;
                let param1 = self[1].try_fold_with(folder)?;
                if param0 == self[0] && param1 == self[1] {
                    Ok(self)
                } else {
                    Ok(folder.tcx().intern_substs(&[param0, param1]))
                }
            }
            0 => Ok(self),
            _ => ty::util::fold_list(self, folder, |tcx, v| tcx.intern_substs(v)),
        }
    }
}

impl<'tcx> TypeFoldable<'tcx> for &'tcx ty::List<Ty<'tcx>> {
    fn try_fold_with<F: FallibleTypeFolder<'tcx>>(self, folder: &mut F) -> Result<Self, F::Error> {
        // This code is fairly hot, though not as hot as `SubstsRef`.
        //
        // When compiling stage 2, I get the following results:
        //
        // len |   total   |   %
        // --- | --------- | -----
        //  2  |  15083590 |  48.1
        //  3  |   7540067 |  24.0
        //  1  |   5300377 |  16.9
        //  4  |   1351897 |   4.3
        //  0  |   1256849 |   4.0
        //
        // I've tried it with some private repositories and got
        // close to the same result, with 4 and 0 swapping places
        // sometimes.
        match self.len() {
            2 => {
                let param0 = self[0].try_fold_with(folder)?;
                let param1 = self[1].try_fold_with(folder)?;
                if param0 == self[0] && param1 == self[1] {
                    Ok(self)
                } else {
                    Ok(folder.tcx().intern_type_list(&[param0, param1]))
                }
            }
            _ => ty::util::fold_list(self, folder, |tcx, v| tcx.intern_type_list(v)),
        }
    }
}

impl<'tcx, T: TypeVisitable<'tcx>> TypeVisitable<'tcx> for &'tcx ty::List<T> {
    #[inline]
    fn visit_with<V: TypeVisitor<'tcx>>(&self, visitor: &mut V) -> ControlFlow<V::BreakTy> {
        self.iter().try_for_each(|t| t.visit_with(visitor))
    }
}

/// Similar to [`super::Binder`] except that it tracks early bound generics, i.e. `struct Foo<T>(T)`
/// needs `T` substituted immediately. This type primarily exists to avoid forgetting to call
/// `subst`.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[derive(Encodable, Decodable, HashStable)]
pub struct EarlyBinder<T>(pub T);

/// For early binders, you should first call `subst` before using any visitors.
impl<'tcx, T> !TypeFoldable<'tcx> for ty::EarlyBinder<T> {}
impl<'tcx, T> !TypeVisitable<'tcx> for ty::EarlyBinder<T> {}

impl<T> EarlyBinder<T> {
    pub fn as_ref(&self) -> EarlyBinder<&T> {
        EarlyBinder(&self.0)
    }

    pub fn map_bound_ref<F, U>(&self, f: F) -> EarlyBinder<U>
    where
        F: FnOnce(&T) -> U,
    {
        self.as_ref().map_bound(f)
    }

    pub fn map_bound<F, U>(self, f: F) -> EarlyBinder<U>
    where
        F: FnOnce(T) -> U,
    {
        let value = f(self.0);
        EarlyBinder(value)
    }

    pub fn try_map_bound<F, U, E>(self, f: F) -> Result<EarlyBinder<U>, E>
    where
        F: FnOnce(T) -> Result<U, E>,
    {
        let value = f(self.0)?;
        Ok(EarlyBinder(value))
    }

    pub fn rebind<U>(&self, value: U) -> EarlyBinder<U> {
        EarlyBinder(value)
    }

    pub fn skip_binder(self) -> T {
        self.0
    }
}

impl<T> EarlyBinder<Option<T>> {
    pub fn transpose(self) -> Option<EarlyBinder<T>> {
        self.0.map(|v| EarlyBinder(v))
    }
}

impl<T, U> EarlyBinder<(T, U)> {
    pub fn transpose_tuple2(self) -> (EarlyBinder<T>, EarlyBinder<U>) {
        (EarlyBinder(self.0.0), EarlyBinder(self.0.1))
    }
}

impl<'tcx, 's, I: IntoIterator> EarlyBinder<I>
where
    I::Item: TypeFoldable<'tcx>,
{
    pub fn subst_iter(
        self,
        tcx: TyCtxt<'tcx>,
        substs: &'s [GenericArg<'tcx>],
    ) -> SubstIter<'s, 'tcx, I> {
        SubstIter { it: self.0.into_iter(), tcx, substs }
    }
}

pub struct SubstIter<'s, 'tcx, I: IntoIterator> {
    it: I::IntoIter,
    tcx: TyCtxt<'tcx>,
    substs: &'s [GenericArg<'tcx>],
}

impl<'tcx, I: IntoIterator> Iterator for SubstIter<'_, 'tcx, I>
where
    I::Item: TypeFoldable<'tcx>,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        Some(EarlyBinder(self.it.next()?).subst(self.tcx, self.substs, HKTSubstType::SubstHKTParamWithType))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.it.size_hint()
    }
}

impl<'tcx, I: IntoIterator> DoubleEndedIterator for SubstIter<'_, 'tcx, I>
where
    I::IntoIter: DoubleEndedIterator,
    I::Item: TypeFoldable<'tcx>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        Some(EarlyBinder(self.it.next_back()?).subst(self.tcx, self.substs, HKTSubstType::SubstHKTParamWithType))
    }
}

impl<'tcx, 's, I: IntoIterator> EarlyBinder<I>
where
    I::Item: Deref,
    <I::Item as Deref>::Target: Copy + TypeFoldable<'tcx>,
{
    pub fn subst_iter_copied(
        self,
        tcx: TyCtxt<'tcx>,
        substs: &'s [GenericArg<'tcx>],
    ) -> SubstIterCopied<'s, 'tcx, I> {
        SubstIterCopied { it: self.0.into_iter(), tcx, substs }
    }
}

pub struct SubstIterCopied<'a, 'tcx, I: IntoIterator> {
    it: I::IntoIter,
    tcx: TyCtxt<'tcx>,
    substs: &'a [GenericArg<'tcx>],
}

impl<'tcx, I: IntoIterator> Iterator for SubstIterCopied<'_, 'tcx, I>
where
    I::Item: Deref,
    <I::Item as Deref>::Target: Copy + TypeFoldable<'tcx>,
{
    type Item = <I::Item as Deref>::Target;

    fn next(&mut self) -> Option<Self::Item> {
        Some(EarlyBinder(*self.it.next()?).subst(self.tcx, self.substs, HKTSubstType::SubstHKTParamWithType))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.it.size_hint()
    }
}

impl<'tcx, I: IntoIterator> DoubleEndedIterator for SubstIterCopied<'_, 'tcx, I>
where
    I::IntoIter: DoubleEndedIterator,
    I::Item: Deref,
    <I::Item as Deref>::Target: Copy + TypeFoldable<'tcx>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        Some(EarlyBinder(*self.it.next_back()?).subst(self.tcx, self.substs, HKTSubstType::SubstHKTParamWithType))
    }
}

pub struct EarlyBinderIter<T> {
    t: T,
}

impl<T: IntoIterator> EarlyBinder<T> {
    pub fn transpose_iter(self) -> EarlyBinderIter<T::IntoIter> {
        EarlyBinderIter { t: self.0.into_iter() }
    }
}

impl<T: Iterator> Iterator for EarlyBinderIter<T> {
    type Item = EarlyBinder<T::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        self.t.next().map(|i| EarlyBinder(i))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.t.size_hint()
    }
}

#[derive(Debug, PartialEq)]
pub enum HKTSubstType<'a, 'tcx> {
    /// Substitute `I<u32>` `[Option<%J>]` to `Option<u32>`
    /// Substitute `A<u32>` `[I<%B>]` to `I<u32>`
    /// Substitute `G<%P>` `[H<%Q>]` to `H<%Q>` when %P is defined in G
    /// Substitute `G<%K>` `[H<%Q>]` to `H<%K>` when %K is not defined in G
    /// Substitute `A, I<A>` `[u32, Option<%J>]` to `u32, Option<u32>`
    /// Substitute `A, I<A>` `[u32, Result<%J, E>]` to `u32, Result<u32, E>`
    SubstHKTParamWithType,
    /// Substitute `I<%J>` `[u32]` to `I<u32>`
    SubstArgumentWithinHKTParam,
    /// Substitute `I<%J>` `[u32]` to `I<u32>`
    /// But if we have arguments that do not correspond to the DefId we do not substitute them
    SubstArgumentWithinHKTParamWithDefId(DefId),

    InstantiateHKT(&'a mut FxHashMap<Ty<'tcx>, Ty<'tcx>>)
}

pub struct OffsetterFolder<'tcx>(pub u32, pub TyCtxt<'tcx>);

impl<'tcx> TypeFolder<'tcx> for OffsetterFolder<'tcx> {
    fn tcx<'a>(&'a self) -> TyCtxt<'tcx> {
        self.1
    }

    fn fold_ty(&mut self, t: Ty<'tcx>) -> Ty<'tcx> {
        match t.kind() {
            ty::Param(ParamTy::Param { index, name }) => {
                return self.1.mk_ty_param(*index + self.0, *name);
            }
            ty::HKT(_, ParamTy::HKT { def_id, index, name }, substs_ref) => {
                return self.1.mk_hkt_param(*def_id, *index + self.0, *name, substs_ref).super_fold_with(self);
            }
            _ => ()
        }

        t.super_fold_with(self)
    }

    fn fold_const(&mut self, c: Const<'tcx>) -> Const<'tcx> {
        match c.kind() {
            ConstKind::Param(p) => {
                return self.1.mk_const(ConstKind::Param(ParamConst::new(p.index + self.0, p.name)), c.ty());
            }
            _ => ()
        }

        c.super_fold_with(self)
    }

    fn fold_region(&mut self, r: Region<'tcx>) -> Region<'tcx> {
        match r.kind() {
            RegionKind::ReEarlyBound(data) => {
                return self.1.mk_region(RegionKind::ReEarlyBound(ty::EarlyBoundRegion {
                    def_id: data.def_id,
                    index: data.index + self.0,
                    name: data.name,
                }));
            }
            _ => ()
        }

        r.super_fold_with(self)
    }
}

pub struct ReplacerFolder<'tcx> {
    replace: Ty<'tcx>,
    with: Ty<'tcx>,
    tcx: TyCtxt<'tcx>
}

#[allow(dead_code)]
impl<'tcx> ReplacerFolder<'tcx> {
    pub fn new(replace: Ty<'tcx>, with: Ty<'tcx>, tcx: TyCtxt<'tcx>) -> Self {

        ReplacerFolder {
            replace,
            with,
            tcx,
        }
    }
}

impl<'tcx> TypeFolder<'tcx> for ReplacerFolder<'tcx> {
    fn tcx<'a>(&'a self) -> TyCtxt<'tcx> {
        self.tcx
    }

    fn fold_ty(&mut self, t: Ty<'tcx>) -> Ty<'tcx> {
        info!("Replace: {:?} with {:?} in {:?}", self.replace, self.with, t);
        if t == self.replace {
            return self.with
        }

        t.super_fold_with(self)
    }
}

impl<'tcx, T: TypeFoldable<'tcx>> ty::EarlyBinder<T> {
    pub fn subst<'a>(self, tcx: TyCtxt<'tcx>, substs: &[GenericArg<'tcx>], hkt_subst_type: HKTSubstType<'a, 'tcx>) -> T {
        let mut folder = SubstFolder { tcx, substs, binders_passed: 0, hkt_subst_type };
        let res = self.clone().0.fold_with(&mut folder);
        //info!("\nSUBST:\n\t{:?}\ngets subst with:\n\t{:?}\nsubsts: {:#?}", self.skip_binder(), res, substs);
        res
    }
}

///////////////////////////////////////////////////////////////////////////
// The actual substitution engine itself is a type folder.

struct SubstFolder<'a, 'tcx> {
    tcx: TyCtxt<'tcx>,
    substs: &'a [GenericArg<'tcx>],

    /// Number of region binders we have passed through while doing the substitution
    binders_passed: u32,

    #[allow(unused)]
    hkt_subst_type: HKTSubstType<'a, 'tcx>,
}

impl<'a, 'tcx> TypeFolder<'tcx> for SubstFolder<'a, 'tcx> {
    #[inline]
    fn tcx<'b>(&'b self) -> TyCtxt<'tcx> {
        self.tcx
    }

    fn fold_binder<T: TypeFoldable<'tcx>>(
        &mut self,
        t: ty::Binder<'tcx, T>,
    ) -> ty::Binder<'tcx, T> {
        self.binders_passed += 1;
        let t = t.super_fold_with(self);
        self.binders_passed -= 1;
        t
    }

    fn fold_region(&mut self, r: ty::Region<'tcx>) -> ty::Region<'tcx> {
        #[cold]
        #[inline(never)]
        fn region_param_out_of_range(data: ty::EarlyBoundRegion, substs: &[GenericArg<'_>]) -> ! {
            bug!(
                "Region parameter out of range when substituting in region {} (index={}, substs = {:?})",
                data.name,
                data.index,
                substs,
            )
        }

        #[cold]
        #[inline(never)]
        fn region_param_invalid(data: ty::EarlyBoundRegion, other: GenericArgKind<'_>) -> ! {
            bug!(
                "Unexpected parameter {:?} when substituting in region {} (index={})",
                other,
                data.name,
                data.index
            )
        }

        // Note: This routine only handles regions that are bound on
        // type declarations and other outer declarations, not those
        // bound in *fn types*. Region substitution of the bound
        // regions that appear in a function signature is done using
        // the specialized routine `ty::replace_late_regions()`.
        match *r {
            ty::ReEarlyBound(data) => {
                let rk = self.substs.get(data.index as usize).map(|k| k.unpack());
                match rk {
                    Some(GenericArgKind::Lifetime(lt)) => self.shift_region_through_binders(lt),
                    Some(other) => region_param_invalid(data, other),
                    None => region_param_out_of_range(data, self.substs),
                }
            }
            _ => r,
        }
    }

    #[instrument(level="info", skip(self))]
    fn fold_ty(&mut self, t: Ty<'tcx>) -> Ty<'tcx> {
        if !t.needs_subst() && !matches!(t.kind(), ty::HKT(..)) {
            info!("just return: {}", t);
            return t;
        }

        //info!("fold type: {}", t);

        let res = match *t.kind() {
            ty::Param(p) => self.ty_for_param(p, t),
            ty::HKT(_, p, substs) if self.hkt_subst_type == HKTSubstType::SubstHKTParamWithType =>
                self.hkt_for_param(p, substs, t),
            ty::HKT(_, p, substs) if matches!(self.hkt_subst_type, HKTSubstType::InstantiateHKT(..)) =>
                self.instantiate_hkt(p, substs, t),
            ty::Argument(arg_def) => {
                match self.hkt_subst_type {
                    HKTSubstType::SubstHKTParamWithType => {
                        t.super_fold_with(self)
                    }
                    HKTSubstType::SubstArgumentWithinHKTParam => {
                        self.ty_for_argument(arg_def, t)
                    }
                    HKTSubstType::SubstArgumentWithinHKTParamWithDefId(did) => {
                        if arg_def.def_id == did {
                            self.ty_for_argument(arg_def, t)
                        } else {
                            t.super_fold_with(self)
                        }
                    }
                    HKTSubstType::InstantiateHKT(..) => {
                        t.super_fold_with(self)
                    }
                }
            }
            _ => t.super_fold_with(self),
        };

        //info!("folded type done: {}, res: {}", t, res);

        res
    }

    fn fold_const(&mut self, c: ty::Const<'tcx>) -> ty::Const<'tcx> {
        if let ty::ConstKind::Param(p) = c.kind() {
            self.const_for_param(p, c)
        } else {
            c.super_fold_with(self)
        }
    }
}

impl<'a, 'tcx> SubstFolder<'a, 'tcx> {
    #[instrument(level="info", skip(self))]
    fn ty_for_param(&self, p: ty::ParamTy, source_ty: Ty<'tcx>) -> Ty<'tcx> {
        // Look up the type in the substitutions. It really should be in there.
        let opt_ty = self.substs.get(p.index() as usize).map(|k| k.unpack());
        //info!("Replace: {:?} with {:?}", source_ty.kind(), opt_ty);

        let ty = match opt_ty {
            Some(GenericArgKind::Type(ty)) => ty,
            Some(kind) => self.type_param_expected(p, source_ty, kind),
            None => self.type_param_out_of_range(p, source_ty),
        };
        //info!("ty_kind: {:#?}", ty.kind());
        self.shift_vars_through_binders(ty)
    }

    fn ty_for_argument(&self, p: ty::ArgumentDef, source_ty: Ty<'tcx>) -> Ty<'tcx> {
        // Look up the type in the substitutions. It really should be in there.
        let opt_ty = self.substs.get(p.index as usize).map(|k| k.unpack());
        info!("ReplaceA: {:?} with {:?}", source_ty.kind(), opt_ty);

        let ty = match opt_ty {
            Some(GenericArgKind::Type(ty)) => ty,
            Some(_kind) => todo!("hoch"),
            None => todo!("hoch"),
        };

        self.shift_vars_through_binders(ty)
    }


    fn instantiate_hkt(&mut self, p: ty::ParamTy, substs: SubstsRef<'tcx>, source_ty: Ty<'tcx>) -> Ty<'tcx> {
        let replaced_to = self.substs.get(p.index() as usize).map(|k| k.unpack());

        let ty = match replaced_to {
            Some(GenericArgKind::Type(ty)) => {
                info!("Source: {:?}, replaced_to: {:?}", source_ty.kind(), ty.kind());
                match ty.kind() {
                    TyKind::HKT(_, other_p, other_substs) => {
                        let substs: SubstsRef<'_> = substs;
                        let other_substs: SubstsRef<'_> = other_substs;
                        assert_eq!(substs.len(), other_substs.len());

                        let generics: &Generics = self.tcx.generics_of(p.def_id());

                        for (inner_param, to) in generics.params.iter().zip(other_substs) {
                            let from = self.tcx.mk_ty(ty::Argument(ArgumentDef {
                                def_id: p.def_id(),
                                index: inner_param.index,
                                name: inner_param.name,
                            }));

                            if let HKTSubstType::InstantiateHKT(map) = &mut self.hkt_subst_type {
                                map.insert(from, to.expect_ty());
                            }
                        }

                        self.tcx.mk_hkt_param(other_p.def_id(), other_p.index(), other_p.name(), substs)
                    }
                    _ => {
                        self.hkt_for_param(p, substs, source_ty)
                    }
                }
            },
            Some(kind) => self.type_param_expected(p, source_ty, kind),
            None => self.type_param_out_of_range(p, source_ty),
        };

        self.shift_vars_through_binders(ty).super_fold_with(self)
    }

    #[allow(dead_code)]
    fn hkt_for_param(&mut self, p: ty::ParamTy, substs: SubstsRef<'tcx>, source_ty: Ty<'tcx>) -> Ty<'tcx> {
        // Substitute `I<u32>` `[Option<%J>]` to `Option<u32>`
        // substs: [u32], source_ty: I<u32>, gens: [%J], replaced_to(self.substs): Option<%J>

        // We set the current type to replaced_to: Option<%J>
        // We replace the first generic (%J) with the first substs (u32) ...
        // We end up with Option<u32>


        // Substitute `I<%J>` `[I<%J>]` to `I<%J>`
        // substs: [%J], source_ty: I<%J>, gens: [%J], replaced_to(self.substs): I<%J>

        // We set the current type to replaced_to: I<%J>
        // We replace the first generic (%J) with the first substs (%J) ...
        // We end up with I<%J>


        // Substitute `A, I<A>` `[u32, Option<%J>]` to `u32, Option<u32>`
        // substs: [A], source_ty: I<A>, gens: [%J], replaced_to(self.substs): Option<%J>

        // We set the current type to replaced_to: Option<%J>
        // We replace the first generic (%J) with the first substs (A) ...
        // We end up with Option<A>


        // Substitute `G<%P>` `[H<%Q>]` to `H<%Q>`
        // substs: [%P], source_ty: G<%P>, gens: [%P], replaced_to(self.substs): H<%Q>

        // We set the current type to replaced_to: H<%P>
        // We replace the first generic (%P) with the first substs (%P) ...
        // We end up with H<%Q>

        // We just change the shell HKT from G to H.
        // For all generics of G, we substitute the argument with the corresponding argument in H assuming they have the same arity.
        // We set the current type to H<%P>
        // We replace the first generic %P with %Q
        // We end up with H<%Q>


        // Substitute `G<%K>` `[H<%Q>]` to `H<%K>`
        // substs: [%K], source_ty: G<%K>, gens: [%P], replaced_to(self.substs): H<%Q>

        // We set the current type to replaced_to: H<%Q>
        // We replace the first generic (%P) with the first substs (%K) ...
        // We end up with H<%Q>

        // We just change the shell HKT from G to H.
        // For all generics of G, we substitute the argument with the corresponding argument in H assuming they have the same arity.
        // We set the current type to H<%K>
        // We replace the first generic %P with %Q
        // We end up with H<%K>



        info!("p: {:?}, source_ty: {:?}, substs: {:?}", p, source_ty.kind(), substs);
        // Look up the type in the substitutions. It really should be in there.
        let replaced_to = self.substs.get(p.index() as usize).map(|k| k.unpack());
        let ty = match replaced_to {
            Some(GenericArgKind::Type(ty)) => {
                info!("replaced_to: {:?}", ty.kind());

                // For each subst in substs
                // Find all arguments with the same def_id as the p.def_id()
                // For all arguments, replace with the replaced_to n'th argument?

                //self.ty_kind_substitution2(source_ty, ty)
                //let mut current = source_ty;
                let mut current = ty;


                for (index, arg) in substs.iter().enumerate() {
                    //current = current.fold_with(&mut ReplacerFolder::new(arg.expect_ty(),ty, self.tcx));
                    current = self.ty_kind_substitution(current, arg.expect_ty(), Some(p.def_id()), index as u32)
                }
                current
            },
            Some(kind) => self.type_param_expected(p, source_ty, kind),
            None => self.type_param_out_of_range(p, source_ty),
        };
        info!("Combine types: {:?} and {:?}, result = {:?}\n", source_ty.kind(), replaced_to, ty);

        //info!("Pre shift: {}, Post shift: {}", ty, self.shift_vars_through_binders(ty));

        self.shift_vars_through_binders(ty).super_fold_with(self)

    }

    /*#[allow(dead_code)]
    fn ty_kind_substitution2(&self, replace: Ty<'tcx>, with: Ty<'tcx>, original: Ty<'tcx>) -> Ty<'tcx> {
        match (ty.kind(), with.kind()) {
            (ty::Bool | ty::Char | ty::Int(_) | ty::Uint(_) | ty::Error(_) | ty::Infer(_) | ty::Param(_) | ty::Float(_), _) => {
                ty
            }
            (ty::Slice(inner), _) => {
                let new_inner = self.ty_kind_substitution2(*inner, with);
                self.tcx.mk_ty(ty::TyKind::Slice(new_inner))
            }
                /*(ty::Adt(a, substs)) => {
                    let substs: &SubstsRef<'_> = substs;

                    let new_substs = substs.iter().map(|a| {
                        match a.unpack() {
                            GenericArgKind::Const(_)
                            | GenericArgKind::Lifetime(_) => a,
                            GenericArgKind::Type(t) => {
                                self.ty_kind_substitution(t, with, def_id, index).into()
                            }
                        }
                    }).collect::<Vec<_>>();

                    self.tcx.mk_ty(ty::Adt(a.clone(), self.tcx.mk_substs(new_substs.into_iter())))
                }*/
            // G<Option<%J>>, H<%Q> =>
            // H<Option<%J>>, %Q =>
            // Option<%J>, %Q =>
            // %J, %Q =>
            // => %Q
            // => Option<%Q>
            // => H<Option<%Q>>
            // === H<Option<%Q>>

            // G<Option<%J>, u32>, H<%Q, %R> =>
            // H<Option<%J>, u32>, [%Q, %R] =>
            // Option<%J>, %Q && u32, %R =>
            // %J, %Q && u32, %R =>
            // => %Q, u32
            // => Option<%Q>, u32
            // => H<Option<%Q>, u32>
            // === H<Option<%Q>, u32>

            // G<Option<%J>, %J>, H<%Q, String> =>
            // H<Option<%J>, %J>, [%Q, String] =>
            // Option<%J>, %J, [%Q, String] =>
            // => Option<%Q>, %Q
            // == H<Option<%Q>, %Q>



            // G<u32>, H<H<%Q>> => H<u32>

            // G<u32>, H<%Q> => H<u32>
            // G<u32>, H<String> => H<String>
            (ty::HKT(_a_did, _a_param, a_substs), ty::HKT(_b_did, _b_param, b_substs)) => {
                let a_substs: &SubstsRef<'_> = a_substs;
                let b_substs: &SubstsRef<'_> = b_substs;
                assert_eq!(a_substs.len(), b_substs.len()); // FIXMIG is this always the case?


                todo!("What the heck")
                /*let new_substs = substs.iter().map(|a| {
                    match a.unpack() {
                        GenericArgKind::Const(_)
                        | GenericArgKind::Lifetime(_) => a,
                        GenericArgKind::Type(t) => {
                            self.ty_kind_substitution(t, with, def_id, index).into()
                        }
                    }
                }).collect::<Vec<_>>();

                self.tcx.mk_ty(ty::HKT(*did, *a, self.tcx.mk_substs(new_substs.into_iter())))*/
            }

            (ty::HKT(did, a, substs), _) => {
                let substs: &SubstsRef<'_> = substs;

                let new_substs = substs.iter().map(|a| {
                    match a.unpack() {
                        GenericArgKind::Const(_)
                        | GenericArgKind::Lifetime(_) => a,
                        GenericArgKind::Type(t) => {
                            self.ty_kind_substitution2(t, with).into()
                        }
                    }
                }).collect::<Vec<_>>();

                self.tcx.mk_ty(ty::HKT(*did, *a, self.tcx.mk_substs(new_substs.into_iter())))
            }

            (_, _) => {
                todo!("here: {:#?} with {:#?}", ty.kind(), with.kind())
            }
        }
    }*/

    // FIXMIG: Make this function a type folder, because that is basically what it should do.
    #[allow(dead_code)]
    fn ty_kind_substitution(&self, ty: Ty<'tcx>, with: Ty<'tcx>, def_id: Option<DefId>, index: u32) -> Ty<'tcx> {
        info!("ty_kind_substitution: {:?}, with: {:?}, def_id: {:?}, index: {}", ty, with, def_id, index);
        let res = match ty.kind() {
            ty::Argument(a) => {
                if let Some(def_id) = def_id {
                    if *a == def_id && *a == index {
                        info!("a = {:?}", a);
                        with
                    } else {
                        info!("not a = {:?}", a);
                        ty
                    }
                } else {
                    with
                }
            }
            ty::Bool
            | ty::Char
            | ty::Int(_)
            | ty::Uint(_)
            | ty::Error(_)
            | ty::Infer(_)
            | ty::Param(_)
            | ty::Float(_) => {
                ty
            }
            ty::Slice(inner) => {
                let new_inner = self.ty_kind_substitution(*inner, with, def_id, index);
                self.tcx.mk_ty(ty::TyKind::Slice(new_inner))
            }
            ty::Ref(region, inner, mutability) => {
                let new_inner = self.ty_kind_substitution(*inner, with, def_id, index);
                self.tcx.mk_ty(ty::TyKind::Ref(*region, new_inner, *mutability))
            }
            ty::Adt(a, substs) => {
                let substs: &SubstsRef<'_> = substs;

                let new_substs = substs.iter().map(|a| {
                    match a.unpack() {
                        GenericArgKind::Const(_)
                        | GenericArgKind::Lifetime(_) => a,
                        GenericArgKind::Type(t) => {
                            self.ty_kind_substitution(t, with, def_id, index).into()
                        }
                    }
                }).collect::<Vec<_>>();

                self.tcx.mk_ty(ty::Adt(a.clone(), self.tcx.mk_substs(new_substs.into_iter())))
            }
            ty::InferHKT( a, substs) => {
                //info!("InferHKT");
                let substs: &SubstsRef<'_> = substs;

                let new_substs = substs.iter().map(|a| {
                    match a.unpack() {
                        GenericArgKind::Const(_)
                        | GenericArgKind::Lifetime(_) => a,
                        GenericArgKind::Type(t) => {
                            let res = self.ty_kind_substitution(t, with, def_id, index);
                            //info!("Subst: {:?}, with {:?}, res: {:?}", t.kind(), with.kind(), res.kind());

                            res.into()
                        }
                    }
                }).collect::<Vec<_>>();

                self.tcx.mk_hkt_infer(*a, self.tcx.mk_substs(new_substs.into_iter()))
            }
            ty::TyKind::Tuple(ty_list) => {
                let ty_list: &'tcx List<Ty<'tcx>> = ty_list;

                let new_ty_list = ty_list.iter().map(|a| {
                    self.ty_kind_substitution(a, with, def_id, index)
                }).collect::<Vec<_>>();
                self.tcx.mk_tup(new_ty_list.into_iter())
            }
            ty::TyKind::Array(ty, size) => {
                let new_inner = self.ty_kind_substitution(*ty, with, def_id, index);
                self.tcx.mk_ty(ty::TyKind::Array(new_inner, *size))
            }
            ty::HKT(did, a, substs) => {
                let substs: &SubstsRef<'_> = substs;

                let new_substs = substs.iter().map(|a| {
                    match a.unpack() {
                        GenericArgKind::Const(_)
                        | GenericArgKind::Lifetime(_) => a,
                        GenericArgKind::Type(t) => {
                            self.ty_kind_substitution(t, with, def_id, index).into()
                        }
                    }
                }).collect::<Vec<_>>();

                self.tcx.mk_ty(ty::HKT(*did, *a, self.tcx.mk_substs(new_substs.into_iter())))
            }
            ty::FnDef(defid, substs) => {
                let substs: &SubstsRef<'_> = substs;

                let new_substs = substs.iter().map(|a| {
                    match a.unpack() {
                        GenericArgKind::Const(_)
                        | GenericArgKind::Lifetime(_) => a,
                        GenericArgKind::Type(t) => {
                            self.ty_kind_substitution(t, with, def_id, index).into()
                        }
                    }
                }).collect::<Vec<_>>();

                self.tcx.mk_ty(ty::TyKind::FnDef(defid.clone(), self.tcx.mk_substs(new_substs.into_iter())))
            }
            ty::FnPtr(polyfnsig) => {
                let poly: PolyFnSig<'tcx> = *polyfnsig;

                let new_poly = poly.map_bound(|ty::FnSig{ inputs_and_output, c_variadic, unsafety, abi }| {
                    ty::FnSig {
                       inputs_and_output: self.tcx.mk_type_list(inputs_and_output.into_iter().map(|t| self.ty_kind_substitution(t, with, def_id, index))),
                       c_variadic,
                       unsafety,
                       abi,
                   }
                });

                self.tcx.mk_fn_ptr(new_poly)
            }
            /*
            ty::Closure(defid, inner) => {
                let closure_subst = ClosureSubsts { substs: inner.clone()};
                closure_subst.sig_as_fn_ptr_ty().fn_sig(self.tcx).map_bound(|ty::FnSig{ inputs_and_output, c_variadic, unsafety, abi }| {
                    ty::FnSig {
                        inputs_and_output: self.tcx.mk_type_list(inputs_and_output.into_iter().map(|t| self.ty_kind_substitution(t, with, def_id, index))),
                        c_variadic,
                        unsafety,
                        abi,
                    }
                });
                self.tcx.mk_closure(defid.clone(), closure_subst.substs)
            }
            */
            _ => {
                todo!("here: {:#?} with {:#?}, def_id: {:?}:{:?}", ty.kind(), with.kind(), def_id, index)
            }
        };

        info!("ty_kind_substitution res: {:?}", res);

        res
    }

    #[cold]
    #[inline(never)]
    fn type_param_expected(&self, p: ty::ParamTy, ty: Ty<'tcx>, kind: GenericArgKind<'tcx>) -> ! {
        bug!(
            "expected type for `{:?}` ({:?}/{}) but found {:?} when substituting, substs={:?}",
            p,
            ty,
            p.index(),
            kind,
            self.substs,
        )
    }

    #[cold]
    #[inline(never)]
    fn type_param_out_of_range(&self, p: ty::ParamTy, ty: Ty<'tcx>) -> ! {
        bug!(
            "type parameter `{:?}` ({:?}/{}) out of range when substituting, substs={:?}",
            p,
            ty,
            p.index(),
            self.substs,
        )
    }

    fn const_for_param(&self, p: ParamConst, source_ct: ty::Const<'tcx>) -> ty::Const<'tcx> {
        // Look up the const in the substitutions. It really should be in there.
        let opt_ct = self.substs.get(p.index as usize).map(|k| k.unpack());
        let ct = match opt_ct {
            Some(GenericArgKind::Const(ct)) => ct,
            Some(kind) => self.const_param_expected(p, source_ct, kind),
            None => self.const_param_out_of_range(p, source_ct),
        };

        self.shift_vars_through_binders(ct)
    }

    #[cold]
    #[inline(never)]
    fn const_param_expected(
        &self,
        p: ty::ParamConst,
        ct: ty::Const<'tcx>,
        kind: GenericArgKind<'tcx>,
    ) -> ! {
        bug!(
            "expected const for `{:?}` ({:?}/{}) but found {:?} when substituting substs={:?}",
            p,
            ct,
            p.index,
            kind,
            self.substs,
        )
    }

    #[cold]
    #[inline(never)]
    fn const_param_out_of_range(&self, p: ty::ParamConst, ct: ty::Const<'tcx>) -> ! {
        bug!(
            "const parameter `{:?}` ({:?}/{}) out of range when substituting substs={:?}",
            p,
            ct,
            p.index,
            self.substs,
        )
    }

    /// It is sometimes necessary to adjust the De Bruijn indices during substitution. This occurs
    /// when we are substituting a type with escaping bound vars into a context where we have
    /// passed through binders. That's quite a mouthful. Let's see an example:
    ///
    /// ```
    /// type Func<A> = fn(A);
    /// type MetaFunc = for<'a> fn(Func<&'a i32>);
    /// ```
    ///
    /// The type `MetaFunc`, when fully expanded, will be
    /// ```ignore (illustrative)
    /// for<'a> fn(fn(&'a i32))
    /// //      ^~ ^~ ^~~
    /// //      |  |  |
    /// //      |  |  DebruijnIndex of 2
    /// //      Binders
    /// ```
    /// Here the `'a` lifetime is bound in the outer function, but appears as an argument of the
    /// inner one. Therefore, that appearance will have a DebruijnIndex of 2, because we must skip
    /// over the inner binder (remember that we count De Bruijn indices from 1). However, in the
    /// definition of `MetaFunc`, the binder is not visible, so the type `&'a i32` will have a
    /// De Bruijn index of 1. It's only during the substitution that we can see we must increase the
    /// depth by 1 to account for the binder that we passed through.
    ///
    /// As a second example, consider this twist:
    ///
    /// ```
    /// type FuncTuple<A> = (A,fn(A));
    /// type MetaFuncTuple = for<'a> fn(FuncTuple<&'a i32>);
    /// ```
    ///
    /// Here the final type will be:
    /// ```ignore (illustrative)
    /// for<'a> fn((&'a i32, fn(&'a i32)))
    /// //          ^~~         ^~~
    /// //          |           |
    /// //   DebruijnIndex of 1 |
    /// //               DebruijnIndex of 2
    /// ```
    /// As indicated in the diagram, here the same type `&'a i32` is substituted once, but in the
    /// first case we do not increase the De Bruijn index and in the second case we do. The reason
    /// is that only in the second case have we passed through a fn binder.
    fn shift_vars_through_binders<T: TypeFoldable<'tcx>>(&self, val: T) -> T {
        debug!(
            "shift_vars(val={:?}, binders_passed={:?}, has_escaping_bound_vars={:?})",
            val,
            self.binders_passed,
            val.has_escaping_bound_vars()
        );

        if self.binders_passed == 0 || !val.has_escaping_bound_vars() {
            return val;
        }

        let result = ty::fold::shift_vars(TypeFolder::tcx(self), val, self.binders_passed);
        debug!("shift_vars: shifted result = {:?}", result);

        result
    }

    fn shift_region_through_binders(&self, region: ty::Region<'tcx>) -> ty::Region<'tcx> {
        if self.binders_passed == 0 || !region.has_escaping_bound_vars() {
            return region;
        }
        ty::fold::shift_region(self.tcx, region, self.binders_passed)
    }
}

/// Stores the user-given substs to reach some fully qualified path
/// (e.g., `<T>::Item` or `<T as Trait>::Item`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, TyEncodable, TyDecodable)]
#[derive(HashStable, TypeFoldable, TypeVisitable, Lift)]
pub struct UserSubsts<'tcx> {
    /// The substitutions for the item as given by the user.
    pub substs: SubstsRef<'tcx>,

    /// The self type, in the case of a `<T>::Item` path (when applied
    /// to an inherent impl). See `UserSelfTy` below.
    pub user_self_ty: Option<UserSelfTy<'tcx>>,
}

/// Specifies the user-given self type. In the case of a path that
/// refers to a member in an inherent impl, this self type is
/// sometimes needed to constrain the type parameters on the impl. For
/// example, in this code:
///
/// ```ignore (illustrative)
/// struct Foo<T> { }
/// impl<A> Foo<A> { fn method() { } }
/// ```
///
/// when you then have a path like `<Foo<&'static u32>>::method`,
/// this struct would carry the `DefId` of the impl along with the
/// self type `Foo<u32>`. Then we can instantiate the parameters of
/// the impl (with the substs from `UserSubsts`) and apply those to
/// the self type, giving `Foo<?A>`. Finally, we unify that with
/// the self type here, which contains `?A` to be `&'static u32`
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, TyEncodable, TyDecodable)]
#[derive(HashStable, TypeFoldable, TypeVisitable, Lift)]
pub struct UserSelfTy<'tcx> {
    pub impl_def_id: DefId,
    pub self_ty: Ty<'tcx>,
}
