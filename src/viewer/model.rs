use glam::{DAffine3, DMat4};
use hex_color::HexColor;
use opencascade::primitives::Shape;
use opencascade_sys::ffi::{
    BRepMesh_IncrementalMesh_ctor, BRep_Tool_Triangulation, Handle_Poly_Triangulation_Get,
    Poly_Triangulation_Node, TopAbs_Orientation, TopAbs_ShapeEnum, TopExp_Explorer_ctor,
    TopLoc_Location_ctor, TopoDS_cast_to_face,
};
use three_d::{Color, CpuMesh, Mat4, Positions, Vector3};

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

    fn mesh(&self, deflection_tolerance: f64) -> Result<CpuMesh, Error> {
        let mut vertices = Vec::new();

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
                    let mut triangle_points = [Vector3::new(0.0, 0.0, 0.0); 3];

                    let triangle = triangulation.Triangle(index + 1);

                    for corner_index in 0..3 {
                        let mut point = Poly_Triangulation_Node(
                            triangulation,
                            triangle.Value(corner_index as i32 + 1),
                        );
                        let point = point.pin_mut();
                        let point =
                            Vector3::new(point.X() as f32, point.Y() as f32, point.Z() as f32);

                        triangle_points[corner_index] = point;
                    }

                    if face_orientation == TopAbs_Orientation::TopAbs_FORWARD {
                        triangle_points.reverse();
                    }

                    vertices.extend(triangle_points);
                }
            }

            face_explorer.pin_mut().Next();
        }

        Ok(CpuMesh {
            positions: Positions::F32(vertices),
            ..Default::default()
        })
    }
}

pub trait ViewableModel {
    fn triangulation_tolerance(&self) -> f64;

    fn components(self) -> Vec<Component>;

    fn into_mesh_model(self) -> MeshModel
    where
        Self: Sized,
    {
        let triangulation_tolerance = self.triangulation_tolerance();

        let mut objects = Vec::new();

        for component in self.components() {
            match component.mesh(triangulation_tolerance) {
                Ok(mesh) => {
                    let HexColor { r, g, b, a } = component.color;
                    let color = Color::new(r, g, b, a);
                    let transformations = component.positions.map(|positions| {
                        positions
                            .into_iter()
                            .map(|position| {
                                let matrix: DMat4 = position.into();
                                matrix.as_mat4().to_cols_array_2d().into()
                            })
                            .collect()
                    });

                    objects.push(CpuObject {
                        mesh,
                        color,
                        transformations,
                    });
                }
                Err(_) => {
                    eprintln!("Warning: Could not triangulate component, ignoring it");
                }
            }
        }

        MeshModel { objects }
    }
}

#[derive(Clone)]
pub struct CpuObject {
    pub mesh: CpuMesh,
    pub color: Color,
    pub transformations: Option<Vec<Mat4>>,
}

#[derive(Clone)]
pub struct MeshModel {
    pub objects: Vec<CpuObject>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Triangulation using BRepMesh_IncrementalMesh_ctor failed
    #[error("Triangulation using BRepMesh_IncrementalMesh_ctor failed")]
    Triangulation,
}
