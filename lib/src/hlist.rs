// ===========
// === Nat ===
// ===========

pub struct Zero;
pub struct Succ<N: Nat>(N);

pub trait Nat {}
impl Nat for Zero {}
impl<N: Nat> Nat for Succ<N> {}

pub type N0 = Zero;
pub type N1 = Succ<N0>;
pub type N2 = Succ<N1>;
pub type N3 = Succ<N2>;
pub type N4 = Succ<N3>;
pub type N5 = Succ<N4>;
pub type N6 = Succ<N5>;
pub type N7 = Succ<N6>;
pub type N8 = Succ<N7>;
pub type N9 = Succ<N8>;


// =============
// === HList ===
// =============

#[derive(Clone, Copy, Debug)]
pub struct Cons<H, T> {
    pub head: H,
    pub tail: T,
}

#[derive(Clone, Copy, Debug)]
pub struct Nil;

// =============
// === Index ===
// =============

pub trait Index<N: Nat> {
    type Item;
}

impl<H, T> Index<Zero> for Cons<H, T> {
    type Item = H;
}

impl<H, T, N: Nat> Index<Succ<N>> for Cons<H, T> where
T: Index<N> {
    type Item = <T as Index<N>>::Item;
}

pub type ItemAt<N, T> = <T as Index<N>>::Item;

// ==============
// === Macros ===
// ==============

#[macro_export]
macro_rules! HList {
    () => { $crate::hlist::Nil };
    ($t:ty $(,$($ts:tt)*)?) => {
        $crate::hlist::Cons<$t, $crate::HList!{$($($ts)*)?}>
    };
}

#[macro_export]
macro_rules! hlist {
    () => { $crate::hlist::Nil };
    ($a:expr $(,$($tok:tt)*)?) => {
        $crate::hlist::Cons {
            head: $a,
            tail: $crate::hlist!{$($($tok)*)?},
        }
    };
}

#[macro_export]
macro_rules! hlist_pat {
    () => { $crate::hlist::Nil };
    ($a:pat $(,$($tok:tt)*)?) => {
        $crate::hlist::Cons {
            head: $a,
            tail: $crate::hlist_pat!{$($($tok)*)?},
        }
    };
}
