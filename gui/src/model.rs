use std::{io::Write, iter::once};

use glam::{DAffine3, DMat4};
use model::{
    matrix_pcb::{
        ClusterConnector, ColumnConnector, KeyConnectors, Segment, CONNECTOR_WIDTH, THICKNESS,
    },
    Mesh as ModelMesh, MeshSettings, Model,
};
use three_d::{CpuMesh, Indices, Mat4, Positions};

pub use model::DisplaySettings;

use crate::Error;

/// The settings for displaying a model.
#[derive(Clone, Default)]
pub struct Settings {
    /// The positions of the finger keys.
    pub finger_key_positions: Vec<Mat4>,
    /// The positions of the thumb keys.
    pub thumb_key_positions: Vec<Mat4>,
    /// The positions of the interface PCBs.
    pub interface_pcb_positions: Vec<Mat4>,
    /// The meshes of the matrix PCBs.
    pub matrix_pcb_meshes: Vec<InstancedMesh>,
    /// The display settings of the model.
    pub display_settings: DisplaySettings,
}

impl From<&Model> for Settings {
    fn from(model: &Model) -> Self {
        let finger_key_positions = model
            .keyboard
            .key_positions
            .columns
            .iter()
            .flat_map(|column| column.iter().flat_map(mirrored_positions))
            .collect();
        let thumb_key_positions = model
            .keyboard
            .key_positions
            .thumb_keys
            .iter()
            .flat_map(mirrored_positions)
            .collect();
        let interface_pcb_positions =
            mirrored_positions(&model.keyboard.interface_pcb_position).to_vec();
        let matrix_pcb = &model.keyboard.matrix_pcb;

        let matrix_pcb_meshes = matrix_pcb
            .key_connectors
            .iter()
            .map(Into::into)
            .chain(matrix_pcb.column_connectors.iter().map(Into::into))
            .chain(once((&matrix_pcb.cluster_connector).into()))
            .collect();

        Self {
            finger_key_positions,
            thumb_key_positions,
            interface_pcb_positions,
            matrix_pcb_meshes,
            display_settings: model.display_settings.clone(),
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

    /// Meshes `self`.
    fn meshes(&self) -> Meshes;
}

impl Mesh for Model {
    fn mesh_settings_preview(&self) -> MeshSettings {
        self.keyboard.preview.mesh_settings(self.resolution)
    }

    fn mesh_preview(&self, settings: MeshSettings) -> CpuMesh {
        self.keyboard.preview.mesh(settings).to_cpu_mesh()
    }

    fn meshes(&self) -> Meshes {
        let settings = self.keyboard.shape.mesh_settings(self.resolution);
        let keyboard = self.keyboard.shape.mesh(settings).to_cpu_mesh();

        let settings = self.keyboard.bottom_plate.mesh_settings(self.resolution);
        let bottom_plate = self.keyboard.bottom_plate.mesh(settings).to_cpu_mesh();

        Meshes {
            keyboard,
            bottom_plate,
        }
    }
}

/// A trait for converting `self` to a `CpuMesh`.
trait ToCpuMesh {
    /// Converts `self` to a `CpuMesh`.
    fn to_cpu_mesh(&self) -> CpuMesh;
}

impl ToCpuMesh for ModelMesh {
    fn to_cpu_mesh(&self) -> CpuMesh {
        let vertices = self
            .vertices
            .iter()
            .map(|vertex| {
                let slice: [_; 3] = vertex
                    .as_slice()
                    .try_into()
                    .expect("the slice has three elements");
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

/// A trait for writing a mesh to a binary STL.
pub trait WriteStl: Write {
    /// Writes the given mesh to a binary STL.
    fn write_stl(&mut self, mesh: CpuMesh) -> Result<(), Error>;
}

impl<W: Write> WriteStl for W {
    fn write_stl(&mut self, mesh: CpuMesh) -> Result<(), Error> {
        const HEADER: &[u8] = b"This is a binary STL file exported by the concavum customizer";

        if let CpuMesh {
            positions: Positions::F32(vertices),
            indices: Indices::U32(indices),
            ..
        } = mesh
        {
            let triangle_count = indices.len() / 3;

            self.write_all(HEADER)?;
            self.write_all(&[0; 80 - HEADER.len()])?;
            #[allow(clippy::cast_possible_truncation)]
            self.write_all(&(triangle_count as u32).to_le_bytes())?;

            for triangle in indices.chunks_exact(3) {
                let a = vertices[triangle[0] as usize];
                let b = vertices[triangle[1] as usize];
                let c = vertices[triangle[2] as usize];

                let normal = (b - a).cross(c - a);

                for vector in [normal, a, b, c] {
                    self.write_all(&vector.x.to_le_bytes())?;
                    self.write_all(&vector.y.to_le_bytes())?;
                    self.write_all(&vector.z.to_le_bytes())?;
                }
                self.write_all(&[0; size_of::<u16>()])?;
            }

            Ok(())
        } else {
            Err(Error::InvalidMesh("The mesh representation is invalid"))
        }
    }
}

/// An instanced mesh.
#[derive(Clone, Default)]
pub struct InstancedMesh {
    /// The mesh itself.
    pub mesh: CpuMesh,
    /// The transformations to apply to the mesh.
    pub transformations: Vec<Mat4>,
}

impl From<&KeyConnectors> for InstancedMesh {
    fn from(connectors: &KeyConnectors) -> Self {
        let mesh = segment_to_mesh(&connectors.connector);
        let transformations = connectors
            .positions
            .iter()
            .flat_map(mirrored_positions)
            .collect();

        Self {
            mesh,
            transformations,
        }
    }
}

impl From<&ColumnConnector> for InstancedMesh {
    fn from(connector: &ColumnConnector) -> Self {
        let (mesh, position) = match connector {
            ColumnConnector::Normal(connector) => (segment_to_mesh(connector), DAffine3::IDENTITY),
            ColumnConnector::Side(connector) => (segment_to_mesh(connector), connector.position),
        };

        let transformations = mirrored_positions(&position).to_vec();

        Self {
            mesh,
            transformations,
        }
    }
}

impl From<&ClusterConnector> for InstancedMesh {
    fn from(cluster_connector: &ClusterConnector) -> Self {
        let mesh = segment_to_mesh(cluster_connector);
        let transformations = mirrored_positions(&DAffine3::IDENTITY).to_vec();

        Self {
            mesh,
            transformations,
        }
    }
}

/// Creates two key positions mirrored along the YZ-plane given a single one.
fn mirrored_positions(position: &DAffine3) -> [Mat4; 2] {
    let matrix: DMat4 = (*position).into();
    let position = matrix.as_mat4().to_cols_array_2d().into();
    let mirror_transform = Mat4::from_nonuniform_scale(-1.0, 1.0, 1.0);

    [position, mirror_transform * position]
}

/// A macro returning the indices of the given quad faces.
macro_rules! face_indices {
    ($base_index:expr, $(($a:expr, $b:expr, $c:expr, $d:expr)),* $(,)?) => {
        [
            $(
                $base_index + $a, $base_index + $b, $base_index + $c,
                $base_index + $c, $base_index + $d, $base_index + $a,
            )*
        ]
    };
}

/// Converts a segment to a `CpuMesh`.
fn segment_to_mesh(segment: &impl Segment) -> CpuMesh {
    let positions = segment.positions();
    let indices = (0..(u32::try_from(positions.len()).expect("length should fit in u32") - 1))
        .flat_map(|segment_index| {
            face_indices!(
                4 * segment_index,
                (0, 1, 5, 4),
                (2, 1, 5, 6),
                (3, 2, 6, 7),
                (0, 3, 7, 4),
            )
        })
        .collect();
    let vertices = positions
        .into_iter()
        .flat_map(|position| {
            let center = position.translation;
            let x = CONNECTOR_WIDTH / 2.0 * position.x_axis;
            let z = THICKNESS / 2.0 * position.z_axis;

            [
                center - x - z,
                center + x - z,
                center + x + z,
                center - x + z,
            ]
        })
        .map(|vertex| vertex.as_vec3().to_array().into())
        .collect();

    CpuMesh {
        positions: Positions::F32(vertices),
        indices: Indices::U32(indices),
        ..Default::default()
    }
}
