#![allow(dead_code)]

mod data;

use data::Ctx;
use struct_split::traits::*;

use data::*;
use struct_split::*;

// =============
// === Tests ===
// =============

#[test]
fn test_types() {
    let mut ctx = Ctx::mock();
    render(ctx.as_ref_mut().fit());
}

fn render(ctx: &mut Ctx![mut *]) {
    render_scene(ctx.fit(), 0);
//     let (scene, ctx) = ctx.extract_scene();
//     for scene in &scene.data {
//         for mesh in &scene.meshes {
//             render_scene(ctx.fit(), *mesh)
//         }
//     }
}
//
fn render_scene(_ctx: &mut Ctx![mesh, mut geometry, mut material], _mesh: usize) {
    // ...
}
