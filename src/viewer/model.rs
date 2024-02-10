use glam::{DAffine3, DMat4, DVec3};
use hex_color::HexColor;
use libfive::{Point3, Region3, Tree, TriangleMesh};
use three_d::{CpuMesh, Indices, Mat4, Positions, Srgba, Vec3};

pub struct Component {
    tree: Tree,
    region: Region3,
    color: HexColor,
    positions: Option<Vec<DAffine3>>,
}

impl Component {
    pub fn new(tree: Tree, region: Region3, color: HexColor) -> Self {
        Self {
            tree,
            region,
            color,
            positions: None,
        }
    }

    pub fn with_positions(&mut self, positions: Vec<DAffine3>) {
        self.positions = Some(positions);
    }

    fn mesh(&self, mesh_resolution: f32) -> Option<CpuMesh> {
        let mesh: TriangleMesh<Vertex> =
            self.tree.to_triangle_mesh(&self.region, mesh_resolution)?;

        let positions = mesh.positions.into_iter().map(Into::into).collect();
        let indices = mesh.triangles.into_iter().flatten().collect();

        let mut mesh = CpuMesh {
            positions: Positions::F32(positions),
            indices: Indices::U32(indices),
            ..Default::default()
        };
        mesh.compute_normals();

        Some(mesh)
    }
}

pub trait Viewable {
    fn components(self) -> Vec<Component>;
    fn light_positions(&self) -> Vec<DVec3>;
    fn background_color(&self) -> HexColor;
    fn mesh_resolution(&self) -> f32;

    fn into_mesh(self) -> Mesh
    where
        Self: Sized,
    {
        let triangulation_tolerance = self.mesh_resolution();
        let light_positions = self
            .light_positions()
            .iter()
            .map(|position| position.as_vec3().to_array().into())
            .collect();
        let HexColor { r, g, b, a } = self.background_color();
        let background_color = Srgba::new(r, g, b, a);

        let mut objects = Vec::new();

        for component in self.components() {
            match component.mesh(triangulation_tolerance) {
                Some(mesh) => {
                    let HexColor { r, g, b, a } = component.color;
                    let color = Srgba::new(r, g, b, a);
                    let transformations = component.positions.map(|positions| {
                        positions
                            .into_iter()
                            .map(|position| {
                                let matrix: DMat4 = position.into();
                                matrix.as_mat4().to_cols_array_2d().into()
                            })
                            .collect()
                    });

                    objects.push(CpuObject {
                        mesh,
                        color,
                        transformations,
                    });
                }
                None => {
                    eprintln!("Warning: Could not triangulate component, ignoring it");
                }
            }
        }

        Mesh {
            objects,
            light_positions,
            background_color,
        }
    }
}

#[derive(Clone)]
pub struct CpuObject {
    pub mesh: CpuMesh,
    pub color: Srgba,
    pub transformations: Option<Vec<Mat4>>,
}

#[derive(Clone)]
pub struct Mesh {
    pub objects: Vec<CpuObject>,
    pub light_positions: Vec<Vec3>,
    pub background_color: Srgba,
}

struct Vertex {
    x: f32,
    y: f32,
    z: f32,
}

impl Point3 for Vertex {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    fn x(&self) -> f32 {
        self.x
    }

    fn y(&self) -> f32 {
        self.y
    }

    fn z(&self) -> f32 {
        self.z
    }
}

impl From<Vertex> for Vec3 {
    fn from(vertex: Vertex) -> Self {
        Vec3::new(vertex.x, vertex.y, vertex.z)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Triangulation failed
    #[error("triangulation failed")]
    Triangulation(#[from] opencascade::Error),
}
