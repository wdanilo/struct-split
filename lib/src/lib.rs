//! <img width="680" alt="banner" src="https://github.com/user-attachments/assets/1740befa-c25d-4428-bda8-c34d437f333e">
//!
//! <br/>
//! <br/>
//!
//! # 🔪 Partial Borrows
//!
//! Zero-overhead
//! ["partial borrows"](https://internals.rust-lang.org/t/notes-on-partial-borrows/20020),
//! borrows of selected fields only, **including partial self-borrows**. It lets you split structs
//! into non-overlapping sets of mutably borrowed fields, like `&<mut field1, field2>MyStruct` and
//! `&<field2, mut field3>MyStruct`. It is similar to
//! [slice::split_at_mut](https://doc.rust-lang.org/std/primitive.slice.html#method.split_at_mut)
//! but more flexible and tailored for structs.
//!
//! <br/>
//! <br/>
//!
//! # 🤩 Why partial borrows? Examples included!
//!
//! Partial borrows offer a variety of advantages. Each of the following points includes a short
//! in-line explanation with a link to an example code with a detailed explanation:
//!
//! #### [🪢 You can partially borrow self in methods (click to see example)](...)
//! You can call a function that takes partially borrowed fields from `&mut self` while holding
//! references to other parts of `Self`, even if it contains private fields.
//!
//! #### [👓 Partial borrows make your code more readable and less error-prone (click to see example).](...)
//! They allow you to drastically shorten function signatures and their usage places. They also
//! enable you to keep the code unchanged, e.g., after adding a new field to a struct, instead of
//! manually refactoring in potentially many places.
//!
//! #### [🚀 Partial borrows make your code faster (click to see example).](...)
//! because passing a single partial reference produces more optimized code than passing many
//! references in separate arguments.
//!
//! <br/>
//! <br/>
//!
//! # 📖 Other literature
//!
//! In real-world applications, lack of partial borrows often affects API design, making code hard
//! to maintain and understand. This issue was described multiple times over the years, some of the
//! most notable discussions include:
//!
//! In real-world applications, the lack of partial borrows often affects API design, making code
//! hard to maintain and understand. This issue has been described multiple times over the years.
//! Some of the most notable discussions include:
//!
//! - [Rust Internals "Notes on partial borrow"](https://internals.rust-lang.org/t/notes-on-partial-borrows/20020).
//! - [The Rustonomicon "Splitting Borrows"](https://doc.rust-lang.org/nomicon/borrow-splitting.html).
//! - [Niko Matsakis Blog Post "After NLL: Interprocedural conflicts"](https://smallcultfollowing.com/babysteps/blog/2018/11/01/after-nll-interprocedural-conflicts/).
//! - [Afternoon Rusting "Multiple Mutable References"](https://oribenshir.github.io/afternoon_rusting/blog/mutable-reference).
//! - [Partial borrows Rust RFC](https://github.com/rust-lang/rfcs/issues/1215#issuecomment-333316998).
//! - [HackMD "My thoughts on (and need for) partial borrows"](https://hackmd.io/J5aGp1ptT46lqLmPVVOxzg?view).
//!
//! <br/>
//! <br/>
//!
//! # 📖 `borrow::Partial` derive macro
//!
//! This crate provides the `borrow::Partial` derive macro, which lets your structs be borrowed
//! partially.
//!
//! <details>
//! <summary>⚠️ Some code was collapsed for brevity, click to expand.</summary>
//!
//! ```
//! use std::vec::Vec;
//!
//! // ============
//! // === Data ===
//! // ============
//!
//! type NodeId = usize;
//! type EdgeId = usize;
//!
//! struct Node {
//!    outputs: Vec<EdgeId>,
//!    inputs:  Vec<EdgeId>,
//! }
//!
//! struct Edge {
//!    from: Option<NodeId>,
//!    to:   Option<NodeId>,
//! }
//!
//! struct Group {
//!    nodes: Vec<NodeId>,
//! }
//!
//! // =============
//! // === Graph ===
//! // =============
//! ```
//!
//! </details>
//!
//! ```
//! # use std::vec::Vec;
//! #
//! # type NodeId = usize;
//! # type EdgeId = usize;
//! #
//! # struct Node {
//! #    outputs: Vec<EdgeId>,
//! #    inputs:  Vec<EdgeId>,
//! # }
//! #
//! # struct Edge {
//! #    from: Option<NodeId>,
//! #    to:   Option<NodeId>,
//! # }
//! #
//! # struct Group {
//! #    nodes: Vec<NodeId>,
//! # }
//! #
//! # fn main() {}
//! #
//! #[derive(borrow::Partial)]
//! #[module(crate)]
//! struct Graph {
//!    pub nodes:  Vec<Node>,
//!    pub edges:  Vec<Edge>,
//!    pub groups: Vec<Group>,
//! }
//! ```
//!
//! The most important code that this macro generates is:
//!
//! ```
//! # pub struct Graph;
//! # pub struct Node;
//! # pub struct Edge;
//! # pub struct Group;
//! #
//! pub struct GraphRef<Nodes, Edges, Groups> {
//!     pub nodes:  Nodes,
//!     pub edges:  Edges,
//!     pub groups: Groups,
//! }
//!
//! impl Graph {
//!     pub fn as_refs_mut(&mut self) ->
//!         GraphRef<
//!             &mut Vec<Node>,
//!             &mut Vec<Edge>,
//!             &mut Vec<Group>,
//!         > {
//!         // ...
//!         # panic!()
//!     }
//! }
//! ```
//!
//! All partial borrows of the `Graph` struct will be represented as `&mut GraphRef<...>` with type
//! parameters instantiated to one of `&T`, `&mut T`, or `Hidden<T>`, a marker for fields
//! inaccessible in the current borrow.
//!
//! <sub></sub>
//!
//! <div class="warning">
//!
//! Please note the usage of the `#[module(...)]` attribute, which specifies the path to the module
//! where the macro is invoked. This attribute is necessary because Rust does not allow procedural
//! macros to automatically detect the path of the module they are used in.
//!
//! If you intend to use the generated macro from another crate, avoid using the `crate::` prefix
//! in the `#[module(...)]` attribute. Instead, refer to your current crate by its name, for
//! example: `#[module(my_crate::data)]` and add `extern crate self as my_crate;` to your `lib.rs`
//! / `main.rs`.
//!
//! </div>
//!
//! <br/>
//! <br/>
//!
//! # 📖 `borrow::partial` (`p!`) macro
//!
//! This crate provides the `borrow::partial` macro, which we recommend importing under a shorter
//! alias `p` for concise syntax. The macro allows you to parameterize borrows similarly to how you
//! parameterize types. Let's see how the macro expansion works:
//!
//! ```
//! // Given:
//! # use std::vec::Vec;
//! # use borrow::partial as p;
//! # use borrow::Hidden;
//! #
//! # struct Node;
//! # struct Edge;
//! # struct Group;
//! #
//! # #[derive(borrow::Partial)]
//! # #[module(crate)]
//! # struct Graph {
//! #   pub nodes:  Vec<Node>,
//! #   pub edges:  Vec<Edge>,
//! #   pub groups: Vec<Group>,
//! # }
//! #
//! # fn main() {}
//! #
//! fn test1(graph: p!(&<nodes, mut edges> Graph)) {}
//!
//! // It will expand to:
//! fn test2(graph: &mut p!(<nodes, mut edges> Graph)) {}
//!
//! // Which will expand to:
//! fn test3(graph: &mut GraphRef<
//!    &Vec<Node>,
//!    &mut Vec<Edge>,
//!    Hidden<Vec<Group>>
//! >) {}
//! ```
//!
//! <sub></sub>
//!
//! The macro implements the syntax proposed in
//! [Rust Internals "Notes on partial borrow"](https://internals.rust-lang.org/t/notes-on-partial-borrows/20020),
//! extended with utilities for increased expressiveness:
//!
//! <sub></sub>
//!
//! 1. **Field References**
//!    You can parameterize a reference by providing field names this reference should contain.
//!
//!    ```
//!    # use std::vec::Vec;
//!    # use borrow::partial as p;
//!    # use borrow::Hidden;
//!    #
//!    # struct Node;
//!    # struct Edge;
//!    # struct Group;
//!    #
//!    # #[derive(borrow::Partial)]
//!    # #[module(crate)]
//!    # struct Graph {
//!    #   pub nodes:  Vec<Node>,
//!    #   pub edges:  Vec<Edge>,
//!    #   pub groups: Vec<Group>,
//!    # }
//!    #
//!    # fn main() {}
//!    #
//!    // Contains:
//!    // 1. Immutable reference to the 'nodes' field.
//!    // 2. Mutable reference to the 'edges' field.
//!    fn test(graph: p!(&<nodes, mut edges> Graph)) { /* ... */ }
//!    ```
//!
//!    <sub></sub>
//!
//! 2. **Field Selectors**
//!    You can use `*` to include all fields and `!` to exclude fields. Later selectors override
//!    previous ones.
//!
//!    ```
//!    # use std::vec::Vec;
//!    # use borrow::partial as p;
//!    # use borrow::Hidden;
//!    #
//!    # struct Node;
//!    # struct Edge;
//!    # struct Group;
//!    #
//!    # #[derive(borrow::Partial)]
//!    # #[module(crate)]
//!    # struct Graph {
//!    #   pub nodes:  Vec<Node>,
//!    #   pub edges:  Vec<Edge>,
//!    #   pub groups: Vec<Group>,
//!    # }
//!    #
//!    # fn main() {}
//!    #
//!    // Contains:
//!    // 1. Mutable references to all, but 'edges' and 'groups' fields.
//!    // 2. Immutable reference to the 'edges' field.
//!    fn test(graph: p!(&<mut *, edges, !groups> Graph)) { /* ... */ }
//!    ```
//!
//!    <sub></sub>
//!
//! 3. **Lifetime Annotations**
//!    You can specify lifetimes for each reference. If a lifetime is not provided, it defaults to
//!    `'_`. You can override the default lifetime (`'_`) by providing it as the first argument.
//!
//!    ```
//!    # use std::vec::Vec;
//!    # use borrow::partial as p;
//!    # use borrow::Hidden;
//!    #
//!    # struct Node;
//!    # struct Edge;
//!    # struct Group;
//!    #
//!    # #[derive(borrow::Partial)]
//!    # #[module(crate)]
//!    # struct Graph {
//!    #   pub nodes:  Vec<Node>,
//!    #   pub edges:  Vec<Edge>,
//!    #   pub groups: Vec<Group>,
//!    # }
//!    #
//!    # fn main() {}
//!    #
//!    // Contains:
//!    // 1. References with the 'b lifetime to all but the 'mesh' fields.
//!    // 2. Reference with the 'c lifetime to the 'edges' field.
//!    //
//!    // Due to explicit partial reference lifetime 'a, the inferred
//!    // lifetime dependencies are 'a:'b and 'a:'c.
//!    fn test<'a, 'b, 'c>(graph: p!(&'a <'b *, 'c edges>Graph)) { /* ... */ }
//!
//!    // Contains:
//!    // 1. Reference with the 't lifetime to the 'nodes' field.
//!    // 2. Reference with the 't lifetime to the 'edges' field.
//!    // 3. Reference with the 'm lifetime to the 'groups' field.
//!    type PathFind<'t, 'm> = p!(<'t, nodes, edges, 'm groups> Graph);
//!    ```
//!
//! <br/>
//! <br/>
//!
//! # 📖 The `partial_borrow`, `split`, and `extract_$field` methods.
//!
//! The `borrow::Partial` derive macro also generates the `partial_borrow`, `split`, and an
//! extraction method per struct field. These methods let you transform one partial borrow
//! into another:
//!
//! <sub></sub>
//!
//! - `partial_borrow` lets you borrow only the fields required by the target type.
//!    ```
//!    # use std::vec::Vec;
//!    # use borrow::partial as p;
//!    # use borrow::traits::*;
//!    #
//!    # struct Node;
//!    # struct Edge;
//!    # struct Group;
//!    #
//!    # #[derive(borrow::Partial)]
//!    # #[module(crate)]
//!    # struct Graph {
//!    #   pub nodes:  Vec<Node>,
//!    #   pub edges:  Vec<Edge>,
//!    #   pub groups: Vec<Group>,
//!    # }
//!    #
//!    # fn main() {}
//!    #
//!    fn test(graph: p!(&<mut *> Graph)) {
//!        let graph2 = graph.partial_borrow::<p!(<mut nodes> Graph)>();
//!    }
//!    ```
//!
//!    <sub></sub>
//!
//! - `split` is like `partial_borrow` but also returns a borrow of the remaining fields.
//!    ```
//!    # use std::vec::Vec;
//!    # use borrow::partial as p;
//!    # use borrow::traits::*;
//!    #
//!    # struct Node;
//!    # struct Edge;
//!    # struct Group;
//!    #
//!    # #[derive(borrow::Partial)]
//!    # #[module(crate)]
//!    # struct Graph {
//!    #   pub nodes:  Vec<Node>,
//!    #   pub edges:  Vec<Edge>,
//!    #   pub groups: Vec<Group>,
//!    # }
//!    #
//!    # fn main() {}
//!    #
//!    fn test(graph: p!(&<mut *> Graph)) {
//!        // The inferred type of `graph3` is `p!(&<mut *, !nodes> Graph)`,
//!        // which expands to `p!(&<mut edges, mut groups> Graph)`
//!        let graph2 = graph.partial_borrow::<p!(<mut nodes> Graph)>();
//!    }
//!    ```
//!
//!    <sub></sub>
//!
//! - `extract_$field` is like split, but for single field only.
//!    ```
//!    # use std::vec::Vec;
//!    # use borrow::partial as p;
//!    #
//!    # struct Node;
//!    # struct Edge;
//!    # struct Group;
//!    #
//!    # #[derive(borrow::Partial)]
//!    # #[module(crate)]
//!    # struct Graph {
//!    #   pub nodes:  Vec<Node>,
//!    #   pub edges:  Vec<Edge>,
//!    #   pub groups: Vec<Group>,
//!    # }
//!    #
//!    # fn main() {}
//!    #
//!    fn test(graph: p!(&<mut *> Graph)) {
//!        // The inferred type of `nodes` is `p!(&<mut nodes> Graph)`.
//!        // The inferred type of `graph2` is `p!(&<mut *, !nodes> Graph)`.
//!        let (nodes, graph2) = graph.extract_nodes();
//!    }
//!    ```
//!
//! <sub></sub>
//!
//! The following example demonstrates usage of these functions. Read the comments in the code to
//! learn more. You can also find this example in the `tests` directory.
//!
//! <details>
//! <summary>⚠️ Some code was collapsed for brevity, click to expand.</summary>
//!
//! ```
//! use std::vec::Vec;
//! use borrow::partial as p;
//! use borrow::traits::*;
//!
//! // ============
//! // === Data ===
//! // ============
//!
//! type NodeId = usize;
//! type EdgeId = usize;
//!
//! #[derive(Debug)]
//! struct Node {
//!     outputs: Vec<EdgeId>,
//!     inputs:  Vec<EdgeId>,
//! }
//!
//! #[derive(Debug)]
//! struct Edge {
//!     from: Option<NodeId>,
//!     to:   Option<NodeId>,
//! }
//!
//! #[derive(Debug)]
//! struct Group {
//!     nodes: Vec<NodeId>,
//! }
//! ```
//!
//! </details>
//!
//! ```
//! # use std::vec::Vec;
//! # use borrow::partial as p;
//! # use borrow::traits::*;
//! #
//! # // ============
//! # // === Data ===
//! # // ============
//! #
//! # type NodeId = usize;
//! # type EdgeId = usize;
//! #
//! # #[derive(Debug)]
//! # struct Node {
//! #     outputs: Vec<EdgeId>,
//! #     inputs:  Vec<EdgeId>,
//! # }
//! #
//! # #[derive(Debug)]
//! # struct Edge {
//! #     from: Option<NodeId>,
//! #     to:   Option<NodeId>,
//! # }
//! #
//! # #[derive(Debug)]
//! # struct Group {
//! #     nodes: Vec<NodeId>,
//! # }
//! #
//! // =============
//! // === Graph ===
//! // =============
//!
//! #[derive(Debug, borrow::Partial)]
//! #[module(crate)]
//! struct Graph {
//!     nodes:  Vec<Node>,
//!     edges:  Vec<Edge>,
//!     groups: Vec<Group>,
//! }
//!
//! // =============
//! // === Utils ===
//! // =============
//!
//! // Requires mutable access to the `graph.edges` field.
//! fn detach_node(graph: p!(&<mut edges> Graph), node: &mut Node) {
//!     for edge_id in std::mem::take(&mut node.outputs) {
//!         graph.edges[edge_id].from = None;
//!     }
//!     for edge_id in std::mem::take(&mut node.inputs) {
//!         graph.edges[edge_id].to = None;
//!     }
//! }
//!
//! // Requires mutable access to all `graph` fields.
//! fn detach_all_nodes(graph: p!(&<mut *> Graph)) {
//!     // Extract the `nodes` field.
//!     // The `graph2` variable has a type of `p!(&<mut *, !nodes> Graph)`.
//!     let (nodes, graph2) = graph.extract_nodes();
//!     for node in nodes {
//!         detach_node(graph2.partial_borrow(), node);
//!     }
//! }
//!
//! // =============
//! // === Tests ===
//! // =============
//!
//! fn main() {
//!    // node0 -----> node1 -----> node2 -----> node0
//!    //       edge0        edge1        edge2
//!     let mut graph = Graph {
//!         nodes: vec![
//!             Node { outputs: vec![0], inputs: vec![2] }, // Node 0
//!             Node { outputs: vec![1], inputs: vec![0] }, // Node 1
//!             Node { outputs: vec![2], inputs: vec![1] }, // Node 2
//!         ],
//!         edges: vec![
//!             Edge { from: Some(0), to: Some(1) }, // Edge 0
//!             Edge { from: Some(1), to: Some(2) }, // Edge 1
//!             Edge { from: Some(2), to: Some(0) }, // Edge 2
//!         ],
//!         groups: vec![]
//!     };
//!
//!     detach_all_nodes(&mut graph.as_refs_mut());
//!
//!     for node in &graph.nodes {
//!         assert!(node.outputs.is_empty());
//!         assert!(node.inputs.is_empty());
//!     }
//!     for edge in &graph.edges {
//!         assert!(edge.from.is_none());
//!         assert!(edge.to.is_none());
//!     }
//! }
//! ```
//!
//! <br/>
//! <br/>
//!
//! # Partial borrows of self in methods
//!
//! The above example can be rewritten to use partial borrows of `self` in methods.
//!
//!
//! <details>
//! <summary>⚠️ Some code was collapsed for brevity, click to expand.</summary>
//!
//! ```
//! use std::vec::Vec;
//! use borrow::partial as p;
//! use borrow::traits::*;
//!
//! // ============
//! // === Data ===
//! // ============
//!
//! type NodeId = usize;
//! type EdgeId = usize;
//!
//! #[derive(Debug)]
//! struct Node {
//! outputs: Vec<EdgeId>,
//! inputs:  Vec<EdgeId>,
//! }
//!
//! #[derive(Debug)]
//! struct Edge {
//! from: Option<NodeId>,
//! to:   Option<NodeId>,
//! }
//!
//! #[derive(Debug)]
//! struct Group {
//! nodes: Vec<NodeId>,
//! }
//!
//! // =============
//! // === Graph ===
//! // =============
//!
//! #[derive(Debug, borrow::Partial)]
//! #[module(crate)]
//! struct Graph {
//!    nodes: Vec<Node>,
//!    edges: Vec<Edge>,
//!    groups: Vec<Group>,
//! }
//! #
//! # fn main() {}
//! ```
//!
//! </details>
//!
//! ```
//! # use std::vec::Vec;
//! # use borrow::partial as p;
//! # use borrow::traits::*;
//! #
//! # // ============
//! # // === Data ===
//! # // ============
//! #
//! # type NodeId = usize;
//! # type EdgeId = usize;
//! #
//! # #[derive(Debug)]
//! # struct Node {
//! #     outputs: Vec<EdgeId>,
//! #     inputs:  Vec<EdgeId>,
//! # }
//! #
//! # #[derive(Debug)]
//! # struct Edge {
//! #     from: Option<NodeId>,
//! #     to:   Option<NodeId>,
//! # }
//! #
//! # #[derive(Debug)]
//! # struct Group {
//! #     nodes: Vec<NodeId>,
//! # }
//! #
//! # // =============
//! # // === Graph ===
//! # // =============
//! #
//! # #[derive(Debug, borrow::Partial)]
//! # #[module(crate)]
//! # struct Graph {
//! #    nodes: Vec<Node>,
//! #    edges: Vec<Edge>,
//! #    groups: Vec<Group>,
//! # }
//! #
//! # fn main() {}
//! #
//! impl p!(<mut edges, mut nodes> Graph) {
//!     fn detach_all_nodes(&mut self) {
//!         let (nodes, self2) = self.extract_nodes();
//!         for node in nodes {
//!             self2.detach_node(node);
//!         }
//!     }
//! }
//!
//! impl p!(<mut edges> Graph) {
//!     fn detach_node(&mut self, node: &mut Node) {
//!         for edge_id in std::mem::take(&mut node.outputs) {
//!             self.edges[edge_id].from = None;
//!         }
//!         for edge_id in std::mem::take(&mut node.inputs) {
//!             self.edges[edge_id].to = None;
//!         }
//!     }
//! }
//! ```
//!
//!
//! <details>
//! <summary>⚠️ Some code was collapsed for brevity, click to expand.</summary>
//!
//! ```
//! # use std::vec::Vec;
//! # use borrow::partial as p;
//! # use borrow::traits::*;
//! #
//! # // ============
//! # // === Data ===
//! # // ============
//! #
//! # type NodeId = usize;
//! # type EdgeId = usize;
//! #
//! # #[derive(Debug)]
//! # struct Node {
//! #     outputs: Vec<EdgeId>,
//! #     inputs:  Vec<EdgeId>,
//! # }
//! #
//! # #[derive(Debug)]
//! # struct Edge {
//! #     from: Option<NodeId>,
//! #     to:   Option<NodeId>,
//! # }
//! #
//! # #[derive(Debug)]
//! # struct Group {
//! #     nodes: Vec<NodeId>,
//! # }
//! #
//! # // =============
//! # // === Graph ===
//! # // =============
//! #
//! # #[derive(Debug, borrow::Partial)]
//! # #[module(crate)]
//! # struct Graph {
//! #    nodes: Vec<Node>,
//! #    edges: Vec<Edge>,
//! #    groups: Vec<Group>,
//! # }
//! #
//! # impl p!(<mut edges, mut nodes> Graph) {
//! #     fn detach_all_nodes(&mut self) {
//! #         let (nodes, self2) = self.extract_nodes();
//! #         for node in nodes {
//! #             self2.detach_node(node);
//! #         }
//! #     }
//! # }
//! #
//! # impl p!(<mut edges> Graph) {
//! #     fn detach_node(&mut self, node: &mut Node) {
//! #         for edge_id in std::mem::take(&mut node.outputs) {
//! #             self.edges[edge_id].from = None;
//! #         }
//! #         for edge_id in std::mem::take(&mut node.inputs) {
//! #             self.edges[edge_id].to = None;
//! #         }
//! #     }
//! # }
//! #
//! // =============
//! // === Tests ===
//! // =============
//!
//! fn main() {
//!    // node0 -----> node1 -----> node2 -----> node0
//!    //       edge0        edge1        edge2
//!    let mut graph = Graph {
//!       nodes: vec![
//!          Node { outputs: vec![0], inputs: vec![2] }, // Node 0
//!          Node { outputs: vec![1], inputs: vec![0] }, // Node 1
//!          Node { outputs: vec![2], inputs: vec![1] }, // Node 2
//!       ],
//!       edges: vec![
//!          Edge { from: Some(0), to: Some(1) }, // Edge 0
//!          Edge { from: Some(1), to: Some(2) }, // Edge 1
//!          Edge { from: Some(2), to: Some(0) }, // Edge 2
//!       ],
//!       groups: vec![],
//!    };
//!
//!    graph.as_refs_mut().partial_borrow().detach_all_nodes();
//!
//!    for node in &graph.nodes {
//!       assert!(node.outputs.is_empty());
//!       assert!(node.inputs.is_empty());
//!    }
//!    for edge in &graph.edges {
//!       assert!(edge.from.is_none());
//!       assert!(edge.to.is_none());
//!    }
//! }
//! ```
//!
//! </details>
//!
//! Please note, that you do not need to provide the partially borrowed type explicitly, it will be
//! inferred automatically. For example, the `detach_all_nodes` method requires self to have the
//! `edges` and `nodes` fields mutably borrowed, but you can simply call it as follows:
//!
//! ```
//! # use std::vec::Vec;
//! # use borrow::partial as p;
//! # use borrow::traits::*;
//! #
//! # #[derive(Default, borrow::Partial)]
//! # #[module(crate)]
//! # struct Graph {
//! #     nodes: Vec<usize>,
//! #     edges: Vec<usize>,
//! # }
//! #
//! # impl p!(<mut nodes> Graph) {
//! #     fn detach_all_nodes(&mut self) {}
//! # }
//! #
//! fn main() {
//!    let mut graph: Graph = Graph::default();
//!    let mut graph_ref: p!(<mut *>Graph) = graph.as_refs_mut();
//!    graph_ref.partial_borrow().detach_all_nodes();
//! }
//! ```
//!
//! <br/>
//! <br/>
//!
//! # Why identity partial borrow is disallowed?
//! Please note, that the `partial_borrow` method does not allow you to request the same fields as
//! the original borrow. This is to enforce the code to be explicit and easy to understand:
//!
//! <sub></sub>
//!
//! 1. Whenever you see the call to `partial_borrow`, you can be sure that target borrow uses
//!    subset of fields from the original borrow:
//!    ```ignore
//!    # use std::vec::Vec;
//!    # use borrow::partial as p;
//!    # use borrow::traits::*;
//!    #
//!    #    #[derive(Default, borrow::Partial)]
//!    #    #[module(crate)]
//!    # struct Graph {
//!    #     nodes: Vec<usize>,
//!    #     edges: Vec<usize>,
//!    # }
//!    #
//!    # impl p!(<mut nodes> Graph) {
//!    #     fn detach_all_nodes(&mut self) {}
//!    # }
//!    #
//!    # fn main() {}
//!    #
//!    fn run(graph: p!(&<mut nodes, mut edges> Graph)) {
//!        // ERROR: Cannot partially borrow the same fields as the original borrow.
//!        // Instead, you should pass `graph` directly as `test(graph)`.
//!        test(graph.partial_borrow())
//!    }
//!
//!    fn test(graph: p!(&<mut nodes, mut edges> Graph)) { /* ... */ }
//!    ```
//!
//! <sub></sub>
//!
//! 2. If you refactor your code and the new version does not require all field references it used
//!    to require, you will get compilation errors in all usage places that were assuming the full
//!    usage. This allows you to easily review the places that either need to introduce a new
//!    partial borrow or need to update their type signatures:
//!    ```
//!    # use std::vec::Vec;
//!    # use borrow::partial as p;
//!    # use borrow::traits::*;
//!    #
//!    #    #[derive(Default, borrow::Partial)]
//!    #    #[module(crate)]
//!    # struct Graph {
//!    #     nodes: Vec<usize>,
//!    #     edges: Vec<usize>,
//!    # }
//!    #
//!    # impl p!(<mut nodes> Graph) {
//!    #     fn detach_all_nodes(&mut self) {}
//!    # }
//!    #
//!    # fn main() {}
//!    #
//!    fn run(graph: p!(&<mut nodes, mut edges> Graph)) {
//!        test(graph)
//!    }
//!
//!    // Changing this signature to `test(graph: p!(&<mut nodes> Graph))` would
//!    // cause a compilation error in the `main` function, as the required borrow
//!    // is smaller than the one provided. There are two possible solutions:
//!    // 1. Change the call site to `test(graph.partial_borrow())`.
//!    // 2. Change the `main` function signature to reflect the new requirements:
//!    //    `main(graph: p!(&<mut nodes> Graph))`.
//!    fn test(graph: p!(&<mut nodes, mut edges> Graph)) { /* ... */ }
//!    ```
//!
//! <sub></sub>
//!
//! 3. In case you want to opt-out from this check, there is also a `partial_borrow_or_identity`
//! method that does not perform this compile-time check. However, we recommend using it only in
//! exceptional cases, as it may lead to confusion and harder-to-maintain code.
//!
//! <br/>
//! <br/>
//!
//! # ⚠️ Limitations
//!
//! Currently, the macro works only with non-parametrized structures. For parametrized structures,
//! please create an issue or submit a pull request.
//!
//! <br/>
//! <br/>


pub mod doc;
pub mod hlist;
pub mod reflect;

use hlist::Cons;
use hlist::Nil;
use std::fmt::Debug;

pub use reflect::*;
pub use borrow_macro::*;


// ==============
// === Traits ===
// ==============

pub mod traits {
    pub use super::Acquire as _;
    pub use super::Partial as _;
    pub use super::PartialHelper as _;
    pub use super::RefCast as _;
    pub use super::AsRefs as _;
    pub use super::AsRefsHelper as _;
}


// ==============
// === AsRefs ===
// ==============

/// Borrow all fields of a struct and output a partially borrowed struct,
/// like `p!(<mut field1, field2>MyStruct)`.
pub trait AsRefs<'t, T> {
    fn as_refs_impl(&'t mut self) -> T;
}

impl<'t, T> AsRefsHelper<'t> for T {}
pub trait AsRefsHelper<'t> {
    /// Borrow all fields of a struct and output a partially borrowed struct,
    /// like `p!(<mut field1, field2>MyStruct)`.
    #[inline(always)]
    fn as_refs<T>(&'t mut self) -> T
    where Self: AsRefs<'t, T> { self.as_refs_impl() }
}


// =========================
// === No Access Wrapper ===
// =========================

/// A phantom type used to mark fields as hidden in the partially borrowed structs.
#[repr(transparent)]
#[derive(Debug)]
pub struct Hidden<T>(*mut T);

impl<T> Copy for Hidden<T> {}
impl<T> Clone for Hidden<T> {
    fn clone(&self) -> Self { Self(self.0) }
}


// ===============
// === RefCast ===
// ===============

pub trait RefCast<'t, T> {
    /// All possible casts of a mutable reference: `&mut T` (identity), `&T`, and `Hidden<T>`.
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


// ===============
// === Acquire ===
// ===============


/// This is a documentation for type-level field borrowing transformation. It involves checking if a
/// field of a partially borrowed struct can be borrowed in a specific form and provides the remaining
/// fields post-borrow.
pub trait           Acquire<Target>                  { type Rest; }
impl<'t, T, S>      Acquire<Hidden<T>> for S         { type Rest = S; }
impl<'t: 's, 's, T> Acquire<&'s mut T> for &'t mut T { type Rest = Hidden<T>; }
impl<'t: 's, 's, T> Acquire<&'s     T> for &'t mut T { type Rest = &'t T; }
impl<'t: 's, 's, T> Acquire<&'s     T> for &'t     T { type Rest = &'t T; }

/// Remaining fields after borrowing a specific field. See the documentation of [`Acquire`] to learn more.
pub type Acquired<This, Target> = <This as Acquire<Target>>::Rest;


// ===================
// === SplitFields ===
// ===================

/// Split HList of borrows into target HList of borrows and a HList of remaining borrows after
/// acquiring the target. See the documentation of [`Acquire`] for more information.
///
/// This trait is automatically implemented for all types.
pub trait          SplitFields<Target>               { type Rest; }
impl               SplitFields<Nil>          for Nil { type Rest = Nil; }
impl<H, H2, T, T2> SplitFields<Cons<H2, T2>> for Cons<H, T> where
T: SplitFields<T2>, H: Acquire<H2> {
    type Rest = Cons<Acquired<H, H2>, <T as SplitFields<T2>>::Rest>;
}

type SplitFieldsRest<T, Target> = <T as SplitFields<Target>>::Rest;


// ===============
// === Partial ===
// ===============

/// Helper trait for [`Partial`]. This trait is automatically implemented by the [`partial_borrow!`]
/// macro. It is used to provide Rust type inferencer with additional type information. In particular, it
/// is used to tell that any partial borrow of a struct results in the same struct type, but parametrized
/// differently. It is needed for Rust to correctly infer target types for associated methods, like:
///
/// ```ignore
/// #[derive(Partial)]
/// #[module(crate)]
/// pub struct Ctx {
///     pub geometry: GeometryCtx,
///     pub material: MaterialCtx,
///     pub mesh: MeshCtx,
///     pub scene: SceneCtx,
/// }
///
/// impl p!(<mut geometry, mut material>Ctx) {
///     fn my_method(&mut self){}
/// }
///
/// fn test(ctx: p!(&<mut *> Ctx)) {
///     ctx.partial_borrow().my_method();
/// }
/// ```
pub trait PartialInferenceGuide<Target> {}

/// Implementation of partial field borrowing. The `Target` type parameter specifies the required
/// partial borrow representation, such as `p!(<mut field1, field2>MyStruct)`.
///
/// This trait is automatically implemented for all partial borrow representations.
pub trait Partial<Target> : PartialInferenceGuide<Target> {
    type Rest;

    /// See the documentation of [`PartialHelper::partial_borrow`].
    #[inline(always)]
    fn partial_borrow_impl(&mut self) -> &mut Target {
        unsafe { &mut *(self as *mut _ as *mut _) }
    }

    /// See the documentation of [`PartialHelper::split`].
    #[inline(always)]
    fn split_impl(&mut self) -> (&mut Target, &mut Self::Rest) {
        let a = unsafe { &mut *(self as *mut _ as *mut _) };
        let b = unsafe { &mut *(self as *mut _ as *mut _) };
        (a, b)
    }
}

impl<Source, Target> Partial<Target> for Source where
Source: PartialInferenceGuide<Target>,
Source: HasFields,
Target: HasFields,
Fields<Source>: SplitFields<Fields<Target>>,
Target: ReplaceFields<SplitFieldsRest<Fields<Source>, Fields<Target>>> {
    type Rest = ReplacedFields<Target, SplitFieldsRest<Fields<Source>, Fields<Target>>>;
}

/// Helper for [`Partial`]. This trait is automatically implemented for all types.
impl<Target> PartialHelper for Target {}
pub trait PartialHelper {
    /// Borrow fields from this partial borrow for the `Target` partial borrow, like
    /// `ctx.partial_borrow::<p!(<mut scene>Ctx)>()`.
    #[inline(always)]
    fn partial_borrow<Target>(&mut self) -> &mut Target
    where Self: PartialNotEq<Target> { self.partial_borrow_impl() }

    /// Borrow fields from this partial borrow for the `Target` partial borrow, like
    /// `ctx.partial_borrow::<p!(<mut scene>Ctx)>()`.
    #[inline(always)]
    fn partial_borrow_or_eq<Target>(&mut self) -> &mut Target
    where Self: Partial<Target> { self.partial_borrow_impl() }

    /// Split this partial borrow into the `Target` partial borrow and the remaining fields, like
    /// `let (scene, ctx2) = ctx.split::<p!(<mut scene>Ctx)>()`.
    #[inline(always)]
    fn split<Target>(&mut self) -> (&mut Target, &mut Self::Rest)
    where Self: Partial<Target> { self.split_impl() }
}


// ====================
// === PartialNotEq ===
// ====================

pub trait PartialNotEq<Target> : Partial<Target> + NotEq<Target> {}
impl<Target, T> PartialNotEq<Target> for T where T: Partial<Target> + NotEq<Target> {}


// =============
// === NotEq ===
// =============

pub trait NotEq<Target> {}
impl<Source, Target> NotEq<Target> for Source where
    Source: HasFields,
    Target: HasFields,
    Fields<Source>: NotEqFields<Fields<Target>> {
}

pub trait NotEqFields<Target> {}
impl<    't, H, T, T2> NotEqFields<Cons<&'t mut H, T>> for Cons<Hidden<H>, T2> {}
impl<    't, H, T, T2> NotEqFields<Cons<&'t     H, T>> for Cons<Hidden<H>, T2> {}
impl<        H, T, T2> NotEqFields<Cons<Hidden<H>, T>> for Cons<Hidden<H>, T2> where T: NotEqFields<T2> {}

impl<    't, H, T, T2> NotEqFields<Cons<Hidden<H>, T>> for Cons<&'t mut H, T2> {}
impl<'s, 't, H, T, T2> NotEqFields<Cons<&'s     H, T>> for Cons<&'t mut H, T2> {}
impl<'s, 't, H, T, T2> NotEqFields<Cons<&'s mut H, T>> for Cons<&'t mut H, T2> where T: NotEqFields<T2> {}

impl<    't, H, T, T2> NotEqFields<Cons<Hidden<H>, T>> for Cons<&'t H, T2> {}
impl<'s, 't, H, T, T2> NotEqFields<Cons<&'s mut H, T>> for Cons<&'t H, T2> {}
impl<'s, 't, H, T, T2> NotEqFields<Cons<&'s     H, T>> for Cons<&'t H, T2> where T: NotEqFields<T2> {}


// ==================
// === UnifyField ===
// ==================

pub trait UnifyField<Other> { type Result; }

impl<'t, T> UnifyField<Hidden<T>> for Hidden<T> { type Result = Hidden<T>; }
impl<'t, T> UnifyField<&'t     T> for Hidden<T> { type Result = &'t     T; }
impl<'t, T> UnifyField<&'t mut T> for Hidden<T> { type Result = &'t mut T; }

impl<'t, T> UnifyField<Hidden<T>> for &'t T { type Result = &'t     T; }
impl<'t, T> UnifyField<&'t     T> for &'t T { type Result = &'t     T; }
impl<'t, T> UnifyField<&'t mut T> for &'t T { type Result = &'t mut T; }

impl<'t, T> UnifyField<Hidden<T>> for &'t mut T { type Result = &'t mut T; }
impl<'t, T> UnifyField<&'t     T> for &'t mut T { type Result = &'t mut T; }
impl<'t, T> UnifyField<&'t mut T> for &'t mut T { type Result = &'t mut T; }

type ConcatenatedField<T, Other> = <T as UnifyField<Other>>::Result;


// ====================
// === UnifyFields ===
// ====================

pub trait UnifyFields<Other> { type Result; }
type ConcatFieldsResult<T, Other> = <T as UnifyFields<Other>>::Result;

impl UnifyFields<Nil> for Nil {
    type Result = Nil;
}

impl<H, H2, T, T2> UnifyFields<Cons<H2, T2>> for Cons<H, T> where
    H: UnifyField<H2>,
    T: UnifyFields<T2> {
    type Result = Cons<ConcatenatedField<H, H2>, <T as UnifyFields<T2>>::Result>;
}

pub trait Unify<Other> {
    type Result;
}

impl<Source, Other> Unify<Other> for Source where
    Source: HasFields,
    Other: HasFields,
    Fields<Source>: UnifyFields<Fields<Other>>,
    Other: ReplaceFields<ConcatFieldsResult<Fields<Source>, Fields<Other>>> {
    type Result = ReplacedFields<Other, ConcatFieldsResult<Fields<Source>, Fields<Other>>>;
}

pub type Union<T, Other> = <T as Unify<Other>>::Result;


// ==============
// === Macros ===
// ==============

#[macro_export]
macro_rules! lifetime_chooser {
    ($lt1:lifetime $lt2:lifetime $($ts:tt)*) => {& $lt2 $($ts)*};
    ($lt1:lifetime $($ts:tt)*) => {& $lt1 $($ts)*};
}

#[macro_export]
macro_rules! partial {
    (& $lt:lifetime $($ts:tt)*)       => { & $lt mut $crate::partial! { $($ts)* } };
    (& $($ts:tt)*)                    => { &     mut $crate::partial! { $($ts)* } };
    (< $($ts:tt)*)                    => {           $crate::partial! { @ [] $($ts)* } };
    (@ [$($xs:tt)*] > $t:ident)       => { $t! { $($xs)* } };
    (@ [$($xs:tt)*] $t:tt $($ts:tt)*) => { $crate::partial! { @ [$($xs)* $t] $($ts)* } };
}