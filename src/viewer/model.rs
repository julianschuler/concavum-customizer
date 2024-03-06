use fidget::{
    context::Node,
    eval::MathShape,
    jit::JitShape,
    mesh::{Octree, Settings},
    Context, Error,
};
use glam::{DAffine3, DMat4, DVec3};
use hex_color::HexColor;
use three_d::{CpuMesh, Indices, Mat4, Positions, Srgba, Vec3};

pub struct Component {
    context: Context,
    root: Node,
    color: HexColor,
    positions: Option<Vec<DAffine3>>,
}

impl Component {
    pub fn new(context: Context, root: Node, color: HexColor) -> Self {
        Self {
            context,
            root,
            color,
            positions: None,
        }
    }

    pub fn with_positions(&mut self, positions: Vec<DAffine3>) {
        self.positions = Some(positions);
    }

    fn mesh(&self, settings: Settings) -> Result<CpuMesh, Error> {
        let shape = JitShape::new(&self.context, self.root)?;
        let mesh = Octree::build(&shape, settings).walk_dual(settings);

        let vertices = mesh
            .vertices
            .iter()
            .map(|vertex| {
                let slice: [_; 3] = vertex.as_slice().try_into().unwrap();
                slice.into()
            })
            .collect();
        let indices = mesh
            .triangles
            .iter()
            .flat_map(|triangle| {
                triangle
                    .iter()
                    .map(|&index| index.try_into().expect("index should fit in u32"))
            })
            .collect();

        let mut mesh = CpuMesh {
            positions: Positions::F32(vertices),
            indices: Indices::U32(indices),
            ..Default::default()
        };
        mesh.compute_normals();

        Ok(mesh)
    }
}

pub trait Viewable {
    fn components(self) -> Vec<Component>;
    fn settings(&self) -> Settings;
    fn light_positions(&self) -> Vec<DVec3>;
    fn background_color(&self) -> HexColor;

    fn into_model(self) -> Model
    where
        Self: Sized,
    {
        let settings = self.settings();
        let light_positions = self
            .light_positions()
            .iter()
            .map(|position| position.as_vec3().to_array().into())
            .collect();
        let HexColor { r, g, b, a } = self.background_color();
        let background_color = Srgba::new(r, g, b, a);

        let mut objects = Vec::new();

        for component in self.components() {
            match component.mesh(settings) {
                Ok(mesh) => {
                    let HexColor { r, g, b, a } = component.color;
                    let color = Srgba::new(r, g, b, a);
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

        Model {
            objects,
            light_positions,
            background_color,
        }
    }
}

#[derive(Clone)]
pub struct CpuObject {
    pub mesh: CpuMesh,
    pub color: Srgba,
    pub transformations: Option<Vec<Mat4>>,
}

#[derive(Clone)]
pub struct Model {
    pub objects: Vec<CpuObject>,
    pub light_positions: Vec<Vec3>,
    pub background_color: Srgba,
}
