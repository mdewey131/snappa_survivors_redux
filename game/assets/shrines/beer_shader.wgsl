#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material_color: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var material_color_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var material_color_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var<uniform> pct: f32;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    var alpha = 1.0;
    if mesh.uv[1] <= (1.0 - pct) {
        alpha = 0.0;
    }
    let adjustment_vec = vec4(1.0, 1.0, 1.0, alpha);
    return textureSample(material_color_texture, material_color_sampler, mesh.uv) * adjustment_vec;
}
