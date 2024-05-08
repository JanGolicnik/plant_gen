struct Camera {
    position: vec2<f32>,
    resolution: vec2<f32>
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct InstanceInput{
    @location(5) position: vec2<f32>,
    @location(6) scale: vec2<f32>,
    @location(7) rotation: f32,
    @location(8) color: vec3<f32>,
}

struct VertexInput{
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) coords: vec2<f32>,
};
@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput{
    var vertex_pos = vertex.position.xy;
    vertex_pos *= instance.scale;

    var position = instance.position + vertex_pos;

    position -= camera.position;
    position /= camera.resolution;

    var out: VertexOutput;
    out.clip_position = vec4<f32>(position, 0.0, 1.0);
    out.color = instance.color;
    out.coords = vertex.position.xy;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32>{
    let p = vec2<f32>(in.coords.x, in.coords.y);
    let len = 1.0 - length(p);

    let val = smoothstep(0.43, 0.44, len );

    return vec4<f32>(in.color, val);
}