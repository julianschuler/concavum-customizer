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
    render::{ThreadPool, View3},
    Context,
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
            FidgetShape::new(&context, root).expect("root node should belong to the same context");
        Self { inner, bounds }
    }

    /// Meshes the shape.
    pub fn mesh(&self, settings: Settings) -> Mesh {
        Octree::build(&self.inner, settings).walk_dual(settings)
    }

    /// Returns the settings to use for meshing the shape with the given resolution.
    pub fn mesh_settings(&self, resolution: f64) -> Settings {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let depth = (self.bounds.size().max_element() / resolution)
            .log2()
            .ceil() as u8;
        #[allow(clippy::cast_possible_truncation)]
        let size = (f64::from(2u32.pow(u32::from(depth.max(1) - 1))) * resolution) as f32;

        let center = self.bounds.center().as_vec3().to_array().into();
        let view = View3::from_center_and_scale(center, size);

        Settings {
            depth,
            view,
            threads: Some(&ThreadPool::Global),
        }
    }

    /// Returns the bounds of the shape.
    pub fn bounds(&self) -> Bounds {
        self.bounds
    }
}

/// Bounded region containing a cluster given by two points.
#[derive(Copy, Clone)]
pub struct Bounds {
    /// Corner point with minimal coordinates.
    pub min: DVec3,
    /// Corner point with maximal coordinates.
    pub max: DVec3,
}

impl Bounds {
    /// Returns the size of the bounds.
    #[must_use]
    pub fn size(&self) -> DVec3 {
        self.max - self.min
    }

    /// Returns the center point of the bounds.
    #[must_use]
    pub fn center(&self) -> DVec3 {
        (self.min + self.max) / 2.0
    }

    /// Returns the diameter of the bounds.
    #[must_use]
    pub fn diameter(&self) -> f64 {
        self.size().length()
    }

    /// Combines two bounds.
    #[must_use]
    pub fn union(&self, other: Self) -> Self {
        let min = self.min.min(other.min);
        let max = self.max.max(other.max);

        Self { min, max }
    }

    /// Mirrors the bound along the YZ-plane.
    #[must_use]
    pub fn mirror_yz(&self) -> Self {
        let min = self.min.with_x(-self.max.x);
        let max = self.max.with_x(-self.min.x);

        Self { min, max }
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
