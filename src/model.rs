use std::path::Path;

use cxx::UniquePtr;
use fj_interop::mesh::{Color, Mesh};
use fj_math::{Aabb, Point, Triangle, Vector};
use opencascade_sys::ffi::{
    new_point, BRepMesh_IncrementalMesh_ctor, BRepPrimAPI_MakeBox_ctor, BRep_Tool_Triangulation,
    Handle_Poly_Triangulation_Get, Poly_Triangulation_Node, TopAbs_Orientation, TopAbs_ShapeEnum,
    TopExp_Explorer_ctor, TopLoc_Location_ctor, TopoDS_Shape, TopoDS_Shape_to_owned,
    TopoDS_cast_to_face,
};

use crate::config::{self, Config};

type Shape = UniquePtr<TopoDS_Shape>;

pub struct Model {
    shape: Shape,
}

impl Into<fj_interop::model::Model> for Model {
    fn into(self) -> fj_interop::model::Model {
        let mut mesh = Mesh::new();

        let mut triangulation = BRepMesh_IncrementalMesh_ctor(&self.shape, 0.01);
        if !triangulation.IsDone() {
            panic!("Call to BRepMesh_IncrementalMesh_ctor failed");
        }

        let mut face_explorer = TopExp_Explorer_ctor(
            triangulation.pin_mut().Shape(),
            TopAbs_ShapeEnum::TopAbs_FACE,
        );
        while face_explorer.More() {
            let mut location = TopLoc_Location_ctor();
            let face = TopoDS_cast_to_face(face_explorer.Current());
            let face_orientation = face.Orientation();

            let triangulation_handle = BRep_Tool_Triangulation(face, location.pin_mut());
            if let Ok(triangulation) = Handle_Poly_Triangulation_Get(&triangulation_handle) {
                for index in 0..triangulation.NbTriangles() {
                    let mut triangle_points = [Point::default(); 3];

                    let triangle = triangulation.Triangle(index + 1);

                    for corner_index in 0..3 {
                        let mut point = Poly_Triangulation_Node(
                            triangulation,
                            triangle.Value(corner_index as i32 + 1),
                        );
                        let point = point.pin_mut();

                        let coords = Vector {
                            components: [point.X().into(), point.Y().into(), point.Z().into()],
                        };
                        triangle_points[corner_index] = Point { coords };
                    }

                    if face_orientation == TopAbs_Orientation::TopAbs_REVERSED {
                        triangle_points.reverse();
                    }

                    if let Ok(triangle) = Triangle::from_points(triangle_points) {
                        mesh.push_triangle(triangle, Color::default())
                    }
                }
            }

            face_explorer.pin_mut().Next();
        }

        let aabb = Aabb::<3>::from_points(mesh.vertices());

        fj_interop::model::Model { mesh, aabb }
    }
}

pub fn dummy_model(config: Config) -> Shape {
    let origin = new_point(0.0, 0.0, 0.0);
    let mut cube = BRepPrimAPI_MakeBox_ctor(&origin, config.width, config.length, config.height);
    TopoDS_Shape_to_owned(cube.pin_mut().Shape())
}

impl Model {
    pub fn try_from_config(config_path: &Path) -> Result<Self, Error> {
        let config = Config::try_from_path(config_path)?;
        let shape = dummy_model(config);

        Ok(Self { shape })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error parsing config
    #[error("Error parsing config")]
    ConfigParsing(#[from] config::Error),
}
