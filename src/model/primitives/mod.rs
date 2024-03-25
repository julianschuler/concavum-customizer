mod operations;
mod shapes;
mod vector;

type Result<T> = std::result::Result<T, fidget::Error>;

use fidget::{
    context::Node,
    eval::{types::Interval, MathShape},
    jit::JitShape,
    mesh::{CellBounds, Mesh, Octree, Settings},
    Context,
};
use glam::DVec3;

pub use operations::*;
pub use shapes::*;

use crate::viewer::MeshSettings;

/// A generic sphere given by a context, root node and bounding box
pub struct Shape {
    inner: JitShape,
    bounding_box: BoundingBox,
}

impl Shape {
    /// Creates a new shape from a context, root node and bounding box.
    ///
    /// Returns [`fidget::Error`] if the root node does not belong to the same context.
    pub fn new(context: &Context, root: Node, bounding_box: BoundingBox) -> Result<Self> {
        let inner = JitShape::new(context, root)?;

        Ok(Self {
            inner,
            bounding_box,
        })
    }

    /// Meshes the shape.
    pub fn mesh(&self, settings: MeshSettings) -> Mesh {
        let center = self.bounding_box.center.as_vec3();
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let depth = (self.bounding_box.size / settings.resolution).log2().ceil() as u8;
        #[allow(clippy::cast_possible_truncation)]
        let offset = (f64::from(2u32.pow(u32::from(depth - 1))) * settings.resolution) as f32;

        let bounds = CellBounds {
            x: Interval::new(center.x - offset, center.x + offset),
            y: Interval::new(center.y - offset, center.y + offset),
            z: Interval::new(center.z - offset, center.z + offset),
        };
        let settings = Settings {
            threads: settings.threads,
            min_depth: depth,
            max_depth: depth,
            bounds,
        };

        Octree::build(&self.inner, settings).walk_dual(settings)
    }
}

/// A bounding box given by a size and a center point.
pub struct BoundingBox {
    size: f64,
    center: DVec3,
}

impl BoundingBox {
    /// Creates a new bounding box from a size and a center point.
    pub fn new(size: f64, center: DVec3) -> Self {
        Self { size, center }
    }
}
