use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::components::{BuildingType, Team, UnitType};
use crate::config::GameConfig;
use crate::fog::{FOG_COLS, FOG_ROWS};

// Helper tạo offscreen HTML canvas element
fn create_offscreen_canvas(
    w: u32,
    h: u32,
) -> Option<(HtmlCanvasElement, CanvasRenderingContext2d)> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let canvas: HtmlCanvasElement = document.create_element("canvas").ok()?.dyn_into().ok()?;
    canvas.set_width(w);
    canvas.set_height(h);
    let ctx: CanvasRenderingContext2d = canvas.get_context("2d").ok()??.dyn_into().ok()?;
    Some((canvas, ctx))
}

pub const COLOR_PLAYER: usize = 0;
pub const COLOR_ALLY: usize = 1;
pub const COLOR_ENEMY_0: usize = 2;
pub const COLOR_ENEMY_1: usize = 3;
pub const COLOR_ENEMY_2: usize = 4;
pub const COLOR_ENEMY_3: usize = 5;
pub const COLOR_ENEMY_4: usize = 6;
pub const COLOR_ENEMY_5: usize = 7;
pub const COLOR_HIT_FLASH: usize = 8;
pub const NUM_COLOR_VARIANTS: usize = 9;

pub fn get_color_index(
    team: Team,
    hit_flash: bool,
    player_faction: u8,
    team_factions: &[u8; 8],
) -> usize {
    if hit_flash {
        return COLOR_HIT_FLASH;
    }
    let t_idx = team.index() % 8;
    if team == Team::Player {
        COLOR_PLAYER
    } else if team_factions[t_idx] == player_faction {
        COLOR_ALLY
    } else {
        COLOR_ENEMY_0 + (t_idx % 6)
    }
}

pub struct GpuCacheManager {
    // 1. Grid Tile Pattern Cache (512x512)
    pub grid_canvas: HtmlCanvasElement,

    // 2. Mineral Sprite (64x64)
    pub mineral_canvas: HtmlCanvasElement,

    // 3. Magic Circle Texture (192x192)
    pub magic_circle_canvas: HtmlCanvasElement,

    // 4. Projectiles Atlas Canvas
    pub bullet_canvas: HtmlCanvasElement,
    pub rocket_canvas: HtmlCanvasElement,
    pub flying_sword_canvas: HtmlCanvasElement,

    // 5. Buildings Atlas: [BuildingType 0..6][ColorVariant 0..9]
    // Mỗi building sprite có kích thước (w + 20, h + 30) để chứa cả bracket + text nhãn đã bake
    pub building_sprites: Vec<Vec<HtmlCanvasElement>>,

    // 6. Units Sprite Atlas: [UnitType 0..10][ColorVariant 0..9][Facing: 0=Left, 1=Right][AnimFrame 0..8]
    // Kích thước chuẩn cho unit sprite: 64x64 (BehemothTitan 96x96)
    pub unit_sprites: Vec<Vec<Vec<Vec<HtmlCanvasElement>>>>,

    // 7. Fog of War Texture Canvas (120x120, được update động chỉ khi fog_dirty)
    pub fog_canvas: HtmlCanvasElement,
    pub fog_ctx: CanvasRenderingContext2d,

    // 8. Minimap Static Base Texture Canvas (160x160: chứa nền + mỏ quặng đã khám phá + tuyến phòng thủ)
    pub minimap_base_canvas: HtmlCanvasElement,
    pub minimap_base_ctx: CanvasRenderingContext2d,

    // 9. Particle Sprite Cache: mỗi màu 1 đốm sáng tròn
    pub particle_blue: HtmlCanvasElement,
    pub particle_cyan: HtmlCanvasElement,
    pub particle_yellow: HtmlCanvasElement,
    pub particle_orange: HtmlCanvasElement,
    pub particle_red: HtmlCanvasElement,
    pub particle_purple: HtmlCanvasElement,
    pub particle_green: HtmlCanvasElement,
    pub particle_slate: HtmlCanvasElement,
    pub particle_white: HtmlCanvasElement,
}

impl GpuCacheManager {
    pub fn new(config: &GameConfig) -> Option<Self> {
        // --- 1. Bake Grid Tile (512x512) ---
        let (grid_canvas, grid_ctx) = create_offscreen_canvas(512, 512)?;
        grid_ctx.set_stroke_style_str(&config.render.grid_line_color);
        grid_ctx.set_line_width(1.0);
        grid_ctx.begin_path();
        let grid_size = config.render.grid_size.max(10.0) as f64;
        let mut x = 0.0;
        while x <= 512.0 {
            grid_ctx.move_to(x, 0.0);
            grid_ctx.line_to(x, 512.0);
            x += grid_size;
        }
        let mut y = 0.0;
        while y <= 512.0 {
            grid_ctx.move_to(0.0, y);
            grid_ctx.line_to(512.0, y);
            y += grid_size;
        }
        grid_ctx.stroke();

        // --- 2. Bake Mineral Sprite (64x64) ---
        let (mineral_canvas, m_ctx) = create_offscreen_canvas(64, 64)?;
        let mx = 32.0;
        let my = 28.0;
        m_ctx.set_stroke_style_str("#38bdf8");
        m_ctx.set_fill_style_str("rgba(56, 189, 248, 0.28)");
        m_ctx.set_line_width(2.0);
        m_ctx.begin_path();
        m_ctx.move_to(mx, my - 18.0);
        m_ctx.line_to(mx + 16.0, my - 5.0);
        m_ctx.line_to(mx + 12.0, my + 16.0);
        m_ctx.line_to(mx - 12.0, my + 16.0);
        m_ctx.line_to(mx - 16.0, my - 5.0);
        m_ctx.close_path();
        m_ctx.fill();
        m_ctx.stroke();

        // --- 3. Bake Magic Circle Texture (192x192) ---
        let (magic_circle_canvas, mc_ctx) = create_offscreen_canvas(192, 192)?;
        let cx = 96.0;
        let cy = 96.0;
        let r = 85.0;
        mc_ctx.set_fill_style_str("rgba(168, 85, 247, 0.22)");
        mc_ctx.begin_path();
        let _ = mc_ctx.arc(cx, cy, r, 0.0, std::f64::consts::PI * 2.0);
        mc_ctx.fill();

        mc_ctx.set_stroke_style_str("rgba(192, 132, 252, 0.95)");
        mc_ctx.set_line_width(2.0);
        mc_ctx.begin_path();
        let _ = mc_ctx.arc(cx, cy, r, 0.0, std::f64::consts::PI * 2.0);
        mc_ctx.stroke();

        mc_ctx.set_line_width(1.0);
        mc_ctx.begin_path();
        let _ = mc_ctx.arc(cx, cy, r - 5.0, 0.0, std::f64::consts::PI * 2.0);
        mc_ctx.stroke();

        mc_ctx.set_stroke_style_str("rgba(232, 121, 249, 0.85)");
        mc_ctx.set_line_width(1.5);
        mc_ctx.begin_path();
        for i in 0..6 {
            let a = (i as f64) * std::f64::consts::PI / 3.0;
            let px = cx + (r - 9.0) * a.cos();
            let py = cy + (r - 9.0) * a.sin();
            if i == 0 {
                mc_ctx.move_to(px, py);
            } else {
                mc_ctx.line_to(px, py);
            }
        }
        mc_ctx.close_path();
        mc_ctx.stroke();

        mc_ctx.set_stroke_style_str("rgba(168, 85, 247, 0.7)");
        mc_ctx.begin_path();
        for i in 0..3 {
            let a = (i as f64) * std::f64::consts::PI * 2.0 / 3.0;
            let px = cx + (r * 0.62) * a.cos();
            let py = cy + (r * 0.62) * a.sin();
            if i == 0 {
                mc_ctx.move_to(px, py);
            } else {
                mc_ctx.line_to(px, py);
            }
        }
        mc_ctx.close_path();
        mc_ctx.stroke();

        mc_ctx.begin_path();
        for i in 0..3 {
            let a = std::f64::consts::PI + (i as f64) * std::f64::consts::PI * 2.0 / 3.0;
            let px = cx + (r * 0.62) * a.cos();
            let py = cy + (r * 0.62) * a.sin();
            if i == 0 {
                mc_ctx.move_to(px, py);
            } else {
                mc_ctx.line_to(px, py);
            }
        }
        mc_ctx.close_path();
        mc_ctx.stroke();

        mc_ctx.set_fill_style_str("rgba(244, 114, 182, 0.95)");
        mc_ctx.begin_path();
        let _ = mc_ctx.arc(cx, cy, 4.5, 0.0, std::f64::consts::PI * 2.0);
        mc_ctx.fill();

        // --- 4. Bake Projectiles (Bullet, Rocket, FlyingSword) ---
        // Bullet (32x16, origin at 20, 8)
        let (bullet_canvas, b_ctx) = create_offscreen_canvas(32, 16)?;
        b_ctx.set_stroke_style_str("rgba(250, 204, 21, 0.45)");
        b_ctx.set_line_width(3.0);
        b_ctx.begin_path();
        b_ctx.move_to(6.0, 8.0);
        b_ctx.line_to(20.0, 8.0);
        b_ctx.stroke();

        b_ctx.set_stroke_style_str("#facc15");
        b_ctx.set_line_width(2.0);
        b_ctx.begin_path();
        b_ctx.move_to(11.0, 8.0);
        b_ctx.line_to(23.0, 8.0);
        b_ctx.stroke();

        b_ctx.set_stroke_style_str("#ffffff");
        b_ctx.set_line_width(1.2);
        b_ctx.begin_path();
        b_ctx.move_to(17.0, 8.0);
        b_ctx.line_to(23.0, 8.0);
        b_ctx.stroke();

        b_ctx.set_fill_style_str("#ffffff");
        b_ctx.begin_path();
        let _ = b_ctx.arc(23.0, 8.0, 1.2, 0.0, std::f64::consts::PI * 2.0);
        b_ctx.fill();

        // Rocket (36x18, origin at 18, 9)
        let (rocket_canvas, rk_ctx) = create_offscreen_canvas(36, 18)?;
        rk_ctx.set_fill_style_str("#ea580c");
        rk_ctx.begin_path();
        rk_ctx.move_to(12.0, 6.8);
        rk_ctx.line_to(6.0, 9.0);
        rk_ctx.line_to(12.0, 11.2);
        rk_ctx.close_path();
        rk_ctx.fill();

        rk_ctx.set_fill_style_str("#fef08a");
        rk_ctx.begin_path();
        rk_ctx.move_to(12.0, 7.8);
        rk_ctx.line_to(8.5, 9.0);
        rk_ctx.line_to(12.0, 10.2);
        rk_ctx.close_path();
        rk_ctx.fill();

        rk_ctx.set_stroke_style_str("#c2410c");
        rk_ctx.set_line_width(1.6);
        rk_ctx.begin_path();
        rk_ctx.move_to(12.0, 5.0);
        rk_ctx.line_to(15.0, 7.0);
        rk_ctx.line_to(15.0, 11.0);
        rk_ctx.line_to(12.0, 13.0);
        rk_ctx.stroke();

        rk_ctx.set_fill_style_str("#475569");
        rk_ctx.fill_rect(12.0, 7.0, 9.0, 4.0);
        rk_ctx.set_stroke_style_str("#ea580c");
        rk_ctx.set_line_width(1.2);
        rk_ctx.stroke_rect(12.0, 7.0, 9.0, 4.0);

        rk_ctx.set_fill_style_str("#ef4444");
        rk_ctx.begin_path();
        rk_ctx.move_to(21.0, 6.8);
        rk_ctx.line_to(25.5, 9.0);
        rk_ctx.line_to(21.0, 11.2);
        rk_ctx.close_path();
        rk_ctx.fill();

        // Flying Sword (36x16, origin at 18, 8)
        let (flying_sword_canvas, fs_ctx) = create_offscreen_canvas(36, 16)?;
        fs_ctx.set_stroke_style_str("rgba(56, 189, 248, 0.35)");
        fs_ctx.set_line_width(3.2);
        fs_ctx.begin_path();
        fs_ctx.move_to(6.0, 8.0);
        fs_ctx.line_to(18.0, 8.0);
        fs_ctx.stroke();

        fs_ctx.set_fill_style_str("#facc15");
        fs_ctx.begin_path();
        let _ = fs_ctx.arc(11.5, 8.0, 1.2, 0.0, std::f64::consts::PI * 2.0);
        fs_ctx.fill();

        fs_ctx.set_stroke_style_str("#64748b");
        fs_ctx.set_line_width(1.8);
        fs_ctx.begin_path();
        fs_ctx.move_to(11.5, 8.0);
        fs_ctx.line_to(14.0, 8.0);
        fs_ctx.stroke();

        fs_ctx.set_stroke_style_str("#facc15");
        fs_ctx.set_line_width(1.8);
        fs_ctx.begin_path();
        fs_ctx.move_to(14.0, 4.8);
        fs_ctx.line_to(14.0, 11.2);
        fs_ctx.stroke();

        fs_ctx.set_stroke_style_str("#38bdf8");
        fs_ctx.set_line_width(2.2);
        fs_ctx.begin_path();
        fs_ctx.move_to(14.0, 8.0);
        fs_ctx.line_to(25.5, 8.0);
        fs_ctx.stroke();

        fs_ctx.set_stroke_style_str("#ffffff");
        fs_ctx.set_line_width(1.0);
        fs_ctx.begin_path();
        fs_ctx.move_to(14.5, 8.0);
        fs_ctx.line_to(25.5, 8.0);
        fs_ctx.stroke();

        // --- 5. Bake Particle Sprites (16x16) ---
        let bake_particle = |color: &str| -> Option<HtmlCanvasElement> {
            let (c, ctx) = create_offscreen_canvas(16, 16)?;
            ctx.set_fill_style_str(color);
            ctx.begin_path();
            let _ = ctx.arc(8.0, 8.0, 6.0, 0.0, std::f64::consts::PI * 2.0);
            ctx.fill();
            Some(c)
        };
        let particle_blue = bake_particle("#38bdf8")?;
        let particle_cyan = bake_particle("#00e5ff")?;
        let particle_yellow = bake_particle("#facc15")?;
        let particle_orange = bake_particle("#fb923c")?;
        let particle_red = bake_particle("#ef4444")?;
        let particle_purple = bake_particle("#c084fc")?;
        let particle_green = bake_particle("#84cc16")?;
        let particle_slate = bake_particle("#64748b")?;
        let particle_white = bake_particle("#ffffff")?;

        // --- 6. Bake Building Sprites ---
        let color_palette = [
            ("#00e5ff", "rgba(0, 229, 255, 0.12)"),   // Player
            ("#10b981", "rgba(16, 185, 129, 0.12)"),  // Ally
            ("#f43f5e", "rgba(244, 63, 94, 0.12)"),   // Enemy 0
            ("#ec4899", "rgba(236, 72, 153, 0.12)"),  // Enemy 1
            ("#8b5cf6", "rgba(139, 92, 246, 0.12)"),  // Enemy 2
            ("#eab308", "rgba(234, 179, 8, 0.12)"),   // Enemy 3
            ("#f97316", "rgba(249, 115, 22, 0.12)"),  // Enemy 4
            ("#06b6d4", "rgba(6, 182, 212, 0.12)"),   // Enemy 5
            ("#ffffff", "rgba(255, 255, 255, 0.35)"), // Hit flash
        ];

        let all_buildings = [
            BuildingType::CommandCenter,
            BuildingType::SupplyDepot,
            BuildingType::PowerReactor,
            BuildingType::Barracks,
            BuildingType::TurretTower,
            BuildingType::EnergyConverter,
        ];

        let mut building_sprites = Vec::with_capacity(all_buildings.len());
        for &bt in &all_buildings {
            let (w, h) = config.get_building_dimensions(bt);
            let cw = (w + 20.0).ceil() as u32;
            let ch = (h + 30.0).ceil() as u32;

            let mut variants = Vec::with_capacity(NUM_COLOR_VARIANTS);
            for &(stroke_col, fill_col) in &color_palette {
                let (c, ctx) = create_offscreen_canvas(cw, ch)?;
                let ox = 10.0;
                let oy = 5.0;

                // Nền nhà
                ctx.set_fill_style_str(fill_col);
                ctx.fill_rect(ox, oy, w as f64, h as f64);

                // Viền nhà
                ctx.set_stroke_style_str(stroke_col);
                ctx.set_line_width(1.5);
                ctx.stroke_rect(ox, oy, w as f64, h as f64);

                // Ký hiệu góc kỹ thuật
                crate::renderer::buildings::draw_blueprint_brackets(
                    &ctx, ox, oy, w as f64, h as f64, stroke_col,
                );

                // Đồ họa lõi
                crate::renderer::buildings::draw_building_graphic(
                    &ctx,
                    bt,
                    ox + (w * 0.5) as f64,
                    oy + (h * 0.5) as f64,
                    w as f64,
                    h as f64,
                    stroke_col,
                );

                // Tên công trình nhãn đã bake
                ctx.set_font("bold 10px monospace");
                ctx.set_text_align("center");
                ctx.set_fill_style_str(stroke_col);
                let title = config.get_building_short_name(bt);
                let _ = ctx.fill_text(title, ox + (w * 0.5) as f64, oy + (h as f64) + 13.0);

                variants.push(c);
            }
            building_sprites.push(variants);
        }

        // --- 7. Bake Unit Sprites ---
        let all_units = [
            UnitType::Engineer,
            UnitType::ShockTrooper,
            UnitType::Rifleman,
            UnitType::Vanguard,
            UnitType::LaserMarksman,
            UnitType::DarkRocketeer,
            UnitType::TaoistSwordmaster,
            UnitType::VoidWarlock,
            UnitType::BehemothTitan,
            UnitType::AcidBaneling,
        ];

        let num_anim_frames = 8;
        let mut unit_sprites = Vec::with_capacity(all_units.len());

        for &ut in &all_units {
            let sprite_dim: u32 = if ut == UnitType::BehemothTitan {
                96
            } else {
                64
            };
            let mut unit_variants = Vec::with_capacity(NUM_COLOR_VARIANTS);

            for &(col, _) in &color_palette {
                let mut facing_variants = Vec::with_capacity(2);
                for facing_idx in 0..2 {
                    let facing_right = facing_idx == 1;
                    let mut frame_variants = Vec::with_capacity(num_anim_frames);

                    for f_idx in 0..num_anim_frames {
                        let (c, ctx) = create_offscreen_canvas(sprite_dim, sprite_dim)?;
                        let cx = (sprite_dim as f64) * 0.5;
                        let cy = (sprite_dim as f64) * 0.75; // Đáy chân tại cy + 1

                        // Tạo 1 mock unit để dùng lại code vẽ tinh vi
                        let mut mock_u = config.create_unit(0, Team::Player, ut, 0.0, 0.0);
                        mock_u.flying_swords = 0; // Để sprite cơ bản không chứa kiếm tĩnh, kiếm sẽ được vẽ động theo số lượng thực tế
                        mock_u.facing_right = facing_right;
                        mock_u.walk_cycle =
                            (f_idx as f32) * (std::f32::consts::TAU / (num_anim_frames as f32));
                        mock_u.attack_anim_timer = if f_idx < 4 {
                            (4 - f_idx) as f32 * 0.25
                        } else {
                            0.0
                        };
                        if f_idx >= 1 {
                            mock_u.order =
                                crate::components::UnitOrder::MoveTo { x: 100.0, y: 100.0 };
                        } else {
                            mock_u.order = crate::components::UnitOrder::Idle;
                        }

                        crate::renderer::units::draw_procedural_stickman_direct(
                            &ctx, &mock_u, col, 0.0, false, cx, cy,
                        );

                        frame_variants.push(c);
                    }
                    facing_variants.push(frame_variants);
                }
                unit_variants.push(facing_variants);
            }
            unit_sprites.push(unit_variants);
        }

        // --- 8. Fog Texture Canvas & Minimap Canvas ---
        let (fog_canvas, fog_ctx) = create_offscreen_canvas(FOG_COLS as u32, FOG_ROWS as u32)?;
        let (minimap_base_canvas, minimap_base_ctx) = create_offscreen_canvas(160, 160)?;

        Some(Self {
            grid_canvas,
            mineral_canvas,
            magic_circle_canvas,
            bullet_canvas,
            rocket_canvas,
            flying_sword_canvas,
            building_sprites,
            unit_sprites,
            fog_canvas,
            fog_ctx,
            minimap_base_canvas,
            minimap_base_ctx,
            particle_blue,
            particle_cyan,
            particle_yellow,
            particle_orange,
            particle_red,
            particle_purple,
            particle_green,
            particle_slate,
            particle_white,
        })
    }

    /// Cập nhật Fog Texture Canvas (chỉ chạy khi fog_dirty = true)
    pub fn update_fog_texture(&self, fog_grid: &[crate::fog::Visibility]) {
        use crate::fog::Visibility;

        self.fog_ctx
            .clear_rect(0.0, 0.0, FOG_COLS as f64, FOG_ROWS as f64);

        for r in 0..FOG_ROWS {
            let ry = r as f64;
            let mut current_vis = Visibility::Visible;
            let mut start_c = 0;
            let mut count = 0;

            for c in 0..FOG_COLS {
                let vis = fog_grid[r * FOG_COLS + c];
                if vis == current_vis {
                    count += 1;
                } else {
                    if current_vis != Visibility::Visible && count > 0 {
                        let rx = start_c as f64;
                        let rw = count as f64;
                        if current_vis == Visibility::Hidden {
                            self.fog_ctx.set_fill_style_str("rgba(2, 6, 13, 0.96)");
                        } else {
                            self.fog_ctx.set_fill_style_str("rgba(3, 13, 26, 0.6)");
                        }
                        self.fog_ctx.fill_rect(rx, ry, rw, 1.0);
                    }
                    current_vis = vis;
                    start_c = c;
                    count = 1;
                }
            }

            if current_vis != Visibility::Visible && count > 0 {
                let rx = start_c as f64;
                let rw = count as f64;
                if current_vis == Visibility::Hidden {
                    self.fog_ctx.set_fill_style_str("rgba(2, 6, 13, 0.96)");
                } else {
                    self.fog_ctx.set_fill_style_str("rgba(3, 13, 26, 0.6)");
                }
                self.fog_ctx.fill_rect(rx, ry, rw, 1.0);
            }
        }
    }
}
