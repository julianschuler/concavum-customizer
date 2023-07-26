use std::ops::Mul;

use fj_interop::{
    mesh::{Color, Mesh as FjMesh},
    model::Model,
};
use fj_math::{Aabb, Point, Triangle as FjTriangle};
use glam::{DAffine3, DVec3};
use hex_color::HexColor;
use opencascade::primitives::Shape;
use opencascade_sys::ffi::{
    BRepMesh_IncrementalMesh_ctor, BRep_Tool_Triangulation, Handle_Poly_Triangulation_Get,
    Poly_Triangulation_Node, TopAbs_Orientation, TopAbs_ShapeEnum, TopExp_Explorer_ctor,
    TopLoc_Location_ctor, TopoDS_cast_to_face,
};

pub struct Triangle {
    points: [DVec3; 3],
}

impl Triangle {
    pub fn new(points: [DVec3; 3]) -> Self {
        Self { points }
    }

    pub fn try_into_triangle(self) -> Option<FjTriangle<3>> {
        let points = self
            .points
            .map(|point| Point::from_array([point.x, point.z, point.y]));

        FjTriangle::from_points(points).ok()
    }
}

impl Mul<&Triangle> for &DAffine3 {
    type Output = Triangle;

    fn mul(self, triangle: &Triangle) -> Self::Output {
        Triangle::new(triangle.points.map(|point| self.transform_point3(point)))
    }
}

pub struct Mesh {
    triangles: Vec<Triangle>,
    positions: Option<Vec<DAffine3>>,
}

pub struct Component {
    shape: Shape,
    color: HexColor,
    positions: Option<Vec<DAffine3>>,
}

impl Component {
    pub fn new(shape: Shape, color: HexColor) -> Self {
        Self {
            shape,
            color,
            positions: None,
        }
    }

    pub fn with_positions(&mut self, positions: Vec<DAffine3>) {
        self.positions = Some(positions);
    }

    fn try_into_mesh(self, deflection_tolerance: f64) -> Result<Mesh, Error> {
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
                    let mut triangle_points = [DVec3::default(); 3];

                    let triangle = triangulation.Triangle(index + 1);

                    for corner_index in 0..3 {
                        let mut point = Poly_Triangulation_Node(
                            triangulation,
                            triangle.Value(corner_index as i32 + 1),
                        );
                        let point = point.pin_mut();
                        let point = DVec3::from_array([point.X(), point.Y(), point.Z()]);

                        triangle_points[corner_index] = point;
                    }

                    if face_orientation == TopAbs_Orientation::TopAbs_FORWARD {
                        triangle_points.reverse();
                    }

                    triangles.push(Triangle::new(triangle_points));
                }
            }

            face_explorer.pin_mut().Next();
        }

        Ok(Mesh {
            triangles,
            positions: self.positions,
        })
    }
}

pub trait ViewableModel {
    fn triangulation_tolerance(&self) -> f64;

    fn components(self) -> Vec<Component>;

    fn into_mesh_model(self) -> Model
    where
        Self: Sized,
    {
        let triangulation_tolerance = self.triangulation_tolerance();
        let mut mesh = FjMesh::new();

        for component in self.components() {
            let color = Color(component.color.to_be_bytes());

            match component.try_into_mesh(triangulation_tolerance) {
                Ok(component_mesh) => {
                    let Mesh {
                        triangles,
                        positions,
                    } = component_mesh;

                    let positions = positions.unwrap_or_else(|| vec![DAffine3::IDENTITY]);

                    for triangle in &triangles {
                        for position in &positions {
                            if let Some(triangle) = (position * triangle).try_into_triangle() {
                                mesh.push_triangle(triangle, color);
                            }
                        }
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
