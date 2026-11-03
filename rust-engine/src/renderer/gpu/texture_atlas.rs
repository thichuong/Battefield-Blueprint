use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::components::BuildingType;
use crate::renderer::gpu_cache::{GpuCacheManager, NUM_COLOR_VARIANTS};

pub const ATLAS_WIDTH: u32 = 2048;
pub const ATLAS_HEIGHT: u32 = 4096;

pub struct UvRect {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

impl UvRect {
    pub fn new(x: f32, y: f32, w: f32, h: f32, atlas_w: f32, atlas_h: f32) -> Self {
        Self {
            min: [x / atlas_w, y / atlas_h],
            max: [(x + w) / atlas_w, (y + h) / atlas_h],
        }
    }
}

pub struct TextureAtlas {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub white_pixel_uv: [f32; 2],
    pub mineral_uv: UvRect,
    pub magic_circle_uv: UvRect,
    pub bullet_uv: UvRect,
    pub rocket_uv: UvRect,
    pub flying_sword_uv: UvRect,
    pub particle_uvs: [UvRect; 9],
    pub building_uvs: Vec<Vec<UvRect>>,
    pub unit_uvs: Vec<Vec<Vec<UvRect>>>, // [unit_type 0..10][color 0..9][frame 0..8]
}

impl TextureAtlas {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        cache: &GpuCacheManager,
    ) -> Option<Self> {
        let window = web_sys::window()?;
        let document = window.document()?;
        let atlas_canvas: HtmlCanvasElement =
            document.create_element("canvas").ok()?.dyn_into().ok()?;
        atlas_canvas.set_width(ATLAS_WIDTH);
        atlas_canvas.set_height(ATLAS_HEIGHT);
        let ctx: CanvasRenderingContext2d =
            atlas_canvas.get_context("2d").ok()??.dyn_into().ok()?;

        let aw = ATLAS_WIDTH as f64;
        let ah = ATLAS_HEIGHT as f64;

        // Xóa atlas với màu trong suốt
        ctx.clear_rect(0.0, 0.0, aw, ah);

        // 1. Pixel trắng 4x4 tại (0, 0)
        ctx.set_fill_style_str("#ffffff");
        ctx.fill_rect(0.0, 0.0, 4.0, 4.0);
        let white_pixel_uv = [2.0 / (ATLAS_WIDTH as f32), 2.0 / (ATLAS_HEIGHT as f32)];

        let mut cur_x = 8.0;
        let mut cur_y = 8.0;
        let mut row_h: f64 = 0.0;

        // 2. Mỏ khoáng sản (64x64)
        let _ = ctx.draw_image_with_html_canvas_element(&cache.mineral_canvas, cur_x, cur_y);
        let mineral_uv = UvRect::new(
            cur_x as f32,
            cur_y as f32,
            64.0,
            64.0,
            ATLAS_WIDTH as f32,
            ATLAS_HEIGHT as f32,
        );
        cur_x += 70.0;
        row_h = row_h.max(70.0);

        // 3. Vòng ma thuật (192x192)
        let _ = ctx.draw_image_with_html_canvas_element(&cache.magic_circle_canvas, cur_x, cur_y);
        let magic_circle_uv = UvRect::new(
            cur_x as f32,
            cur_y as f32,
            192.0,
            192.0,
            ATLAS_WIDTH as f32,
            ATLAS_HEIGHT as f32,
        );
        cur_x += 200.0;
        row_h = row_h.max(200.0);

        // 4. Đạn / Tên lửa / Phi kiếm
        let _ = ctx.draw_image_with_html_canvas_element(&cache.bullet_canvas, cur_x, cur_y);
        let bullet_uv = UvRect::new(
            cur_x as f32,
            cur_y as f32,
            32.0,
            16.0,
            ATLAS_WIDTH as f32,
            ATLAS_HEIGHT as f32,
        );
        cur_x += 40.0;

        let _ = ctx.draw_image_with_html_canvas_element(&cache.rocket_canvas, cur_x, cur_y);
        let rocket_uv = UvRect::new(
            cur_x as f32,
            cur_y as f32,
            36.0,
            18.0,
            ATLAS_WIDTH as f32,
            ATLAS_HEIGHT as f32,
        );
        cur_x += 45.0;

        let _ = ctx.draw_image_with_html_canvas_element(&cache.flying_sword_canvas, cur_x, cur_y);
        let flying_sword_uv = UvRect::new(
            cur_x as f32,
            cur_y as f32,
            36.0,
            16.0,
            ATLAS_WIDTH as f32,
            ATLAS_HEIGHT as f32,
        );
        cur_x += 45.0;

        // 5. Hạt hạt nổ (9 loại màu)
        let particle_canvases = [
            &cache.particle_blue,
            &cache.particle_cyan,
            &cache.particle_yellow,
            &cache.particle_orange,
            &cache.particle_red,
            &cache.particle_purple,
            &cache.particle_green,
            &cache.particle_slate,
            &cache.particle_white,
        ];
        let mut particle_uvs = [
            UvRect::new(0.0, 0.0, 0.0, 0.0, 1.0, 1.0),
            UvRect::new(0.0, 0.0, 0.0, 0.0, 1.0, 1.0),
            UvRect::new(0.0, 0.0, 0.0, 0.0, 1.0, 1.0),
            UvRect::new(0.0, 0.0, 0.0, 0.0, 1.0, 1.0),
            UvRect::new(0.0, 0.0, 0.0, 0.0, 1.0, 1.0),
            UvRect::new(0.0, 0.0, 0.0, 0.0, 1.0, 1.0),
            UvRect::new(0.0, 0.0, 0.0, 0.0, 1.0, 1.0),
            UvRect::new(0.0, 0.0, 0.0, 0.0, 1.0, 1.0),
            UvRect::new(0.0, 0.0, 0.0, 0.0, 1.0, 1.0),
        ];
        for (i, p_canvas) in particle_canvases.iter().enumerate() {
            let _ = ctx.draw_image_with_html_canvas_element(p_canvas, cur_x, cur_y);
            particle_uvs[i] = UvRect::new(
                cur_x as f32,
                cur_y as f32,
                16.0,
                16.0,
                ATLAS_WIDTH as f32,
                ATLAS_HEIGHT as f32,
            );
            cur_x += 20.0;
        }

        // Xuống hàng tiếp theo cho Công trình
        cur_x = 8.0;
        cur_y += row_h + 8.0;
        row_h = 0.0;

        // 6. Công trình (6 types x 9 colors)
        let all_buildings = [
            BuildingType::CommandCenter,
            BuildingType::SupplyDepot,
            BuildingType::PowerReactor,
            BuildingType::Barracks,
            BuildingType::TurretTower,
            BuildingType::EnergyConverter,
        ];

        let mut building_uvs = Vec::with_capacity(all_buildings.len());
        for b_type_idx in 0..cache.building_sprites.len() {
            let mut color_variants = Vec::with_capacity(NUM_COLOR_VARIANTS);
            for c_idx in 0..cache.building_sprites[b_type_idx].len() {
                let sprite = &cache.building_sprites[b_type_idx][c_idx];
                let sw = sprite.width() as f64;
                let sh = sprite.height() as f64;

                if cur_x + sw + 4.0 > aw {
                    cur_x = 8.0;
                    cur_y += row_h + 6.0;
                    row_h = 0.0;
                }

                let _ = ctx.draw_image_with_html_canvas_element(sprite, cur_x, cur_y);
                color_variants.push(UvRect::new(
                    cur_x as f32,
                    cur_y as f32,
                    sw as f32,
                    sh as f32,
                    ATLAS_WIDTH as f32,
                    ATLAS_HEIGHT as f32,
                ));

                cur_x += sw + 4.0;
                row_h = row_h.max(sh);
            }
            building_uvs.push(color_variants);
        }

        // Xuống hàng tiếp theo cho Quân lính (Units)
        cur_x = 8.0;
        cur_y += row_h + 8.0;
        row_h = 0.0;

        // 7. Quân lính (11 unit types x 9 colors x 8 animation frames)
        // Lưu ý: Chỉ cần lưu hướng facing_right = true (facing_idx = 1)
        // Hướng facing_left sẽ được lật UV tự động khi nạp SpriteBatch
        let mut unit_uvs = Vec::with_capacity(cache.unit_sprites.len());
        for u_type_idx in 0..cache.unit_sprites.len() {
            let mut color_variants = Vec::with_capacity(NUM_COLOR_VARIANTS);
            for c_idx in 0..cache.unit_sprites[u_type_idx].len() {
                let facing_idx = 1; // Luôn dùng facing_right
                let mut frame_variants = Vec::with_capacity(8);

                for f_idx in 0..cache.unit_sprites[u_type_idx][c_idx][facing_idx].len() {
                    let sprite = &cache.unit_sprites[u_type_idx][c_idx][facing_idx][f_idx];
                    let sw = sprite.width() as f64;
                    let sh = sprite.height() as f64;

                    if cur_x + sw + 4.0 > aw {
                        cur_x = 8.0;
                        cur_y += row_h + 4.0;
                        row_h = 0.0;
                    }

                    let _ = ctx.draw_image_with_html_canvas_element(sprite, cur_x, cur_y);
                    frame_variants.push(UvRect::new(
                        cur_x as f32,
                        cur_y as f32,
                        sw as f32,
                        sh as f32,
                        ATLAS_WIDTH as f32,
                        ATLAS_HEIGHT as f32,
                    ));

                    cur_x += sw + 4.0;
                    row_h = row_h.max(sh);
                }
                color_variants.push(frame_variants);
            }
            unit_uvs.push(color_variants);
        }

        // 8. Đọc toàn bộ buffer pixel RGBA từ canvas atlas
        let image_data = ctx.get_image_data(0.0, 0.0, aw, ah).ok()?.data();
        let pixel_bytes: Vec<u8> = image_data.0;

        // 9. Tạo GPU Texture và nạp pixel
        let texture_size = wgpu::Extent3d {
            width: ATLAS_WIDTH,
            height: ATLAS_HEIGHT,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("WebGPU Atlas Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &pixel_bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * ATLAS_WIDTH),
                rows_per_image: Some(ATLAS_HEIGHT),
            },
            texture_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("WebGPU Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Some(Self {
            texture,
            view,
            sampler,
            white_pixel_uv,
            mineral_uv,
            magic_circle_uv,
            bullet_uv,
            rocket_uv,
            flying_sword_uv,
            particle_uvs,
            building_uvs,
            unit_uvs,
        })
    }
}
