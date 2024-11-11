# 🔪 Partial Borrows

Zero-overhead, safe implementation of [partial borrows](https://internals.rust-lang.org/t/notes-on-partial-borrows/20020). This crate allows you to borrow selected fields from a struct and split structs into non-overlapping sets of borrowed fields. The splitting functionality is similar to [slice::split_at_mut](https://doc.rust-lang.org/std/primitive.slice.html#method.split_at_mut), but tailored for structs.

# 😵‍💫 Problem

Suppose you're building a rendering engine that needs to store geometries, materials, meshes, and scenes. These entities form a reference graph (e.g., two meshes can use the same material). To handle this, you can either:

- Use `Rc`/`Arc` to share ownership, which bypasses Rust's borrow checker and can make your code error-prone at runtime.
- Store the entities in registries and use their indices as references.

We opt for the latter approach and create a root registry called `Ctx`:

```rust
// === Data ===
pub struct Geometry { /* ... */ }
pub struct Material { /* ... */ }
pub struct Mesh     { pub geometry: usize, pub material: usize }
pub struct Scene    { pub meshes: Vec<usize> }

// === Registries ===
pub struct GeometryCtx { pub data: Vec<Geometry> }
pub struct MaterialCtx { pub data: Vec<Material> }
pub struct MeshCtx     { pub data: Vec<Mesh> }
pub struct SceneCtx    { pub data: Vec<Scene> }

// === Global Registry ===
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

At first glance, this seems reasonable. However, it will be rejected by the compiler:

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

Passing each field separately works but becomes cumbersome and error-prone as the number of fields grows:

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

In real-world applications, this problem affects API design, making code hard to maintain and understand. This issue is also explored in the following sources:

- [Rust Internals "Notes on partial borrow"](https://internals.rust-lang.org/t/notes-on-partial-borrows/20020).
- [The Rustonomicon "Splitting Borrows"](https://doc.rust-lang.org/nomicon/borrow-splitting.html).
- [Niko Matsakis Blog Post "After NLL: Interprocedural conflicts"](https://smallcultfollowing.com/babysteps/blog/2018/11/01/after-nll-interprocedural-conflicts/).
- [Afternoon Rusting "Multiple Mutable References"](https://oribenshir.github.io/afternoon_rusting/blog/mutable-reference).
- [Partial borrows Rust RFC](https://github.com/rust-lang/rfcs/issues/1215#issuecomment-333316998).
- [HackMD "My thoughts on (and need for) partial borrows"](https://hackmd.io/J5aGp1ptT46lqLmPVVOxzg?view).
- [Dozens of threads on different platforms](https://www.google.com/search?client=safari&rls=en&q=rust+multiple+mut+ref+struct+fields&ie=UTF-8&oe=UTF-8).

## 🤩 Solution: Partial Borrow

This crate provides the `partial_borrow` macro, which we recommend importing under a shorter alias for concise syntax:

```rust
use struct_split::PartialBorrow;
use struct_split::partial_borrow as p;
use struct_split::traits::*;

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
   fn test(ctx: &mut p!(<geometry, mut material> Ctx)) {
       // ...
   }
   ```

2. **Field Selectors**: Use `*` to include all fields and `!` to exclude fields. Later selectors override previous ones.

   ```rust
   // Immutable reference to all fields except `geometry`.
   fn test1(ctx: &mut p!(<*, !geometry> Ctx)) {
       // ...
   }

   // Immutable reference to `material` and mutable reference 
   // to all other fields.
   fn test2(ctx: &mut p!(<mut *, material> Ctx)) {
       // ...
   }

   // Mutable reference to all fields.
   fn test3(ctx: &mut p!(<mut *> Ctx)) {
       // ...
   }
   ```

3. **Lifetime Annotations**: You can specify lifetimes for each reference. If a lifetime is not provided, it defaults to `'_`.

   ```rust
   // Reference to `mesh` with lifetime `'c` and references to 
   // other fields with lifetime `'b`. The inferred lifetime 
   // dependencies are `'a: 'b` and `'a: 'c`.
   fn test<'a, 'b, 'c>(ctx: &'a mut p!(<'b *, 'c mesh> Ctx)) {
       // ...
   }
   ```

4. **Default Lifetime**: Provide an alternative default lifetime as the first argument.

   ```rust
   // Alias for immutable references to `geometry` and `material` 
   // with lifetime `'t`, and to `mesh` with lifetime `'m`.
   type GlyphCtx<'t, 'm> = p!(<'t, geometry, material, 'm mesh> Ctx);
   ```

Let's apply these concepts to our rendering engine example:

```rust
use struct_split::PartialBorrow;
use struct_split::partial_borrow as p;
use struct_split::traits::*;

pub struct GeometryCtx { pub data: Vec<String> }
pub struct MaterialCtx { pub data: Vec<String> }
pub struct Mesh        { pub geometry: usize, pub material: usize }
pub struct MeshCtx     { pub data: Vec<Mesh> }
pub struct Scene       { pub meshes: Vec<usize> }
pub struct SceneCtx    { pub data: Vec<Scene> }

#[derive(PartialBorrow)]
#[module(crate::data)] // Current module, see explanation below.
pub struct Ctx {
    pub geometry: GeometryCtx,
    pub material: MaterialCtx,
    pub mesh:     MeshCtx,
    pub scene:    SceneCtx,
}

fn main() {
    let mut ctx = Ctx::new();
    // Obtain a mutable reference to all fields.
    render(ctx.as_refs_mut().partial_borrow());
}

fn render_pass1(ctx: &mut p!(<mut *> Ctx)) {
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

fn render_pass2(ctx: &mut p!(<mut *> Ctx)) {
    // ...
}

// Take a ref to `mesh` and mut ref to `geometry` and `material`.
fn render_scene(
    ctx: &mut p!(<mesh, mut geometry, mut material> Ctx), 
    mesh: usize
) {
    // ...
}
```

## 🛠 Batteries Included

Consider the following struct to demonstrate the key tools provided by the macro:

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

The `Ctx` struct is equipped with the following methods:

```rust
impl Ctx {
    /// Converts `Ctx` into `CtxRef` where each field can 
    /// be either a mutable or immutable reference.
    pub fn as_refs<Target>(&mut self) -> Target {
        // ...
    }
   
    /// Converts `Ctx` into `CtxRef` where every field is 
    /// a mutable reference.
    pub fn as_refs_mut(&mut self) -> p!(<mut *> Ctx) {
        // ...
    }
}
```

The partially borrowed struct provides borrowing and splitting capabilities:

```rust
impl CtxRef</* ... */> {
    /// Re-borrows fields to match the target type. The
    /// target type is a partial ref to the current 
    /// struct, allowing for simple, explicit syntax:
    /// `ctx.partial_borrow::<p!(<*, mut mesh> Ctx)>()`.
    fn partial_borrow<Target>(&mut self) -> &mut Target {
        // ...
    }
   
    /// Re-borrows fields to match the target type and 
    /// returns a struct of the remaining references.
    fn partial_borrow_rest<Target>(&mut self) -> &mut Self::Rest {
        // ...
    }
   
    /// Splits the struct into two parts: one matching the 
    /// target type and one containing the remaining references.
    fn split<Target>(&mut self) -> (&mut Target, &mut Self::Rest) {
        // ...
    }
}
```

The partially borrowed struct also provides methods for concatenating partial borrows:

```rust
impl CtxRef</* ... */> {
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
fn render_pass1(ctx: &mut p!(<mut *> Ctx)) {
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
fn render_pass1(ctx: &mut p!(<mut *> Ctx)) {
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

## 👓 `#[module(...)]` Attribute

In the example above, we used the `#[module(...)]` attribute, which specifies the path to the module where the macro is invoked. This attribute is necessary because, currently, Rust does not allow procedural macros to automatically detect the path of the module they are used in. This limitation applies to both stable and unstable Rust versions.

If you intend to use the generated macro from another crate, avoid using the `crate::` prefix in the `#[module(...)]` attribute. Instead, refer to your current crate by its name, for example: `#[module(my_crate::data)]`. However, Rust does not permit referring to the current crate by name by default. To enable this, add the following line to your `lib.rs` file:

```rust
extern crate self as my_crate;
```

## 🛠 How It Works Under the Hood

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

Each type parameter is instantiated with one of `&`, `&mut`, or `Hidden<T>`:

```rust
#[repr(transparent)]
#[derive(Debug)]
pub struct Hidden<T>(*mut T);
```

The `Hidden<T>` type is used to safely hide fields that are not part of the current borrow.

The macro generates `as_refs_mut` and `as_refs` methods for flexible reference creation:

```rust
impl Ctx {
    pub fn as_refs_mut(&mut self) -> CtxRef<
        &mut GeometryCtx, 
        &mut MaterialCtx,
        &mut MeshCtx,
        &mut SceneCtx
    > {
        // ...
    }
    
    // `T` is a parametrized `CtxRef` struct. Bounds are 
    // skipped for brevity.
    pub fn as_refs<T>(&self) -> T where /* ... */ {
        // ...
    }
}
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

An `extract_$field` method is generated for each field:

```rust
impl CtxRef</* ... */> {
    pub fn extract_geometry(&mut self) -> (
        &mut GeometryCtx,
        &mut <Self as Split</* ... */>>::Rest
    ) {
        // ...
    }
}
```

Finally, a helper macro with the same name as the struct is generated and is used by the `partial_borrow` macro.

## ⚠️ Limitations

Currently, the macro works only with non-parametrized structures. For parametrized structures, please create an issue or submit a pull request.