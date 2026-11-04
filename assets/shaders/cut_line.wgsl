// x = how much of the line is already gone, y = where the finish zone starts
@group(1) @binding(0) var<uniform> line: vec2<f32>;
@group(1) @binding(1) var<uniform> finish_color: vec4<f32>;
@group(1) @binding(2) var image: texture_2d<f32>;
@group(1) @binding(3) var image_sampler: sampler;

@fragment
fn fragment(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let along_line = mix(line.x, 1.0, uv.x);
    let color = textureSample(image, image_sampler, vec2(along_line, uv.y));

    if along_line >= line.y {
        return vec4(finish_color.rgb, color.a);
    }
    return color;
}
