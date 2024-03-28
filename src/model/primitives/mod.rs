mod operations;
mod shapes2d;
mod shapes3d;
mod vector;

pub type Result<T> = std::result::Result<T, fidget::Error>;

use fidget::{
    context::Node,
    eval::MathShape,
    jit::JitShape,
    mesh::{Mesh, Octree, Settings},
    shape, Context,
};
use glam::DVec3;

pub use operations::*;
pub use shapes2d::*;
pub use shapes3d::*;

use crate::viewer::MeshSettings;

/// A generic shape
pub struct Shape {
    inner: JitShape,
    bounds: Bounds,
}

impl Shape {
    /// Creates a new shape from a context, root node and bounds.
    ///
    /// Returns [`fidget::Error`] if the root node does not belong to the same context.
    pub fn new(context: &Context, root: Node, bounds: Bounds) -> Result<Self> {
        let inner = JitShape::new(context, root)?;
        Ok(Self { inner, bounds })
    }

    /// Meshes the shape.
    pub fn mesh(&self, settings: MeshSettings) -> Mesh {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let depth = ((self.bounds.size / settings.resolution).log2().ceil() as u8).max(1);
        #[allow(clippy::cast_possible_truncation)]
        let size = (f64::from(2u32.pow(u32::from(depth - 1))) * settings.resolution) as f32;

        let center = self.bounds.center.as_vec3().to_array().into();
        let bounds = shape::Bounds { center, size };
        let settings = Settings {
            threads: settings.threads,
            min_depth: depth,
            max_depth: depth,
            bounds,
        };

        Octree::build(&self.inner, settings).walk_dual(settings)
    }
}

/// A cubical bounded region used for meshing.
pub struct Bounds {
    size: f64,
    center: DVec3,
}

impl Bounds {
    /// Creates new bounds given a size and center point.
    pub fn new(size: f64, center: DVec3) -> Self {
        Self { size, center }
    }
}
