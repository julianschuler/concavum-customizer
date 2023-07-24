use fj_interop::{
    mesh::{Color, Mesh},
    model::Model,
};
use fj_math::{Aabb, Point, Triangle, Vector};
use hex_color::HexColor;
use opencascade::primitives::Shape;
use opencascade_sys::ffi::{
    BRepMesh_IncrementalMesh_ctor, BRep_Tool_Triangulation, Handle_Poly_Triangulation_Get,
    Poly_Triangulation_Node, TopAbs_Orientation, TopAbs_ShapeEnum, TopExp_Explorer_ctor,
    TopLoc_Location_ctor, TopoDS_cast_to_face,
};

pub struct Component {
    shape: Shape,
    color: HexColor,
}

impl Component {
    pub fn new(shape: Shape, color: HexColor) -> Self {
        Self { shape, color }
    }

    fn try_into_triangles(self, deflection_tolerance: f64) -> Result<Vec<Triangle<3>>, Error> {
        let mut triangles = Vec::new();

        let mut triangulation =
            BRepMesh_IncrementalMesh_ctor(&self.shape.inner, deflection_tolerance);
        if !triangulation.IsDone() {
            return Err(Error::Triangulation);
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
                            components: [point.X().into(), point.Z().into(), point.Y().into()],
                        };
                        triangle_points[corner_index] = Point { coords };
                    }

                    if face_orientation == TopAbs_Orientation::TopAbs_FORWARD {
                        triangle_points.reverse();
                    }

                    if let Ok(triangle) = Triangle::from_points(triangle_points) {
                        triangles.push(triangle);
                    }
                }
            }

            face_explorer.pin_mut().Next();
        }

        Ok(triangles)
    }
}

pub trait ViewableModel {
    fn triangulation_tolerance(&self) -> f64;

    fn components(self) -> Vec<Component>;

    fn into_mesh_model(self) -> fj_interop::model::Model
    where
        Self: Sized,
    {
        let triangulation_tolerance = self.triangulation_tolerance();
        let mut mesh = Mesh::new();

        for component in self.components() {
            let color = Color(component.color.to_be_bytes());

            match component.try_into_triangles(triangulation_tolerance) {
                Ok(triangles) => {
                    for triangle in triangles {
                        mesh.push_triangle(triangle, color);
                    }
                }
                Err(_) => {
                    eprintln!("Warning: Could not triangulate component, ignoring it");
                }
            }
        }

        let aabb = Aabb::<3>::from_points(mesh.vertices());

        Model { mesh, aabb }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Triangulation using BRepMesh_IncrementalMesh_ctor failed
    #[error("Triangulation using BRepMesh_IncrementalMesh_ctor failed")]
    Triangulation,
}
