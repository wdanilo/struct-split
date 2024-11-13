#![allow(dead_code)]

use std::vec::Vec;
use borrow::PartialBorrow;
use borrow::partial_borrow as p;
use borrow::traits::*;

// ============
// === Test ===
// ============

type NodeId = usize;
type EdgeId = usize;

#[derive(Debug)]
struct Node {
    outputs: Vec<EdgeId>,
    inputs: Vec<EdgeId>,
}

#[derive(Debug)]
struct Edge {
    from: Option<NodeId>,
    to: Option<NodeId>,
}

#[derive(Debug, PartialBorrow)]
#[module(crate)]
struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

// Requires mutable access to the `graph.edges` field.
fn detach_node(graph: &mut p!(<mut edges> Graph), node: &mut Node) {
    for edge_id in std::mem::take(&mut node.outputs) {
        graph.edges[edge_id].from = None;
    }
    for edge_id in std::mem::take(&mut node.inputs) {
        graph.edges[edge_id].to = None;
    }
}

// Requires mutable access to all `graph` fields.
fn detach_all_nodes(graph: &mut p!(<mut *> Graph)) {
    // Extract the `nodes` field. The `graph2` variable has a type
    // of `&mut p!(<mut *, !nodes> Graph)`.
    let (nodes, graph2) = graph.extract_nodes();
    for node in nodes {
        detach_node(graph2.partial_borrow(), node);
    }
}

#[test]
fn test() {
    // 0 -> 1 -> 2 -> 0
    let mut graph = Graph {
        nodes: vec![
            Node { outputs: vec![0], inputs: vec![2] }, // Node 0
            Node { outputs: vec![1], inputs: vec![0] }, // Node 1
            Node { outputs: vec![2], inputs: vec![1] }, // Node 2
        ],
        edges: vec![
            Edge { from: Some(0), to: Some(1) }, // Edge 0
            Edge { from: Some(1), to: Some(2) }, // Edge 1
            Edge { from: Some(2), to: Some(0) }, // Edge 2
        ],
    };

    detach_all_nodes(&mut graph.as_refs_mut().partial_borrow());

    for node in &graph.nodes {
        assert!(node.outputs.is_empty());
        assert!(node.inputs.is_empty());
    }
    for edge in &graph.edges {
        assert!(edge.from.is_none());
        assert!(edge.to.is_none());
    }
}
