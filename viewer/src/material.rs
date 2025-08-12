use config::Color;
use three_d::{
    lights_shader_source, ColorMapping, EffectMaterialId, Material, MaterialType, Program,
    RenderStates, Srgba, ToneMapping, Viewer,
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
    fn id(&self) -> EffectMaterialId {
        EffectMaterialId(0)
    }

    fn fragment_shader_source(&self, lights: &[&dyn three_d::Light]) -> String {
        let mut output = lights_shader_source(lights);
        output.push_str(ToneMapping::fragment_shader_source());
        output.push_str(ColorMapping::fragment_shader_source());
        output.push_str(include_str!("material.frag"));
        output
    }

    fn use_uniforms(&self, program: &Program, viewer: &dyn Viewer, lights: &[&dyn three_d::Light]) {
        program.use_uniform_if_required("lightingModel", 2u32);
        viewer.tone_mapping().use_uniforms(program);
        viewer.color_mapping().use_uniforms(program);
        program.use_uniform_if_required("cameraPosition", viewer.position());
        for (i, light) in lights.iter().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            light.use_uniforms(program, i as u32);
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
