use std::{ops::Deref, path::Path};

use fj_core::{
    algorithms::{
        approx::{InvalidTolerance, Tolerance},
        bounding_volume::BoundingVolume,
        sweep::Sweep,
        triangulate::Triangulate,
    },
    objects::{Cycle, Region, Sketch, Solid},
    operations::{
        BuildCycle, BuildRegion, BuildSketch, Insert, Reverse, UpdateRegion, UpdateSketch,
    },
    services::Services,
    storage::Handle,
};
use fj_interop::mesh::Mesh;
use fj_math::{Aabb, Point, Scalar, Vector};

use crate::config::{self, Config};

pub struct Model {
    handle: Handle<Solid>,
}

fn triangulate_model<M>(
    model: impl Deref<Target = M>,
    aabb: &Aabb<3>,
) -> Result<Mesh<Point<3>>, InvalidTolerance>
where
    for<'r> (&'r M, Tolerance): Triangulate,
    M: BoundingVolume<3>,
{
    let min_extent = aabb
        .size()
        .components
        .into_iter()
        .filter(|extent| *extent != Scalar::ZERO)
        .min()
        .unwrap_or(Scalar::MAX);

    let tolerance = Tolerance::from_scalar(min_extent / Scalar::from_f64(1000.))?;

    Ok((model.deref(), tolerance).triangulate())
}

impl Into<fj_interop::model::Model> for Model {
    fn into(self) -> fj_interop::model::Model {
        let model = self.handle;

        let aabb = model.aabb().unwrap_or(Aabb {
            min: Point::origin(),
            max: Point::origin(),
        });
        let mesh = triangulate_model(model, &aabb).expect("tolerance should be valid");

        fj_interop::model::Model { mesh, aabb }
    }
}

pub fn dummy_model(config: Config) -> Handle<Solid> {
    let services = &mut Services::new();
    let sketch = Sketch::empty()
        .add_region(
            Region::circle(Point::origin(), config.outer, services)
                .add_interiors([Cycle::circle(Point::origin(), config.inner, services)
                    .reverse(services)
                    .insert(services)])
                .insert(services),
        )
        .insert(services);

    let surface = services.objects.surfaces.xy_plane();
    let path = Vector::from([0., 0., config.height]);
    (sketch, surface).sweep(path, services)
}

impl Model {
    pub fn try_from_config(config_path: &Path) -> Result<Self, Error> {
        let config = Config::try_from_path(config_path)?;
        let handle = dummy_model(config);

        Ok(Self { handle })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error parsing config
    #[error("Error parsing config")]
    ConfigParsing(#[from] config::Error),
}
