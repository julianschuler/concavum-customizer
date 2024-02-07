use fidget::{
    context::{IntoNode, Node},
    Context, Error,
};

pub struct Sphere {
    radius: f32,
}

impl Sphere {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl IntoNode for Sphere {
    fn into_node(self, context: &mut Context) -> Result<Node, Error> {
        let x = context.x();
        let y = context.y();
        let z = context.z();
        let x_square = context.square(x)?;
        let y_square = context.square(y)?;
        let z_square = context.square(z)?;
        let sum = context.add(y_square, z_square)?;
        let sum = context.add(x_square, sum)?;
        let length = context.sqrt(sum)?;
        let root = context.sub(length, self.radius)?;

        Ok(root)
    }
}
