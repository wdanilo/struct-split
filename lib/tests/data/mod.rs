use borrow::PartialBorrow;

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

#[derive(Debug, Default, PartialBorrow)]
#[module(crate::data)]
pub struct Ctx {
    pub geometry: GeometryCtx,
    pub material: MaterialCtx,
    pub mesh: MeshCtx,
    pub scene: SceneCtx,
}

impl Ctx {
    pub fn new_geometry(&mut self, label: &str) -> usize {
        let label = label.to_string();
        self.geometry.data.push(Geometry { label });
        self.geometry.data.len() - 1
    }

    pub fn new_material(&mut self, label: &str) -> usize {
        let label = label.to_string();
        self.material.data.push(Material { label });
        self.material.data.len() - 1
    }

    pub fn new_mesh(&mut self, geometry: usize, material: usize) -> usize {
        self.mesh.data.push(Mesh { geometry, material });
        self.mesh.data.len() - 1
    }

    pub fn new_scene(&mut self, meshes: &[usize]) -> usize {
        let meshes = meshes.to_vec();
        self.scene.data.push(Scene { meshes });
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
