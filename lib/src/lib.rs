pub mod hlist;

use hlist::Cons;
use hlist::Nil;

use std::fmt::Debug;
pub use borrow_macro::*;


// ==============
// === Traits ===
// ==============

pub mod traits {
    pub use super::Acquire as _;
    pub use super::PartialBorrow as _;
    pub use super::PartialBorrowHelper as _;
    pub use super::RefCast as _;
    pub use super::AsRefs as _;
    pub use super::AsRefsHelper as _;
}


// ==============
// === AsRefs ===
// ==============

pub trait AsRefs<'t, T> {
    fn as_refs_impl(&'t mut self) -> T;
}

impl<'t, T> AsRefsHelper<'t> for T {}
pub trait AsRefsHelper<'t> {
    #[inline(always)]
    fn as_refs<T>(&'t mut self) -> T
    where Self: AsRefs<'t, T> { self.as_refs_impl() }
}


// =======================
// === Struct Generics ===
// =======================

pub trait HasFields { type Fields; }
type Fields<T> = <T as HasFields>::Fields;

pub trait FromFields<Fields> { type Result; }
type WithFields<T, Fields> = <T as FromFields<Fields>>::Result;


// =========================
// === No Access Wrapper ===
// =========================

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

pub trait           Acquire<Target>                  { type Rest; }
impl<'t, T, S>      Acquire<Hidden<T>> for S         { type Rest = S; }
impl<'t: 's, 's, T> Acquire<&'s mut T> for &'t mut T { type Rest = Hidden<T>; }
impl<'t: 's, 's, T> Acquire<&'s     T> for &'t mut T { type Rest = &'t T; }
impl<'t: 's, 's, T> Acquire<&'s     T> for &'t     T { type Rest = &'t T; }

pub type Acquired<This, Target> = <This as Acquire<Target>>::Rest;


// ===================
// === SplitFields ===
// ===================

pub trait SplitFields<Target> { type Rest; }
type SplitFieldsRest<T, Target> = <T as SplitFields<Target>>::Rest;

impl SplitFields<Nil> for Nil {
    type Rest = Nil;
}

impl<H, H2, T, T2> SplitFields<Cons<H2, T2>> for Cons<H, T> where
H: Acquire<H2>,
T: SplitFields<T2> {
    type Rest = Cons<Acquired<H, H2>, <T as SplitFields<T2>>::Rest>;
}


// =====================
// === PartialBorrow ===
// =====================

pub trait PartialBorrow<Target> {
    type Rest;

    #[inline(always)]
    fn partial_borrow_impl(&mut self) -> &mut Target {
        unsafe { &mut *(self as *mut _ as *mut _) }
    }

    #[inline(always)]
    fn partial_borrow_rest_impl(&mut self) -> &mut Self::Rest {
        unsafe { &mut *(self as *mut _ as *mut _) }
    }

    #[inline(always)]
    fn split_impl(&mut self) -> (&mut Target, &mut Self::Rest) {
        let a = unsafe { &mut *(self as *mut _ as *mut _) };
        let b = unsafe { &mut *(self as *mut _ as *mut _) };
        (a, b)
    }
}

impl<Source, Target> PartialBorrow<Target> for Source where
Source: HasFields,
Target: HasFields,
Fields<Source>: SplitFields<Fields<Target>>,
Target: FromFields<SplitFieldsRest<Fields<Source>, Fields<Target>>> {
    type Rest = WithFields<Target, SplitFieldsRest<Fields<Source>, Fields<Target>>>;
}

impl<T> PartialBorrowHelper for T {}
pub trait PartialBorrowHelper {
    #[inline(always)]
    fn partial_borrow<Target>(&mut self) -> &mut Target
    where Self: PartialBorrow<Target> { self.partial_borrow_impl() }

    #[inline(always)]
    fn partial_borrow_rest<Target>(&mut self) -> &mut Self::Rest
    where Self: PartialBorrow<Target> { self.partial_borrow_rest_impl() }

    #[inline(always)]
    fn split<Target>(&mut self) -> (&mut Target, &mut Self::Rest)
    where Self: PartialBorrow<Target> { self.split_impl() }
}


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
    Other: FromFields<ConcatFieldsResult<Fields<Source>, Fields<Other>>> {
    type Result = WithFields<Other, ConcatFieldsResult<Fields<Source>, Fields<Other>>>;
}

pub type Union<T, Other> = <T as Unify<Other>>::Result;


// ======================
// === UnifyFieldImpl ===
// ======================

// NOTE:
// This impl is pretty complex. Maybe it is possible to parametrize everything differently
// to make it nicer.
pub trait UnifyFieldImpl<'t, Other> {
    type Result;
    fn unify_field(&'t mut self, other: &'t mut Other) -> Self::Result;
}

// === for Hidden<T> ===

impl<'t, T> UnifyFieldImpl<'t, Hidden<T>> for Hidden<T> {
    type Result = Hidden<T>;
    fn unify_field(&'t mut self, _: &'t mut Hidden<T>) -> Self::Result { *self }
}

impl<'t, 's, T> UnifyFieldImpl<'t, &'s T> for Hidden<T> {
    type Result = &'s T;
    fn unify_field(&'t mut self, other: &'t mut &'s T) -> Self::Result { other }
}

impl<'t, 's, T: 't> UnifyFieldImpl<'t, &'s mut T> for Hidden<T> {
    type Result = &'t mut T;
    fn unify_field(&'t mut self, other: &'t mut &'s mut T) -> Self::Result { other }
}

// === for &'s T ===

impl<'t, 's, T> UnifyFieldImpl<'t, Hidden<T>> for &'s T {
    type Result = &'s T;
    fn unify_field(&'t mut self, _: &'t mut Hidden<T>) -> Self::Result { self }
}

impl<'t, 's, T> UnifyFieldImpl<'t, &'s T> for &'s T {
    type Result = &'s T;
    fn unify_field(&'t mut self, _: &'t mut &'s T) -> Self::Result { self }
}

impl<'t, 's, T: 't> UnifyFieldImpl<'t, &'s mut T> for &'s T {
    type Result = &'t mut T;
    fn unify_field(&'t mut self, other: &'t mut &'s mut T) -> Self::Result { other }
}

// === for &'s mut T ===

impl<'t, 's, T: 't> UnifyFieldImpl<'t, Hidden<T>> for &'s mut T {
    type Result = &'t mut T;
    fn unify_field(&'t mut self, _: &'t mut Hidden<T>) -> Self::Result { self }
}

impl<'t, 's, T: 't> UnifyFieldImpl<'t, &'s T> for &'s mut T {
    type Result = &'t mut T;
    fn unify_field(&'t mut self, _: &'t mut &'s T) -> Self::Result { self }
}

impl<'t, 's, T: 't> UnifyFieldImpl<'t, &'s mut T> for &'s mut T {
    type Result = &'t mut T;
    fn unify_field(&'t mut self, _: &'t mut &'s mut T) -> Self::Result { self }
}


// =================
// === UnifyImpl ===
// =================

pub trait UnifyImpl<Other> {
    type Result;
    fn union(self, other: Other) -> Self::Result;
}

// This should be the same as `Union`, but the implementation of `unify` requires
// complex bounds, so `Union` uses simpler logic.
pub type UnionImpl<T, Other> = <T as UnifyImpl<Other>>::Result;


// ==============
// === Macros ===
// ==============

#[macro_export]
macro_rules! lifetime_chooser {
    ($lt1:lifetime $lt2:lifetime $($ts:tt)*) => {& $lt2 $($ts)*};
    ($lt1:lifetime $($ts:tt)*) => {& $lt1 $($ts)*};
}

#[macro_export]
macro_rules! partial_borrow {
    (< $($ts:tt)*)                    => { $crate::partial_borrow! { @ [] $($ts)* } };
    (@ [$($xs:tt)*] > $t:ident)       => { $t! { $($xs)* } };
    (@ [$($xs:tt)*] $t:tt $($ts:tt)*) => { $crate::partial_borrow! { @ [$($xs)* $t] $($ts)* } };
}