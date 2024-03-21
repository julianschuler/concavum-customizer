mod operations;
mod shapes;
mod vector;

type Result<T> = std::result::Result<T, fidget::Error>;

use fidget::{
    context::Node,
    eval::MathShape,
    jit::JitShape,
    mesh::{Mesh, Octree, Settings},
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
    pub fn new(context: Context, root: Node, bounding_box: BoundingBox) -> Result<Self> {
        let inner = JitShape::new(&context, root)?;

        Ok(Self {
            inner,
            bounding_box,
        })
    }

    /// Meshes the shape.
    pub fn mesh(&self, settings: MeshSettings) -> Mesh {
        let settings = Settings {
            threads: settings.threads,
            min_depth: settings.min_depth,
            max_depth: settings.max_depth,
        };
        Octree::build(&self.inner, settings).walk_dual(settings)
    }
}

/// A bounding box given by two corner points
#[derive(Clone)]
pub struct BoundingBox {
    a: DVec3,
    b: DVec3,
}

impl BoundingBox {
    pub fn new(a: DVec3, b: DVec3) -> Self {
        Self { a, b }
    }

    pub fn size(&self) -> DVec3 {
        self.b - self.a
    }
}
