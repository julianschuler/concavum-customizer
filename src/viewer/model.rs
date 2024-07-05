use fidget::mesh::{Mesh as FidgetMesh, Settings};
use glam::{DAffine3, DMat4};
use three_d::{CpuMesh, Indices, Mat4, Positions};

use crate::{
    config::{Colors, Preview},
    model,
};

#[derive(Clone)]
pub struct Model {
    pub finger_key_positions: Vec<Mat4>,
    pub thumb_key_positions: Vec<Mat4>,
    pub interface_pcb_positions: Vec<Mat4>,
    pub colors: Colors,
    pub settings: Preview,
}

impl From<&model::Model> for Model {
    fn from(model: &model::Model) -> Self {
        let finger_key_positions = model
            .keyboard
            .key_positions
            .columns
            .iter()
            .flat_map(|column| column.iter().copied().flat_map(mirrored_positions))
            .collect();
        let thumb_key_positions = model
            .keyboard
            .key_positions
            .thumb_keys
            .iter()
            .copied()
            .flat_map(mirrored_positions)
            .collect();
        let interface_pcb_positions =
            mirrored_positions(model.keyboard.interface_pcb_position).to_vec();

        Self {
            finger_key_positions,
            thumb_key_positions,
            interface_pcb_positions,
            colors: model.colors.clone(),
            settings: model.settings.clone(),
        }
    }
}

#[derive(Clone)]
pub struct Meshes {
    pub keyboard: CpuMesh,
    pub bottom_plate: CpuMesh,
}

/// A trait for meshing a model.
pub trait Mesh {
    /// Returns the mesh settings for meshing the preview at the set resolution.
    fn mesh_settings_preview(&self) -> Settings;

    /// Meshes the preview using the given mesh settings.
    fn mesh_preview(&self, settings: Settings) -> CpuMesh;

    /// Meshes self.
    fn meshes(&self) -> Meshes;
}

impl Mesh for model::Model {
    fn mesh_settings_preview(&self) -> Settings {
        self.keyboard
            .preview
            .mesh_settings(self.settings.resolution.into())
    }

    fn mesh_preview(&self, settings: Settings) -> CpuMesh {
        self.keyboard.preview.mesh(settings).into_cpu_mesh()
    }

    fn meshes(&self) -> Meshes {
        let settings = self
            .keyboard
            .shape
            .mesh_settings(self.settings.resolution.into());
        let keyboard = self.keyboard.shape.mesh(settings).into_cpu_mesh();

        let settings = self
            .keyboard
            .bottom_plate
            .mesh_settings(self.settings.resolution.into());
        let bottom_plate = self.keyboard.bottom_plate.mesh(settings).into_cpu_mesh();

        Meshes {
            keyboard,
            bottom_plate,
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
