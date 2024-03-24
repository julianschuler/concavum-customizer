use glam::{DAffine3, DMat4, DVec3};
use hex_color::HexColor;
use three_d::{CpuMesh, Indices, Mat4, Positions, Srgba, Vec3};

use crate::model::Shape;

/// A component which can be meshed and may have multiple positions.
pub struct Component {
    shape: Shape,
    color: HexColor,
    positions: Option<Vec<DAffine3>>,
}

impl Component {
    /// Creates a new component from a given shape and color.
    pub fn new(shape: Shape, color: HexColor) -> Self {
        Self {
            shape,
            color,
            positions: None,
        }
    }

    /// Sets the positions of the component.
    pub fn with_positions(&mut self, positions: Vec<DAffine3>) {
        self.positions = Some(positions);
    }

    /// Meshes the component.
    fn mesh(&self, settings: MeshSettings) -> CpuMesh {
        let mesh = self.shape.mesh(settings);

        let vertices = mesh
            .vertices
            .iter()
            .map(|vertex| {
                let slice: [_; 3] = vertex.as_slice().try_into().unwrap();
                slice.into()
            })
            .collect();
        let indices = mesh
            .triangles
            .iter()
            .flat_map(|triangle| {
                triangle
                    .iter()
                    .map(|&index| index.try_into().expect("index should fit in u32"))
            })
            .collect();

        let mut mesh = CpuMesh {
            positions: Positions::F32(vertices),
            indices: Indices::U32(indices),
            ..Default::default()
        };
        mesh.compute_normals();

        mesh
    }
}

/// A trait for displaying components with settings.
pub trait Viewable {
    fn components(self) -> Vec<Component>;
    fn settings(&self) -> MeshSettings;
    fn light_positions(&self) -> Vec<DVec3>;
    fn background_color(&self) -> HexColor;

    fn into_model(self) -> Model
    where
        Self: Sized,
    {
        let settings = self.settings();
        let light_positions = self
            .light_positions()
            .iter()
            .map(|position| position.as_vec3().to_array().into())
            .collect();
        let HexColor { r, g, b, a } = self.background_color();
        let background_color = Srgba::new(r, g, b, a);

        let objects = self
            .components()
            .into_iter()
            .map(|component| {
                let mesh = component.mesh(settings);
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

                CpuObject {
                    mesh,
                    color,
                    transformations,
                }
            })
            .collect();

        Model {
            objects,
            light_positions,
            background_color,
        }
    }
}

/// Settings to use for meshing
#[derive(Copy, Clone)]
pub struct MeshSettings {
    pub threads: u8,
    pub resolution: f64,
}

/// A CPU-side version of an object.
#[derive(Clone)]
pub struct CpuObject {
    pub mesh: CpuMesh,
    pub color: Srgba,
    pub transformations: Option<Vec<Mat4>>,
}

/// A model consisting of objects and other settings.
#[derive(Clone)]
pub struct Model {
    pub objects: Vec<CpuObject>,
    pub light_positions: Vec<Vec3>,
    pub background_color: Srgba,
}
