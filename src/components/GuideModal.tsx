import React, { useState } from 'react';
import {
  X,
  BookOpen,
  Users,
  Building2,
  Cpu,
  Bot,
  Swords,
  Shield,
  Zap,
  Gem,
  Crosshair,
  Layers,
  Sparkles,
  Flame,
  Compass,
} from 'lucide-react';
import { GameConfigData } from '../bridge/wasmBridge';

interface GuideModalProps {
  isOpen: boolean;
  onClose: () => void;
  gameConfig?: GameConfigData | null;
  autoPause?: boolean;
  isInGame?: boolean;
}

type GuideTab = 'overview' | 'units' | 'buildings' | 'techtree' | 'autoplay' | 'presets';

export const GuideModal: React.FC<GuideModalProps> = ({
  isOpen,
  onClose,
  gameConfig,
  autoPause = true,
  isInGame = false,
}) => {
  const [activeTab, setActiveTab] = useState<GuideTab>('overview');
  const [unitFilter, setUnitFilter] = useState<'all' | 'basic' | 'science' | 'magic' | 'monster'>('all');

  if (!isOpen) return null;

  // Fallback data nếu gameConfig chưa kịp load
  const unitsData = gameConfig?.units || {};
  const buildingsData = gameConfig?.buildings || {};
  const ecoData = gameConfig?.economy || {
    starting_minerals: 200,
    starting_energy: 50,
    starting_supply: 4,
    base_max_supply: 10,
    base_energy_per_sec: 2.0,
    converter_threshold: 1000,
    converter_energy_per_sec: 100,
    converter_minerals_per_100_energy: 50,
  };

  const unitList = [
    {
      id: 'Engineer',
      category: 'basic',
      categoryName: 'Cơ Bản // Kinh Tế',
      fallbackName: 'ENGINEER // WORKER',
      fallbackDesc: 'Huấn luyện thợ mỏ / kỹ sư xây dựng. Thu thập khoáng sản và kiến thiết các công trình căn cứ.',
      hp: 90,
      speed: 120,
      range: 30,
      dmg: 5,
      cd: 1.0,
      dmgType: 'Physical',
      aoe: 0,
      minerals: 50,
      energy: 0,
      time: 5,
      res: { physical: 0.05, energy: 0, explosive: 0, arcane: 0 },
      reqTech: null,
      special: 'Có khả năng khai thác mỏ quặng và thi công xây dựng công trình.',
    },
    {
      id: 'ShockTrooper',
      category: 'basic',
      categoryName: 'Cơ Bản // Tiên Phong',
      fallbackName: 'STRIKER // SHOCK TROOPER',
      fallbackDesc: 'Kiếm sĩ Plasma, chạy nhanh, chém mạnh.',
      hp: 160,
      speed: 140,
      range: 40,
      dmg: 22,
      cd: 0.8,
      dmgType: 'Energy',
      aoe: 0,
      minerals: 75,
      energy: 0,
      time: 7,
      res: { physical: 0.15, energy: 0.2, explosive: 0.05, arcane: 0.1 },
      reqTech: null,
      special: 'Tốc độ di chuyển cao (140), chém nhanh sát thương Năng Lượng.',
    },
    {
      id: 'Rifleman',
      category: 'basic',
      categoryName: 'Cơ Bản // Tầm Xa',
      fallbackName: 'MARKSMAN // RIFLEMAN',
      fallbackDesc: 'Xạ thủ súng trường tầm xa.',
      hp: 120,
      speed: 110,
      range: 160,
      dmg: 14,
      cd: 1.1,
      dmgType: 'Physical',
      aoe: 0,
      minerals: 100,
      energy: 20,
      time: 8,
      res: { physical: 0.1, energy: 0.05, explosive: 0.05, arcane: 0.05 },
      reqTech: null,
      special: 'Tầm bắn cơ bản 160px. Cường hóa thêm tầm bắn qua nâng cấp Đạn Từ Tính.',
    },
    {
      id: 'Vanguard',
      category: 'science',
      categoryName: 'Khoa Học // Phòng Hộ',
      fallbackName: 'VANGUARD // SHIELDER',
      fallbackDesc: 'Vệ binh khiên chắn năng lượng, mỗi 5s nhận khiên 100.',
      hp: 320,
      speed: 90,
      range: 45,
      dmg: 18,
      cd: 1.2,
      dmgType: 'Physical',
      aoe: 0,
      minerals: 150,
      energy: 50,
      time: 12,
      res: { physical: 0.4, energy: 0.5, explosive: 0.4, arcane: 0.3 },
      reqTech: 'Khiên Lượng Tử (Energy Shielding)',
      special: 'Tự động hồi phục lá chắn Khiên Năng Lượng 100 HP mỗi 5 giây.',
    },
    {
      id: 'LaserMarksman',
      category: 'science',
      categoryName: 'Khoa Học // Công Kích',
      fallbackName: 'SIEGE // LASER MARKSMAN',
      fallbackDesc: 'Tia Laser liên tục mỗi 0.2s, sát thương tăng dần theo thời gian.',
      hp: 110,
      speed: 95,
      range: 220,
      dmg: 1,
      cd: 0.2,
      dmgType: 'Energy',
      aoe: 0,
      minerals: 175,
      energy: 100,
      time: 15,
      res: { physical: 0.05, energy: 0.3, explosive: 0, arcane: 0.15 },
      reqTech: 'Tụ Quang Lượng Tử (Focused Laser)',
      special: 'Mỗi đòn bắn trúng tích stack tăng sát thương dồn dập lên cùng mục tiêu.',
    },
    {
      id: 'DarkRocketeer',
      category: 'science',
      categoryName: 'Khoa Học // Hắc Khoa Kỹ',
      fallbackName: 'DARK TECH // ROCKETEER',
      fallbackDesc: 'Bệ phóng Rocket nổ AoE tầm xa.',
      hp: 140,
      speed: 95,
      range: 200,
      dmg: 50,
      cd: 1.5,
      dmgType: 'Explosive',
      aoe: 50,
      minerals: 160,
      energy: 80,
      time: 14,
      res: { physical: 0.2, energy: 0.1, explosive: 0.35, arcane: 0.05 },
      reqTech: 'Bệ Phóng Rocket (Dark Matter Rockets)',
      special: 'Sát thương Nổ AoE bán kính 50px cực mạnh quét sạch cụm lính địch.',
    },
    {
      id: 'TaoistSwordmaster',
      category: 'magic',
      categoryName: 'Ma Thuật // Tiên Đạo',
      fallbackName: 'CELESTIAL // SWORDMASTER',
      fallbackDesc: 'Ngự phi kiếm chân khí Arcane, tích 1 kiếm/s (tối đa 7), tốc đánh 0.1s.',
      hp: 170,
      speed: 130,
      range: 140,
      dmg: 40,
      cd: 0.1,
      dmgType: 'Arcane',
      aoe: 10,
      minerals: 180,
      energy: 90,
      time: 15,
      res: { physical: 0.15, energy: 0.35, explosive: 0.15, arcane: 0.45 },
      reqTech: 'Ngự Kiếm Thuật (Celestial Flying Swords)',
      special: 'Tích trữ tối đa 7 phi kiếm (hồi 1 kiếm/s). Phóng kiếm liên thanh bỏ qua 50% Kháng Vật Lý.',
    },
    {
      id: 'VoidWarlock',
      category: 'magic',
      categoryName: 'Ma Thuật // Hư Không',
      fallbackName: 'VOID // SHADOW WARLOCK',
      fallbackDesc: 'Mỗi 2s triệu hồi vòng ma thuật tồn tại 5s, đốt 10% HP tối đa mỗi giây.',
      hp: 135,
      speed: 100,
      range: 175,
      dmg: 0,
      cd: 2.0,
      dmgType: 'Arcane',
      aoe: 75,
      minerals: 200,
      energy: 120,
      time: 16,
      res: { physical: 0.1, energy: 0.25, explosive: 0.2, arcane: 0.5 },
      reqTech: 'Bộc Phá Hư Không (Void Cataclysm)',
      special: 'Triệu hồi pháp trận bán kính 75px tồn tại 5s, đốt 10% Max HP/giây của tất cả kẻ địch bên trong!',
    },
    {
      id: 'BehemothTitan',
      category: 'monster',
      categoryName: 'Quái Vật // Cự Thú',
      fallbackName: 'BEHEMOTH // TITAN',
      fallbackDesc: 'Cự thú HP khủng, dậm chấn động diện rộng.',
      hp: 650,
      speed: 60,
      range: 55,
      dmg: 40,
      cd: 1.5,
      dmgType: 'Physical',
      aoe: 35,
      minerals: 250,
      energy: 100,
      time: 20,
      res: { physical: 0.5, energy: 0.2, explosive: 0.2, arcane: 0.1 },
      reqTech: 'Biến Dị Khổng Lồ (Behemoth Mutation)',
      special: 'Máu cực trâu (650 HP), Kháng vật lý 50%, đòn đánh lan diện rộng 35px.',
    },
    {
      id: 'AcidBaneling',
      category: 'monster',
      categoryName: 'Quái Vật // Cảm Tử',
      fallbackName: 'SWARM // ACID BANELING',
      fallbackDesc: 'Bọ cảm tử tự nổ axit AoE cực lớn, x2 sát thương lên công trình.',
      hp: 100,
      speed: 160,
      range: 25,
      dmg: 90,
      cd: 0.5,
      dmgType: 'Explosive',
      aoe: 70,
      minerals: 60,
      energy: 30,
      time: 6,
      res: { physical: 0.05, energy: 0.05, explosive: 0.2, arcane: 0.2 },
      reqTech: 'Bọ Độc Cảm Tử (Acid Banelings)',
      special: 'Tự nổ AoE 70px gây 90 sát thương Nổ, ĐẶC BIỆT GẤP ĐÔI SÁT THƯƠNG (180 DMG) lên Công Trình!',
    },
  ];

  const buildingList = [
    {
      id: 'CommandCenter',
      fallbackName: 'HQ COMMAND CENTER',
      fallbackDesc: 'Trung tâm chỉ huy đầu não. Sản xuất ENGINEER // WORKER, tiếp nhận khoáng sản và tự trang bị vũ khí phòng vệ tầm xa.',
      hp: 2000,
      minerals: 400,
      energy: 0,
      time: 15,
      width: 90,
      height: 90,
      supply: 10,
      energyRate: 0,
      weapon: 'Súng phòng thủ 100 DMG, tầm bắn 250px, hồi 1.0s',
      res: { physical: 0.8, energy: 0.6, explosive: 0.25, arcane: 0.2 },
    },
    {
      id: 'SupplyDepot',
      fallbackName: 'SUPPLY DEPOT',
      fallbackDesc: 'Kho cung ứng hậu cần quân đội. Mở rộng giới hạn trần dân số để duy trì quân đoàn đông đảo.',
      hp: 800,
      minerals: 100,
      energy: 0,
      time: 8,
      width: 60,
      height: 60,
      supply: 8,
      energyRate: 0,
      weapon: 'Không trang bị vũ khí',
      res: { physical: 0.2, energy: 0.15, explosive: 0.1, arcane: 0.05 },
    },
    {
      id: 'PowerReactor',
      fallbackName: 'POWER REACTOR',
      fallbackDesc: 'Lò phản ứng năng lượng lượng tử. Liên tục sản sinh Năng lượng (Energy) để phục vụ công nghệ và quân chủng cấp cao.',
      hp: 700,
      minerals: 120,
      energy: 0,
      time: 10,
      width: 60,
      height: 60,
      supply: 0,
      energyRate: 8,
      weapon: 'Không trang bị vũ khí',
      res: { physical: 0.15, energy: 0.25, explosive: 0.05, arcane: 0.15 },
    },
    {
      id: 'Barracks',
      fallbackName: 'BARRACKS',
      fallbackDesc: 'Doanh trại quân đội tối tân. Nơi đào tạo và huấn luyện toàn bộ 9 binh chủng chiến đấu từ bộ binh đến quái thú viễn tưởng.',
      hp: 1200,
      minerals: 150,
      energy: 0,
      time: 12,
      width: 80,
      height: 70,
      supply: 0,
      energyRate: 0,
      weapon: 'Không trang bị vũ khí',
      res: { physical: 0.35, energy: 0.2, explosive: 0.25, arcane: 0.15 },
    },
    {
      id: 'TurretTower',
      fallbackName: 'TURRET TOWER',
      fallbackDesc: 'TURRET TOWER phòng thủ tự động. Tự động truy quét và bắn tỉa các mục tiêu thâm nhập với chỉ số chống chịu kiên cố.',
      hp: 900,
      minerals: 125,
      energy: 50,
      time: 10,
      width: 50,
      height: 50,
      supply: 0,
      energyRate: 0,
      weapon: 'Pháo Năng Lượng 25 DMG, tầm bắn 250px, hồi 1.0s (Nâng cấp bắn AoE)',
      res: { physical: 0.85, energy: 0.6, explosive: 0.35, arcane: 0.3 },
    },
    {
      id: 'EnergyConverter',
      fallbackName: 'ENERGY CONVERTER',
      fallbackDesc: 'Trạm chuyển hóa vật chất năng lượng. Khi năng lượng tích lũy vượt quá 1000⚡, tự động chuyển đổi sang Khoáng sản Vàng💎.',
      hp: 850,
      minerals: 150,
      energy: 50,
      time: 12,
      width: 65,
      height: 65,
      supply: 0,
      energyRate: 0,
      weapon: 'Không trang bị vũ khí',
      res: { physical: 0.25, energy: 0.35, explosive: 0.1, arcane: 0.25 },
    },
  ];

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/85 backdrop-blur-md p-3 md:p-6 font-mono text-slate-200">
      <div className="relative bg-[#030d1a] border-2 border-cyan-500/70 rounded-2xl w-[96vw] max-w-6xl h-[90vh] min-h-[680px] max-h-[920px] shadow-[0_0_50px_rgba(0,229,255,0.25)] flex flex-col overflow-hidden">
        {/* Header Modal */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-cyan-900/60 bg-slate-950/80 shrink-0">
          <div className="flex items-center gap-3">
            <div className="p-2.5 rounded-xl bg-cyan-950/90 border border-cyan-500/80 text-cyan-400 shadow-sm">
              <BookOpen className="w-6 h-6" />
            </div>
            <div>
              <h2 className="text-base md:text-xl font-black text-white tracking-wider flex items-center gap-2">
                BATTLEFIELD BLUEPRINT <span className="text-cyan-400">// CẨM NANG CHIẾN THUẬT</span>
              </h2>
              <p className="text-xs md:text-sm text-slate-400 mt-0.5">
                Hướng dẫn luật chơi, thông số đơn vị, công nghệ và tự động hóa theo thời gian thực
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-2 rounded-lg bg-slate-900/80 border border-slate-700 hover:border-cyan-400 text-slate-400 hover:text-white transition-all cursor-pointer"
          >
            <X className="w-6 h-6" />
          </button>
        </div>

        {/* Thanh điều hướng Buttons (Tabs) */}
        <div className="px-6 py-3 bg-slate-950/60 border-b border-slate-800 flex items-center gap-2.5 overflow-x-auto shrink-0 select-none">
          {[
            { id: 'overview', label: 'TỔNG QUAN & ĐIỀU KHIỂN', icon: Compass },
            { id: 'units', label: 'ĐƠN VỊ BINH CHỦNG', icon: Users },
            { id: 'buildings', label: 'CÔNG TRÌNH CĂN CỨ', icon: Building2 },
            { id: 'techtree', label: 'CÂY CÔNG NGHỆ [T]', icon: Cpu },
            { id: 'autoplay', label: 'TỰ ĐỘNG HÓA AUTO [B]', icon: Bot },
            { id: 'presets', label: 'ĐA PHE & 8 NGƯỜI CHƠI', icon: Swords },
          ].map((tab) => {
            const Icon = tab.icon;
            const isActive = activeTab === tab.id;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id as GuideTab)}
                className={`flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs md:text-sm font-bold transition-all shrink-0 cursor-pointer border ${
                  isActive
                    ? 'bg-cyan-950 border-cyan-400 text-cyan-300 shadow-[0_0_12px_rgba(0,229,255,0.3)]'
                    : 'bg-slate-900/50 border-slate-800 text-slate-400 hover:text-slate-200 hover:border-slate-700'
                }`}
              >
                <Icon className={`w-4 h-4 md:w-5 md:h-5 ${isActive ? 'text-cyan-400' : 'text-slate-500'}`} />
                <span>{tab.label}</span>
              </button>
            );
          })}
        </div>

        {/* Nội dung Tab */}
        <div className="flex-1 overflow-y-auto p-6 space-y-6 text-sm">
          {/* TAB 1: TỔNG QUAN & ĐIỀU KHIỂN */}
          {activeTab === 'overview' && (
            <div className="space-y-6">
              {/* Box Giới thiệu */}
              <div className="p-4 rounded-xl border border-cyan-800/60 bg-cyan-950/20">
                <h3 className="text-sm md:text-base font-bold text-cyan-300 mb-2 flex items-center gap-2">
                  <Sparkles className="w-5 h-5 text-cyan-400" /> MỤC TIÊU CHIẾN DỊCH:
                </h3>
                <p className="text-xs md:text-sm text-slate-200 leading-relaxed">
                  Bạn là Chỉ Huy tối cao của một tiền đồn quân sự. Mục tiêu của bạn là khai thác mỏ tài nguyên,
                  sản sinh năng lượng, củng cố hàng rào phòng ngự, nghiên cứu các công nghệ tiến hóa và chỉ huy
                  đội quân quét sạch mọi công trình, căn cứ của các phe đối địch trên bản đồ!
                </p>
              </div>

              {/* Hệ thống Tài nguyên & Kinh tế */}
              <div>
                <h3 className="text-xs md:text-sm font-bold text-slate-200 uppercase tracking-wider mb-3 flex items-center gap-2">
                  <Gem className="w-4 h-4 text-cyan-400" /> Hệ Thống Kinh Tế Chiến Trường:
                </h3>
                <div className="grid grid-cols-1 md:grid-cols-3 gap-3.5">
                  <div className="p-4 rounded-xl border border-sky-800/60 bg-sky-950/25">
                    <div className="flex items-center gap-2 text-cyan-400 font-bold mb-1.5 text-xs md:text-sm">
                      <span>💎 KHOÁNG SẢN (MINERALS)</span>
                    </div>
                    <p className="text-xs md:text-sm text-slate-300 leading-relaxed">
                      Khai thác từ các mỏ quặng xanh dương bằng ENGINEER // WORKER. Tài nguyên thiết yếu để xây dựng
                      công trình và tuyển mộ quân đội.
                    </p>
                  </div>

                  <div className="p-4 rounded-xl border border-purple-800/60 bg-purple-950/25">
                    <div className="flex items-center gap-2 text-purple-400 font-bold mb-1.5 text-xs md:text-sm">
                      <span>⚡ NĂNG LƯỢNG (ENERGY)</span>
                    </div>
                    <p className="text-xs md:text-sm text-slate-300 leading-relaxed">
                      Sản sinh liên tục qua các Lò Phản Ứng (POWER REACTOR). Dùng để nghiên cứu công nghệ, đào tạo
                      binh chủng cao cấp và nạp đạn cho TURRET TOWER.
                    </p>
                  </div>

                  <div className="p-4 rounded-xl border border-amber-800/60 bg-amber-950/25">
                    <div className="flex items-center gap-2 text-amber-400 font-bold mb-1.5 text-xs md:text-sm">
                      <span>👥 DÂN SỐ (SUPPLY)</span>
                    </div>
                    <p className="text-xs md:text-sm text-slate-300 leading-relaxed">
                      Giới hạn quân số hiện tại. Khởi đầu là {ecoData.starting_supply}/{ecoData.base_max_supply}. Xây thêm SUPPLY DEPOT (+8) hoặc HQ COMMAND CENTER (+10) để nâng trần dân số.
                    </p>
                  </div>
                </div>

                <div className="mt-3.5 p-3.5 rounded-xl border border-amber-500/40 bg-amber-950/20 text-xs md:text-sm text-amber-200 flex items-start gap-3">
                  <Zap className="w-5 h-5 text-amber-400 shrink-0 mt-0.5" />
                  <div className="leading-relaxed">
                    <span className="font-bold text-amber-300">Cơ chế Trạm Chuyển Hóa (ENERGY CONVERTER): </span>
                    Khi tích lũy Năng lượng vượt ngưỡng{' '}
                    <span className="text-cyan-300 font-bold">{ecoData.converter_threshold}⚡</span>, trạm sẽ tự động
                    đổi năng lượng dôi dư thành Khoáng sản Vàng theo tỉ lệ{' '}
                    <span className="text-emerald-400 font-bold">100⚡ ➔ {ecoData.converter_minerals_per_100_energy}💎</span>!
                  </div>
                </div>
              </div>

              {/* Bảng Phím tắt & Điều khiển */}
              <div>
                <h3 className="text-xs md:text-sm font-bold text-slate-200 uppercase tracking-wider mb-3 flex items-center gap-2">
                  <Crosshair className="w-4 h-4 text-cyan-400" /> Bảng Thao Tác Chuột & Phím Tắt:
                </h3>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-2.5 text-xs md:text-sm">
                  {[
                    { key: 'Kéo Chuột Trái', desc: 'Quét hộp (Box Selection) để chọn đồng loạt nhiều đơn vị quân.' },
                    { key: 'Chuột Phải', desc: 'Di chuyển / Tấn công mục tiêu / Gán thợ mỏ đào quặng / Đặt điểm tập kết Rally.' },
                    { key: 'Chuột Phải (Command Card)', desc: 'Hủy bớt đơn vị trong hàng đợi đang chờ và hoàn 100% tài nguyên.' },
                    { key: 'Phím A + Chuột Trái', desc: 'Chế độ Tấn công khi di chuyển (Attack-Move) - tự động giao tranh kẻ địch gặp trên đường.' },
                    { key: 'Phím F', desc: 'Chuyển đổi luân phiên 6 Đội hình chiến thuật (Line, Wedge, Box, Scatter, Circle, Column).' },
                    { key: 'Phím T', desc: 'Bật / Tắt sơ đồ Cây Công Nghệ (Tech Tree).' },
                    { key: 'Phím B', desc: 'Bật / Tắt bảng điều khiển Chế độ Tự Động (Auto-Play).' },
                    { key: 'Phím H / Click HUD', desc: 'Bật / Tắt bảng Cẩm Nang Hướng Dẫn Chơi Game này.' },
                    { key: 'Phím Space', desc: 'Tạm dừng (Pause) hoặc Tiếp tục diễn biến trận đấu.' },
                    { key: 'Phím ESC', desc: 'Mở Bảng Cài Đặt Game / Đóng cửa sổ đang mở / Hủy lệnh xây dựng.' },
                    { key: 'Phím F3 hoặc ~', desc: 'Bật / Tắt bảng giám sát hiệu năng thực tế (FPS, Sim Time ms, Render Time ms).' },
                    { key: 'Cuộn Chuột', desc: 'Phóng to / Thu nhỏ góc nhìn chiến trường (Zoom 0.5x - 2.0x).' },
                  ].map((item, idx) => (
                    <div key={idx} className="p-3 rounded-xl border border-slate-800 bg-slate-900/50 flex items-start gap-3">
                      <span className="px-2.5 py-1 rounded-lg bg-cyan-950 border border-cyan-800 text-cyan-300 font-bold shrink-0 text-xs md:text-sm">
                        {item.key}
                      </span>
                      <span className="text-slate-200 text-xs md:text-sm leading-snug">{item.desc}</span>
                    </div>
                  ))}
                </div>
              </div>

              {/* Bảng Tương khắc Sát thương & Kháng */}
              <div>
                <h3 className="text-xs md:text-sm font-bold text-slate-200 uppercase tracking-wider mb-3 flex items-center gap-2">
                  <Shield className="w-4 h-4 text-cyan-400" /> Cơ Chế 4 Hệ Sát Thương & Kháng (Resistances):
                </h3>
                <div className="grid grid-cols-2 md:grid-cols-4 gap-3 text-xs md:text-sm">
                  <div className="p-3.5 rounded-xl border border-slate-700 bg-slate-900/60">
                    <span className="text-amber-400 font-bold block mb-1 text-xs md:text-sm">⚔️ VẬT LÝ (Physical)</span>
                    <p className="text-xs text-slate-300 leading-relaxed">
                      Đòn đánh cơ bản từ súng trường, TURRET TOWER và cự thú. Bị giảm trừ trực tiếp bởi Kháng Vật Lý.
                    </p>
                  </div>
                  <div className="p-3.5 rounded-xl border border-cyan-700 bg-cyan-950/40">
                    <span className="text-cyan-400 font-bold block mb-1 text-xs md:text-sm">⚡ NĂNG LƯỢNG (Energy)</span>
                    <p className="text-xs text-slate-300 leading-relaxed">
                      Chùm tia plasma và laser xuyên thấu. Gây áp lực cực lớn lên các mục tiêu có giáp vật lý dày.
                    </p>
                  </div>
                  <div className="p-3.5 rounded-xl border border-orange-700 bg-orange-950/40">
                    <span className="text-orange-400 font-bold block mb-1 text-xs md:text-sm">💥 NỔ AOE (Explosive)</span>
                    <p className="text-xs text-slate-300 leading-relaxed">
                      Rocket và bọ nổ axit. Sát thương lan diện rộng, đặc biệt bọ nổ gây x2 sát thương lên nhà cửa!
                    </p>
                  </div>
                  <div className="p-3.5 rounded-xl border border-purple-700 bg-purple-950/40">
                    <span className="text-purple-400 font-bold block mb-1 text-xs md:text-sm">✨ MA THUẬT (Arcane)</span>
                    <p className="text-xs text-slate-300 leading-relaxed">
                      Phi kiếm và pháp trận hư không. Đặc tính: <span className="text-cyan-300 font-bold">Bỏ qua 50% Kháng Vật Lý</span> của nạn nhân!
                    </p>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* TAB 2: ĐƠN VỊ BINH CHỦNG */}
          {activeTab === 'units' && (
            <div className="space-y-4">
              {/* Bộ lọc đơn vị */}
              <div className="flex items-center gap-2.5 flex-wrap pb-3 border-b border-slate-800">
                <span className="text-xs md:text-sm text-slate-300 font-bold uppercase mr-1">Bộ lọc nhánh:</span>
                {[
                  { id: 'all', label: 'Tất cả (10)' },
                  { id: 'basic', label: 'Cơ bản (3)' },
                  { id: 'science', label: 'Khoa học (3)' },
                  { id: 'magic', label: 'Ma thuật (2)' },
                  { id: 'monster', label: 'Quái vật (2)' },
                ].map((f) => (
                  <button
                    key={f.id}
                    onClick={() => setUnitFilter(f.id as any)}
                    className={`px-3.5 py-1.5 rounded-xl text-xs md:text-sm font-bold transition-all cursor-pointer border ${
                      unitFilter === f.id
                        ? 'bg-cyan-950 border-cyan-400 text-cyan-300 shadow-[0_0_8px_rgba(0,229,255,0.3)]'
                        : 'bg-slate-900 border-slate-800 text-slate-400 hover:text-slate-200'
                    }`}
                  >
                    {f.label}
                  </button>
                ))}
              </div>

              {/* Grid 10 đơn vị */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {unitList
                  .filter((u) => unitFilter === 'all' || u.category === unitFilter)
                  .map((u) => {
                    const cfg = unitsData[u.id];
                    const name = cfg?.name || u.fallbackName;
                    const desc = cfg?.description || u.fallbackDesc;
                    const hp = cfg?.hp ?? u.hp;
                    const speed = cfg?.speed ?? u.speed;
                    const range = cfg?.attack_range ?? u.range;
                    const dmg = cfg?.attack_damage ?? u.dmg;
                    const cd = cfg?.attack_cooldown ?? u.cd;
                    const minCost = cfg?.cost_minerals ?? u.minerals;
                    const enCost = cfg?.cost_energy ?? u.energy;
                    const trainTime = cfg?.training_time ?? u.time;
                    const res = cfg?.resistances ?? u.res;

                    return (
                      <div
                        key={u.id}
                        className="p-4 rounded-xl border border-slate-800 bg-slate-950/70 hover:border-cyan-500/50 transition-all flex flex-col justify-between shadow-sm"
                      >
                        <div>
                          <div className="flex items-center justify-between mb-2">
                            <span className="text-xs font-bold text-cyan-400 uppercase tracking-wider">
                              {u.categoryName}
                            </span>
                            <span className="text-xs md:text-sm font-mono text-amber-300 font-bold bg-amber-950/40 border border-amber-900/60 px-2.5 py-0.5 rounded-lg">
                              {minCost}💎 {enCost > 0 ? `${enCost}⚡` : ''} • {trainTime}s
                            </span>
                          </div>

                          <h4 className="text-base md:text-lg font-black text-white mb-1.5">{name}</h4>
                          <p className="text-xs md:text-sm text-slate-200 mb-2.5 leading-relaxed">{desc}</p>

                          {/* Đặc tính độc quyền */}
                          <div className="text-xs md:text-sm text-cyan-200 bg-cyan-950/50 p-2.5 rounded-lg border border-cyan-800/60 mb-3.5 leading-relaxed">
                            <span className="font-bold text-cyan-300">⚡ Đặc tính: </span>
                            {u.special}
                          </div>
                        </div>

                        <div>
                          {/* Chỉ số chiến đấu */}
                          <div className="grid grid-cols-4 gap-2 text-xs md:text-sm font-mono text-center mb-3 bg-slate-900/90 p-2.5 rounded-lg border border-slate-800">
                            <div>
                              <span className="text-slate-400 block text-xs font-sans font-bold">MÁU</span>
                              <span className="text-emerald-400 font-black">{hp} HP</span>
                            </div>
                            <div>
                              <span className="text-slate-400 block text-xs font-sans font-bold">SÁT THƯƠNG</span>
                              <span className="text-rose-400 font-black">
                                {u.id === 'VoidWarlock' ? '10%/s' : dmg}
                              </span>
                            </div>
                            <div>
                              <span className="text-slate-400 block text-xs font-sans font-bold">TẦM / HỒI</span>
                              <span className="text-sky-300 font-black">{range}px / {cd}s</span>
                            </div>
                            <div>
                              <span className="text-slate-400 block text-xs font-sans font-bold">TỐC ĐỘ</span>
                              <span className="text-amber-300 font-black">{speed}</span>
                            </div>
                          </div>

                          {/* Kháng 4 hệ */}
                          <div className="flex items-center justify-between text-xs font-mono text-slate-300 border-t border-slate-800/80 pt-2.5">
                            <span className="text-slate-400 font-bold">Kháng:</span>
                            <span className="text-amber-400">Vật lý: {Math.round(res.physical * 100)}%</span>
                            <span className="text-cyan-400">N.Lượng: {Math.round(res.energy * 100)}%</span>
                            <span className="text-orange-400">Nổ: {Math.round(res.explosive * 100)}%</span>
                            <span className="text-purple-400">M.Thuật: {Math.round(res.arcane * 100)}%</span>
                          </div>

                          {u.reqTech && (
                            <div className="text-xs text-amber-300 mt-2 font-mono">
                              * Yêu cầu công nghệ: <span className="font-bold underline">{u.reqTech}</span>
                            </div>
                          )}
                        </div>
                      </div>
                    );
                  })}
              </div>
            </div>
          )}

          {/* TAB 3: CÔNG TRÌNH CĂN CỨ */}
          {activeTab === 'buildings' && (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {buildingList.map((b) => {
                const cfg = buildingsData[b.id];
                const name = cfg?.name || b.fallbackName;
                const desc = cfg?.description || b.fallbackDesc;
                const hp = cfg?.hp ?? b.hp;
                const minCost = cfg?.cost_minerals ?? b.minerals;
                const enCost = cfg?.cost_energy ?? b.energy;
                const buildTime = cfg?.build_time ?? b.time;
                const res = cfg?.resistances ?? b.res;

                return (
                  <div
                    key={b.id}
                    className="p-4 rounded-xl border border-slate-800 bg-slate-950/70 hover:border-cyan-500/50 transition-all flex flex-col justify-between shadow-sm"
                  >
                    <div>
                      <div className="flex items-center justify-between mb-2">
                        <span className="text-base md:text-lg font-black text-cyan-400">{name}</span>
                        <span className="text-xs md:text-sm font-mono text-amber-300 font-bold bg-amber-950/40 border border-amber-900/60 px-2.5 py-0.5 rounded-lg">
                          {minCost}💎 {enCost > 0 ? `${enCost}⚡` : ''} • {buildTime}s
                        </span>
                      </div>
                      <p className="text-xs md:text-sm text-slate-200 mb-3.5 leading-relaxed">{desc}</p>
                    </div>

                    <div>
                      <div className="p-3 rounded-lg bg-slate-900/90 border border-slate-800 text-xs md:text-sm space-y-2 mb-3.5 font-mono">
                        <div className="flex items-center justify-between">
                          <span className="text-slate-400">Độ bền kết cấu:</span>
                          <span className="text-emerald-400 font-black">{hp} HP</span>
                        </div>
                        <div className="flex items-center justify-between">
                          <span className="text-slate-400">Kích thước móng:</span>
                          <span className="text-slate-200">{b.width} x {b.height} px</span>
                        </div>
                        {b.supply > 0 && (
                          <div className="flex items-center justify-between">
                            <span className="text-slate-400">Hỗ trợ dân số:</span>
                            <span className="text-cyan-400 font-bold">+{b.supply} Supply</span>
                          </div>
                        )}
                        {b.energyRate > 0 && (
                          <div className="flex items-center justify-between">
                            <span className="text-slate-400">Sản lượng năng lượng:</span>
                            <span className="text-purple-400 font-bold">+{b.energyRate}⚡ / giây</span>
                          </div>
                        )}
                        <div className="flex items-center justify-between">
                          <span className="text-slate-400">Hệ thống vũ khí:</span>
                          <span className="text-rose-300 font-semibold">{b.weapon}</span>
                        </div>
                      </div>

                      {/* Kháng 4 hệ */}
                      <div className="flex items-center justify-between text-xs font-mono text-slate-300 border-t border-slate-800/80 pt-2.5">
                        <span className="text-slate-400 font-bold">Chỉ số giáp:</span>
                        <span className="text-amber-400">Vật lý: {Math.round(res.physical * 100)}%</span>
                        <span className="text-cyan-400">N.Lượng: {Math.round(res.energy * 100)}%</span>
                        <span className="text-orange-400">Nổ: {Math.round(res.explosive * 100)}%</span>
                        <span className="text-purple-400">M.Thuật: {Math.round(res.arcane * 100)}%</span>
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          )}

          {/* TAB 4: CÂY CÔNG NGHỆ */}
          {activeTab === 'techtree' && (
            <div className="space-y-6">
              <div className="p-4 rounded-xl border border-sky-800/60 bg-sky-950/25 text-xs md:text-sm text-sky-200 leading-relaxed">
                <span className="font-bold text-sky-300">💡 Cơ Chế Nghiên Cứu Đa Nhánh Đồng Thời: </span>
                Cây công nghệ chia làm 4 nhánh độc lập. Bạn hoàn toàn có thể nghiên cứu song song nhiều công nghệ
                cùng lúc nếu chúng thuộc các nhánh khác nhau (ví dụ: vừa nghiên cứu Khoa Học vừa nghiên cứu Ma Thuật)!
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {/* Nhánh 1: Khoa học */}
                <div className="p-4 rounded-xl border border-cyan-800/60 bg-cyan-950/20">
                  <h4 className="text-sm md:text-base font-bold text-cyan-300 mb-3 flex items-center gap-2">
                    <Zap className="w-5 h-5 text-cyan-400" /> 1. NHÁNH KHOA HỌC (SCIENCE)
                  </h4>
                  <ul className="space-y-2.5 text-xs md:text-sm text-slate-200">
                    <li className="p-2.5 rounded-lg bg-slate-900/70 border border-slate-800 leading-relaxed">
                      <span className="font-bold text-cyan-400">Tụ Quang Lượng Tử (Focused Laser):</span> Mở khóa SIEGE // LASER MARKSMAN & tăng +25% sát thương Energy toàn quân.
                    </li>
                    <li className="p-2.5 rounded-lg bg-slate-900/70 border border-slate-800 leading-relaxed">
                      <span className="font-bold text-cyan-400">Khiên Lượng Tử (Energy Shielding):</span> Mở khóa VANGUARD // SHIELDER & tăng +40% Kháng Năng Lượng toàn quân.
                    </li>
                    <li className="p-2.5 rounded-lg bg-slate-900/70 border border-slate-800 leading-relaxed">
                      <span className="font-bold text-cyan-400">Bệ Phóng Rocket (Dark Matter Rockets):</span> Mở khóa DARK TECH // ROCKETEER & đòn bắn TURRET TOWER chuyển sang Nổ AoE 40px.
                    </li>
                    <li className="p-2.5 rounded-lg bg-slate-900/70 border border-slate-800 leading-relaxed">
                      <span className="font-bold text-cyan-400">Khung Titan (Titan Alloy Plating):</span> Tăng +35% Kháng Vật Lý toàn quân và +30% Máu toàn bộ Công Trình.
                    </li>
                  </ul>
                </div>

                {/* Nhánh 2: Ma thuật */}
                <div className="p-4 rounded-xl border border-purple-800/60 bg-purple-950/20">
                  <h4 className="text-sm md:text-base font-bold text-purple-300 mb-3 flex items-center gap-2">
                    <Sparkles className="w-5 h-5 text-purple-400" /> 2. NHÁNH MA THUẬT (MAGIC)
                  </h4>
                  <ul className="space-y-2.5 text-xs md:text-sm text-slate-200">
                    <li className="p-2.5 rounded-lg bg-slate-900/70 border border-slate-800 leading-relaxed">
                      <span className="font-bold text-purple-400">Ngự Kiếm Thuật (Celestial Swords):</span> Mở khóa CELESTIAL // SWORDMASTER (7 phi kiếm Arcane xuyên giáp).
                    </li>
                    <li className="p-2.5 rounded-lg bg-slate-900/70 border border-slate-800 leading-relaxed">
                      <span className="font-bold text-purple-400">Kim Cang Hộ Thể (Golden Bell Shield):</span> Tăng +40% Kháng Ma Thuật/Năng Lượng và tự hồi phục +3 HP/giây.
                    </li>
                    <li className="p-2.5 rounded-lg bg-slate-900/70 border border-slate-800 leading-relaxed">
                      <span className="font-bold text-purple-400">Chú Ấn Ăn Mòn (Corrosive Curse):</span> Đòn đánh toàn quân làm tan chảy vĩnh viễn mọi chỉ số Kháng của kẻ địch.
                    </li>
                    <li className="p-2.5 rounded-lg bg-slate-900/70 border border-slate-800 leading-relaxed">
                      <span className="font-bold text-purple-400">Bộc Phá Hư Không (Void Cataclysm):</span> Mở khóa VOID // SHADOW WARLOCK (Triệu hồi pháp trận đốt 10% Max HP/giây).
                    </li>
                  </ul>
                </div>

                {/* Nhánh 3: Quái vật */}
                <div className="p-4 rounded-xl border border-emerald-800/60 bg-emerald-950/20">
                  <h4 className="text-sm md:text-base font-bold text-emerald-300 mb-3 flex items-center gap-2">
                    <Flame className="w-5 h-5 text-emerald-400" /> 3. NHÁNH QUÁI VẬT (MONSTER)
                  </h4>
                  <ul className="space-y-2.5 text-xs md:text-sm text-slate-200">
                    <li className="p-2.5 rounded-lg bg-slate-900/70 border border-slate-800 leading-relaxed">
                      <span className="font-bold text-emerald-400">Vảy Cổ Đại (Primeval Scales):</span> Tăng +35% Kháng Vật Lý và +20% Lượng Máu tối đa của toàn quân.
                    </li>
                    <li className="p-2.5 rounded-lg bg-slate-900/70 border border-slate-800 leading-relaxed">
                      <span className="font-bold text-emerald-400">Biến Dị Khổng Lồ (Behemoth Mutation):</span> Mở khóa BEHEMOTH // TITAN (650 HP, dậm chấn động diện rộng).
                    </li>
                    <li className="p-2.5 rounded-lg bg-slate-900/70 border border-slate-800 leading-relaxed">
                      <span className="font-bold text-emerald-400">Bọ Độc Axit (Acid Banelings):</span> Mở khóa SWARM // ACID BANELING (Tự nổ AoE 70px x2 sát thương công trình).
                    </li>
                    <li className="p-2.5 rounded-lg bg-slate-900/70 border border-slate-800 leading-relaxed">
                      <span className="font-bold text-emerald-400">Bầy Đàn Phân Hạch (Hive Proliferation):</span> Giảm -35% Chi Phí và Thời Gian sản xuất toàn bộ binh chủng!
                    </li>
                  </ul>
                </div>

                {/* Nhánh 4: Kinh tế & Phụ trợ */}
                <div className="p-4 rounded-xl border border-amber-800/60 bg-amber-950/20">
                  <h4 className="text-sm md:text-base font-bold text-amber-300 mb-3 flex items-center gap-2">
                    <Gem className="w-5 h-5 text-amber-400" /> 4. KINH TẾ & PHỤ TRỢ (LOGISTICS)
                  </h4>
                  <ul className="space-y-2.5 text-xs md:text-sm text-slate-200">
                    <li className="p-2 rounded-lg bg-slate-900/70 border border-slate-800 leading-relaxed">
                      <span className="font-bold text-amber-400">Khoan Quang Học (Optical Drills):</span> ENGINEER // WORKER đào nhanh hơn và tăng sức chứa quặng (10 ➔ 15).
                    </li>
                    <li className="p-2 rounded-lg bg-slate-900/70 border border-slate-800 leading-relaxed">
                      <span className="font-bold text-amber-400">Lò Phản Ứng Siêu Nạp (Overcharged Reactors):</span> Tăng +50% sản lượng năng lượng của POWER REACTOR.
                    </li>
                    <li className="p-2 rounded-lg bg-slate-900/70 border border-slate-800 leading-relaxed">
                      <span className="font-bold text-amber-400">Bản Vẽ Gia Cố (Fortified Blueprints):</span> Tăng +30% Máu công trình và tốc độ thi công.
                    </li>
                    <li className="p-2 rounded-lg bg-slate-900/70 border border-slate-800 leading-relaxed">
                      <span className="font-bold text-amber-400">Đạn Từ Tính (Magnetic Rounds):</span> Tăng tầm bắn cho MARKSMAN // RIFLEMAN và TURRET TOWER.
                    </li>
                    <li className="p-2 rounded-lg bg-slate-900/70 border border-slate-800 leading-relaxed">
                      <span className="font-bold text-amber-400">Động Cơ Tăng Tốc (Exo Boosters):</span> Toàn quân di chuyển nhanh hơn +20%.
                    </li>
                    <li className="p-2 rounded-lg bg-slate-900/70 border border-slate-800 leading-relaxed">
                      <span className="font-bold text-amber-400">Kích Nạp Quá Tải (Overload Strike):</span> Tăng +66% tốc độ bắn cho TURRET TOWER & HQ COMMAND CENTER.
                    </li>
                  </ul>
                </div>
              </div>
            </div>
          )}

          {/* TAB 5: TỰ ĐỘNG HÓA AUTO-PLAY */}
          {activeTab === 'autoplay' && (
            <div className="space-y-5">
              <div className="p-4 rounded-xl border border-emerald-800/60 bg-emerald-950/20">
                <h4 className="text-sm md:text-base font-bold text-emerald-300 mb-2 flex items-center gap-2">
                  <Bot className="w-5 h-5 text-emerald-400" /> CƠ CHẾ LẬP TRÌNH TỰ ĐỘNG (AUTO-PLAY BOT):
                </h4>
                <p className="text-xs md:text-sm text-slate-200 leading-relaxed">
                  Hệ thống Auto-Play cho phép bạn tự thiết lập chiến lược vĩ mô để máy tự vận hành thay cho các thao tác lặp lại.
                  Bao gồm 2 vòng lặp cốt lõi: <span className="text-cyan-300 font-bold">Vòng Lặp Sản Xuất</span> (kinh tế, nhà cửa, lính, công nghệ)
                  và <span className="text-rose-400 font-bold">Vòng Lặp Quân Sự</span> (tự gom quân và mở đợt tổng tấn công khi đạt đủ lực lượng).
                </p>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="p-4 rounded-xl border border-slate-800 bg-slate-950/60">
                  <h5 className="text-xs md:text-sm font-bold text-cyan-300 uppercase mb-2 flex items-center gap-2">
                    <Layers className="w-4 h-4 text-cyan-400" /> 1. Thẻ Nhiệm Vụ (Action Cards):
                  </h5>
                  <p className="text-xs md:text-sm text-slate-200 leading-relaxed mb-3">
                    Mỗi thẻ nhiệm vụ hoạt động theo nguyên lý: <span className="text-amber-300 font-bold">ĐIỀU KIỆN (If) ➔ HÀNH ĐỘNG (Then)</span>.
                  </p>
                  <ul className="text-xs md:text-sm text-slate-300 space-y-2 list-disc pl-4 leading-relaxed">
                    <li>Nếu số lượng ENGINEER // WORKER &lt; 7 ➔ Huấn luyện thêm ENGINEER // WORKER.</li>
                    <li>Nếu sắp kẹt dân số (Supply Near Cap) ➔ Xây SUPPLY DEPOT.</li>
                    <li>Nếu Khoáng sản &gt; 150 ➔ Huấn luyện binh lính tấn công.</li>
                    <li>Nếu Quân đoàn chiến đấu &gt; 40 đơn vị ➔ Phát động hành quân tổng tiến công!</li>
                  </ul>
                </div>

                <div className="p-4 rounded-xl border border-slate-800 bg-slate-950/60">
                  <h5 className="text-xs md:text-sm font-bold text-amber-300 uppercase mb-2 flex items-center gap-2">
                    <Shield className="w-4 h-4 text-amber-400" /> 2. Tuyến Phòng Thủ TURRET TOWER (Defense Line):
                  </h5>
                  <p className="text-xs md:text-sm text-slate-200 leading-relaxed mb-3">
                    Trong bảng Auto-Play, bạn có thể thiết lập tọa độ đường kẻ nối 2 điểm A và B bảo vệ lối vào căn cứ.
                  </p>
                  <p className="text-xs md:text-sm text-slate-300 leading-relaxed">
                    Bot sẽ tự động tính toán khoảng cách và xây dựng hàng rào TURRET TOWER trải dài dọc tuyến rào chắn này,
                    ngăn chặn hữu hiệu các đợt càn quét bất ngờ từ AI đối thủ!
                  </p>
                </div>
              </div>

              <div className="p-4 rounded-xl border border-slate-800 bg-slate-900/50 text-xs md:text-sm text-slate-200 leading-relaxed">
                <span className="font-bold text-cyan-400">⭐ Hàng Đợi Công Nghệ Ưu Tiên [★]: </span>
                Khi xem Cây Công Nghệ [T], bạn có thể đánh dấu sao vào các công nghệ quan trọng (ví dụ: Laser, Tên lửa, Phi kiếm).
                Thẻ Auto "Nâng cấp Cây Công Nghệ" sẽ ưu tiên chi tiền nghiên cứu chính xác các công nghệ được đánh dấu này trước tiên!
              </div>
            </div>
          )}

          {/* TAB 6: ĐA PHE & 8 NGƯỜI CHƠI */}
          {activeTab === 'presets' && (
            <div className="space-y-5">
              <div className="p-4 rounded-xl border border-cyan-800/60 bg-cyan-950/20">
                <h4 className="text-sm md:text-base font-bold text-cyan-300 mb-2 flex items-center gap-2">
                  <Swords className="w-5 h-5 text-cyan-400" /> CHIẾN TRƯỜNG ĐA PHE TỐI ĐA 8 NGƯỜI CHƠI:
                </h4>
                <p className="text-xs md:text-sm text-slate-200 leading-relaxed">
                  Bản đồ mở rộng quy mô lớn <span className="text-cyan-300 font-bold">4800 x 4800</span> chia đều 8 vị trí xuất phát.
                  Bạn có thể linh hoạt thiết lập các kịch bản đối kháng hoặc chia phe tùy ý ngay tại Menu chính trước khi xuất quân!
                </p>
              </div>

              <div className="grid grid-cols-2 md:grid-cols-3 gap-3.5 text-xs md:text-sm">
                {[
                  { label: '1 vs 1 (Đối đầu)', desc: 'Chỉ Huy vs 1 AI Địch ngẫu nhiên.' },
                  { label: '2 vs 2 (Song đấu)', desc: 'Bạn + 1 AI Đồng Minh đấu 2 AI Địch.' },
                  { label: '3 vs 3 (Chiến đoàn)', desc: 'Liên quân 3 người giáp chiến 3 kẻ thù.' },
                  { label: '4 vs 4 (Đại chiến 8P)', desc: 'Đại chiến quy mô tối đa 8 người chia làm 2 phe lớn.' },
                  { label: '2v2v2 (Tam giác phe)', desc: 'Chiến tranh 3 thế lực chia đều 3 góc bàn cờ.' },
                  { label: '8-FFA (Hỗn chiến 8 phe)', desc: 'Tất cả 8 người chơi đều là đối thủ của nhau!' },
                ].map((p, i) => (
                  <div key={i} className="p-3.5 rounded-xl border border-slate-800 bg-slate-900/60">
                    <span className="font-bold text-cyan-400 block mb-1 text-sm md:text-base">{p.label}</span>
                    <p className="text-xs md:text-sm text-slate-300 leading-relaxed">{p.desc}</p>
                  </div>
                ))}
              </div>

              <div className="p-4 rounded-xl border border-slate-800 bg-slate-900/50 text-xs md:text-sm text-slate-200 leading-relaxed">
                <span className="font-bold text-amber-400">🎨 Màu sắc nhận diện: </span>
                Mỗi slot có màu sắc đại diện riêng (Slot 1 bạn luôn là Cyan Neon #00e5ff, Slot 2 Đỏ Crimson, Slot 3 Tím Arcane,...).
                Minimap và đơn vị trên chiến trường sẽ hiển thị màu sắc tương ứng theo phe để bạn dễ dàng nắm bắt tình hình mặt trận!
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-slate-800 bg-slate-950 flex flex-wrap items-center justify-between gap-3 shrink-0">
          <div className="flex items-center gap-4 text-xs md:text-sm text-slate-400 font-mono">
            {isInGame && (
              <span className="flex items-center gap-2">
                <span
                  className={`w-2.5 h-2.5 rounded-full ${
                    autoPause ? 'bg-cyan-400 animate-pulse' : 'bg-emerald-400 animate-ping'
                  }`}
                />
                <span className={autoPause ? 'text-cyan-300 font-semibold' : 'text-emerald-300 font-semibold'}>
                  {autoPause
                    ? 'Trận đấu đang tạm dừng (Chiến trường đóng băng)'
                    : 'Trận đấu đang tiếp tục (Cập nhật thời gian thực)'}
                </span>
              </span>
            )}
            <span className="text-slate-500">
              Phím tắt mở nhanh: <span className="text-cyan-400 font-bold">[H]</span> hoặc click nút GUIDE trên HUD
            </span>
          </div>
          <button
            onClick={onClose}
            className="px-6 py-2.5 rounded-xl bg-cyan-600 hover:bg-cyan-500 text-white font-bold text-xs md:text-sm transition-colors cursor-pointer shadow-md"
          >
            ĐÃ HIỂU // ĐÓNG
          </button>
        </div>
      </div>
    </div>
  );
};
