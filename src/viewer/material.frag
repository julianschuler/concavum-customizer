uniform vec3 cameraPosition;
uniform vec4 albedo;

in vec3 pos;
in vec4 col;

layout (location = 0) out vec4 outColor;

void main() {
    vec4 surface_color = albedo * col;
    vec3 normal = normalize(cross(dFdx(pos), dFdy(pos)));

    float metallic = 0.0;
    float roughness = 1.0;
    float occlusion = 1.0;

    outColor.rgb = calculate_lighting(cameraPosition, surface_color.rgb, pos, normal, metallic, roughness, occlusion);
    outColor.rgb = tone_mapping(outColor.rgb);
    outColor.rgb = color_mapping(outColor.rgb);
    outColor.a = surface_color.a;
}
