use three_d::{
    lights_shader_source, Camera, ColorMapping, FragmentAttributes, LightingModel, Material,
    MaterialType, Program, RenderStates, Srgba, ToneMapping,
};

pub struct Physical {
    albedo: Srgba,
}

impl Physical {
    pub fn new(albedo: Srgba) -> Self {
        Self { albedo }
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
                light.use_uniforms(program, i as u32);
            }
        }
        program.use_uniform("albedo", self.albedo.to_linear_srgb());
    }

    fn render_states(&self) -> RenderStates {
        RenderStates::default()
    }

    fn material_type(&self) -> MaterialType {
        MaterialType::Opaque
    }
}
