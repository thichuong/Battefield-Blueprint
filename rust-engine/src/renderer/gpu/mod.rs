pub mod fog_pass;
pub mod sprite_batch;
pub mod texture_atlas;
pub mod wgpu_state;

use bytemuck::{Pod, Zeroable};
use web_sys::HtmlCanvasElement;
use wgpu::util::DeviceExt;

use crate::components::{HarvestPhase, Team, UnitOrder, UnitType};
use crate::renderer::gpu_cache::{get_color_index, GpuCacheManager};
use crate::GameEngine;

use fog_pass::FogPass;
use sprite_batch::SpriteBatch;
use texture_atlas::TextureAtlas;
use wgpu_state::WgpuState;

const PLAYER_LASER_COLORS: [[f32; 4]; 8] = [
    [0.00, 0.90, 1.00, 0.95], // 0: Cyan Neon (#00e5ff)
    [0.96, 0.25, 0.37, 0.95], // 1: Crimson Red (#f43f5e)
    [0.66, 0.33, 0.97, 0.95], // 2: Arcane Purple (#a855f7)
    [0.96, 0.62, 0.04, 0.95], // 3: Amber Orange (#f59e0b)
    [0.06, 0.73, 0.51, 0.95], // 4: Emerald Green (#10b981)
    [0.93, 0.28, 0.60, 0.95], // 5: Hot Pink (#ec4899)
    [0.23, 0.51, 0.96, 0.95], // 6: Royal Blue (#3b82f6)
    [0.92, 0.70, 0.03, 0.95], // 7: Gold Yellow (#eab308)
];

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_proj: [f32; 16],
    pub viewport: [f32; 2],
    pub zoom: f32,
    pub show_grid: f32,
}

pub struct WgpuRenderer {
    pub state: WgpuState,
    pub atlas: TextureAtlas,
    pub sprite_batch: SpriteBatch,
    pub fog_pass: FogPass,

    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_atlas_bind_group: wgpu::BindGroup,
    pub camera_only_bind_group: wgpu::BindGroup,

    pub grid_pipeline: wgpu::RenderPipeline,
    pub sprite_pipeline: wgpu::RenderPipeline,
    pub fog_pipeline: wgpu::RenderPipeline,
}

impl WgpuRenderer {
    pub async fn new(
        canvas: HtmlCanvasElement,
        cache: &GpuCacheManager,
    ) -> Result<Self, String> {
        let state = WgpuState::new(canvas).await?;

        // 1. Tạo Texture Atlas từ GPU Cache Manager
        let atlas = TextureAtlas::new(&state.device, &state.queue, cache)
            .ok_or_else(|| "Failed to create WebGPU Texture Atlas".to_string())?;

        // 2. Tạo SpriteBatch
        let sprite_batch = SpriteBatch::new(&state.device);

        // 3. Camera Uniform Buffer & Bind Groups
        let initial_camera = CameraUniform {
            view_proj: [0.0; 16],
            viewport: [state.width as f32, state.height as f32],
            zoom: 1.0,
            show_grid: 1.0,
        };

        let camera_buffer = state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("WebGPU Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[initial_camera]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("WebGPU Camera Bind Group Layout"),
                    entries: &[
                        // binding 0: Uniform buffer
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        // binding 1: Diffuse Texture
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        // binding 2: Diffuse Sampler
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let camera_only_bind_group_layout =
            state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("WebGPU Camera Only Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let camera_atlas_bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("WebGPU Camera + Atlas Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&atlas.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&atlas.sampler),
                },
            ],
        });

        let camera_only_bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("WebGPU Camera Only Bind Group"),
            layout: &camera_only_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // 4. Load WGSL Shaders
        let shader = state
            .device
            .create_shader_module(wgpu::include_wgsl!("shaders.wgsl"));

        // 5. Grid Pipeline
        let grid_pipeline_layout =
            state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("WebGPU Grid Pipeline Layout"),
                    bind_group_layouts: &[&camera_only_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let grid_pipeline =
            state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("WebGPU Grid Render Pipeline"),
                    layout: Some(&grid_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_grid"),
                        buffers: &[],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_grid"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: state.config.format,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        ..Default::default()
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                    cache: None,
                });

        // 6. Sprite Pipeline
        let sprite_pipeline_layout =
            state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("WebGPU Sprite Pipeline Layout"),
                    bind_group_layouts: &[&camera_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let sprite_vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<sprite_batch::QuadVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        };

        let sprite_pipeline =
            state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("WebGPU Sprite Render Pipeline"),
                    layout: Some(&sprite_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_sprite"),
                        buffers: &[sprite_vertex_layout, sprite_batch::SpriteInstance::desc()],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_sprite"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: state.config.format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        ..Default::default()
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                    cache: None,
                });

        // 7. Fog of War Pass & Pipeline
        let fog_pass = FogPass::new(&state.device);

        let fog_pipeline_layout =
            state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("WebGPU Fog Pipeline Layout"),
                    bind_group_layouts: &[
                        &camera_only_bind_group_layout,
                        &fog_pass.bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let fog_pipeline =
            state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("WebGPU Fog Render Pipeline"),
                    layout: Some(&fog_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_fog"),
                        buffers: &[],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_fog"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: state.config.format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        ..Default::default()
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                    cache: None,
                });

        Ok(Self {
            state,
            atlas,
            sprite_batch,
            fog_pass,
            camera_buffer,
            camera_bind_group_layout,
            camera_atlas_bind_group,
            camera_only_bind_group,
            grid_pipeline,
            sprite_pipeline,
            fog_pipeline,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.state.resize(width, height);
    }

    pub fn render_frame(&mut self, engine: &mut GameEngine) -> Result<(), wgpu::SurfaceError> {
        let vw = engine.camera.viewport_w.max(1.0);
        let vh = engine.camera.viewport_h.max(1.0);
        let zoom = engine.camera.zoom;
        let cx = engine.camera.x;
        let cy = engine.camera.y;

        // 1. Tính toán ma trận Camera Orthographic
        let sx = (2.0 * zoom) / vw;
        let sy = -(2.0 * zoom) / vh;
        let tx = -cx * sx;
        let ty = -cy * sy;

        #[rustfmt::skip]
        let view_proj = [
            sx,  0.0, 0.0, 0.0,
            0.0, sy,  0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            tx,  ty,  0.0, 1.0,
        ];

        let show_grid_val = if engine.settings.show_grid { 1.0 } else { 0.0 };
        let camera_data = CameraUniform {
            view_proj,
            viewport: [vw, vh],
            zoom,
            show_grid: show_grid_val,
        };
        self.state.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_data]),
        );

        // 2. Cập nhật GPU Fog Texture nếu sương mù thay đổi
        if engine.fog_dirty {
            self.fog_pass.update(&self.state.queue, &engine.fog.grid);
            engine.fog_dirty = false;
        }

        // 3. Gom toàn bộ thực thể game vào SpriteBatch (Instanced Buffer)
        self.sprite_batch.clear();
        let white_uv = self.atlas.white_pixel_uv;
        let alpha = engine.render_alpha;

        // 3.1. Địa hình vật cản (Obstacles)
        for obs in &engine.obstacles {
            if !engine.camera.is_rect_visible(obs.x, obs.y, obs.width, obs.height, 40.0) {
                continue;
            }
            let center = [obs.x + obs.width * 0.5, obs.y + obs.height * 0.5];
            let size = [obs.width, obs.height];
            // Khối nền vật cản
            self.sprite_batch.push_rect(
                center,
                size,
                [0.08, 0.12, 0.18, 0.85],
                white_uv,
            );
            // Viền viền Blueprint
            self.sprite_batch.push_stroke_rect(
                center,
                size,
                2.0,
                [0.15, 0.28, 0.42, 0.9],
                white_uv,
            );
        }

        // 3.2. Mỏ khoáng sản (Minerals)
        for m in &engine.minerals {
            if !engine.camera.is_point_visible(m.x, m.y, 60.0) {
                continue;
            }
            if !engine.fog.is_explored(m.x, m.y) {
                continue;
            }
            let uv = &self.atlas.mineral_uv;
            let alpha_col = if engine.fog.is_visible(m.x, m.y) { 1.0 } else { 0.5 };
            self.sprite_batch.push(
                [m.x, m.y],
                [64.0, 64.0],
                uv.min,
                uv.max,
                [1.0, 1.0, 1.0, alpha_col],
                0.0,
            );
        }

        // 3.3. Vòng ma thuật (Magic Circles)
        for mc in &engine.magic_circles {
            if !engine.camera.is_point_visible(mc.x, mc.y, 120.0) {
                continue;
            }
            let uv = &self.atlas.magic_circle_uv;
            let mc_rot = (engine.game_time * 0.8) % std::f32::consts::TAU;
            self.sprite_batch.push(
                [mc.x, mc.y],
                [192.0, 192.0],
                uv.min,
                uv.max,
                [1.0, 1.0, 1.0, 0.85],
                mc_rot,
            );
        }

        // 3.4. Công trình (Buildings)
        for b in &engine.buildings {
            if !engine.camera.is_rect_visible(b.x - b.width * 0.5, b.y - b.height * 0.5, b.width, b.height, 60.0) {
                continue;
            }
            let is_visible = engine.are_allies(Team::Player, b.team) || engine.fog.is_visible(b.x, b.y);
            if !is_visible && !engine.fog.is_explored(b.x, b.y) {
                continue;
            }

            let b_type_idx = b.building_type as usize;
            if b_type_idx < self.atlas.building_uvs.len() {
                let is_hit_flash = b.hit_flash_timer > 0.0;
                let color_idx = get_color_index(b.team, is_hit_flash);
                let uv = &self.atlas.building_uvs[b_type_idx][color_idx];

                // b.x, b.y là tâm công trình trong toàn bộ hệ thống mô phỏng
                // Sprite canvas có thêm 30px phía dưới cho nhãn tên đã bake nên offset nhẹ theo trục Y
                let cx = b.x;
                let cy = b.y + 10.0;
                let sw = b.width + 20.0;
                let sh = b.height + 30.0;

                let tint = if is_visible {
                    [1.0, 1.0, 1.0, 1.0]
                } else {
                    [0.55, 0.65, 0.75, 0.6] // Tối mờ nếu trong sương mù đã khám phá
                };

                self.sprite_batch.push([cx, cy], [sw, sh], uv.min, uv.max, tint, 0.0);

                // Khung chọn công trình
                if engine.selected_building_id == Some(b.id) {
                    self.sprite_batch.push_stroke_rect(
                        [b.x, b.y],
                        [b.width + 8.0, b.height + 8.0],
                        2.0,
                        [0.0, 0.9, 1.0, 0.9],
                        white_uv,
                    );
                }

                // Thanh máu nếu còn thấy
                if is_visible && b.max_hp > 0.0 {
                    let hp_pct = (b.hp / b.max_hp).clamp(0.0, 1.0);
                    let bar_w = b.width;
                    let bar_h = 4.0;
                    let bar_y = b.y - b.height * 0.5 - 6.0;
                    // Nền đen
                    self.sprite_batch.push_rect(
                        [b.x, bar_y],
                        [bar_w, bar_h],
                        [0.1, 0.1, 0.1, 0.8],
                        white_uv,
                    );
                    // Máu xanh/vàng/đỏ
                    let hp_col = if hp_pct > 0.5 {
                        [0.1, 0.85, 0.3, 0.9]
                    } else if hp_pct > 0.25 {
                        [0.95, 0.8, 0.1, 0.9]
                    } else {
                        [0.9, 0.2, 0.2, 0.9]
                    };
                    let cur_w = bar_w * hp_pct;
                    self.sprite_batch.push_rect(
                        [b.x - (bar_w - cur_w) * 0.5, bar_y],
                        [cur_w, bar_h],
                        hp_col,
                        white_uv,
                    );
                }
            }
        }

        // 3.5. Quân lính (Units)
        engine.visible_unit_indices.clear();
        for (idx, u) in engine.units.iter().enumerate() {
            let draw_x = (u.prev_x + (u.x - u.prev_x) * alpha) as f64;
            let draw_y = (u.prev_y + (u.y - u.prev_y) * alpha) as f64;
            if !engine.camera.is_point_visible(draw_x as f32, draw_y as f32, 40.0) {
                continue;
            }
            let is_visible = engine.are_allies(Team::Player, u.team) || engine.fog.is_visible(u.x, u.y);
            if !is_visible {
                continue;
            }
            engine.visible_unit_indices.push((idx, draw_x, draw_y));
        }

        // Sắp xếp chiều sâu 2.5D theo trục Y
        engine
            .visible_unit_indices
            .sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

        let num_units = engine.visible_unit_indices.len();
        for v_i in 0..num_units {
            let (u_idx, draw_x, draw_y) = engine.visible_unit_indices[v_i];
            let u = &engine.units[u_idx];

            let dx = draw_x as f32;
            let dy = draw_y as f32;

            let is_selected = engine.selected_unit_set.contains(&u.id);
            if is_selected {
                // Vòng tròn chọn dưới chân
                self.sprite_batch.push_circle_stroke(
                    [dx, dy + 4.0],
                    14.0,
                    7.0,
                    1.5,
                    [1.0, 1.0, 1.0, 0.9],
                    16,
                    white_uv,
                );
            }

            let ut_idx = u.unit_type as usize;
            if ut_idx < self.atlas.unit_uvs.len() {
                let is_hit_flash = u.hit_flash_timer > 0.0;
                let color_idx = get_color_index(u.team, is_hit_flash);

                let is_moving = matches!(
                    u.order,
                    UnitOrder::MoveTo { .. }
                        | UnitOrder::AttackMove { .. }
                        | UnitOrder::Harvest {
                            phase: HarvestPhase::MovingToNode | HarvestPhase::ReturningToBase,
                            ..
                        }
                        | UnitOrder::Build { .. }
                );

                let frame_idx = if u.attack_anim_timer > 0.0 {
                    let p = (1.0 - u.attack_anim_timer).clamp(0.0, 1.0);
                    ((p * 3.99) as usize).min(3)
                } else if is_moving {
                    let norm = (u.walk_cycle / std::f32::consts::TAU).fract();
                    ((norm * 7.99) as usize).clamp(0, 7)
                } else {
                    0
                };

                let uv = &self.atlas.unit_uvs[ut_idx][color_idx][frame_idx];
                let sprite_dim = if u.unit_type == UnitType::BehemothTitan { 96.0 } else { 64.0 };

                // Xử lý lật mặt trái / phải bằng cách đảo ngược trục UV X
                let (uv_min, uv_max) = if u.facing_right {
                    (uv.min, uv.max)
                } else {
                    ([uv.max[0], uv.min[1]], [uv.min[0], uv.max[1]])
                };

                let spr_y = dy - sprite_dim * 0.25;
                self.sprite_batch.push(
                    [dx, spr_y],
                    [sprite_dim, sprite_dim],
                    uv_min,
                    uv_max,
                    [1.0, 1.0, 1.0, 1.0],
                    0.0,
                );

                // 1. TaoistSwordmaster: Phi kiếm bay lượn xung quanh tương ứng số kiếm tích trữ (0..7)
                if u.unit_type == UnitType::TaoistSwordmaster {
                    let swords_count = (u.flying_swords as usize).min(7);
                    if swords_count > 0 {
                        let flip = if u.facing_right { 1.0 } else { -1.0 };
                        let sword_uv = &self.atlas.flying_sword_uv;
                        let (s_uv_min, s_uv_max) = if u.facing_right {
                            (sword_uv.min, sword_uv.max)
                        } else {
                            ([sword_uv.max[0], sword_uv.min[1]], [sword_uv.min[0], sword_uv.max[1]])
                        };

                        for i in 0..swords_count {
                            let arc_offset = ((i as f32) - (swords_count as f32 - 1.0) * 0.5) * 6.5;
                            let bob = (engine.game_time * 3.5 + (i as f32) * 0.8).sin() * 2.5;
                            let sw_x = dx - (11.0 + (i as f32) * 2.0) * flip;
                            let sw_y = dy - 16.0 + arc_offset + bob;
                            let sword_rot = if u.facing_right { -0.35 } else { 0.35 };

                            self.sprite_batch.push(
                                [sw_x, sw_y],
                                [28.0, 12.0],
                                s_uv_min,
                                s_uv_max,
                                [1.0, 1.0, 1.0, 0.95],
                                sword_rot,
                            );

                            // Điểm sáng chuôi kiếm xanh ngọc
                            self.sprite_batch.push(
                                [sw_x - 6.0 * flip, sw_y],
                                [4.0, 4.0],
                                self.atlas.particle_uvs[1].min,
                                self.atlas.particle_uvs[1].max,
                                [0.2, 0.9, 1.0, 0.9],
                                0.0,
                            );
                        }
                    }
                }

                // 2. Engineer: Hiệu ứng tia hàn nối vào mục tiêu và tia lửa bắn tung tóe khi khai thác/xây dựng
                if u.unit_type == UnitType::Engineer {
                    let is_welding = match u.order {
                        UnitOrder::Harvest {
                            phase: HarvestPhase::Mining(_),
                            ..
                        } => true,
                        UnitOrder::Build { target_building_id } => {
                            if let Some(b) = engine.get_building_by_id(target_building_id) {
                                let b_dx = b.x - u.x;
                                let b_dy = b.y - u.y;
                                let dist_sq = b_dx * b_dx + b_dy * b_dy;
                                let max_reach = b.width.max(b.height) * 0.5 + 45.0;
                                dist_sq <= max_reach * max_reach && b.build_progress < 1.0
                            } else {
                                false
                            }
                        }
                        _ => false,
                    };

                    if is_welding {
                        let flip = if u.facing_right { 1.0 } else { -1.0 };
                        let nozzle_x = dx + 12.0 * flip;
                        let nozzle_y = dy - 8.0;

                        // Tìm mục tiêu đang hàn (mỏ quặng hoặc công trình)
                        let target_pos = match u.order {
                            UnitOrder::Harvest {
                                target_mineral_id,
                                phase: HarvestPhase::Mining(_),
                            } => engine
                                .minerals
                                .iter()
                                .find(|m| m.id == target_mineral_id)
                                .map(|m| (m.x, m.y)),
                            UnitOrder::Build { target_building_id } => engine
                                .get_building_by_id(target_building_id)
                                .map(|b| (b.x, b.y)),
                            _ => None,
                        };

                        if let Some((tx, ty)) = target_pos {
                            // Tia năng lượng hàn ngoài (Cyan)
                            self.sprite_batch.push_line(
                                [nozzle_x, nozzle_y],
                                [tx, ty],
                                2.4,
                                [0.0, 0.85, 1.0, 0.85],
                                white_uv,
                            );
                            // Lõi tia trắng rực
                            self.sprite_batch.push_line(
                                [nozzle_x, nozzle_y],
                                [tx, ty],
                                1.0,
                                [1.0, 1.0, 1.0, 0.95],
                                white_uv,
                            );
                        }

                        // Đốm sáng đầu mỏ hàn
                        self.sprite_batch.push(
                            [nozzle_x, nozzle_y],
                            [8.0, 8.0],
                            self.atlas.particle_uvs[2].min, // yellow
                            self.atlas.particle_uvs[2].max,
                            [1.0, 1.0, 0.8, 1.0],
                            0.0,
                        );

                        // Các tia lửa hàn bắn tung tóe theo thời gian thực (Sparks)
                        let seed = (engine.game_time * 25.0).floor();
                        for i in 0..5 {
                            let ang = ((seed * 1.7 + (i as f32) * 1.6).sin() * 1.2) - 0.2;
                            let len = 5.0 + ((seed * 2.3 + (i as f32)).cos().abs() * 7.0);
                            let sp_x = nozzle_x + ang.cos() * len * flip;
                            let sp_y = nozzle_y + ang.sin() * len;
                            self.sprite_batch.push_line(
                                [nozzle_x, nozzle_y],
                                [sp_x, sp_y],
                                1.2,
                                [1.0, 0.95, 0.4, 0.9],
                                white_uv,
                            );
                        }
                    }
                }

                // 3. LaserMarksman: Tia laser công kích liên tục
                if u.unit_type == UnitType::LaserMarksman {
                    if let Some((target_x, target_y)) = u.last_attack_pos {
                        let flip = if u.facing_right { 1.0 } else { -1.0 };
                        let gun_tip_x = dx + 23.0 * flip;
                        let gun_tip_y = dy - 17.5;
                        let stacks = (u.laser_stacks as f32).min(25.0);
                        let beam_width = 1.6 + stacks * 0.25;

                        // Tia ngoài màu tím neon
                        self.sprite_batch.push_line(
                            [gun_tip_x, gun_tip_y],
                            [target_x, target_y],
                            beam_width + 1.6,
                            [0.75, 0.52, 0.99, 0.85],
                            white_uv,
                        );
                        // Lõi trắng rực
                        self.sprite_batch.push_line(
                            [gun_tip_x, gun_tip_y],
                            [target_x, target_y],
                            beam_width * 0.6,
                            [1.0, 1.0, 1.0, 0.95],
                            white_uv,
                        );
                        // Điểm sáng lóe nòng súng
                        self.sprite_batch.push(
                            [gun_tip_x, gun_tip_y],
                            [7.0, 7.0],
                            self.atlas.particle_uvs[1].min,
                            self.atlas.particle_uvs[1].max,
                            [0.85, 0.6, 1.0, 0.9],
                            0.0,
                        );
                        // Điểm sáng va chạm trên mục tiêu
                        self.sprite_batch.push(
                            [target_x, target_y],
                            [10.0 + stacks * 0.4, 10.0 + stacks * 0.4],
                            self.atlas.particle_uvs[0].min,
                            self.atlas.particle_uvs[0].max,
                            [1.0, 1.0, 1.0, 0.95],
                            0.0,
                        );
                    }
                }

                // Thanh máu và khiên trên đầu lính
                if u.max_hp > 0.0 {
                    let hp_pct = (u.hp / u.max_hp).clamp(0.0, 1.0);
                    let bar_w = 20.0;
                    let bar_h = 2.5;
                    let bar_y = dy - sprite_dim * 0.5 - 4.0;
                    self.sprite_batch.push_rect(
                        [dx, bar_y],
                        [bar_w, bar_h],
                        [0.1, 0.1, 0.1, 0.8],
                        white_uv,
                    );
                    let cur_w = bar_w * hp_pct;
                    let hp_color = if hp_pct > 0.4 {
                        [0.1, 0.85, 0.3, 0.9]
                    } else {
                        [0.9, 0.2, 0.2, 0.9]
                    };
                    self.sprite_batch.push_rect(
                        [dx - (bar_w - cur_w) * 0.5, bar_y],
                        [cur_w, bar_h],
                        hp_color,
                        white_uv,
                    );

                    // Thanh khiên (Shield Bar)
                    if u.max_shield > 0.0 && u.shield > 0.0 {
                        let shield_pct = (u.shield / u.max_shield).clamp(0.0, 1.0);
                        let shield_y = bar_y - 3.0;
                        self.sprite_batch.push_rect(
                            [dx, shield_y],
                            [bar_w, 2.0],
                            [0.05, 0.2, 0.35, 0.8],
                            white_uv,
                        );
                        let cur_sw = bar_w * shield_pct;
                        self.sprite_batch.push_rect(
                            [dx - (bar_w - cur_sw) * 0.5, shield_y],
                            [cur_sw, 2.0],
                            [0.0, 0.8, 1.0, 0.9],
                            white_uv,
                        );
                    }
                }
            }
        }

        // 3.6. Đạn & Tia chém (Projectiles) với đồ họa riêng biệt cho từng loại
        for p in &engine.projectiles {
            // Đòn cận chiến không hiển thị viên đạn
            if p.is_melee {
                continue;
            }

            // Tọa độ nội suy mượt mà giữa 2 tick
            let draw_x = p.prev_x + (p.x - p.prev_x) * alpha;
            let draw_y = p.prev_y + (p.y - p.prev_y) * alpha;

            if !engine.camera.is_point_visible(draw_x, draw_y, 50.0)
                && !engine.camera.is_point_visible(p.target_x, p.target_y, 50.0)
            {
                continue;
            }

            let is_visible = engine.are_allies(Team::Player, p.team)
                || engine.fog.is_visible(draw_x, draw_y)
                || engine.fog.is_visible(p.target_x, p.target_y);
            if !is_visible {
                continue;
            }

            // 1. Tia Laser (Tháp TurretTower hoặc vũ khí năng lượng)
            if p.is_laser || p.visual == crate::components::ProjectileVisual::Laser {
                let color_idx = get_color_index(p.team, false);
                let laser_color = PLAYER_LASER_COLORS[color_idx.min(7)];

                // Tia viền màu phe
                self.sprite_batch.push_line(
                    [p.x, p.y],
                    [p.target_x, p.target_y],
                    3.6,
                    laser_color,
                    white_uv,
                );
                // Lõi laser trắng rực
                self.sprite_batch.push_line(
                    [p.x, p.y],
                    [p.target_x, p.target_y],
                    1.5,
                    [1.0, 1.0, 1.0, 0.98],
                    white_uv,
                );

                // Điểm lóe sáng đầu nòng tháp
                self.sprite_batch.push(
                    [p.x, p.y],
                    [9.0, 9.0],
                    self.atlas.particle_uvs[1].min,
                    self.atlas.particle_uvs[1].max,
                    laser_color,
                    0.0,
                );
                // Điểm nổ sáng tại mục tiêu
                self.sprite_batch.push(
                    [p.target_x, p.target_y],
                    [14.0, 14.0],
                    self.atlas.particle_uvs[0].min,
                    self.atlas.particle_uvs[0].max,
                    [1.0, 1.0, 1.0, 0.98],
                    0.0,
                );
                continue;
            }

            // 2. Các loại đạn bay có đồ họa riêng theo enum ProjectileVisual
            match p.visual {
                crate::components::ProjectileVisual::Bullet => {
                    // Đạn súng thường / Rifleman
                    let uv = &self.atlas.bullet_uv;
                    self.sprite_batch.push(
                        [draw_x, draw_y],
                        [32.0, 16.0],
                        uv.min,
                        uv.max,
                        [1.0, 1.0, 1.0, 1.0],
                        p.angle,
                    );
                    // Vệt sáng vàng ở mũi đạn
                    self.sprite_batch.push(
                        [draw_x, draw_y],
                        [5.0, 5.0],
                        self.atlas.particle_uvs[2].min, // yellow
                        self.atlas.particle_uvs[2].max,
                        [1.0, 0.9, 0.4, 0.9],
                        0.0,
                    );
                }
                crate::components::ProjectileVisual::Rocket => {
                    // Tên lửa DarkRocketeer / Tháp tên lửa
                    let uv = &self.atlas.rocket_uv;
                    self.sprite_batch.push(
                        [draw_x, draw_y],
                        [36.0, 18.0],
                        uv.min,
                        uv.max,
                        [1.0, 1.0, 1.0, 1.0],
                        p.angle,
                    );

                    // Luồng lửa phản lực ở đuôi tên lửa
                    let tail_x = draw_x - p.angle.cos() * 14.0;
                    let tail_y = draw_y - p.angle.sin() * 14.0;
                    let flame_size = 11.0 + ((engine.game_time * 30.0).sin() * 2.5).abs();
                    self.sprite_batch.push(
                        [tail_x, tail_y],
                        [flame_size, flame_size * 0.7],
                        self.atlas.particle_uvs[3].min, // orange flame
                        self.atlas.particle_uvs[3].max,
                        [1.0, 0.6, 0.1, 0.95],
                        p.angle,
                    );
                }
                crate::components::ProjectileVisual::FlyingSword => {
                    // Phi kiếm TaoistSwordmaster phóng ra
                    let uv = &self.atlas.flying_sword_uv;
                    self.sprite_batch.push(
                        [draw_x, draw_y],
                        [36.0, 16.0],
                        uv.min,
                        uv.max,
                        [1.0, 1.0, 1.0, 1.0],
                        p.angle,
                    );

                    // Đốm hào quang kiếm khí ở mũi và đuôi
                    let tip_x = draw_x + p.angle.cos() * 12.0;
                    let tip_y = draw_y + p.angle.sin() * 12.0;
                    self.sprite_batch.push(
                        [tip_x, tip_y],
                        [8.0, 8.0],
                        self.atlas.particle_uvs[1].min, // cyan
                        self.atlas.particle_uvs[1].max,
                        [0.2, 0.85, 1.0, 0.9],
                        0.0,
                    );
                }
                crate::components::ProjectileVisual::Laser => {}
            }
        }

        // 3.7. Hiệu ứng hạt (Particles)
        for part in &engine.particles {
            if !engine.camera.is_point_visible(part.x, part.y, 20.0) {
                continue;
            }
            let alpha_col = (part.lifetime / part.max_lifetime.max(0.01)).clamp(0.0, 1.0);
            let p_idx = (part.color as usize).min(8);
            let p_uv = &self.atlas.particle_uvs[p_idx];
            self.sprite_batch.push(
                [part.x, part.y],
                [part.size, part.size],
                p_uv.min,
                p_uv.max,
                [1.0, 1.0, 1.0, alpha_col],
                0.0,
            );
        }

        // 3.8. Bóng công trình khi đặt móng (Ghost Building)
        if let Some(b_type) = engine.build_mode {
            let (wx, wy) = engine.screen_to_world(engine.mouse_screen_x, engine.mouse_screen_y);
            let (gw, gh) = engine.config.get_building_dimensions(b_type);

            let b_type_idx = b_type as usize;
            if b_type_idx < self.atlas.building_uvs.len() {
                let uv = &self.atlas.building_uvs[b_type_idx][0];
                let sw = gw + 20.0;
                let sh = gh + 30.0;
                // Bóng mờ màu xanh blueprint của chính công trình
                self.sprite_batch.push(
                    [wx, wy + 10.0],
                    [sw, sh],
                    uv.min,
                    uv.max,
                    [0.0, 0.9, 1.0, 0.65],
                    0.0,
                );
            }

            // Khung giới hạn đặt móng
            self.sprite_batch.push_rect(
                [wx, wy],
                [gw, gh],
                [0.0, 0.8, 1.0, 0.2],
                white_uv,
            );
            self.sprite_batch.push_stroke_rect(
                [wx, wy],
                [gw, gh],
                2.0,
                [0.0, 0.9, 1.0, 0.85],
                white_uv,
            );

            // Hiển thị vòng tầm bắn nếu có
            let range = engine
                .config
                .get_building(b_type)
                .map(|c| c.turret_range)
                .unwrap_or(0.0);
            if range > 0.0 {
                self.sprite_batch.push_circle_stroke(
                    [wx, wy],
                    range,
                    range,
                    1.5,
                    [0.0, 0.9, 1.0, 0.5],
                    36,
                    white_uv,
                );
            }
        }

        // 4. Chuẩn bị Instance Buffer lên GPU
        self.sprite_batch.prepare(&self.state.device, &self.state.queue);

        // 5. Render Pass
        let output = self.state.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("WebGPU World Render Encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("WebGPU World Main Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.031,
                            g: 0.047,
                            b: 0.078,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // 5.1. Render Blueprint Grid & Map Borders
            render_pass.set_pipeline(&self.grid_pipeline);
            render_pass.set_bind_group(0, &self.camera_only_bind_group, &[]);
            render_pass.draw(0..3, 0..1); // Fullscreen triangle

            // 5.2. Render Instanced Sprites (World Entities)
            render_pass.set_pipeline(&self.sprite_pipeline);
            render_pass.set_bind_group(0, &self.camera_atlas_bind_group, &[]);
            self.sprite_batch.render(&mut render_pass);

            // 5.3. Render Fog of War Overlay
            render_pass.set_pipeline(&self.fog_pipeline);
            render_pass.set_bind_group(0, &self.camera_only_bind_group, &[]);
            render_pass.set_bind_group(1, &self.fog_pass.bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }

        self.state.queue.submit(Some(encoder.finish()));
        output.present();

        Ok(())
    }
}
