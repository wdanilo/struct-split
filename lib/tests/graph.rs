#![allow(dead_code)]

use std::vec::Vec;
use borrow::partial as p;
use borrow::traits::*;

// ============
// === Data ===
// ============

type NodeId = usize;
type EdgeId = usize;

#[derive(Debug)]
struct Node {
    outputs: Vec<EdgeId>,
    inputs:  Vec<EdgeId>,
}

#[derive(Debug)]
struct Edge {
    from: Option<NodeId>,
    to:   Option<NodeId>,
}

#[derive(Debug)]
struct Group {
    nodes: Vec<NodeId>,
}

// =============
// === Graph ===
// =============

#[derive(Debug, borrow::Partial)]
#[module(crate)]
struct Graph {
    nodes:  Vec<Node>,
    edges:  Vec<Edge>,
    groups: Vec<Group>,
}

// =============
// === Utils ===
// =============

// Requires mutable access to the `graph.edges` field.
fn detach_node(graph: p!(&<mut edges> Graph), node: &mut Node) {
    for edge_id in std::mem::take(&mut node.outputs) {
        graph.edges[edge_id].from = None;
    }
    for edge_id in std::mem::take(&mut node.inputs) {
        graph.edges[edge_id].to = None;
    }
}

// Requires mutable access to all `graph` fields.
fn detach_all_nodes(graph: p!(&<mut *> Graph)) {
    // Extract the `nodes` field.
    // The `graph2` variable has a type of `p!(&<mut *, !nodes> Graph)`.
    let (nodes, graph2) = graph.extract_nodes();
    for node in nodes {
        detach_node(graph2.partial_borrow(), node);
    }
}

// =============
// === Tests ===
// =============

#[test]
fn test() {
    // node0 -----> node1 -----> node2 -----> node0
    //       edge0        edge1        edge2
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
        groups: vec![]
    };

    detach_all_nodes(&mut graph.as_refs_mut());

    for node in &graph.nodes {
        assert!(node.outputs.is_empty());
        assert!(node.inputs.is_empty());
    }
    for edge in &graph.edges {
        assert!(edge.from.is_none());
        assert!(edge.to.is_none());
    }
}
