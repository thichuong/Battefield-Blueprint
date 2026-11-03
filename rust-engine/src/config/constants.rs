// =========================================================================
// HẰNG SỐ CẤU HÌNH HỆ THỐNG (CENTRALIZED CONSTANTS)
// =========================================================================

// === BẢN ĐỒ & CHIẾN TRƯỜNG (MAP & ENVIRONMENT) ===
pub const MAP_WIDTH: f32 = 4800.0;
pub const MAP_HEIGHT: f32 = 4800.0;

// === SƯƠNG MÙ CHIẾN TRANH (FOG OF WAR) ===
pub const FOG_COLS: usize = 96;
pub const FOG_ROWS: usize = 96;
pub const CELL_W: f32 = MAP_WIDTH / (FOG_COLS as f32);
pub const CELL_H: f32 = MAP_HEIGHT / (FOG_ROWS as f32);

// === LƯỚI KHÔNG GIAN TÌM KIẾM ĐƠN VỊ (SPATIAL GRID) ===
pub const CELL_SIZE: f32 = 150.0;
pub const COLS: usize = ((MAP_WIDTH / CELL_SIZE) as usize) + 1; // 33
pub const ROWS: usize = ((MAP_HEIGHT / CELL_SIZE) as usize) + 1; // 33

// === KINH TẾ & CHUYỂN ĐỔI NĂNG LƯỢNG (ECONOMY & CONVERSION) ===
pub const ENERGY_CONVERSION_THRESHOLD: f32 = 1000.0;
pub const CONVERT_ENERGY_PER_SEC: f32 = 30.0;
pub const CONVERT_MINERALS_PER_SEC: f32 = 10.0;

// === TỐC ĐỘ XÂY DỰNG & HIỆU ỨNG BỊ TẤN CÔNG ===
pub const BUILD_SPEED_BASE: f32 = 0.05; // Tốc độ xây tự động cơ bản khi không có thợ (1/20s)
pub const BUILD_SPEED_WORKER: f32 = 0.20; // Tốc độ xây khi có Engineer ở gần (1/5s)
pub const WORKER_BUILD_RANGE: f32 = 90.0; // Khoảng cách thợ hỗ trợ xây dựng
pub const BUILDING_UNDER_ATTACK_DURATION: f32 = 5.0; // Thời gian duy trì hiệu ứng bị tấn công (5s)
pub const BUILDING_UNDER_ATTACK_SPEED_MULTIPLIER: f32 = 0.20; // Giảm còn 30% tốc độ xây khi bị tấn công

// === NGHIÊN CỨU ĐA NHÁNH CÔNG NGHỆ ===
pub const MAX_CONCURRENT_RESEARCH_PER_BRANCH: usize = 1; // 1 công nghệ mỗi nhánh cùng lúc
pub const MAX_TOTAL_CONCURRENT_RESEARCH: usize = 4; // Tối đa 4 nhánh chạy đồng thời

// === KỸ NĂNG ĐƠN VỊ ĐẶC BIỆT (SPECIAL UNIT ABILITIES) ===
pub const VOID_WARLOCK_DEFAULT_MAX_HP_DPS: f32 = 0.10; // 10% máu tối đa mỗi giây
pub const VOID_WARLOCK_CIRCLE_LIFETIME: f32 = 5.0; // Thời gian duy trì vòng ma thuật (5s)
pub const VOID_WARLOCK_CIRCLE_RADIUS: f32 = 55.0; // Bán kính vòng ma thuật (55px)

// === GIAO DIỆN ĐÁY MÀN HÌNH (BOTTOM HUD CANVASES) ===
pub const HUD_MARGIN: f32 = 10.0;
pub const MINIMAP_SIZE: f32 = 220.0;

pub const SELECTION_PANEL_WIDTH: f32 = 480.0;
pub const SELECTION_PANEL_HEIGHT: f32 = 180.0;

pub const COMMAND_CARD_WIDTH: f32 = 285.0;
pub const COMMAND_CARD_HEIGHT: f32 = 180.0;
pub const COMMAND_CARD_PADDING: f32 = 6.0;
pub const COMMAND_CARD_GAP: f32 = 6.0;
pub const COMMAND_CARD_BTN_WIDTH: f32 = 87.0;
pub const COMMAND_CARD_BTN_HEIGHT: f32 = 52.0;
pub const COMMAND_CARD_FORMATION_BTN_WIDTH: f32 = 273.0; // 87.0 * 3 + 6.0 * 2

