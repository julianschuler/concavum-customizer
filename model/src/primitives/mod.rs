mod operations;
mod shapes2d;
mod shapes3d;
mod vector;

#[cfg(not(target_arch = "wasm32"))]
use fidget::jit::JitShape as FidgetShape;
#[cfg(target_arch = "wasm32")]
use fidget::vm::VmShape as FidgetShape;

use fidget::{
    context::Tree,
    mesh::{Mesh, Octree, Settings},
    shape, Context,
};
use glam::DVec3;

pub use operations::*;
pub use shapes2d::*;
pub use shapes3d::*;

pub const EPSILON: f64 = 0.001;

/// A generic shape.
pub struct Shape {
    inner: FidgetShape,
    bounds: Bounds,
}

impl Shape {
    /// Creates a new shape from a context, root node and bounds.
    ///
    /// Returns [`fidget::Error`] if the root node does not belong to the same context.
    pub fn new(tree: &Tree, bounds: Bounds) -> Self {
        let mut context = Context::new();
        let root = context.import(tree);
        let inner =
            FidgetShape::new(&context, root).expect("root node should belong to same context");
        Self { inner, bounds }
    }

    /// Meshes the shape.
    pub fn mesh(&self, settings: Settings) -> Mesh {
        Octree::build(&self.inner, settings).walk_dual(settings)
    }

    /// Returns the settings to use for meshing the shape with the given resolution.
    pub fn mesh_settings(&self, resolution: f64) -> Settings {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let depth = (self.bounds.size / resolution).log2().ceil() as u8;
        #[allow(clippy::cast_possible_truncation)]
        let size = (f64::from(2u32.pow(u32::from(depth.max(1) - 1))) * resolution) as f32;

        let center = self.bounds.center.as_vec3().to_array().into();
        let bounds = shape::Bounds { center, size };

        Settings {
            depth,
            bounds,
            #[cfg(not(target_arch = "wasm32"))]
            threads: std::thread::available_parallelism()
                .expect("available parallelism should be known"),
        }
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

/// A trait for converting a shape into a `Tree`.
pub trait IntoTree {
    /// Converts `self` into a `Tree`
    fn into_tree(self) -> Tree;
}

impl<T: Into<Tree>> IntoTree for T {
    fn into_tree(self) -> Tree {
        self.into()
    }
}
