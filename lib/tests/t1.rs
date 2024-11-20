use std::vec::Vec;
use borrow::partial as p;
use borrow::traits::*;

//
//
// #[derive(Debug, borrow::Partial)]
// #[module(crate)]
// struct Graph {
//     nodes: usize,
//     edges: usize,
// }
//
//
//
// type Foo = p!(<mut *> Graph);


struct Graph;

trait Field1 {
    type Type;
}

impl Field1 for Graph {
    type Type = usize;
}