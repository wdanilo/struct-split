use struct_split::Split;
use struct_split as lib;

#[derive(Debug, Default)]
pub struct GeometryCtx {
    pub data: Vec<String>
}

#[derive(Debug, Default)]
pub struct MaterialCtx {
    pub data: Vec<String>
}

#[derive(Debug, Default)]
pub struct Mesh {
    pub geometry: usize,
    pub material: usize,
}

#[derive(Debug, Default)]
pub struct MeshCtx {
    pub data: Vec<Mesh>,
}

#[derive(Debug, Default)]
pub struct Scene {
    pub meshes: Vec<usize>,
}

#[derive(Debug, Default)]
pub struct SceneCtx {
    pub data: Vec<Scene>,
}

#[derive(Debug, Default, Split)]
#[module(crate::data)]
pub struct Ctx {
    pub geometry: GeometryCtx,
    pub material: MaterialCtx,
    pub mesh: MeshCtx,
    pub scene: SceneCtx,
}

impl<'t, geometry, material, mesh, scene>
    lib::AsRefs<'t, CtxRef<geometry, material, mesh, scene>> for Ctx
where
    GeometryCtx: lib::RefCast<'t, geometry>,
    MaterialCtx: lib::RefCast<'t, material>,
    MeshCtx:     lib::RefCast<'t, mesh>,
    SceneCtx:    lib::RefCast<'t, scene>,
{
    fn as_refs_impl(&'t mut self) -> CtxRef<geometry, material, mesh, scene> {
        CtxRef {
            geometry: lib::RefCast::ref_cast(&mut self.geometry),
            material: lib::RefCast::ref_cast(&mut self.material),
            mesh:     lib::RefCast::ref_cast(&mut self.mesh),
            scene:    lib::RefCast::ref_cast(&mut self.scene),
        }
    }
}

impl Ctx {
    pub fn as_ref_mut(&mut self) -> CtxRef<&mut GeometryCtx, &mut MaterialCtx, &mut MeshCtx, &mut SceneCtx> {
        CtxRef {
            geometry: &mut self.geometry,
            material: &mut self.material,
            mesh:     &mut self.mesh,
            scene:    &mut self.scene,
        }
    }
}

impl<geometry, material, mesh, scene>
lib::IntoFields for CtxRef<geometry, material, mesh, scene> {
    type Fields = lib::HList![geometry, material, mesh, scene];
}

impl<geometry_target, material_target, mesh_target, scene_target,
     geometry,        material,        mesh,        scene>
lib::FromFields<lib::HList![geometry_target, material_target, mesh_target, scene_target]>
for CtxRef<geometry, material, mesh, scene> {
    type Result = CtxRef<geometry_target, material_target, mesh_target, scene_target>;
}

impl Ctx {
    pub fn new_geometry(&mut self, data: &str) -> usize {
        self.geometry.data.push(data.to_string());
        self.geometry.data.len() - 1
    }

    pub fn new_material(&mut self, data: &str) -> usize {
        self.material.data.push(data.to_string());
        self.material.data.len() - 1
    }

    pub fn new_mesh(&mut self, geometry: usize, material: usize) -> usize {
        self.mesh.data.push(Mesh { geometry, material });
        self.mesh.data.len() - 1
    }

    pub fn new_scene(&mut self, meshes: &[usize]) -> usize {
        self.scene.data.push(Scene { meshes: meshes.to_vec() });
        self.scene.data.len() - 1
    }

    pub fn mock() -> Self {
        let mut ctx = Self::default();
        let geo1 = ctx.new_geometry("geo1");
        let geo2 = ctx.new_geometry("geo2");
        let mat1 = ctx.new_material("mat1");
        let mat2 = ctx.new_material("mat2");
        let mesh1 = ctx.new_mesh(geo1, mat1);
        let mesh2 = ctx.new_mesh(geo2, mat2);
        let _scene1 = ctx.new_scene(&[mesh1, mesh2]);
        ctx
    }
}

#[macro_export]
macro_rules! _Ctx {
    (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [,* $($xs:tt)*]) => {
        _Ctx! { @ $lt [[& $lt GeometryCtx] [& $lt MaterialCtx] [& $lt MeshCtx] [& $lt SceneCtx]] [$ ($xs) *] }
    };
    (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, mut * $ ($xs:tt) *]) => {
        _Ctx! { @ $lt [[& $lt mut GeometryCtx] [& $lt mut MaterialCtx] [& $lt mut MeshCtx] [& $lt mut SceneCtx]] [$ ($xs) *] }
    };


    (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, $(ref)? geometry $ ($xs:tt) *]) => {
        _Ctx! { @ $lt [[& $lt GeometryCtx] $t1 $t2 $t3] [$ ($xs) *] }
    };
    (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, $(ref)? material $ ($xs:tt) *]) => {
        _Ctx! { @ $lt [$t0 [& $lt MaterialCtx] $t2 $t3] [$ ($xs) *] }
    };
    (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, $(ref)? mesh $ ($xs:tt) *]) => {
        _Ctx! { @ $lt [$t0 $t1 [& $lt MeshCtx] $t3] [$ ($xs) *] }
    };
    (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, $(ref)? scene $ ($xs:tt) *]) => {
        _Ctx! { @ $lt [$t0 $t1 $t2 [& $lt SceneCtx]] [$ ($xs) *] }
    };


    (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, mut geometry $ ($xs:tt) *]) => {
        _Ctx! { @ $lt [[& $lt mut GeometryCtx] $t1 $t2 $t3] [$ ($xs) *] }
    };
    (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, mut material $ ($xs:tt) *]) => {
        _Ctx! { @ $lt [$t0 [& $lt mut MaterialCtx] $t2 $t3] [$ ($xs) *] }
    };
    (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, mut mesh $ ($xs:tt) *]) => {
        _Ctx! { @ $lt [$t0 $t1 [& $lt mut MeshCtx] $t3] [$ ($xs) *] }
    };
    (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, mut scene $ ($xs:tt) *]) => {
        _Ctx! { @ $lt [$t0 $t1 $t2 [& $lt mut SceneCtx]] [$ ($xs) *] }
    };


    (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, ! geometry $ ($xs:tt) *]) => {
        _Ctx! { @ $lt [[Hidden<GeometryCtx>] $t1 $t2 $t3] [$ ($xs) *] }
    };
    (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, ! material $ ($xs:tt) *]) => {
        _Ctx! { @ $lt [$t0 [Hidden<MaterialCtx>] $t2 $t3] [$ ($xs) *] }
    };
    (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, ! mesh $ ($xs:tt) *]) => {
        _Ctx! { @ $lt [$t0 $t1 [Hidden<MeshCtx>] $t3] [$ ($xs) *] }
    };
    (@ $lt:lifetime [$t0:tt $t1:tt $t2:tt $t3:tt] [, ! scene $ ($xs:tt) *]) => {
        _Ctx! { @ $lt [$t0 $t1 $t2 [Hidden<SceneCtx>]] [$ ($xs) *] }
    };


    (@ $lt:lifetime [$ ([$ ($ts:tt) *]) *] [$ (,) *]) => {
        CtxRef < $ ($ ($ts) *), * >
    };

    (@ $($ts:tt)*) => {
        error
    };

    ($lt:lifetime $ ($ts:tt) *) => {
        _Ctx! { @ $lt [[Hidden<GeometryCtx>] [Hidden<MaterialCtx>] [Hidden<MeshCtx>] [Hidden<SceneCtx>]] [$($ts)*] }
    };

    ($($ts:tt)*) => {
        _Ctx! { @ '_ [[Hidden<GeometryCtx>] [Hidden<MaterialCtx>] [Hidden<MeshCtx>] [Hidden<SceneCtx>]] [, $ ($ts) *] }
    };
}
pub use _Ctx as Ctx;