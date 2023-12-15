use glam::{DAffine3, DMat4, DVec3};
use hex_color::HexColor;
use opencascade::{mesh::Mesh, primitives::Shape};
use three_d::{CpuMesh, Indices, Mat4, Positions, Srgba, Vec3};

pub struct Component {
    shape: Shape,
    color: HexColor,
    positions: Option<Vec<DAffine3>>,
}

impl Component {
    pub fn new(shape: Shape, color: HexColor) -> Self {
        Self {
            shape,
            color,
            positions: None,
        }
    }

    pub fn with_positions(&mut self, positions: Vec<DAffine3>) {
        self.positions = Some(positions);
    }

    fn mesh(&self, triangulation_tolerance: f64) -> Result<CpuMesh, Error> {
        let Mesh {
            vertices, indices, ..
        } = self.shape.mesh_with_tolerance(triangulation_tolerance)?;

        let vertices = vertices
            .iter()
            .map(|vertex| vertex.as_vec3().to_array().into())
            .collect();
        let indices = indices
            .iter()
            .map(|&index| index.try_into().expect("index does not fit in u32"))
            .collect();

        let mut mesh = CpuMesh {
            positions: Positions::F32(vertices),
            indices: Indices::U32(indices),
            ..Default::default()
        };
        mesh.compute_normals();

        Ok(mesh)
    }
}

pub trait ViewableModel {
    fn components(self) -> Vec<Component>;
    fn light_positions(&self) -> Vec<DVec3>;
    fn background_color(&self) -> HexColor;
    fn triangulation_tolerance(&self) -> f64;

    fn into_mesh_model(self) -> MeshModel
    where
        Self: Sized,
    {
        let triangulation_tolerance = self.triangulation_tolerance();
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
                Ok(mesh) => {
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
                Err(_) => {
                    eprintln!("Warning: Could not triangulate component, ignoring it");
                }
            }
        }

        MeshModel {
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
pub struct MeshModel {
    pub objects: Vec<CpuObject>,
    pub light_positions: Vec<Vec3>,
    pub background_color: Srgba,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Triangulation failed
    #[error("triangulation failed")]
    Triangulation(#[from] opencascade::Error),
}
