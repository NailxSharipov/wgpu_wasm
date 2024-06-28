struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) @interpolate(flat) brush_index: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) brush_index: u32,
    @location(1) frag_position: vec2<f32>,
};

struct Brush {
    x: f32,
    y: f32,
    w: f32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
    f: f32
};

@group(0) @binding(0)
var<uniform> transform: mat4x4<f32>;

@group(0) @binding(1)
var<uniform> brushes: array<Brush, 16>;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = transform * vec4<f32>(in.position, 0.0, 1.0);
    out.brush_index = in.brush_index;
    out.frag_position = in.position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let index = in.brush_index & 15;
    let brush = brushes[index];
    if in.brush_index >= 16 {
        return vec4<f32>(brush.r, brush.g, brush.b, 1.0);
    }

    // Convert fragment position to local coordinates
    let local_position = in.frag_position;

    // Calculate the distance to the nearest line
    let distance = abs(brush.y * local_position.x - brush.x * local_position.y);

    var pattern: f32;
    if fract(0.16 * distance / brush.w) < 0.5 {
        pattern = 1.0;
    } else {
        pattern = 0.0;
    }

    return vec4<f32>(brush.r, brush.g, brush.b, pattern);
}
