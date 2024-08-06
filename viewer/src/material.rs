use config::Color;
use three_d::{
    lights_shader_source, Camera, ColorMapping, FragmentAttributes, LightingModel, Material,
    MaterialType, Program, RenderStates, Srgba, ToneMapping,
};

/// A simple physical material with an albedo.
pub struct Physical {
    albedo: Color,
}

impl Physical {
    /// Creates a new physical material given an albedo.
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }

    /// Updates the albedo of the physical material.
    pub fn update(&mut self, albedo: Color) {
        self.albedo = albedo;
    }
}

impl Material for Physical {
    fn id(&self) -> u16 {
        0
    }

    fn fragment_shader_source(&self, lights: &[&dyn three_d::Light]) -> String {
        let mut output = lights_shader_source(lights, LightingModel::Blinn);
        output.push_str(ToneMapping::fragment_shader_source());
        output.push_str(ColorMapping::fragment_shader_source());
        output.push_str(include_str!("material.frag"));
        output
    }

    fn fragment_attributes(&self) -> FragmentAttributes {
        FragmentAttributes {
            position: true,
            normal: false,
            color: false,
            uv: false,
            tangents: false,
        }
    }

    fn use_uniforms(&self, program: &Program, camera: &Camera, lights: &[&dyn three_d::Light]) {
        camera.tone_mapping.use_uniforms(program);
        camera.color_mapping.use_uniforms(program);
        if !lights.is_empty() {
            program.use_uniform_if_required("cameraPosition", camera.position());
            for (i, light) in lights.iter().enumerate() {
                #[allow(clippy::cast_possible_truncation)]
                light.use_uniforms(program, i as u32);
            }
        }
        let Color { r, g, b, a } = self.albedo;
        program.use_uniform("albedo", Srgba::new(r, g, b, a).to_linear_srgb());
    }

    fn render_states(&self) -> RenderStates {
        RenderStates::default()
    }

    fn material_type(&self) -> MaterialType {
        MaterialType::Opaque
    }
}
