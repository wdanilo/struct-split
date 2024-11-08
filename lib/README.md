## struct-split
Efficiently split struct fields into distinct subsets of references, ensuring **zero overhead** and **strict borrow checker compliance** (non-overlapping mutable references).

## Example Usage
Suppose we’re building a rendering engine with registries for geometry, material, and scenes. Entities reference each other by ID (usize), stored within various registries:

```rust
use struct_split::Split;

pub struct GeometryCtx { pub data: Vec<String> }
pub struct MaterialCtx { pub data: Vec<String> }
pub struct Mesh        { pub geometry: usize, pub material: usize }
pub struct MeshCtx     { pub data: Vec<Mesh> }
pub struct Scene       { pub meshes: Vec<usize> }
pub struct SceneCtx    { pub data: Vec<Scene> }

#[derive(Split)]
#[module(crate::data)]
pub struct Ctx {
    pub geometry: GeometryCtx,
    pub material: MaterialCtx,
    pub mesh:     MeshCtx,
    pub scene:    SceneCtx,
}
```

With `struct-split`, you can separate `Ctx` into subsets of field references:

```rust
fn main() {
    let mut ctx = Ctx::new();
    // Obtain a mutable reference to all fields.
    render(&mut ctx.as_ref_mut());
}

fn render(ctx: &mut Ctx![mut *]) {
    // Extract a mutable reference to `scene`, excluding it from `ctx`.
    let (scene, ctx) = ctx.extract_scene();
    for scene in &scene.data {
        for mesh in &scene.meshes {
            // Extract references from `ctx` and pass them to `render_scene`.
            render_scene(ctx.fit(), *mesh)
        }
    }
}

// Take immutable reference to `mesh` and mutable references to both `geometry` 
// and `material`.
fn render_scene(ctx: &mut Ctx![mesh, mut geometry, mut material], mesh: usize) {
    // ...
}
```

## `#[module(...)]` Attribute
In the example above, we used the `#[module(...)]` attribute, which specifies the path to the module where the macro is invoked. This attribute is necessary because, as of now, Rust does not allow procedural macros to automatically detect the path of the module they are used in. This limitation applies to both stable and unstable Rust versions.

If you intend to use the generated macro from another crate, avoid using the `crate::` prefix in the `#[module(...)]` attribute. Instead, refer to your current crate by its name, for example: `#[module(my_crate::data)]`. However, Rust does not permit referring to the current crate by name by default. To enable this, add the following line to your `lib.rs` file:

```rust
extern crate self as my_crate;
```

## Generated Macro Syntax
A macro with the same name as the target struct is generated, allowing flexible reference specifications. The syntax follows these rules:

   1. **Lifetime:** The first argument can be an optional lifetime, which will be used for all references. If no lifetime is provided, '_ is used as the default.
   2. **Mutability:** Each field name can be prefixed with mut for a mutable reference or ref for an immutable reference. If no prefix is specified, the reference is immutable by default.
   3. **Symbols:**
      - `*` can be used to include all fields.
      - `!` can be used to exclude a field (providing neither an immutable nor mutable reference).
   4. **Override Capability:** Symbols can override previous specifications, allowing flexible configurations. For example, `Ctx![mut *, geometry, !scene]` will provide a mutable reference to all fields except `geometry` and `scene`, with geometry having an immutable reference and scene being completely inaccessible.


## How it works under the hood
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
pub struct CtxRef<'t, geometry: Access, material: Access, mesh: Access, scene: Access> {
    geometry: Value<'t, geometry, GeometryCtx>,
    material: Value<'t, material, MaterialCtx>,
    mesh:     Value<'t, mesh,     MeshCtx>,
    scene:    Value<'t, scene,    SceneCtx>,
}
```

The `Value` type adapts to either a reference, a mutable reference, or an inaccessible private mutable pointer, based on parameterization:

```rust
#[repr(transparent)]
#[derive(Debug)]
pub struct NoAccess<T>(*mut T);

pub trait Access            { type Value<'t, T: 't + Debug>: Debug; }
impl      Access for Ref    { type Value<'t, T: 't + Debug> = &'t T; }
impl      Access for RefMut { type Value<'t, T: 't + Debug> = &'t mut T; }
impl      Access for None   { type Value<'t, T: 't + Debug> = NoAccess<T>; }

pub type Value<'t, L, T> = <L as Access>::Value<'t, T>;
```

The macro generates `as_ref_mut` and `as_refs` methods for flexible reference creation:

```rust
impl Ctx {
    pub fn as_ref_mut<'t>(&'t mut self) -> CtxRef<'t, RefMut, RefMut, RefMut, RefMut> {
        // ...
    }
    
    // T is a parametrized `CtxRef` struct. Bounds skipped for brevity.
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

Finally, the macro generates the `Ctx!` macro itself.

## Limitations
Currently, the macro works only with non-parametrized structures. For parameterized structures, please create an issue or submit a PR.