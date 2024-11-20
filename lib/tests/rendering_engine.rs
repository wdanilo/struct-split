#![allow(dead_code)]

mod data;

use data::Ctx;
use borrow::partial as p;

use borrow::traits::*;
use borrow::Union;

// =============
// === Tests ===
// =============

#[test]
fn test_types() {
    let mut ctx = Ctx::mock();
    render_pass1(&mut ctx.as_refs_mut());
}

fn render_pass1(ctx: p!(&<mut *> Ctx)) {
    let (scene, ctx2) = ctx.extract_scene();
    for scene in &scene.data {
        for mesh in &scene.meshes {
            render_scene(ctx2.partial_borrow(), *mesh)
        }
    }
    render_pass2(ctx);
    render_pass3(ctx.partial_borrow());
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
