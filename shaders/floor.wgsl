// Floor shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    time: f32,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.world_pos = input.position;
    output.clip_position = camera.view_proj * vec4<f32>(input.position, 1.0);
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Procedural grid
    let grid_size = 10.0;
    let grid_x = fract(input.world_pos.x / grid_size);
    let grid_z = fract(input.world_pos.z / grid_size);

    let line_width = 0.02;
    let grid = step(grid_x, line_width) + step(grid_z, line_width);

    // Distance fade
    let distance = length(camera.camera_pos - input.world_pos);
    let fade = exp(-distance * 0.005);

    let grid_color = vec3<f32>(0.1, 0.3, 0.5);
    let intensity = grid * 0.3 * fade;

    return vec4<f32>(grid_color * intensity, intensity);
}
