// =================
// === HasFields ===
// =================

pub trait HasFields { type Fields; }
pub type Fields<T> = <T as HasFields>::Fields;


// =====================
// === ReplaceFields ===
// =====================

pub trait ReplaceFields<Fields> { type Result; }
pub type ReplacedFields<T, Fields> = <T as ReplaceFields<Fields>>::Result;