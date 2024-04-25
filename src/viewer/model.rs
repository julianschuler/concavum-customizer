use fidget::mesh::{Mesh as FidgetMesh, Settings};
use glam::DMat4;
use three_d::{CpuMesh, Indices, Mat4, Positions, Vec3};

use crate::{config::Colors, model};

pub struct Model {
    pub keyboard: CpuMesh,
    pub finger_key_positions: Vec<Mat4>,
    pub thumb_key_positions: Vec<Mat4>,
    pub light_positions: Vec<Vec3>,
    pub colors: Colors,
}

/// A trait for meshing a model.
pub trait Mesh {
    /// Returns the mesh settings for meshing at the set resolution.
    fn mesh_settings(&self) -> Settings;

    /// Converts self into a mesh model using the given mesh settings.
    fn mesh(&self, settings: Settings) -> Model;
}

impl Mesh for model::Model {
    fn mesh_settings(&self) -> Settings {
        self.keyboard.mesh_settings(self.resolution)
    }

    fn mesh(&self, settings: Settings) -> Model {
        let mesh = self.keyboard.mesh(settings);
        let keyboard = mesh.into_cpu_mesh();

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
            colors: self.colors.clone(),
        }
    }
}

/// A trait for converting a mesh into a `CpuMesh`.
trait IntoCpuMesh {
    /// Converts self into a `CpuMesh`.
    fn into_cpu_mesh(self) -> CpuMesh;
}

impl IntoCpuMesh for FidgetMesh {
    fn into_cpu_mesh(self) -> CpuMesh {
        let vertices = self
            .vertices
            .iter()
            .map(|vertex| {
                let slice: [_; 3] = vertex.as_slice().try_into().unwrap();
                slice.into()
            })
            .collect();
        let indices = self
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
