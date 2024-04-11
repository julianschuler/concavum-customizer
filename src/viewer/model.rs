use glam::DMat4;
use three_d::{CpuMesh, Indices, Mat4, Positions, Vec3};

use crate::{
    config::Colors,
    model::{self, MeshSettings, Shape},
};

pub struct Model {
    pub keyboard: CpuMesh,
    pub finger_key_positions: Vec<Mat4>,
    pub thumb_key_positions: Vec<Mat4>,
    pub light_positions: Vec<Vec3>,
    pub colors: Colors,
}

/// A trait for displaying a keyboard
pub trait IntoShape {
    /// Converts self into a model.
    fn into_model(self) -> Model;
}

impl IntoShape for model::Model {
    fn into_model(self) -> Model {
        let settings = MeshSettings {
            threads: 12,
            resolution: self.resolution,
        };
        let keyboard = Mesh::mesh(&self.keyboard, settings);
        let light_positions = self
            .light_positions
            .iter()
            .map(|position| position.as_vec3().to_array().into())
            .collect();

        let finger_key_positions = self
            .key_positions
            .columns
            .iter()
            .flat_map(|column| {
                column.iter().map(|&position| {
                    let matrix: DMat4 = position.into();
                    matrix.as_mat4().to_cols_array_2d().into()
                })
            })
            .collect();
        let thumb_key_positions = self
            .key_positions
            .thumb_keys
            .iter()
            .map(|&position| {
                let matrix: DMat4 = position.into();
                matrix.as_mat4().to_cols_array_2d().into()
            })
            .collect();

        Model {
            keyboard,
            finger_key_positions,
            thumb_key_positions,
            light_positions,
            colors: self.colors,
        }
    }
}

/// A trait for meshing a shape.
trait Mesh {
    /// Meshes the shape.
    fn mesh(&self, settings: MeshSettings) -> CpuMesh;
}

impl Mesh for Shape {
    fn mesh(&self, settings: MeshSettings) -> CpuMesh {
        let mesh = self.mesh(settings);

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

        CpuMesh {
            positions: Positions::F32(vertices),
            indices: Indices::U32(indices),
            ..Default::default()
        }
    }
}
