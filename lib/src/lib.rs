// #![doc = include_str!("../README.md")]

pub mod hlist;
pub mod reflect;

use hlist::Cons;
use hlist::Nil;
use std::fmt::Debug;

pub use reflect::*;
pub use borrow_macro::*;


// ==============
// === Traits ===
// ==============

pub mod traits {
    pub use super::Acquire as _;
    pub use super::Partial as _;
    pub use super::PartialHelper as _;
    pub use super::RefCast as _;
    pub use super::AsRefs as _;
    pub use super::AsRefsHelper as _;
}


// ==============
// === AsRefs ===
// ==============

/// Borrow all fields of a struct and output a partially borrowed struct,
/// like `p!(<mut field1, field2>MyStruct)`.
pub trait AsRefs<'t, T> {
    fn as_refs_impl(&'t mut self) -> T;
}

impl<'t, T> AsRefsHelper<'t> for T {}
pub trait AsRefsHelper<'t> {
    /// Borrow all fields of a struct and output a partially borrowed struct,
    /// like `p!(<mut field1, field2>MyStruct)`.
    #[inline(always)]
    fn as_refs<T>(&'t mut self) -> T
    where Self: AsRefs<'t, T> { self.as_refs_impl() }
}


// =========================
// === No Access Wrapper ===
// =========================

/// A phantom type used to mark fields as hidden in the partially borrowed structs.
#[repr(transparent)]
#[derive(Debug)]
pub struct Hidden<T>(*mut T);

impl<T> Copy for Hidden<T> {}
impl<T> Clone for Hidden<T> {
    fn clone(&self) -> Self { Self(self.0) }
}


// ===============
// === RefCast ===
// ===============

pub trait RefCast<'t, T> {
    /// All possible casts of a mutable reference: `&mut T` (identity), `&T`, and `Hidden<T>`.
    fn ref_cast(&'t mut self) -> T;
}

impl<'t, T> RefCast<'t, &'t T> for T {
    #[inline(always)]
    fn ref_cast(&'t mut self) -> &'t T { self }
}

impl<'t, T> RefCast<'t, &'t mut T> for T {
    #[inline(always)]
    fn ref_cast(&'t mut self) -> &'t mut T { self }
}

impl<'t, T> RefCast<'t, Hidden<T>> for T {
    #[inline(always)]
    fn ref_cast(&'t mut self) -> Hidden<T> { Hidden(self) }
}


// ===============
// === Acquire ===
// ===============


/// This is a documentation for type-level field borrowing transformation. It involves checking if a
/// field of a partially borrowed struct can be borrowed in a specific form and provides the remaining
/// fields post-borrow.
pub trait           Acquire<Target>                  { type Rest; }
impl<'t, T, S>      Acquire<Hidden<T>> for S         { type Rest = S; }
impl<'t: 's, 's, T> Acquire<&'s mut T> for &'t mut T { type Rest = Hidden<T>; }
impl<'t: 's, 's, T> Acquire<&'s     T> for &'t mut T { type Rest = &'t T; }
impl<'t: 's, 's, T> Acquire<&'s     T> for &'t     T { type Rest = &'t T; }

/// Remaining fields after borrowing a specific field. See the documentation of [`Acquire`] to learn more.
pub type Acquired<This, Target> = <This as Acquire<Target>>::Rest;


// ===================
// === SplitFields ===
// ===================

/// Split HList of borrows into target HList of borrows and a HList of remaining borrows after
/// acquiring the target. See the documentation of [`Acquire`] for more information.
///
/// This trait is automatically implemented for all types.
pub trait          SplitFields<Target>               { type Rest; }
impl               SplitFields<Nil>          for Nil { type Rest = Nil; }
impl<H, H2, T, T2> SplitFields<Cons<H2, T2>> for Cons<H, T> where
T: SplitFields<T2>, H: Acquire<H2> {
    type Rest = Cons<Acquired<H, H2>, <T as SplitFields<T2>>::Rest>;
}

type SplitFieldsRest<T, Target> = <T as SplitFields<Target>>::Rest;


// ===============
// === Partial ===
// ===============

/// Helper trait for [`Partial`]. This trait is automatically implemented by the [`partial_borrow!`]
/// macro. It is used to provide Rust type inferencer with additional type information. In particular, it
/// is used to tell that any partial borrow of a struct results in the same struct type, but parametrized
/// differently. It is needed for Rust to correctly infer target types for associated methods, like:
///
/// ```ignore
/// #[derive(Partial)]
/// #[module(crate)]
/// pub struct Ctx {
///     pub geometry: GeometryCtx,
///     pub material: MaterialCtx,
///     pub mesh: MeshCtx,
///     pub scene: SceneCtx,
/// }
///
/// impl p!(<mut geometry, mut material>Ctx) {
///     fn my_method(&mut self){}
/// }
///
/// fn test(ctx: p!(&<mut *> Ctx)) {
///     ctx.partial_borrow().my_method();
/// }
/// ```
pub trait PartialInferenceGuide<Target> {}

/// Implementation of partial field borrowing. The `Target` type parameter specifies the required
/// partial borrow representation, such as `p!(<mut field1, field2>MyStruct)`.
///
/// This trait is automatically implemented for all partial borrow representations.
pub trait Partial<Target> : PartialInferenceGuide<Target> {
    type Rest;

    /// See the documentation of [`PartialHelper::partial_borrow`].
    #[inline(always)]
    fn partial_borrow_impl(&mut self) -> &mut Target {
        unsafe { &mut *(self as *mut _ as *mut _) }
    }

    /// See the documentation of [`PartialHelper::split`].
    #[inline(always)]
    fn split_impl(&mut self) -> (&mut Target, &mut Self::Rest) {
        let a = unsafe { &mut *(self as *mut _ as *mut _) };
        let b = unsafe { &mut *(self as *mut _ as *mut _) };
        (a, b)
    }
}

impl<Source, Target> Partial<Target> for Source where
Source: PartialInferenceGuide<Target>,
Source: HasFields,
Target: HasFields,
Fields<Source>: SplitFields<Fields<Target>>,
Target: ReplaceFields<SplitFieldsRest<Fields<Source>, Fields<Target>>> {
    type Rest = ReplacedFields<Target, SplitFieldsRest<Fields<Source>, Fields<Target>>>;
}

/// Helper for [`Partial`]. This trait is automatically implemented for all types.
impl<Target> PartialHelper for Target {}
pub trait PartialHelper {
    /// Borrow fields from this partial borrow for the `Target` partial borrow, like
    /// `ctx.partial_borrow::<p!(<mut scene>Ctx)>()`.
    #[inline(always)]
    fn partial_borrow<Target>(&mut self) -> &mut Target
    where Self: PartialNotEq<Target> { self.partial_borrow_impl() }

    /// Borrow fields from this partial borrow for the `Target` partial borrow, like
    /// `ctx.partial_borrow::<p!(<mut scene>Ctx)>()`.
    #[inline(always)]
    fn partial_borrow_or_eq<Target>(&mut self) -> &mut Target
    where Self: Partial<Target> { self.partial_borrow_impl() }

    /// Split this partial borrow into the `Target` partial borrow and the remaining fields, like
    /// `let (scene, ctx2) = ctx.split::<p!(<mut scene>Ctx)>()`.
    #[inline(always)]
    fn split<Target>(&mut self) -> (&mut Target, &mut Self::Rest)
    where Self: Partial<Target> { self.split_impl() }
}


// ====================
// === PartialNotEq ===
// ====================

pub trait PartialNotEq<Target> : Partial<Target> + NotEq<Target> {}
impl<Target, T> PartialNotEq<Target> for T where T: Partial<Target> + NotEq<Target> {}


// =============
// === NotEq ===
// =============

pub trait NotEq<Target> {}
impl<Source, Target> NotEq<Target> for Source where
    Source: HasFields,
    Target: HasFields,
    Fields<Source>: NotEqFields<Fields<Target>> {
}

pub trait NotEqFields<Target> {}
impl<    't, H, T, T2> NotEqFields<Cons<&'t mut H, T>> for Cons<Hidden<H>, T2> {}
impl<    't, H, T, T2> NotEqFields<Cons<&'t     H, T>> for Cons<Hidden<H>, T2> {}
impl<        H, T, T2> NotEqFields<Cons<Hidden<H>, T>> for Cons<Hidden<H>, T2> where T: NotEqFields<T2> {}

impl<    't, H, T, T2> NotEqFields<Cons<Hidden<H>, T>> for Cons<&'t mut H, T2> {}
impl<'s, 't, H, T, T2> NotEqFields<Cons<&'s     H, T>> for Cons<&'t mut H, T2> {}
impl<'s, 't, H, T, T2> NotEqFields<Cons<&'s mut H, T>> for Cons<&'t mut H, T2> where T: NotEqFields<T2> {}

impl<    't, H, T, T2> NotEqFields<Cons<Hidden<H>, T>> for Cons<&'t H, T2> {}
impl<'s, 't, H, T, T2> NotEqFields<Cons<&'s mut H, T>> for Cons<&'t H, T2> {}
impl<'s, 't, H, T, T2> NotEqFields<Cons<&'s     H, T>> for Cons<&'t H, T2> where T: NotEqFields<T2> {}


// ==================
// === UnifyField ===
// ==================

pub trait UnifyField<Other> { type Result; }

impl<'t, T> UnifyField<Hidden<T>> for Hidden<T> { type Result = Hidden<T>; }
impl<'t, T> UnifyField<&'t     T> for Hidden<T> { type Result = &'t     T; }
impl<'t, T> UnifyField<&'t mut T> for Hidden<T> { type Result = &'t mut T; }

impl<'t, T> UnifyField<Hidden<T>> for &'t T { type Result = &'t     T; }
impl<'t, T> UnifyField<&'t     T> for &'t T { type Result = &'t     T; }
impl<'t, T> UnifyField<&'t mut T> for &'t T { type Result = &'t mut T; }

impl<'t, T> UnifyField<Hidden<T>> for &'t mut T { type Result = &'t mut T; }
impl<'t, T> UnifyField<&'t     T> for &'t mut T { type Result = &'t mut T; }
impl<'t, T> UnifyField<&'t mut T> for &'t mut T { type Result = &'t mut T; }

type ConcatenatedField<T, Other> = <T as UnifyField<Other>>::Result;


// ====================
// === UnifyFields ===
// ====================

pub trait UnifyFields<Other> { type Result; }
type ConcatFieldsResult<T, Other> = <T as UnifyFields<Other>>::Result;

impl UnifyFields<Nil> for Nil {
    type Result = Nil;
}

impl<H, H2, T, T2> UnifyFields<Cons<H2, T2>> for Cons<H, T> where
    H: UnifyField<H2>,
    T: UnifyFields<T2> {
    type Result = Cons<ConcatenatedField<H, H2>, <T as UnifyFields<T2>>::Result>;
}

pub trait Unify<Other> {
    type Result;
}

impl<Source, Other> Unify<Other> for Source where
    Source: HasFields,
    Other: HasFields,
    Fields<Source>: UnifyFields<Fields<Other>>,
    Other: ReplaceFields<ConcatFieldsResult<Fields<Source>, Fields<Other>>> {
    type Result = ReplacedFields<Other, ConcatFieldsResult<Fields<Source>, Fields<Other>>>;
}

pub type Union<T, Other> = <T as Unify<Other>>::Result;


// ==============
// === Macros ===
// ==============

#[macro_export]
macro_rules! lifetime_chooser {
    ($lt1:lifetime $lt2:lifetime $($ts:tt)*) => {& $lt2 $($ts)*};
    ($lt1:lifetime $($ts:tt)*) => {& $lt1 $($ts)*};
}

#[macro_export]
macro_rules! partial {
    (& $lt:lifetime $($ts:tt)*)       => { & $lt mut $crate::partial! { $($ts)* } };
    (& $($ts:tt)*)                    => { &     mut $crate::partial! { $($ts)* } };
    (< $($ts:tt)*)                    => {           $crate::partial! { @ [] $($ts)* } };
    (@ [$($xs:tt)*] > $t:ident)       => { $t! { $($xs)* } };
    (@ [$($xs:tt)*] $t:tt $($ts:tt)*) => { $crate::partial! { @ [$($xs)* $t] $($ts)* } };
}