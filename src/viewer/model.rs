use fidget::mesh::{Mesh as FidgetMesh, Settings};
use glam::{DAffine3, DMat4};
use three_d::{CpuMesh, Indices, Mat4, Positions};

use crate::{
    config::{Colors, Preview},
    model,
};

pub struct Model {
    pub keyboard: CpuMesh,
    pub finger_key_positions: Vec<Mat4>,
    pub thumb_key_positions: Vec<Mat4>,
    pub colors: Colors,
    pub settings: Preview,
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
        self.keyboard.mesh_settings(self.settings.resolution.into())
    }

    fn mesh(&self, settings: Settings) -> Model {
        let mesh = self.keyboard.mesh(settings);
        let keyboard = mesh.into_cpu_mesh();

        let finger_key_positions = self
            .key_positions
            .columns
            .iter()
            .flat_map(|column| column.iter().copied().flat_map(mirrored_positions))
            .collect();
        let thumb_key_positions = self
            .key_positions
            .thumb_keys
            .iter()
            .copied()
            .flat_map(mirrored_positions)
            .collect();

        Model {
            keyboard,
            finger_key_positions,
            thumb_key_positions,
            colors: self.colors.clone(),
            settings: self.settings.clone(),
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

/// Creates two key positions mirrored along the xy-plane given a single one.
fn mirrored_positions(position: DAffine3) -> [Mat4; 2] {
    let matrix: DMat4 = position.into();
    let position = matrix.as_mat4().to_cols_array_2d().into();
    let mirror_transform = Mat4::from_nonuniform_scale(-1.0, 1.0, 1.0);

    [position, mirror_transform * position]
}
