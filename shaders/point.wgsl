// Vertex shader for point rendering with bloom

struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    time: f32,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct PointData {
    position: vec3<f32>,
    color: vec3<f32>,
    brightness: f32,
    size: f32,
}

@group(1) @binding(0)
var<storage, read> points: array<PointData>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) brightness: f32,
    @location(2) distance: f32,
    @location(3) uv: vec2<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    let point = points[instance_index];

    // Quad vertices for point sprite
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0),
    );

    let quad_pos = positions[vertex_index];

    // Billboard: face camera
    let view_pos = (camera.view_proj * vec4<f32>(point.position, 1.0)).xyz;
    let distance = length(camera.camera_pos - point.position);

    // Better size attenuation - more stable at distance
    let size = point.size / (1.0 + distance * 0.002);

    // Billboard offset with better scaling
    let offset = vec4<f32>(quad_pos * size * 0.008, 0.0, 0.0);
    let final_pos = camera.view_proj * vec4<f32>(point.position, 1.0) + offset;

    var output: VertexOutput;
    output.clip_position = final_pos;
    output.color = point.color;
    output.brightness = point.brightness;
    output.distance = distance;
    output.uv = quad_pos * 0.5 + 0.5; // Convert from [-1,1] to [0,1]

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Calculate distance from center (0.5, 0.5)
    let center = vec2<f32>(0.5, 0.5);
    let dist = length(input.uv - center) * 2.0;

    // Simple circular point - NO BLOOM
    let circle = 1.0 - smoothstep(0.0, 0.5, dist);

    // Pure red
    let red = vec3<f32>(1.0, 0.0, 0.0);

    // Stronger fog for smooth fade to black at distance
    let fog_density = 0.003;
    let fog_factor = exp(-input.distance * fog_density);

    // Smooth fade to black
    let final_color = red * circle * input.brightness * fog_factor;

    // Alpha with stronger distance attenuation
    let alpha = circle * fog_factor * input.brightness;

    // Discard pixels outside circle
    if (dist > 1.0) {
        discard;
    }

    return vec4<f32>(final_color, alpha);
}
