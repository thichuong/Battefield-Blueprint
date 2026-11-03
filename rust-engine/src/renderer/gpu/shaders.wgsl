// Shaders WGSL cho WebGPU 2D Battlefield Blueprint Renderer

struct CameraUniform {
    view_proj: mat4x4<f32>,
    viewport: vec2<f32>,
    zoom: f32,
    _pad: f32,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(0) @binding(1)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(2)
var s_diffuse: sampler;

// -------------------------------------------------------------
// 1. INSTANCED SPRITE PIPELINE
// -------------------------------------------------------------
struct VertexInput {
    @location(0) quad_pos: vec2<f32>, // [-0.5, 0.5]
    @location(1) quad_uv: vec2<f32>,  // [0.0, 1.0]
};

struct InstanceInput {
    @location(2) pos: vec2<f32>,
    @location(3) size: vec2<f32>,
    @location(4) uv_min: vec2<f32>,
    @location(5) uv_max: vec2<f32>,
    @location(6) color: vec4<f32>,
    @location(7) rotation: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) world_pos: vec2<f32>,
};

@vertex
fn vs_sprite(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    // Scale trước theo instance.size, sau đó quay quanh tâm theo instance.rotation
    let scaled = model.quad_pos * instance.size;
    let cos_r = cos(instance.rotation);
    let sin_r = sin(instance.rotation);
    let rot_x = scaled.x * cos_r - scaled.y * sin_r;
    let rot_y = scaled.x * sin_r + scaled.y * cos_r;

    let world_p = instance.pos + vec2<f32>(rot_x, rot_y);
    out.world_pos = world_p;
    out.clip_position = camera.view_proj * vec4<f32>(world_p, 0.0, 1.0);
    out.uv = mix(instance.uv_min, instance.uv_max, model.quad_uv);
    out.color = instance.color;
    return out;
}

@fragment
fn fs_sprite(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_diffuse, s_diffuse, in.uv);
    let final_color = tex_color * in.color;
    if (final_color.a < 0.005) {
        discard;
    }
    return final_color;
}

// -------------------------------------------------------------
// 2. PROCEDURAL BLUEPRINT GRID PIPELINE
// -------------------------------------------------------------
struct GridVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec2<f32>,
};

@vertex
fn vs_grid(@builtin(vertex_index) in_vertex_index: u32) -> GridVertexOutput {
    var out: GridVertexOutput;
    // Fullscreen Triangle: 0: (-1, -1), 1: (3, -1), 2: (-1, 3)
    let x = f32(i32(in_vertex_index == 1u) * 4 - 1);
    let y = f32(i32(in_vertex_index == 2u) * 4 - 1);
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);

    // Tính ngược lại tọa độ world từ NDC
    // clip_position = (world_p * s + t) => world_p = (clip_position - t) / s
    let sx = camera.view_proj[0][0];
    let sy = camera.view_proj[1][1];
    let tx = camera.view_proj[3][0];
    let ty = camera.view_proj[3][1];
    out.world_pos = vec2<f32>((x - tx) / sx, (y - ty) / sy);
    return out;
}

@fragment
fn fs_grid(in: GridVertexOutput) -> @location(0) vec4<f32> {
    let wp = in.world_pos;

    // Giới hạn lưới trong ranh giới map [0, 4800]
    let map_w = 4800.0;
    let map_h = 4800.0;
    let is_inside_map = wp.x >= 0.0 && wp.x <= map_w && wp.y >= 0.0 && wp.y <= map_h;

    // Màu nền không gian Blueprint
    let bg_color = vec4<f32>(0.031, 0.047, 0.078, 1.0); // #080c14
    let outside_color = vec4<f32>(0.02, 0.03, 0.05, 1.0);

    if (!is_inside_map) {
        return outside_color;
    }

    // Lưới nhỏ 20px & Lưới lớn 100px
    let d = fwidth(wp);
    let grid_small = 20.0;
    let grid_large = 100.0;

    let coord_s = wp / grid_small;
    let line_s = abs(fract(coord_s - 0.5) - 0.5) / (d / grid_small);
    let s_factor = 1.0 - min(min(line_s.x, line_s.y), 1.0);

    let coord_l = wp / grid_large;
    let line_l = abs(fract(coord_l - 0.5) - 0.5) / (d / grid_large);
    let l_factor = 1.0 - min(min(line_l.x, line_l.y), 1.0);

    let line_color_small = vec4<f32>(0.07, 0.18, 0.32, 0.35); // Xanh nhạt mờ
    let line_color_large = vec4<f32>(0.0, 0.9, 1.0, 0.18);     // Cyan sáng Blueprint

    var color = bg_color;
    color = mix(color, line_color_small, s_factor * 0.4);
    color = mix(color, line_color_large, l_factor * 0.7);

    // Vẽ đường viền map 4800x4800
    let border_dist_x = min(wp.x, map_w - wp.x);
    let border_dist_y = min(wp.y, map_h - wp.y);
    let border_dist = min(border_dist_x, border_dist_y);
    if (border_dist < 4.0) {
        color = vec4<f32>(0.008, 0.518, 0.78, 0.85); // #0284c7
    }

    return color;
}

// -------------------------------------------------------------
// 3. FOG OF WAR OVERLAY PIPELINE
// -------------------------------------------------------------
@group(1) @binding(0)
var t_fog: texture_2d<f32>;
@group(1) @binding(1)
var s_fog: sampler;

struct FogVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_fog(@builtin(vertex_index) in_vertex_index: u32) -> FogVertexOutput {
    var out: FogVertexOutput;
    // Map quad phủ toàn bộ bản đồ 4800x4800
    // 0: (0, 0), 1: (4800, 0), 2: (4800, 4800), 3: (0, 0), 4: (4800, 4800), 5: (0, 4800)
    var pos = vec2<f32>(0.0, 0.0);
    var uv = vec2<f32>(0.0, 0.0);
    let map_dim = 4800.0;

    switch in_vertex_index {
        case 0u: { pos = vec2<f32>(0.0, 0.0); uv = vec2<f32>(0.0, 0.0); }
        case 1u: { pos = vec2<f32>(map_dim, 0.0); uv = vec2<f32>(1.0, 0.0); }
        case 2u: { pos = vec2<f32>(map_dim, map_dim); uv = vec2<f32>(1.0, 1.0); }
        case 3u: { pos = vec2<f32>(0.0, 0.0); uv = vec2<f32>(0.0, 0.0); }
        case 4u: { pos = vec2<f32>(map_dim, map_dim); uv = vec2<f32>(1.0, 1.0); }
        default: { pos = vec2<f32>(0.0, map_dim); uv = vec2<f32>(0.0, 1.0); }
    }

    out.clip_position = camera.view_proj * vec4<f32>(pos, 0.0, 1.0);
    out.uv = uv;
    return out;
}

@fragment
fn fs_fog(in: FogVertexOutput) -> @location(0) vec4<f32> {
    // Giá trị r của fog: 0.0 = Hidden (chưa mở), 0.5 = Explored (đã mở), 1.0 = Visible (đang thấy)
    let fog_sample = textureSample(t_fog, s_fog, in.uv).r;

    // Màu tối Blueprint của sương mù
    let fog_dark = vec3<f32>(0.02, 0.04, 0.08);

    if (fog_sample >= 0.98) {
        // Đang nhìn thấy hoàn toàn -> Không che
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    } else if (fog_sample <= 0.02) {
        // Hoàn toàn chưa mở -> Che đen tối Blueprint
        return vec4<f32>(fog_dark, 0.94);
    } else {
        // Hòa trộn mượt giữa Hidden, Explored (0.5), và Visible (1.0)
        let alpha = mix(0.94, 0.45, clamp(fog_sample * 2.0, 0.0, 1.0));
        let final_alpha = mix(alpha, 0.0, clamp((fog_sample - 0.5) * 2.0, 0.0, 1.0));
        return vec4<f32>(fog_dark, final_alpha);
    }
}

