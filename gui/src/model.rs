use config::{Colors, Preview};
use glam::{DAffine3, DMat4};
use model::{Mesh as ModelMesh, MeshSettings, Model};
use three_d::{CpuMesh, Indices, Mat4, Positions};

/// The settings for displaying a model.
#[derive(Clone)]
pub struct Settings {
    /// The positions of the finger keys.
    pub finger_key_positions: Vec<Mat4>,
    /// The positions of the thumb keys.
    pub thumb_key_positions: Vec<Mat4>,
    /// Positions of the interface PCBs.
    pub interface_pcb_positions: Vec<Mat4>,
    /// The resolution used for meshing.
    pub resolution: f64,
    /// The colors of the model.
    pub colors: Colors,
    /// The preview settings of the model.
    pub settings: Preview,
}

impl From<&Model> for Settings {
    fn from(model: &Model) -> Self {
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
            resolution: model.resolution,
            colors: model.colors.clone(),
            settings: model.settings.clone(),
        }
    }
}

/// The meshes of the keyboard and bottom plate.
#[derive(Clone)]
pub struct Meshes {
    /// The mesh of the keyboard.
    pub keyboard: CpuMesh,
    /// The mesh of the bottom plate.
    pub bottom_plate: CpuMesh,
}

/// A trait for meshing a model.
pub trait Mesh {
    /// Returns the mesh settings for meshing the preview at the set resolution.
    fn mesh_settings_preview(&self) -> MeshSettings;

    /// Meshes the preview using the given mesh settings.
    fn mesh_preview(&self, settings: MeshSettings) -> CpuMesh;

    /// Meshes self.
    fn meshes(&self) -> Meshes;
}

impl Mesh for Model {
    fn mesh_settings_preview(&self) -> MeshSettings {
        self.keyboard.preview.mesh_settings(self.resolution)
    }

    fn mesh_preview(&self, settings: MeshSettings) -> CpuMesh {
        self.keyboard.preview.mesh(settings).into_cpu_mesh()
    }

    fn meshes(&self) -> Meshes {
        let settings = self.keyboard.shape.mesh_settings(self.resolution);
        let keyboard = self.keyboard.shape.mesh(settings).into_cpu_mesh();

        let settings = self.keyboard.bottom_plate.mesh_settings(self.resolution);
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

impl IntoCpuMesh for ModelMesh {
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

/// Creates two key positions mirrored along the XY-plane given a single one.
fn mirrored_positions(position: DAffine3) -> [Mat4; 2] {
    let matrix: DMat4 = position.into();
    let position = matrix.as_mat4().to_cols_array_2d().into();
    let mirror_transform = Mat4::from_nonuniform_scale(-1.0, 1.0, 1.0);

    [position, mirror_transform * position]
}
