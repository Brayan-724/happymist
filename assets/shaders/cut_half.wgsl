
@group(1) @binding(0) var<uniform> cut: vec4<f32>;
@group(1) @binding(1) var image: texture_2d<f32>;
@group(1) @binding(2) var image_sampler: sampler;

@fragment
fn fragment(
    @location(0) uv: vec2<f32>,
    @location(4) @interpolate(flat) size: vec2<f32>,
) -> @location(0) vec4<f32> {
    let distance_to_cut = dot((uv - 0.5) * size, cut.xy) * cut.z;
    let color = textureSample(image, image_sampler, uv);
    return vec4(color.rgb, color.a * cut.w * saturate(distance_to_cut + 0.5));
}
