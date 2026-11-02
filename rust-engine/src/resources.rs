use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerEconomy {
    pub minerals: f32,
    pub energy: f32,
    pub supply: u32,
    pub max_supply: u32,
    pub minerals_per_sec: f32,
    pub energy_per_sec: f32,
}

impl Default for PlayerEconomy {
    fn default() -> Self {
        Self {
            minerals: 200.0,
            energy: 50.0,
            supply: 4,
            max_supply: 10,
            minerals_per_sec: 0.0,
            energy_per_sec: 2.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ActiveResearch {
    pub tech_id: String,
    pub branch: String,
    pub progress: f32, // 0.0 -> 1.0
    pub total_time: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TechTreeState {
    // === NHÁNH KHOA HỌC (SCIENCE) ===
    // 1. Công nghệ tương lai (Future Tech)
    pub focused_laser: bool, // Mở LaserMarksman & +25% Sát thương Năng lượng
    pub energy_shielding: bool, // Mở Vanguard & +40% Kháng Năng lượng
    // 2. Hắc khoa kỹ (Dark Tech)
    pub dark_matter_rockets: bool, // Mở DarkRocketeer & đạn nổ AoE
    pub titan_alloy_plating: bool, // +35% Kháng Vật lý & +30% Máu công trình

    // === NHÁNH MA THUẬT (MAGIC) ===
    // 1. Tiên pháp (Celestial / Taoist Arts)
    pub celestial_flying_swords: bool, // Mở TaoistSwordmaster & phi kiếm chân khí Arcane
    pub golden_bell_shield: bool, // Kim cang linh quang (+40% Kháng Năng lượng & Arcane, hồi 3 HP/s)
    // 2. Pháp thuật bóng tối (Dark Arts)
    pub corrosive_curse: bool, // Lời nguyền ăn mòn (-20% mọi chỉ số kháng của kẻ địch khi trúng đòn)
    pub void_cataclysm: bool,  // Mở VoidWarlock & cầu ma thuật bộc phá AoE

    // === NHÁNH QUÁI VẬT (MONSTERS) ===
    // 1. Khổng lồ (Behemoth / Titan)
    pub primeval_scales: bool, // Vảy cổ đại (+50% Kháng Vật lý, +40% Kháng Nổ)
    pub behemoth_mutation: bool, // Mở BehemothTitan (HP 650, chấn động diện rộng)
    // 2. Bọ bầy đàn (Swarm / Insectoid)
    pub acid_banelings: bool, // Mở AcidBaneling (Bọ nổ axit cảm tử AoE cực mạnh)
    pub hive_proliferation: bool, // Sinh sôi siêu tốc (-35% chi phí & thời gian tạo lính, +25% tốc chạy)

    // Các nâng cấp kinh tế / phụ trợ
    pub optical_drills: bool,       // +30% tốc độ khai thác
    pub overcharged_reactors: bool, // +50% energy/s
    pub fortified_blueprints: bool, // +30% máu nhà
    pub magnetic_rounds: bool,      // +25% DMG Rifleman
    pub exo_boosters: bool,         // +20% tốc độ chạy lính
    pub superheated_blades: bool,   // Xuyên giáp kiếm sĩ
    pub overload_strike: bool,      // +40% tốc bắn Tháp canh & Laser

    // Danh sách các công nghệ đang nghiên cứu song song
    pub active_researches: Vec<ActiveResearch>,

    // Đang nghiên cứu (giữ tương thích ngược với single research)
    pub current_research: Option<String>,
    pub research_progress: f32, // 0.0 -> 1.0
    pub research_total_time: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameSettings {
    pub sound_volume: f32,
    pub camera_speed: f32,
    pub auto_pause_on_tech: bool,
    pub show_grid: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            sound_volume: 0.7,
            camera_speed: 1.0,
            auto_pause_on_tech: true,
            show_grid: true,
        }
    }
}

pub struct CameraState {
    pub x: f32,
    pub y: f32,
    pub zoom: f32,
    pub viewport_w: f32,
    pub viewport_h: f32,
    pub dpr: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            x: 550.0,
            y: 4250.0, // Nhìn vào căn cứ Player góc Tây-Nam trên bản đồ 4800x4800
            zoom: 1.0,
            viewport_w: 1280.0,
            viewport_h: 720.0,
            dpr: 1.0,
        }
    }
}

impl CameraState {
    /// Trả về (min_x, min_y, max_x, max_y) trong tọa độ thế giới (world space)
    #[inline]
    pub fn viewport_bounds(&self, margin: f32) -> (f32, f32, f32, f32) {
        let half_w = (self.viewport_w * 0.5) / self.zoom + margin;
        let half_h = (self.viewport_h * 0.5) / self.zoom + margin;
        (
            self.x - half_w,
            self.y - half_h,
            self.x + half_w,
            self.y + half_h,
        )
    }

    /// Kiểm tra điểm (wx, wy) có nằm trong viewport hay không
    #[inline]
    pub fn is_point_visible(&self, wx: f32, wy: f32, margin: f32) -> bool {
        let (min_x, min_y, max_x, max_y) = self.viewport_bounds(margin);
        wx >= min_x && wx <= max_x && wy >= min_y && wy <= max_y
    }

    /// Kiểm tra hình chữ nhật AABB có giao với viewport hay không
    #[inline]
    pub fn is_rect_visible(&self, rx: f32, ry: f32, rw: f32, rh: f32, margin: f32) -> bool {
        let (min_x, min_y, max_x, max_y) = self.viewport_bounds(margin);
        rx + rw >= min_x && rx <= max_x && ry + rh >= min_y && ry <= max_y
    }
}
