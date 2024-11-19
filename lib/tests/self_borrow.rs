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
    inputs: Vec<EdgeId>,
}

#[derive(Debug)]
struct Edge {
    from: Option<NodeId>,
    to: Option<NodeId>,
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
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    groups: Vec<Group>,
}


// =============
// === Utils ===
// =============

impl p!(<mut *> Graph) {
    fn detach_all_nodes(&mut self) {
        let (nodes, self2) = self.extract_nodes();
        for node in nodes {
            self2.partial_borrow().detach_node(node);
        }
    }
}

impl p!(<mut edges> Graph) {
    fn detach_node(&mut self, node: &mut Node) {
        for edge_id in std::mem::take(&mut node.outputs) {
            self.edges[edge_id].from = None;
        }
        for edge_id in std::mem::take(&mut node.inputs) {
            self.edges[edge_id].to = None;
        }
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
        groups: vec![],
    };

    graph.as_refs_mut().detach_all_nodes();

    for node in &graph.nodes {
        assert!(node.outputs.is_empty());
        assert!(node.inputs.is_empty());
    }
    for edge in &graph.edges {
        assert!(edge.from.is_none());
        assert!(edge.to.is_none());
    }
}
