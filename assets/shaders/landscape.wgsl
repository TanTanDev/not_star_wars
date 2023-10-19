#import bevy_pbr::mesh_functions as mesh_functions
#import bevy_pbr::mesh_bindings       mesh, globals
#import bevy_pbr::mesh_vertex_output  MeshVertexOutput
#import bevy_pbr::pbr_functions as pbr_functions
#import bevy_pbr::pbr_bindings as pbr_bindings
#import bevy_pbr::pbr_types as pbr_types
#import bevy_pbr::mesh_view_bindings       view, fog, screen_space_ambient_occlusion_texture
#import bevy_pbr::mesh_view_types          FOG_MODE_OFF
#import bevy_core_pipeline::tonemapping    screen_space_dither, powsafe, tone_mapping

struct LandscapeMaterial {
    time: f32,
    speed: f32,
    terrain_height: f32,
    terrain_size: f32,
    uv_scaling: f32,
    quad_size: f32,
};

@group(1) @binding(0)
var<uniform> material: LandscapeMaterial;
@group(1) @binding(1)
var color_texture: texture_2d<f32>;
@group(1) @binding(2)
var color_sampler: sampler;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> MeshVertexOutput {
    var vertex = vertex;
    var out: MeshVertexOutput;
    var model = mesh.model;

    let moving_offset = vec2<f32>(0.0, material.time * material.speed);
    let noise = simplexNoise2((vertex.position.xz + moving_offset) * material.terrain_size * 0.01) * 0.5 + 0.5;
    vertex.position.y += noise * material.terrain_height;

    out.world_position = mesh_functions::mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
    out.position = mesh_functions::mesh_position_world_to_clip(out.world_position);
    out.world_normal = mesh_functions::mesh_normal_local_to_world(vertex.normal);
    out.uv = vertex.uv;

    return out;
}

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    var output_color: vec4<f32> = vec4<f32>(0.0,1.0,0.0, 1.0);
    var pbr_input = pbr_functions::pbr_input_new();
    pbr_input.world_position = in.world_position;
    pbr_input.world_normal = in.world_normal;
    pbr_input.frag_coord = in.position;
    pbr_input.material.reflectance = 0.1;
    pbr_input.N = in.world_normal;
    pbr_input.V = pbr_functions::calculate_view(in.world_position, false);

    // green
    // pbr_input.material.base_color = vec4<f32>(0.0,1.0,0.0,1.0);
    let moving_offset = vec2<f32>(0.0, material.time * material.speed);
    pbr_input.material.base_color = textureSample(color_texture, color_sampler, (in.uv + moving_offset * (1.0 / material.quad_size)) * material.uv_scaling);

    output_color = pbr_functions::pbr(pbr_input);
    output_color = pbr_functions::apply_fog(fog, output_color, in.world_position.xyz, view.world_position.xyz);
    return output_color;
}


//  MIT License. © Ian McEwan, Stefan Gustavson, Munrocket, Johan Helsing
//
fn mod289(x: vec2f) -> vec2f {
    return x - floor(x * (1. / 289.)) * 289.;
}

fn mod289_3(x: vec3f) -> vec3f {
    return x - floor(x * (1. / 289.)) * 289.;
}

fn permute3(x: vec3f) -> vec3f {
    return mod289_3(((x * 34.) + 1.) * x);
}

//  MIT License. © Ian McEwan, Stefan Gustavson, Munrocket
fn simplexNoise2(v: vec2f) -> f32 {
    let C = vec4(
        0.211324865405187, // (3.0-sqrt(3.0))/6.0
        0.366025403784439, // 0.5*(sqrt(3.0)-1.0)
        -0.577350269189626, // -1.0 + 2.0 * C.x
        0.024390243902439 // 1.0 / 41.0
    );

    // First corner
    var i = floor(v + dot(v, C.yy));
    let x0 = v - i + dot(i, C.xx);

    // Other corners
    var i1 = select(vec2(0., 1.), vec2(1., 0.), x0.x > x0.y);

    // x0 = x0 - 0.0 + 0.0 * C.xx ;
    // x1 = x0 - i1 + 1.0 * C.xx ;
    // x2 = x0 - 1.0 + 2.0 * C.xx ;
    var x12 = x0.xyxy + C.xxzz;
    x12.x = x12.x - i1.x;
    x12.y = x12.y - i1.y;

    // Permutations
    i = mod289(i); // Avoid truncation effects in permutation

    var p = permute3(permute3(i.y + vec3(0., i1.y, 1.)) + i.x + vec3(0., i1.x, 1.));
    var m = max(0.5 - vec3(dot(x0, x0), dot(x12.xy, x12.xy), dot(x12.zw, x12.zw)), vec3(0.));
    m *= m;
    m *= m;

    // Gradients: 41 points uniformly over a line, mapped onto a diamond.
    // The ring size 17*17 = 289 is close to a multiple of 41 (41*7 = 287)
    let x = 2. * fract(p * C.www) - 1.;
    let h = abs(x) - 0.5;
    let ox = floor(x + 0.5);
    let a0 = x - ox;

    // Normalize gradients implicitly by scaling m
    // Approximation of: m *= inversesqrt( a0*a0 + h*h );
    m *= 1.79284291400159 - 0.85373472095314 * (a0 * a0 + h * h);

    // Compute final noise value at P
    let g = vec3(a0.x * x0.x + h.x * x0.y, a0.yz * x12.xz + h.yz * x12.yw);
    return 130. * dot(m, g);
}