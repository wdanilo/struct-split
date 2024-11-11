#![allow(dead_code)]

mod data;

use data::Ctx;
use struct_split::partial_borrow as p;

use struct_split::traits::*;
use struct_split::Join;
use struct_split::Joined;

// =============
// === Tests ===
// =============

#[test]
fn test_types() {
    let mut ctx = Ctx::mock();
    render_pass1(ctx.as_refs_mut().partial_borrow());
}

fn render_pass1(ctx: &mut p!(<mut *> Ctx)) {
    let (scene, ctx2) = ctx.extract_scene();
    for scene in &scene.data {
        for mesh in &scene.meshes {
            render_scene(ctx2.partial_borrow(), *mesh)
        }
    }
    render_pass2(ctx);
}

fn render_pass1_alt(ctx: &mut p!(<mut *> Ctx)) {
    let (scene_ctx, ctx2) = ctx.split::<p!(<mut scene> Ctx)>();
    for scene in &scene_ctx.scene.data {
        for mesh in &scene.meshes {
            render_scene(ctx2.partial_borrow(), *mesh)
        }
    }
    let mut merged_ctx = ctx2.join(scene_ctx);
    render_pass2(&mut merged_ctx);
}

fn render_pass2(ctx: &mut p!(<mut *> Ctx)) {}

fn render_scene(_ctx: &mut p!(<mesh, mut geometry, mut material> Ctx), _mesh: usize) {
    // ...
}


type RenderCtx<'t> = p!(<'t, scene> Ctx);
type GlyphCtx<'t> = p!(<'t, geometry, material, mesh> Ctx);
type GlyphRenderCtx<'t> = Joined<RenderCtx<'t>, GlyphCtx<'t>>;