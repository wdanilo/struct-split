use std::fmt::Debug;
use std::marker::PhantomData;
pub use struct_split_macro::*;

// ==============
// === Traits ===
// ==============

pub mod traits {
    pub use super::Access as _;
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

#[derive(Debug)]
pub struct None;

#[derive(Debug)]
pub struct Ref;

#[derive(Debug)]
pub struct RefMut;


// =========================
// === No Access Wrapper ===
// =========================

#[repr(transparent)]
#[derive(Debug)]
pub struct NoAccess<T>(*mut T);


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

impl<'t, T> RefCast<'t, NoAccess<T>> for T {
    #[inline(always)]
    fn ref_cast(&'t mut self) -> NoAccess<T> { NoAccess(self) }
}


// ==============
// === Access ===
// ==============

pub trait Access            { type Value<'t, T: 't + Debug>: Debug; }
impl      Access for Ref    { type Value<'t, T: 't + Debug> = &'t T; }
impl      Access for RefMut { type Value<'t, T: 't + Debug> = &'t mut T; }
impl      Access for None   { type Value<'t, T: 't + Debug> = NoAccess<T>; }
impl<L, S> Access for Labeled<L, S>
where S: Access {
    type Value<'t, T: 't + Debug> = S::Value<'t, T>;
}

pub type Value<'t, L, T> = <L as Access>::Value<'t, T>;


// ===============
// === Acquire ===
// ===============

pub trait       Acquire<Target: Access>    { type Rest: Access; }
impl<T: Access> Acquire<None>   for T      { type Rest = T; }
impl            Acquire<RefMut> for RefMut { type Rest = None; }
impl            Acquire<Ref>    for RefMut { type Rest = Ref; }
impl            Acquire<Ref>    for Ref    { type Rest = Ref; }

pub type Acquired<This, Target> = <This as Acquire<Target>>::Rest;


// =============
// === Split ===
// =============

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
