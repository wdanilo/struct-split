pub mod hlist;

use hlist::Cons;
use hlist::Nil;

use std::fmt::Debug;
pub use struct_split_macro::*;


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


// =======================
// === Struct Generics ===
// =======================

pub trait IntoFields {
    type Fields;
    fn into_fields(&mut self) -> Self::Fields;
}
type Fields<T> = <T as IntoFields>::Fields;

pub trait FromFields<Fields> {
    type Result;
    fn from_fields(fields: Fields) -> Self::Result;
}
type WithFields<T, Fields> = <T as FromFields<Fields>>::Result;


// =========================
// === No Access Wrapper ===
// =========================

#[repr(transparent)]
#[derive(Debug)]
pub struct Hidden<T>(*mut T);


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




// impl<X, T1, T2> JoinField<Cons<Hidden<X>, T2>> for Cons<Hidden<X>, T1>
// where T1: JoinField<T2> {
//     type Result = Cons<Hidden<X>, <T1 as JoinField<T2>>::Result>;
//     fn join_field(self, other: Cons<Hidden<X>, T2>) -> Self::Result {
//         Cons { head: self.head, tail: self.tail.join_field(other.tail) }
//     }
// }

pub trait JoinField<Other> {
    type Result;
    fn join_field(self, other: Other) -> Self::Result;
}


pub trait JoinFields<Other> {
    type Result;
    fn join_fields(self, other: Other) -> Self::Result;
}

impl JoinFields<Nil> for Nil where {
    type Result = Nil;
    fn join_fields(self, _: Nil) -> Self::Result {
        Nil
    }
}

impl<A, B, T1, T2> JoinFields<Cons<B, T2>> for Cons<A, T1> where
    T1: JoinFields<T2>,
    A: JoinField<B>,
{
    type Result = Cons<<A as JoinField<B>>::Result, <T1 as JoinFields<T2>>::Result>;
    fn join_fields(self, other: Cons<B, T2>) -> Self::Result {
        Cons { head: self.head.join_field(other.head), tail: self.tail.join_fields(other.tail) }
    }
}


// === for Hidden<T> ===

impl<T> JoinField<Hidden<T>> for Hidden<T> {
    type Result = Hidden<T>;
    fn join_field(self, _: Hidden<T>) -> Self::Result { self }
}

impl<'t, T> JoinField<&'t T> for Hidden<T> {
    type Result = &'t T;
    fn join_field(self, other: &'t T) -> Self::Result { other }
}

impl<'t, T> JoinField<&'t mut T> for Hidden<T> {
    type Result = &'t mut T;
    fn join_field(self, other: &'t mut T) -> Self::Result { other }
}

// === for &'s T ===

impl<'s, T> JoinField<Hidden<T>> for &'s T {
    type Result = &'s T;
    fn join_field(self, _: Hidden<T>) -> Self::Result { self }
}

impl<'t, 's, T> JoinField<&'t T> for &'s T {
    type Result = &'s T;
    fn join_field(self, _: &'t T) -> Self::Result { self }
}

impl<'t, 's, T> JoinField<&'t mut T> for &'s T {
    type Result = &'t mut T;
    fn join_field(self, other: &'t mut T) -> Self::Result { other }
}

// === for &'s mut T ===

impl<'s, T> JoinField<Hidden<T>> for &'s mut T {
    type Result = &'s mut T;
    fn join_field(self, _: Hidden<T>) -> Self::Result { self }
}

impl<'t, 's, T> JoinField<&'t T> for &'s mut T {
    type Result = &'s mut T;
    fn join_field(self, _: &'t T) -> Self::Result { self }
}

impl<'t, 's, T> JoinField<&'t mut T> for &'s mut T {
    type Result = &'s mut T;
    fn join_field(self, _: &'t mut T) -> Self::Result { self }
}



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
Source: IntoFields,
Target: IntoFields,
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


// ============
// === Join ===
// ============

pub trait Join<Other> {
    type Result;
    fn join(&mut self, other: &mut Other) -> &mut Self::Result;
}

// impl<This, That> Join<That> for This where
//     This: IntoFields,
//     That: IntoFields,
//     Fields<This>: JoinFields<Fields<That>>,
//     That: FromFields<<Fields<This> as JoinFields<Fields<That>>>::Result> {
//     type Result = WithFields<That, <Fields<This> as JoinFields<Fields<That>>>::Result>;
//     fn join(&mut self, other: &mut That) -> &mut Self::Result {
//         self.into_fields();
//     }
// }

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