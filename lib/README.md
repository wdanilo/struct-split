# 🔪 struct-split
Zero overhead, safe implementation of the [partial borrow idea](https://internals.rust-lang.org/t/notes-on-partial-borrows/20020). It lets you parametrize struct borrows with field names. It’s similar to [slice::split_at_mut](https://doc.rust-lang.org/std/primitive.slice.html#method.split_at_mut), but tailored for structs.

# 😵‍💫 Problem
Suppose you’re building a rendering engine with registries for geometry, materials, and scenes. Entities reference each other by ID (`usize`), stored within various registries:

```rust
pub struct GeometryCtx { pub data: Vec<String> }
pub struct MaterialCtx { pub data: Vec<String> }
pub struct Mesh        { pub geometry: usize, pub material: usize }
pub struct MeshCtx     { pub data: Vec<Mesh> }
pub struct Scene       { pub meshes: Vec<usize> }
pub struct SceneCtx    { pub data: Vec<Scene> }

pub struct Ctx {
    pub geometry: GeometryCtx,
    pub material: MaterialCtx,
    pub mesh:     MeshCtx,
    pub scene:    SceneCtx,
    // Possibly many more fields...
}
```

Some functions require mutable access to only part of this structure. Should they take a mutable reference to the entire Ctx struct, or should each field be passed separately? The former approach is inflexible and impractical. Consider the following code:

```rust
fn render_scene(ctx: &mut Ctx, mesh: usize) {
      // ...
}
```

At first glance, this may seem reasonable. However, using it like this:

```rust
fn render(ctx: &mut Ctx) {
    for scene in &ctx.scene.data {
        for mesh in &scene.meshes {
            render_scene(ctx, *mesh)
        }
    }
}
```

will be rejected by the compiler:

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

The approach of passing each field separately is functional but cumbersome and error-prone, especially as the number of fields grows:

```rust
fn render(
    geometry: &mut GeometryCtx, 
    material: &mut MaterialCtx,
    mesh:     &mut MeshCtx,
    scene:    &mut SceneCtx,
    // Possibly many more fields...
) {
    for scene in &scene.data {
        for mesh_ix in &scene.meshes {
            render_scene(geometry, material, mesh, *mesh_ix)
        }
    }
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

In real-world use, this problem commonly impacts API design, making code hard to maintain and understand. This issue is also explored in the following sources:

- [Rust Internals "Notes on partial borrow"](https://internals.rust-lang.org/t/notes-on-partial-borrows/20020).
- [The Rustonomicon "Splitting Borrows"](https://doc.rust-lang.org/nomicon/borrow-splitting.html).
- [Niko Matsakis Blog Post "After NLL: Interprocedural conflicts"](https://smallcultfollowing.com/babysteps/blog/2018/11/01/after-nll-interprocedural-conflicts/).
- [Afternoon Rusting "Multiple Mutable References"](https://oribenshir.github.io/afternoon_rusting/blog/mutable-reference).
- [Partial borrows Rust RFC](https://github.com/rust-lang/rfcs/issues/1215#issuecomment-333316998).
- [HackMD "My thoughts on (and need for) partial borrows"](https://hackmd.io/J5aGp1ptT46lqLmPVVOxzg?view).
- [Dozens of threads on different platforms](https://www.google.com/search?client=safari&rls=en&q=rust+multiple+mut+ref+struct+fields&ie=UTF-8&oe=UTF-8).

## 🤩 Solution, the `partial_borrow` macro

This crate exposes the `partial_borrow` macro which enables the syntax proposed in [Rust Internals "Notes on partial borrow"](https://internals.rust-lang.org/t/notes-on-partial-borrows/20020), allowing you to parametrize borrows, in a very similar way to how you can parametrize types. We recommend importing the macro in a renamed form for a simple and concise syntax:

```rust
use struct_split::partial_borrow as b;
```

1. **Transparency:** The type `&mut Ctx` is equivalent to `b!(&mut Ctx)`.
2. **Reference parametrization:** You can parametrize the reference with field names. For example `b!(&<geometry, mut material>Ctx)` will provide an immutable reference to the `geometry` field and a mutable one to the `material` field.
3. **A lifetime per field reference:** You can specify lifetimes per reference. For example, `b!(&'a<'b geometry, 'c mut material>Ctx)` uses three lifetimes, where `'a: 'b` and `'a: 'c`. In case a lifetime is not specified, it defaults to `'_`.
4. **Field selectors:** You can use the `*` symbol to include all fields, and `!` to exclude a field. For example, `b!(&<mut *, !geometry>Ctx)` will provide a mutable reference to all fields except `geometry`.
5. **Field specialization:** Symbols can override previous specifications, allowing flexible configurations. For example, `b!(<mut *, geometry, !scene> Ctx)` will provide a mutable reference to all fields except `geometry` and `scene`, with `geometry` having an immutable reference and scene being completely inaccessible.

Let's apply the above ideas to the rendering engine example:

```rust
use struct_split::PartialBorrow;
use struct_split::partial_borrow as b;

pub struct GeometryCtx { pub data: Vec<String> }
pub struct MaterialCtx { pub data: Vec<String> }
pub struct Mesh        { pub geometry: usize, pub material: usize }
pub struct MeshCtx     { pub data: Vec<Mesh> }
pub struct Scene       { pub meshes: Vec<usize> }
pub struct SceneCtx    { pub data: Vec<Scene> }

#[derive(PartialBorrow)]
#[module(crate::data)] // See explanation below.
pub struct Ctx {
      pub geometry: GeometryCtx,
      pub material: MaterialCtx,
      pub mesh:     MeshCtx,
      pub scene:    SceneCtx,
}

fn main() {
    let mut ctx = Ctx::new();
    // Obtain a mutable reference to all fields.
    render(&mut ctx.as_ref_mut());
}

fn render(ctx: b!(&<mut *>Ctx)) {
    // Extract a mutable reference to `scene`, excluding it from 
    // `ctx`.
    let (scene, ctx) = ctx.extract_scene();
    for scene in &scene.data {
        for mesh in &scene.meshes {
            // Extract references from `ctx` and pass them to 
            // `render_scene`.
            render_scene(ctx.fit(), *mesh)
        }
    }
}

// Take immutable reference to `mesh` and mutable references to 
// both `geometry` and `material`.
fn render_scene(
      ctx: b!(&<mesh, mut geometry, mut material>Ctx), 
      mesh: usize
) {
    // ...
}
```

## 👓 `#[module(...)]` Attribute
In the example above, we used the `#[module(...)]` attribute, which specifies the path to the module where the macro is invoked. This attribute is necessary because, as of now, Rust does not allow procedural macros to automatically detect the path of the module they are used in. This limitation applies to both stable and unstable Rust versions.

If you intend to use the generated macro from another crate, avoid using the `crate::` prefix in the `#[module(...)]` attribute. Instead, refer to your current crate by its name, for example: `#[module(my_crate::data)]`. However, Rust does not permit referring to the current crate by name by default. To enable this, add the following line to your `lib.rs` file:

```rust
extern crate self as my_crate;
```

## 🛠 Batteries included
Let's consider the following struct to demonstrate the most important tools provided by the macro:

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

Both the `Ctx` struct and the generated `CtxRef` one provide the following methods:

```rust
/// Perform partial field borrowing. The `Target` type is
/// a parametrized `CtxRef` struct, so you can use this
/// method with explicit parametrization, like
/// `partial_borrow::<b!(&<scene> Ctx)>()`.
pub fn partial_borrow<Target>(&self) -> Target where /*...*/ { 
    // ...
}

/// Returns a mutable reference to all fields.
pub fn partial_borrow_mut(&mut self) -> b!(&<mut*> Ctx) {
   self.partial_borrow::<b!(&<mut*> Ctx)>()
}

/// Perform field re-borrowing to match the target type
/// and return a struct of all references left.
fn partial_borrow_rest<Target>(&mut self) -> &mut Self::Rest {
    // ...
}

/// Split the struct into two parts, one matching the target
/// type, and one containing all other references.
fn split<Target>(&mut self) -> (
   &mut Target,
   &mut Self::Rest
) {
    // ...
}
```

2. The borrowed struct provides reshaping capabilities:
      ```rust
      impl CtxRef</*...*/> {
          /// Perform field re-borrowing to match the target type.
          fn partial_borrow<Target>(&mut self) -> &mut Target { 
              // ...
          }
   
          /// Perform field re-borrowing to match the target type
          /// and return a struct of all references left.
          fn partial_borrow_rest<Target>(&mut self) -> &mut Self::Rest { 
              // ...
          }
   
          /// Split the struct into two parts, one matching the target
          /// type, and one containing all other references.
          fn split<Target>(&mut self) -> (
              &mut Target, 
              &mut Self::Rest
          ) { 
              // ...
          }
      }
      ```


```rust
    type GlyphCtx<'s, 't> = b!(&'s, <'t, mut geometry, mut material> Ctx);
    type PlotCtx<'s, 't> = Joined<b!(&'s, <'t, mut mesh> Ctx), GlyphCtx<'s, 't>>;
```


```rust
    type GlyphCtx<'t> = b!(<'t, mut geometry, mut material> Ctx);
    type PlotCtx<'t> = Joined<b!(<'t, mut mesh> Ctx), GlyphCtx>;
```


## 🛠 How it works under the hood
This macro performs a set of straightforward transformations. Consider the struct from the example above:

```rust
#[derive(Debug, Default, Split)]
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

Each type variable will be instantiated with one of `&`, `&mut`, or `Hidden`:

```rust
#[repr(transparent)]
#[derive(Debug)]
pub struct Hidden<T>(*mut T);
```

The macro generates `as_ref_mut` and `as_refs` methods for flexible reference creation:

```rust
impl Ctx {
    pub fn as_ref_mut(mut self) -> CtxRef<
          &mut GeometryCtx, 
          &mut MaterialCtx,
          &mut MeshCtx,
          &mut SceneCtx
    > {
        // ...
    }
    
    // T is a parametrized `CtxRef` struct, so it can be useed as. Bounds skipped for brevity.
    pub fn as_refs<T>(&self) -> T where /*...*/ {
        // ...
    }
}
```

The CtxRef struct provides `fit`, `fit_rest`, and `split` methods:

```rust
impl CtxRef</*...*/> {
    /// Returns a reshaped struct fitting the target type.
    fn fit<Target>(&mut self) -> &mut Target { /*...*/ }
    /// Returns a new struct of all references left after fitting the target type.
    fn fit_rest<Target>(&mut self) -> &mut Self::Rest { /*...*/ }
    /// Returns a reshaped struct fitting the target type and a struct of all references left.
    fn split<Target>(&mut self) -> (&mut Target, &mut Self::Rest) { /*...*/ }
}

```

An `extract_$field` method is generated for each field:

```rust
impl CtxRef</*...*/> {
    pub fn extract_geometry(&mut self) -> 
        ( &mut GeometryCtx
        , &mut <Self as Split</*...*/>>::Rest
        ) {
    // ...
    }
}
```

## ⚠️ Limitations
Currently, the macro works only with non-parametrized structures. For parameterized structures, please create an issue or submit a PR.