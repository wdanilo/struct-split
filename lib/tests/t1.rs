use borrow::partial as p;
use borrow::traits::*;
use borrow::*;
use std::fmt::Debug;
use std::vec::Vec;

// ============
// === Data ===
// ============

#[derive(Debug, Default)]
pub struct Geometry {
    label: String,
}

#[derive(Debug, Default)]
pub struct Material {
    label: String,
}

#[derive(Debug, Default)]
pub struct Mesh {
    pub geometry: usize,
    pub material: usize,
}

#[derive(Debug, Default)]
pub struct Scene {
    pub meshes: Vec<usize>,
}

// ==================
// === Registries ===
// ==================

#[derive(Debug, Default)]
pub struct GeometryCtx {
    pub data: Vec<Geometry>,
}

#[derive(Debug, Default)]
pub struct MaterialCtx {
    pub data: Vec<Material>,
}

#[derive(Debug, Default)]
pub struct MeshCtx {
    pub data: Vec<Mesh>,
}

#[derive(Debug, Default)]
pub struct SceneCtx {
    pub data: Vec<Scene>,
}

// =====================
// === Root Registry ===
// =====================

#[derive(Debug, borrow::Partial)]
#[module(crate)]
pub struct Ctx<'v, V: Debug> {
    version: &'v V,
    pub geometry: GeometryCtx,
    pub material: MaterialCtx,
    pub mesh: MeshCtx,
    pub scene: SceneCtx,
}

fn render_pass1<'v2, V2: Debug>(ctx: p!(&<mut *> Ctx<'v2, V2>)) {
    // let x: usize = ctx.extract_version();
    // fn render_pass1<'v2, V2: Debug>(ctx: p!(&<mut *> Ctx<'v2, V2>)) {
    // fn render_pass1<'v2, V2: Debug>(ctx: &mut Ctx![['v2, V2] mut *]) {
    // let (scene, ctx2) = ctx.extract_scene();
    // for scene in &scene.data {
    //     for mesh in &scene.meshes {
    //         render_scene(ctx2.partial_borrow(), *mesh)
    //     }
    // }
}




#[allow(non_camel_case_types)]
impl<'t, version, geometry, material, mesh, scene>
    CtxRef<version, geometry, material, mesh, scene>
where
{
    #[inline(always)]
    pub fn extract_version2(&'t mut self) -> (
        <version as RefFlatten<'t>>::Output,
        &'t mut CtxRef<Acquired<version, version>, geometry, material, mesh, scene>
    ) where version: Acquire<version> + RefFlatten<'t> {
        let rest = unsafe { &mut *(self as *mut _ as *mut _) };
        (self.version.ref_flatten(), rest)
    }
}
