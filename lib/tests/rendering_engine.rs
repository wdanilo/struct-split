#![allow(dead_code)]

mod data;

use data::Ctx;
use borrow::partial_borrow as p;

use borrow::traits::*;
use borrow::UnifyImpl;
use borrow::Union;

// =============
// === Tests ===
// =============

#[test]
fn test_types() {
    let mut ctx = Ctx::mock();
    render_pass1(ctx.as_refs_mut().partial_borrow());
    render_pass1_explicit(ctx.as_refs_mut().partial_borrow());
}

fn render_pass1(ctx: p!(&<mut *> Ctx)) {
    let (scene, ctx2) = ctx.extract_scene();
    for scene in &scene.data {
        for mesh in &scene.meshes {
            render_scene(ctx2.partial_borrow(), *mesh)
        }
    }
    render_pass2(ctx.partial_borrow());
    render_pass3(ctx.partial_borrow());
}

fn render_pass1_explicit(ctx: p!(&<mut *> Ctx)) {
    let (scene_ctx, ctx2) = ctx.split::<p!(<mut scene> Ctx)>();
    for scene in &scene_ctx.scene.data {
        for mesh in &scene.meshes {
            render_scene(ctx2.partial_borrow(), *mesh)
        }
    }
    let mut merged_ctx = ctx2.union(scene_ctx);
    render_pass2(&mut merged_ctx);
}

fn render_pass2(_ctx: p!(&<mut *> Ctx)) {}
fn render_pass3(_ctx: &mut GlyphRenderCtx) {}
fn render_scene(_ctx: p!(&<mesh, mut geometry, mut material> Ctx), _mesh: usize) {
    // ...
}

// === Type Aliases ===

type RenderCtx<'t> = p!(<'t, scene> Ctx);
type GlyphCtx<'t> = p!(<'t, geometry, material, mesh> Ctx);
type GlyphRenderCtx<'t> = Union<RenderCtx<'t>, GlyphCtx<'t>>;