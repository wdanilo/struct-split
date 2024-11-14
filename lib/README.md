<img width="735" alt="partial3" src="https://github.com/user-attachments/assets/6203e7e2-3520-4fc8-911b-9d0dddf7ff16">

<br/>

# 🔪 Partial Borrows

Zero-overhead ["partial borrows"](https://internals.rust-lang.org/t/notes-on-partial-borrows/20020), borrows of selected fields only, like `&<mut field1, mut field2>MyStruct`. It lets you split structs into non-overlapping sets of mutably borrowed fields, similar to [slice::split_at_mut](https://doc.rust-lang.org/std/primitive.slice.html#method.split_at_mut), but more flexible and tailored for structs.

<br/>

# 📌 TL;DR (Full documentation below)

If you prefer a concise guide over reading the entire README, here's a quick setup demonstrating the core concepts. Important lines are marked with "⚠️". Copy the following code into your `lib.rs` or `main.rs`. Adjust the path in the `#[module(...)]` attribute if using in a module:


```rust
#![allow(dead_code)]

use std::vec::Vec;
use borrow::PartialBorrow;
use borrow::partial_borrow as p;
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

#[derive(Debug, PartialBorrow)] // ⚠️
#[module(crate)] // ⚠️ USE HERE THE PATH TO MODULE OF THIS FILE
struct Graph {
   nodes: Vec<Node>,
   edges: Vec<Edge>,
}

// =============
// === Utils ===
// =============

// Requires mutable access to the `graph.edges` field.
fn detach_node(
   graph: p!(&<mut edges> Graph), // ⚠️
   node: &mut Node
) {
   for edge_id in std::mem::take(&mut node.outputs) {
      graph.edges[edge_id].from = None;
   }
   for edge_id in std::mem::take(&mut node.inputs) {
      graph.edges[edge_id].to = None;
   }
}

// Requires mutable access to all `graph` fields.
fn detach_all_nodes(graph: p!(&<mut *> Graph)) { // ⚠️
   // Extract the `nodes` field. The `graph2` variable has a type
   // of `p!(&<mut *, !nodes> Graph)`.
   let (nodes, graph2) = graph.extract_nodes(); // ⚠️
   for node in nodes {
      detach_node(graph2.partial_borrow(), node); // ⚠️
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
   };

   detach_all_nodes(&mut graph.as_refs_mut().partial_borrow());

   for node in &graph.nodes {
      assert!(node.outputs.is_empty() && node.inputs.is_empty());
   }
   for edge in &graph.edges {
      assert!(edge.from.is_none() && edge.to.is_none());
   }
}
```

<br/>

# 😵‍💫 What problem does it solve?

Consider a rendering engine requiring storage for geometries, materials, meshes, and scenes. These entities often form a reference graph (e.g., two meshes can use the same material). To handle this, you can either:

- Use `Rc<RefCell<...>>`/`Arc<RefCell<...>>` for shared ownership, which risks runtime errors.
- Store the entities in registries and use their indices as references. 

We opt for the latter approach and create a root registry called `Ctx`:

```rust
// === Data ===
pub struct Geometry { /* ... */ }
pub struct Material { /* ... */ }
pub struct Mesh     { 
    /// Index of the geometry in the `GeometryCtx` registry.
    pub geometry: usize,
    /// Index of the material in the `MaterialCtx` registry.
    pub material: usize 
}
pub struct Scene    { 
    /// Indexes of meshes in the `MeshCtx` registry.
    pub meshes: Vec<usize> 
}

// === Registries ===
pub struct GeometryCtx { pub data: Vec<Geometry> }
pub struct MaterialCtx { pub data: Vec<Material> }
pub struct MeshCtx     { pub data: Vec<Mesh> }
pub struct SceneCtx    { pub data: Vec<Scene> }

// === Root Registry ===
pub struct Ctx {
    pub geometry: GeometryCtx,
    pub material: MaterialCtx,
    pub mesh:     MeshCtx,
    pub scene:    SceneCtx,
    // Possibly many more fields...
}
````

Some functions require mutable access to only part of the root registry. Should they take a mutable reference to the entire `Ctx` struct, or should each field be passed separately? Passing the entire `Ctx` is inflexible and impractical. Consider the following code:

```rust
fn render_pass1(ctx: &mut Ctx) {
   for scene in &ctx.scene.data {
      for mesh in &scene.meshes {
         render_scene(ctx, *mesh)
      }
   }
   render_pass2(ctx);
}

fn render_pass2(ctx: &mut Ctx) {
   // ...
}

fn render_scene(ctx: &mut Ctx, mesh: usize) {
    // ...
}
```

At first glance, this might seem reasonable, but it will be rejected by the compiler:

```rust
Cannot borrow `*ctx` as mutable because it is also borrowed as 
immutable:

|  for scene in &ctx.scene.data {
|               ---------------
|               |
|               immutable borrow occurs here
|               immutable borrow later used here
|      for mesh in &scene.meshes {
|          render_scene(ctx, *mesh)
|          ^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
```

Passing each field separately compiles, but becomes cumbersome and error-prone as the number of fields grows:

```rust
fn render_pass1(
    geometry: &mut GeometryCtx, 
    material: &mut MaterialCtx,
    mesh:     &mut MeshCtx,
    scene:    &mut SceneCtx,
    // Possibly many more fields...
) {
    for scene in &scene.data {
        for mesh_ix in &scene.meshes {
            render_scene(
                geometry, 
                material, 
                mesh,
                // Possibly many more fields...
                *mesh_ix
            )
        }
    }
   render_pass2(
      geometry, 
      material, 
      mesh, 
      scene,
      // Possibly many more fields...
   );
}

fn render_pass2(
   geometry: &mut GeometryCtx,
   material: &mut MaterialCtx,
   mesh:     &mut MeshCtx,
   scene:    &mut SceneCtx,
   // Possibly many more fields...
) {
   // ...
}

fn render_scene(
    geometry: &mut GeometryCtx, 
    material: &mut MaterialCtx,
    mesh:     &mut MeshCtx,
    // Possibly many more fields...
    mesh_ix:  usize
) {
    // ...
}
```

In real-world applications, this problem often affects API design, making code hard to maintain and understand. This issue was described multiple times over the years, some of the most notable discussions include:

- [Rust Internals "Notes on partial borrow"](https://internals.rust-lang.org/t/notes-on-partial-borrows/20020).
- [The Rustonomicon "Splitting Borrows"](https://doc.rust-lang.org/nomicon/borrow-splitting.html).
- [Niko Matsakis Blog Post "After NLL: Interprocedural conflicts"](https://smallcultfollowing.com/babysteps/blog/2018/11/01/after-nll-interprocedural-conflicts/).
- [Afternoon Rusting "Multiple Mutable References"](https://oribenshir.github.io/afternoon_rusting/blog/mutable-reference).
- [Partial borrows Rust RFC](https://github.com/rust-lang/rfcs/issues/1215#issuecomment-333316998).
- [HackMD "My thoughts on (and need for) partial borrows"](https://hackmd.io/J5aGp1ptT46lqLmPVVOxzg?view).
- [Dozens of threads on different platforms](https://www.google.com/search?client=safari&rls=en&q=rust+multiple+mut+ref+struct+fields&ie=UTF-8&oe=UTF-8).

<br/>

# 🤩 Partial borrows for the rescue!

This crate provides the `partial_borrow` macro, which we recommend importing under a shorter alias for concise syntax:

```rust
// CURRENT FILE: src/data.rs

use borrow::PartialBorrow;
use borrow::partial_borrow as p;
use borrow::traits::*;

#[derive(PartialBorrow)]
#[module(crate::data)] // Current module, see explanation below.
pub struct Ctx {
    pub geometry: GeometryCtx,
    pub material: MaterialCtx,
    pub mesh:     MeshCtx,
    pub scene:    SceneCtx,
}
```

The macro allows you to parameterize borrows similarly to how you parameterize types. It implements the syntax proposed in [Rust Internals "Notes on partial borrow"](https://internals.rust-lang.org/t/notes-on-partial-borrows/20020), extended with utilities for increased expressiveness:

1. **Field References**: You can parameterize the reference with field names.

   ```rust
   // Immutable reference to `geometry` and mutable reference 
   // to `material`.
   fn test(ctx: p!(&<geometry, mut material> Ctx)) {
       // ...
   }
   ```

2. **Field Selectors**: Use `*` to include all fields and `!` to exclude fields. Later selectors override previous ones.

   ```rust
   // Immutable reference to all fields except `geometry`.
   fn test1(ctx: p!(&<*, !geometry> Ctx)) {
       // ...
   }

   // Immutable reference to `material` and mutable reference 
   // to all other fields.
   fn test2(ctx: p!(&<mut *, material> Ctx)) {
       // ...
   }

   // Mutable reference to all fields.
   fn test3(ctx: p!(&<mut *> Ctx)) {
       // ...
   }
   ```

3. **Lifetime Annotations**: You can specify lifetimes for each reference. If a lifetime is not provided, it defaults to `'_`.

   ```rust
   // Reference to `mesh` with lifetime `'c` and references to 
   // other fields with lifetime `'b`. The inferred lifetime 
   // dependencies are `'a: 'b` and `'a: 'c`.
   fn test<'a, 'b, 'c>(ctx: p!(&'a <'b *, 'c mesh> Ctx)) {
       // ...
   }
   ```

4. **Default Lifetime**: Provide an alternative default lifetime as the first argument.

   ```rust
   // Alias for immutable references to `geometry` and `material` 
   // with lifetime `'t`, and to `mesh` with lifetime `'m`.
   type GlyphCtx<'t, 'm> = p!(<'t, geometry, material, 'm mesh> Ctx);
   ```

5. **Flexible Macro Expansion**: Please note that `p!(&<...>MyStruct)` always expands to `&mut p!(<...>MyStruct)`, which expands to `&mut MyStructRef<...>`, a generated struct containing references to fields. This allows for concise type alias syntax.

   ```rust
   type RenderCtx<'t> = p!(<'t, scene> Ctx);
   type GlyphCtx<'t> = p!(<'t, geometry, material, mesh> Ctx);
   type GlyphRenderCtx<'t> = Union<RenderCtx<'t>, GlyphCtx<'t>>;
   
   fn test(ctx: &mut GlyphRenderCtx) {
       // ...
   }
   ```

Let's apply these concepts to our rendering engine example:

```rust
// CURRENT FILE: src/data.rs

use borrow::PartialBorrow;
use borrow::partial_borrow as p;
use borrow::traits::*;

// === Data ===
pub struct Geometry { /* ... */ }
pub struct Material { /* ... */ }
pub struct Mesh     {
   /// Index of the geometry in the `GeometryCtx` registry.
   pub geometry: usize,
   /// Index of the material in the `MaterialCtx` registry.
   pub material: usize
}
pub struct Scene    {
   /// Indexes of meshes in the `MeshCtx` registry.
   pub meshes: Vec<usize>
}

// === Registries ===
pub struct GeometryCtx { pub data: Vec<Geometry> }
pub struct MaterialCtx { pub data: Vec<Material> }
pub struct MeshCtx     { pub data: Vec<Mesh> }
pub struct SceneCtx    { pub data: Vec<Scene> }

// === Root Registry ===
#[derive(PartialBorrow)]
#[module(crate::data)] // Current module, see explanation below.
pub struct Ctx {
   pub geometry: GeometryCtx,
   pub material: MaterialCtx,
   pub mesh:     MeshCtx,
   pub scene:    SceneCtx,
   // Possibly many more fields...
}

fn main() {
    let mut ctx = Ctx::new();
    // Obtain a mutable reference to all fields.
    render(ctx.as_refs_mut().partial_borrow());
}

fn render_pass1(ctx: p!(&<mut *> Ctx)) {
    // Extract a mut ref to `scene`, excluding it from `ctx`.
    let (scene, ctx2) = ctx.extract_scene();
    for scene in &scene.data {
        for mesh in &scene.meshes {
            // Extract references required by `render_scene`.
            render_scene(ctx2.partial_borrow(), *mesh)
        }
    }
    // As `ctx2` is no longer used, we can use `ctx` again.
    render_pass2(ctx);
}

fn render_pass2(ctx: p!(&<mut *> Ctx)) {
    // ...
}

// Take a ref to `mesh` and mut refs to `geometry` and `material`.
fn render_scene(
    ctx: p!(&<mesh, mut geometry, mut material> Ctx), 
    mesh: usize
) {
    // ...
}
```

<br/>

# 🔋 Batteries Included

Consider the following struct to demonstrate the key tools provided by the macro:

```rust
#[derive(PartialBorrow)]
#[module(crate::data)] // Current module, see explanation below.
pub struct Ctx {
   pub geometry: GeometryCtx,
   pub material: MaterialCtx,
   pub mesh:     MeshCtx,
   pub scene:    SceneCtx,
   // Possibly many more fields...
}
```

The `Ctx` struct is equipped with the following methods:

```rust
impl Ctx {
    /// Borrows all fields. The target type needs to be known,
    /// e.g., `ctx.as_refs::<p!(<*, mut mesh> Ctx)>()`.
    pub fn as_refs<Target>(&mut self) -> Target {
        // ...
    }
   
    /// Borrows all fields mutably.
    pub fn as_refs_mut(&mut self) -> p!(<mut *> Ctx) {
        // ...
    }
}
```

The partially borrowed struct provides borrowing and splitting capabilities:

```rust
impl p!(</* ... */>Ctx) {
    /// Borrows required fields. The target type needs to be known,
    /// e.g., `ctx.partial_borrow::<p!(<*, mut mesh> Ctx)>()`.
    fn partial_borrow<Target>(&mut self) -> &mut Target {
        // ...
    }
   
    /// Borrows fields required by `Target` and returns borrows of 
    /// all remaining fields. Please note, that if `Target` requires
    /// an immutable borrow of a field, the remaining fields will also 
    /// include an immutable borrow of that field.
    fn partial_borrow_rest<Target>(&mut self) -> 
        &mut <Self as ParialBorrow<Target>>::Rest {
        // ...
    }
   
    /// Borrows fields required by `Target` and returns borrows of 
    /// all remaining fields. Please refer to the `partial_borrow` and
    /// `partial_borrow_rest` methods for more details.
    fn split<Target>(&mut self) -> (
       &mut Target, 
       &mut <Self as ParialBorrow<Target>>::Rest
    ) {
        // ...
    }

    // Extract the `geometry` field and return it along with the rest 
    // of the borrowed fields.
    pub fn extract_geometry(&mut self) -> (
        &mut GeometryCtx,
        &mut <Self as PartialBorrow<p!(<mut geometry> Ctx)>>::Rest
    ) {
        // ...
    }

    // Other `extract_$field` methods are generated similarly.
}
```

The partially borrowed struct also provides methods for concatenating partial borrows:

```rust
impl p!(</* ... */>Ctx) {
    /// Concatenates the current partial borrow with 
    /// another one.
    fn union<Other>(&mut self, other: &mut Other) 
       -> Union<Self, Other> {
        // ...
    }
}



/// The `Union` type is particularly useful when defining 
/// type aliases.
type RenderCtx<'t> = p!(<'t, scene> Ctx);
type GlyphCtx<'t> = p!(<'t, geometry, material, mesh> Ctx);
type GlyphRenderCtx<'t> = Union<RenderCtx<'t>, GlyphCtx<'t>>;
```

Please note, that while the `union` operation might seem useful, in most cases it is better to re-structure your code to avoid it. For example, let's consider the previous implementation of `render_pass1`: 

```rust
fn render_pass1(ctx: p!(&<mut *> Ctx)) {
    let (scene, ctx2) = ctx.extract_scene();
    for scene in &scene.data {
        for mesh in &scene.meshes {
            render_scene(ctx2.partial_borrow(), *mesh)
        }
    }
    render_pass2(ctx);
}
```

It could also be implemented using explicit split and union, but it would make the code less readable:

```rust
fn render_pass1(ctx: p!(&<mut *> Ctx)) {
    // The `ctx` var shadows the original one here.
    let (scene_ctx, ctx) = ctx.split::<p!(<mut scene> Ctx)>();
    for scene in &scene_ctx.scene.data {
        for mesh in &scene.meshes {
            render_scene(ctx.partial_borrow(), *mesh)
        }
    }
    // Because the original var is inaccessible, we need to
    // unify the parts back together.
    let mut ctx = ctx.union(&scene_ctx);
    render_pass2(&mut ctx);
}
```

<br/>

# 👓 `#[module(...)]` Attribute

In the example above, we used the `#[module(...)]` attribute, which specifies the path to the module where the macro is invoked. This attribute is necessary because, currently, Rust does not allow procedural macros to automatically detect the path of the module they are used in. This limitation applies to both stable and unstable Rust versions.

If you intend to use the generated macro from another crate, avoid using the `crate::` prefix in the `#[module(...)]` attribute. Instead, refer to your current crate by its name, for example: `#[module(my_crate::data)]`. However, Rust does not permit referring to the current crate by name by default. To enable this, add the following line to your `lib.rs` file:

```rust
extern crate self as my_crate;
```

<br/>

# 🛠 How It Works Under the Hood

This macro performs straightforward transformations. Consider the `Ctx` struct from the example above:

```rust
#[derive(Debug, Default, PartialBorrow)]
#[module(crate::data)]
pub struct Ctx {
    pub geometry: GeometryCtx,
    pub material: MaterialCtx,
    pub mesh:     MeshCtx,
    pub scene:    SceneCtx,
}
```

The macro defines a `CtxRef` struct:

```rust
#[repr(C)]
pub struct CtxRef<Geometry, Material, Mesh, Scene> {
    geometry: Geometry,
    material: Material,
    mesh:     Mesh,
    scene:    Scene,
}
```

Each type parameter is instantiated with one of `&`, `&mut`, or `Hidden<T>`, a type used to safely hide fields that are not part of the current borrow:

```rust
#[repr(transparent)]
#[derive(Debug)]
pub struct Hidden<T>(*mut T);
```

The `partial_borrow`, `partial_borrow_rest`, and `split` methods are implemented using inlined pointer casts, with safety guarantees enforced by the type system:

```rust
pub trait PartialBorrow<Target> {
    type Rest;

    #[inline(always)]
    fn partial_borrow_impl(&mut self) -> &mut Target {
        unsafe { &mut *(self as *mut _ as *mut _) }
    }

    #[inline(always)]
    fn partial_borrow_rest_impl(&mut self) -> &mut Self:: Rest {
        unsafe { &mut *(self as *mut _ as *mut _) }
    }

    #[inline(always)]
    fn split_impl(&mut self) -> (&mut Target, &mut Self::Rest) {
        let a = unsafe { &mut *(self as *mut _ as *mut _) };
        let b = unsafe { &mut *(self as *mut _ as *mut _) };
        (a, b)
    }
}
```

Finally, a helper macro with the same name as the struct is generated and is used by the `partial_borrow` macro.

<br/>

# ⚠️ Limitations

Currently, the macro works only with non-parametrized structures. For parametrized structures, please create an issue or submit a pull request.
