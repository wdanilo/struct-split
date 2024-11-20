// =================
// === HasFields ===
// =================

use crate::hlist;

pub trait HasFields { type Fields; }
pub type Fields<T> = <T as HasFields>::Fields;
pub type FieldAt<N, T> = hlist::ItemAt<N, Fields<T>>;


// =====================
// === ReplaceFields ===
// =====================

pub trait ReplaceFields<Fields> { type Result; }
pub type ReplacedFields<T, Fields> = <T as ReplaceFields<Fields>>::Result;