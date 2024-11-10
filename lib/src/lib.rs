pub mod hlist;

use hlist::Cons;
use hlist::Nil;

use std::fmt::Debug;
use std::marker::PhantomData;
pub use struct_split_macro::*;

// ==============
// === Traits ===
// ==============

pub mod traits {
    pub use super::Acquire as _;
    pub use super::Split as _;
    pub use super::SplitHelper as _;
    pub use super::RefCast as _;
    pub use super::AsRefs as _;
    pub use super::AsRefsHelper as _;
}


// ===============
// === Labeled ===
// ===============

#[repr(transparent)]
pub struct Labeled<L, T> {
    label: PhantomData<L>,
    data: T,
}


// ===================
// === Access Flag ===
// ===================

// #[derive(Debug)]
// pub struct None;
//
// #[derive(Debug)]
// pub struct Ref;
//
// #[derive(Debug)]
// pub struct RefMut;


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


// ==============
// === Access ===
// ==============

// pub trait Access            { type Value<'t, T: 't + Debug>: Debug; }
// impl      Access for Ref    { type Value<'t, T: 't + Debug> = &'t T; }
// impl      Access for RefMut { type Value<'t, T: 't + Debug> = &'t mut T; }
// impl      Access for None   { type Value<'t, T: 't + Debug> = Hidden<T>; }
// impl<L, S> Access for Labeled<L, S>
// where S: Access {
//     type Value<'t, T: 't + Debug> = S::Value<'t, T>;
// }
//
// pub type Value<'t, L, T> = <L as Access>::Value<'t, T>;


// ===============
// === Acquire ===
// ===============

pub trait   Acquire<Target>                  { type Rest; }
impl<'t, T> Acquire<Hidden<T>> for &'t mut T { type Rest = &'t mut T; }
impl<'t, T> Acquire<Hidden<T>> for &'t     T { type Rest = &'t     T; }
impl<'t, T> Acquire<&'t mut T> for &'t mut T { type Rest = Hidden<T>; }
impl<'t, T> Acquire<&'t     T> for &'t mut T { type Rest = &'t T; }
impl<'t, T> Acquire<&'t     T> for &'t     T { type Rest = &'t T; }

pub type Acquired<This, Target> = <This as Acquire<Target>>::Rest;


// ==============
// === Fields ===
// ==============

pub trait IntoFields { type Fields; }
type Fields<T> = <T as IntoFields>::Fields;

pub trait FromFields<Fields> { type Result; }
type WithFields<T, Fields> = <T as FromFields<Fields>>::Result;

// =============
// === Split ===
// =============

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

pub trait Split<Target> {
    type Rest;

    #[inline(always)]
    fn fit_impl(&mut self) -> &mut Target {
        unsafe { &mut *(self as *mut _ as *mut _) }
    }

    #[inline(always)]
    fn fit_rest_impl(&mut self) -> &mut Self::Rest {
        unsafe { &mut *(self as *mut _ as *mut _) }
    }

    #[inline(always)]
    fn split_impl(&mut self) -> (&mut Target, &mut Self::Rest) {
        let a = unsafe { &mut *(self as *mut _ as *mut _) };
        let b = unsafe { &mut *(self as *mut _ as *mut _) };
        (a, b)
    }
}

impl<Source, Target> Split<Target> for Source where
Source: IntoFields,
Target: IntoFields,
Fields<Source>: SplitFields<Fields<Target>>,
Target: FromFields<SplitFieldsRest<Fields<Source>, Fields<Target>>> {
    type Rest = WithFields<Target, SplitFieldsRest<Fields<Source>, Fields<Target>>>;
}

impl<T> SplitHelper for T {}
pub trait SplitHelper {
    #[inline(always)]
    fn fit<Target>(&mut self) -> &mut Target
    where Self: Split<Target> { self.fit_impl() }

    #[inline(always)]
    fn fit_rest<Target>(&mut self) -> &mut Self::Rest
    where Self: Split<Target> { self.fit_rest_impl() }

    #[inline(always)]
    fn split<Target>(&mut self) -> (&mut Target, &mut Self::Rest)
    where Self: Split<Target> { self.split_impl() }
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
