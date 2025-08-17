use std::iter::once;

use config::KeySize;
use glam::{DAffine3, DMat4};
use model::{
    matrix_pcb::{
        ClusterConnector, ColumnConnector, ColumnKeyConnectors, Segment, ThumbKeyConnectors,
        CONNECTOR_WIDTH, THICKNESS,
    },
    Bounds, Mesh as ModelMesh, MeshSettings, Model,
};
use three_d::{CpuMesh, Indices, Mat4, Positions, Vec3};

pub use model::DisplaySettings;

/// The settings for displaying a model.
#[derive(Clone, Default)]
pub struct Settings {
    /// The positions of the finger keys.
    pub finger_key_positions: Vec<Mat4>,
    /// The settings for the thumb keys.
    pub thumb_key_settings: ThumbKeySettings,
    /// The positions of the interface PCBs.
    pub interface_pcb_positions: Vec<Mat4>,
    /// The meshes of the matrix PCBs.
    pub matrix_pcb_meshes: Vec<InstancedMesh>,
    /// The positions of the FFC connector pad.
    pub ffc_pad_positions: Vec<Mat4>,
    /// The display settings of the model.
    pub display_settings: DisplaySettings,
    /// The positions of the illuminating point lights.
    pub light_positions: Vec<Vec3>,
}

#[derive(Clone, Default)]
/// The settings for the thumb keys.
pub struct ThumbKeySettings {
    /// The size of the thumb keys.
    pub key_size: KeySize,
    /// The positions of the thumb keys.
    pub thumb_key_positions: Vec<Mat4>,
}

pub fn make_settings(model: &Model, config: &config::Config) -> Settings {
    let finger_key_positions = model
        .key_positions
        .columns
        .iter()
        .flat_map(|column| column.iter().flat_map(mirrored_positions))
        .collect();
    let thumb_key_positions = model
        .key_positions
        .thumb_keys
        .iter()
        .flat_map(mirrored_positions)
        .collect();
    let thumb_key_settings = ThumbKeySettings {
        thumb_key_positions,
        key_size: config.thumb_cluster.key_size,
    };
    let interface_pcb_positions =
        mirrored_positions(&model.keyboard.interface_pcb_position).to_vec();

    let matrix_pcb_meshes = model
        .matrix_pcb
        .column_key_connectors
        .iter()
        .map(Into::into)
        .chain(once((&model.matrix_pcb.thumb_key_connectors).into()))
        .chain(model.matrix_pcb.column_connectors.iter().map(Into::into))
        .chain(once((&model.matrix_pcb.cluster_connector).into()))
        .collect();
    let ffc_pad_positions = mirrored_positions(&model.matrix_pcb.ffc_pad_position).to_vec();
    let light_positions = light_positions_from_bounds(model.keyboard.case.bounds());

    Settings {
        finger_key_positions,
        thumb_key_settings,
        interface_pcb_positions,
        matrix_pcb_meshes,
        ffc_pad_positions,
        display_settings: model.display_settings.clone(),
        light_positions,
    }
}

/// The meshes of the keyboard and bottom plate.
#[derive(Clone)]
pub struct Meshes {
    /// The mesh of the case.
    pub case: CpuMesh,
    /// The mesh of the bottom plate.
    pub bottom_plate: CpuMesh,
}

/// A trait for meshing a model.
pub trait Mesh<'a> {
    /// Returns the mesh settings for meshing the preview at the set resolution.
    fn mesh_settings_preview(&self) -> MeshSettings<'_>;

    /// Meshes the preview using the given mesh settings.
    fn mesh_preview(&self, settings: MeshSettings) -> CpuMesh;

    /// Meshes `self`.
    fn meshes(&self) -> Meshes;
}

impl Mesh<'_> for Model {
    fn mesh_settings_preview(&self) -> MeshSettings<'_> {
        self.keyboard.preview.mesh_settings(self.resolution)
    }

    fn mesh_preview(&self, settings: MeshSettings) -> CpuMesh {
        self.keyboard.preview.mesh(settings).to_cpu_mesh()
    }

    fn meshes(&self) -> Meshes {
        let settings = self.keyboard.case.mesh_settings(self.resolution);
        let case = self.keyboard.case.mesh(settings).to_cpu_mesh();

        let settings = self.keyboard.bottom_plate.mesh_settings(self.resolution);
        let bottom_plate = self.keyboard.bottom_plate.mesh(settings).to_cpu_mesh();

        Meshes { case, bottom_plate }
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

/// An instanced mesh.
#[derive(Clone, Default)]
pub struct InstancedMesh {
    /// The mesh itself.
    pub mesh: CpuMesh,
    /// The transformations to apply to the mesh.
    pub transformations: Vec<Mat4>,
}

impl From<&ColumnKeyConnectors> for InstancedMesh {
    fn from(connectors: &ColumnKeyConnectors) -> Self {
        let mesh = segment_to_mesh(&connectors.connector);
        let transformations = connectors
            .positions
            .iter()
            .flat_map(|(left, right)| [left, right])
            .flat_map(mirrored_positions)
            .collect();

        Self {
            mesh,
            transformations,
        }
    }
}

impl From<&ThumbKeyConnectors> for InstancedMesh {
    fn from(connectors: &ThumbKeyConnectors) -> Self {
        let mesh = segment_to_mesh(&connectors.connector);
        let transformations = connectors
            .positions
            .iter()
            .flat_map(|(left, right)| [left, right])
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

/// Creates the light positios from the bounds of a keyboard half.
fn light_positions_from_bounds(bounds: Bounds) -> Vec<Vec3> {
    const DISTANCE: f32 = 100.0;

    let min = bounds.min.as_vec3();
    let max = bounds.max.as_vec3();
    let center = bounds.center().as_vec3();

    let from_below = Vec3::new(center.x, center.y, -DISTANCE);
    let from_above_back = Vec3::new(max.x + DISTANCE, max.y + DISTANCE, max.z + DISTANCE);
    let from_above_front = Vec3::new(from_above_back.x, min.y - DISTANCE, from_above_back.z);

    [from_below, from_above_front, from_above_back]
        .into_iter()
        .flat_map(|position| [position, Vec3::new(-position.x, position.y, position.z)])
        .collect()
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
