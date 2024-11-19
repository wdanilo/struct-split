<img width="735" alt="partial3" src="https://github.com/user-attachments/assets/6203e7e2-3520-4fc8-911b-9d0dddf7ff16">

<br/>

# 🔪 Partial Borrows

Zero-overhead ["partial borrows"](https://internals.rust-lang.org/t/notes-on-partial-borrows/20020), borrows of selected fields only, **including partial self-borrows**. It lets you split structs into non-overlapping sets of mutably borrowed fields, like `&<mut field1, field2>MyStruct` and `&<field2, mut field3>MyStruct`. It is similar to [slice::split_at_mut](https://doc.rust-lang.org/std/primitive.slice.html#method.split_at_mut), but more flexible and tailored for structs.

<br/>

# 🤩 Why partial borrows? Examples included!

Partial borrows has variety of advantages. Each of the following points has a short in-line explanation with a link to an example code with detailed explanation:

### [🪢 You can partially borrow self in methods (click to see example)](...)
You can call a function that takes partially borrowed fields from `&mut self` while holding references to other parts of `Self`, even if it contains private fields.

### [👓 Partial borrows make your code more readable and less error-prone (click to see example).](...)
They allow you to drastically shorten function signatures and their usage places. They also allow you to keep the code unchanged, e.g. after adding a new field to a struct, instead of manual refactoring in potentially many places.

### [🚀 Partial borrows make your code faster (click to see example).](...) 
because passing a single partial reference produces more optimized code than passing many references in separate arguments.

<br/>

# 📖 Other literature

In real-world applications, lack of partial borrows often affects API design, making code hard to maintain and understand. This issue was described multiple times over the years, some of the most notable discussions include:

- [Rust Internals "Notes on partial borrow"](https://internals.rust-lang.org/t/notes-on-partial-borrows/20020).
- [The Rustonomicon "Splitting Borrows"](https://doc.rust-lang.org/nomicon/borrow-splitting.html).
- [Niko Matsakis Blog Post "After NLL: Interprocedural conflicts"](https://smallcultfollowing.com/babysteps/blog/2018/11/01/after-nll-interprocedural-conflicts/).
- [Afternoon Rusting "Multiple Mutable References"](https://oribenshir.github.io/afternoon_rusting/blog/mutable-reference).
- [Partial borrows Rust RFC](https://github.com/rust-lang/rfcs/issues/1215#issuecomment-333316998).
- [HackMD "My thoughts on (and need for) partial borrows"](https://hackmd.io/J5aGp1ptT46lqLmPVVOxzg?view).

<br/>

# 🖋️ `borrow::Partial` derive macro

This crate provides the `borrow::Partial` derive macro, which lets your structs be borrowed partially.

<details>
<summary>⚠️ Some code was collapsed for brevity, click to expand.</summary>

```rust
use std::vec::Vec;

// ============
// === Data ===
// ============

type NodeId = usize;
type EdgeId = usize;

struct Node {
   outputs: Vec<EdgeId>,
   inputs:  Vec<EdgeId>,
}

struct Edge {
   from: Option<NodeId>,
   to:   Option<NodeId>,
}

struct Group {
   nodes: Vec<NodeId>,
}
```
   
</details>

```rust
#[derive(borrow::Partial)]
#[module(crate)]
struct Graph {
   pub nodes:  Vec<Node>,
   pub edges:  Vec<Edge>,
   pub groups: Vec<Group>,
}
```

The most important code that this macro generates is:

```rust
pub struct GraphRef<Nodes, Edges, Groups> {
    pub nodes:  Nodes,
    pub edges:  Edges,
    pub groups: Groups,
}

impl Graph {
    pub fn as_refs_mut(&mut self) -> 
        GraphRef<
            &mut Vec<Node>,
            &mut Vec<Edge>,
            &mut Vec<Group>,
        > 
    { /* ... */ }
}
```

All partial borrows of the `Graph` struct will be represented as `&mut GraphRef<...>` with type parameters instantiated to one of `&T`, `&mut T`, or `Hidden<T>`, a marker for fields inaccessible in the current borrow.

### The `#[module(...)]` Attribute

Please note the usage of the `#[module(...)]` attribute, which specifies the path to the module where the macro is invoked. This attribute is necessary because Rust does not allow procedural macros to automatically detect the path of the module they are used in.

If you intend to use the generated macro from another crate, avoid using the `crate::` prefix in the `#[module(...)]` attribute. Instead, refer to your current crate by its name, for example: `#[module(my_crate::data)]` and add `extern crate self as my_crate;` to your `lib.rs` / `main.rs`.

<br/>

# 🖋️ `borrow::partial` (`p!`) macro

This crate provides the `borrow::partial` macro, which we recommend importing under a shorter alias `p` for concise syntax. The macro allows you to parameterize borrows similarly to how you parameterize types. Let's see how the macro expansion works:

```rust
// Given:
p!(&<nodes, mut edges> Graph)

// It will expand to:
&mut p!(<nodes, mut edges> Graph)

// Which will expand to:
&mut GraphRef<
   &Vec<Node>,
   &mut Vec<Edge>, 
   Hidden<Vec<Group>>
>
```

### Supported Syntax 
The macro implements the syntax proposed in [Rust Internals "Notes on partial borrow"](https://internals.rust-lang.org/t/notes-on-partial-borrows/20020), extended with utilities for increased expressiveness:

1. **Field References**  
   You can parameterize a reference by providing field names this reference should contain.

   ```rust
   // Contains:
   // 1. Immutable reference to the 'nodes' field.
   // 2. Mutable reference to the 'edges' field.
   fn test(graph: p!(&<nodes, mut edges> Graph)) { /* ... */ }
   ```

2. **Field Selectors**  
   You can use `..` to include all fields and `!` to exclude fields. Later selectors override previous ones.

   ```rust
   // Contains:
   // 1. Mutable references to all, but 'edges' and 'groups' fields.
   // 2. Immutable reference to the 'edges' field.
   fn test(graph: p!(&<mut .., edges, !groups> Graph)) { /* ... */ }
   ```

3. **Lifetime Annotations**  
   You can specify lifetimes for each reference. If a lifetime is not provided, it defaults to `'_`. You can override the default lifetime (`'_`) by providing it as the first argument.

   ```rust
   // Contains:
   // 1. References with the 'b lifetime to all but the 'mesh' fields.
   // 2. Reference with the 'c lifetime to the 'edges' field.
   //
   // Due to explicit partial reference lifetime 'a, the inferred
   // lifetime dependencies are 'a:'b and 'a:'c.
   fn test<'a, 'b, 'c>(graph: p!(&'a <'b .., 'c edges> Graph)) { /* ... */ }
   
   // Contains:
   // 1. Reference with the 't lifetime to the 'nodes' field.
   // 2. Reference with the 't lifetime to the 'edges' field.
   // 3. Reference with the 'm lifetime to the 'groups' field.
   type PathFind<'t, 'm> = p!(<'t, nodes, edges, 'm groups> Graph);
   ```


<br/>

# The `partial_borrow`, `split`, and `extract_$field` methods.

The `borrow::Partial` derive macro generates also the `partial_borrow`, `split`, and an extraction method per struct field. These methods let you transform one partial borrow into another:

- `partial_borrow` lets you borrow only the fields required by the target type.
   ```rust
   fn test(graph: p!(&<mut ..> Graph)) {
       let graph2 = graph.partial_borrow::<p!(<mut nodes> Graph)>();
   }
   ```

- `split` is like `partial_borrow` but also returns a borrow of the remaining fields.
   ```rust
   fn test(graph: p!(&<mut ..> Graph)) {
       // The inferred type of `graph3` is `p!(&<mut .., !nodes> Graph)`, 
       // which expands to `p!(&<mut edges, mut groups> Graph)`
       let (graph2, graph3) = graph.partial_borrow::<p!(<mut nodes> Graph)>();
   }
   ```

- `extract_$field` is like split, but for single field only.
   ```rust
   fn test(graph: p!(&<mut ..> Graph)) {
       // The inferred type of `nodes` is `p!(&<mut nodes> Graph)`.
       // The inferred type of `graph2` is `p!(&<mut .., !nodes> Graph)`.
       let (nodes, graph2) = graph.extract_nodes();
   }
   ```

The following example demonstrates usage of these functions. Read the comments in the code to learn more. You can also find this example in the `tests` directory.

<details>
<summary>⚠️ Some code was collapsed for brevity, click to expand.</summary>

```rust
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
```

</details>

```rust
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
fn detach_all_nodes(graph: p!(&<mut ..> Graph)) {
    // Extract the `nodes` field.
    // The `graph2` variable has a type of `p!(&<mut .., !nodes> Graph)`.
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
```

<br/>

# Partial borrows of self in methods

The above example can be rewritten to use partial borrows of `self` in methods.


<details>
<summary>⚠️ Some code was collapsed for brevity, click to expand.</summary>

```rust
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
   nodes: Vec<Node>,
   edges: Vec<Edge>,
   groups: Vec<Group>,
}
```

</details>

```rust
impl p!(<mut edges, mut nodes> Graph) {
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
```


<details>
<summary>⚠️ Some code was collapsed for brevity, click to expand.</summary>

```rust
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

   graph.as_refs_mut().partial_borrow().detach_all_nodes();

   for node in &graph.nodes {
      assert!(node.outputs.is_empty());
      assert!(node.inputs.is_empty());
   }
   for edge in &graph.edges {
      assert!(edge.from.is_none());
      assert!(edge.to.is_none());
   }
}
```

</details>

Please note, that you do not need to provide the partially borrowed type explicitly, it will be inferred automatically. For example, the `detach_all_nodes` method requires self to have the `edges` and `nodes` fields mutably borrowed, but you can simply call it as follows:

```rust
fn test() {
   let mut graph: Graph = /* ... */;
   let mut graph_ref: p!(<mut ..>Graph) = graph.as_refs_mut();
   graph_ref.partial_borrow().detach_all_nodes();
}
```

<br/>

# Why identity partial borrow is disallowed
Please note, that the `partial_borrow` method does not allow you to request the same fields as the original borrow. This is to enforce the code to be explicit and easy to understand:

1. Whenever you see the call to `partial_borrow`, you can be sure that target borrow uses subset of fields from the original borrow:
   ```rust
   fn main(graph: p!(&<mut nodes, mut edges> Graph)) {
       // ERROR: Cannot partially borrow the same fields as the original borrow.
       // Instead, you should pass `graph` directly as `test(graph)`.
       test(graph.partial_borrow())
   }
   
   fn test(graph: p!(&<mut nodes, mut edges> Graph)) { /* ... */ }
   ```

2. If you refactor your code and the new version does not require all field references it used to require, you will get compilation errors in all usage places that were assuming the full usage. This allows you to easily review the places that either need to introduce a new partial borrow or need to update their type signatures:
   ```rust
   fn main(graph: p!(&<mut nodes, mut edges> Graph)) {
       test(graph)
   }
   
   // Changing this signature to `test(graph: p!(&<mut nodes> Graph))` would 
   // cause a compilation error in the `main` function, as the required borrow
   // is smaller than the one provided. There are two possible solutions:
   // 1. Change the call site to `test(graph.partial_borrow())`.
   // 2. Change the `main` function signature to reflect the new requirements:
   //    `main(graph: p!(&<mut nodes> Graph))`.
   fn test(graph: p!(&<mut nodes, mut edges> Graph)) { /* ... */ }
   ```

3. In case you want to opt-out from this check, there is also a `partial_borrow_or_identity` method that does not perform this compile-time check. However, we recommend using it only in exceptional cases, as it may lead to confusion and harder-to-maintain code.

<br/>

# ⚠️ Limitations

Currently, the macro works only with non-parametrized structures. For parametrized structures, please create an issue or submit a pull request.
