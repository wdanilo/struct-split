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
