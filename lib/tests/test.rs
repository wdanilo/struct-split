#![allow(dead_code)]

mod data;

use data::Ctx;
use struct_split::partial_borrow as p;

use struct_split::traits::*;

// =============
// === Tests ===
// =============

#[test]
fn test_types() {
    let mut ctx = Ctx::mock();
    render(ctx.as_refs_mut().partial_borrow());
}

fn render(ctx: &mut p!(<mut *> Ctx)) {
    render_scene(ctx.partial_borrow(), 0);
    let (scene, ctx) = ctx.extract_scene();
    for scene in &scene.data {
        for mesh in &scene.meshes {
            render_scene(ctx.partial_borrow(), *mesh)
        }
    }
}

fn render_scene(_ctx: &mut p!(<mesh, mut geometry, mut material> Ctx), _mesh: usize) {
    // ...
}

