#![allow(dead_code)]

mod data;

use data::Ctx;
use data::CtxRef;
use borrow::partial as p;

use borrow::traits::*;
use borrow::Union;
use borrow::Hidden;
use crate::data::{GeometryCtx, MaterialCtx, MeshCtx, SceneCtx};

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

// pub struct Ctx {
//     pub geometry: GeometryCtx,
//     pub material: MaterialCtx,
//     pub mesh: MeshCtx,
//     pub scene: SceneCtx,
// }

impl p!(<mut geometry, mut material>Ctx) {
    fn foo(&mut self){}
}

fn test(ctx: p!(&<mut *> Ctx)) {
    ctx.partial_borrow().foo();
}
