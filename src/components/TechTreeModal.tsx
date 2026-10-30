import React, { useState, useMemo, useCallback } from 'react';
import {
  X,
  Zap,
  CheckCircle2,
  Loader2,
  Lock,
  Atom,
  Sparkles,
  Bug,
  Pause,
  Play,
} from 'lucide-react';
import { TechState } from '../bridge/wasmBridge';

interface TechTreeModalProps {
  isOpen: boolean;
  onClose: () => void;
  techState: TechState | null;
  minerals: number;
  energy: number;
  onResearch: (techId: string) => void;
  autoPause?: boolean;
  onToggleAutoPause?: (val: boolean) => void;
}

export type MajorBranch = 'science' | 'magic' | 'monster' | 'logistics';

export interface TechNode {
  id: string;
  name: string;
  subBranch: string;
  majorBranch: MajorBranch;
  minerals: number;
  energy: number;
  time: number;
  desc: string;
  damageTypeBadge?: { label: string; color: string };
  resistanceBadge?: { label: string; color: string };
  requires?: string;
}

export const TECH_NODES: TechNode[] = [
  // ==========================================
  // 1. NHÁNH KHOA HỌC (SCIENCE)
  // ==========================================
  // Tiểu nhánh: Công nghệ tương lai
  {
    id: 'focused_laser',
    name: 'Tụ Quang Lượng Tử',
    subBranch: 'CÔNG NGHỆ TƯƠNG LAI',
    majorBranch: 'science',
    minerals: 250,
    energy: 200,
    time: 20,
    damageTypeBadge: { label: '⚡ NĂNG LƯỢNG', color: 'bg-sky-950 text-sky-400 border-sky-600' },
    desc: 'Mở khóa đơn vị Laser Marksman và tăng +25% Sát thương Năng lượng xuyên thấu cho toàn quân.',
  },
  {
    id: 'energy_shielding',
    name: 'Khiên Lượng Tử Phản Xạ',
    subBranch: 'CÔNG NGHỆ TƯƠNG LAI',
    majorBranch: 'science',
    minerals: 200,
    energy: 150,
    time: 16,
    requires: 'focused_laser',
    resistanceBadge: { label: '🛡️ +40% KHÁNG NĂNG LƯỢNG', color: 'bg-cyan-950 text-cyan-400 border-cyan-600' },
    desc: 'Mở khóa Vệ Binh Khiên Chắn (Vanguard) và cường hóa +40% Kháng Năng Lượng cho toàn bộ đơn vị.',
  },
  // Tiểu nhánh: Hắc khoa kỹ
  {
    id: 'dark_matter_rockets',
    name: 'Bệ Phóng Rocket Hắc Ám',
    subBranch: 'HẮC KHOA KỸ',
    majorBranch: 'science',
    minerals: 240,
    energy: 160,
    time: 18,
    damageTypeBadge: { label: '💥 NỔ DIỆN RỘNG (AOE)', color: 'bg-orange-950 text-orange-400 border-orange-600' },
    desc: 'Mở khóa Lính Bệ Phóng Tên Lửa (Dark Rocketeer). Đòn bắn của Tháp Pháo chuyển sang Sát thương Nổ AoE 40px.',
  },
  {
    id: 'titan_alloy_plating',
    name: 'Khung Xương Titan Cường Lực',
    subBranch: 'HẮC KHOA KỸ',
    majorBranch: 'science',
    minerals: 280,
    energy: 180,
    time: 22,
    requires: 'dark_matter_rockets',
    resistanceBadge: { label: '🛡️ +35% KHÁNG VẬT LÝ', color: 'bg-amber-950 text-amber-400 border-amber-600' },
    desc: 'Cường hóa hợp kim siêu bền: +35% Kháng Vật lý cho toàn quân và tăng +30% Lượng Máu tối đa của tất cả Công trình.',
  },

  // ==========================================
  // 2. NHÁNH MA THUẬT (MAGIC)
  // ==========================================
  // Tiểu nhánh: Tiên pháp
  {
    id: 'celestial_flying_swords',
    name: 'Ngự Kiếm Thuật Chân Khí',
    subBranch: 'TIÊN PHÁP',
    majorBranch: 'magic',
    minerals: 220,
    energy: 160,
    time: 18,
    damageTypeBadge: { label: '✨ MA THUẬT (ARCANE)', color: 'bg-purple-950 text-purple-400 border-purple-600' },
    desc: 'Mở khóa Tu Tiên Giả (Taoist Swordmaster), phi kiếm chân khí tầm xa gây sát thương Ma thuật bỏ qua 50% kháng vật lý.',
  },
  {
    id: 'golden_bell_shield',
    name: 'Kim Cang Hộ Thể Linh Quang',
    subBranch: 'TIÊN PHÁP',
    majorBranch: 'magic',
    minerals: 260,
    energy: 200,
    time: 22,
    requires: 'celestial_flying_swords',
    resistanceBadge: { label: '🛡️ +40% KHÁNG PHÉP & HỒI MÁU', color: 'bg-emerald-950 text-emerald-400 border-emerald-600' },
    desc: 'Bảo hộ linh khí: Tăng +40% Kháng Năng lượng & Ma thuật, đồng thời toàn quân tự hồi phục +3.0 HP/s.',
  },
  // Tiểu nhánh: Pháp thuật bóng tối
  {
    id: 'corrosive_curse',
    name: 'Hắc Ám Chú Ấn Ăn Mòn',
    subBranch: 'PHÁP THUẬT BÓNG TỐI',
    majorBranch: 'magic',
    minerals: 190,
    energy: 150,
    time: 16,
    desc: 'Đòn đánh của toàn quân găm chú ấn hắc ám lên mục tiêu, làm tan chảy vĩnh viễn mọi chỉ số kháng của kẻ địch khi trúng đòn.',
  },
  {
    id: 'void_cataclysm',
    name: 'Bộc Phá Hư Không',
    subBranch: 'PHÁP THUẬT BÓNG TỐI',
    majorBranch: 'magic',
    minerals: 290,
    energy: 240,
    time: 25,
    requires: 'corrosive_curse',
    damageTypeBadge: { label: '🔮 NỔ BÓNG TỐI (AOE)', color: 'bg-fuchsia-950 text-fuchsia-400 border-fuchsia-600' },
    desc: 'Mở khóa Vu Sư Hư Không (Void Warlock) bắn cầu ma thuật bộc phá bóng tối gây sát thương Ma thuật nổ diện rộng 35px.',
  },

  // ==========================================
  // 3. NHÁNH QUÁI VẬT (MONSTERS)
  // ==========================================
  // Tiểu nhánh: Khổng lồ
  {
    id: 'primeval_scales',
    name: 'Vảy Cổ Đại Hóa Đá',
    subBranch: 'KHỔNG LỒ',
    majorBranch: 'monster',
    minerals: 220,
    energy: 120,
    time: 18,
    resistanceBadge: { label: '🛡️ +50% VẬT LÝ & +40% NỔ', color: 'bg-amber-950 text-amber-400 border-amber-600' },
    desc: 'Lớp da vảy titan cổ đại giúp binh chủng cận chiến nhận thêm +50% Kháng Vật lý và +40% Kháng Nổ.',
  },
  {
    id: 'behemoth_mutation',
    name: 'Đột Biến Cự Thú Titan',
    subBranch: 'KHỔNG LỒ',
    majorBranch: 'monster',
    minerals: 320,
    energy: 180,
    time: 26,
    requires: 'primeval_scales',
    damageTypeBadge: { label: '🔨 DẬM CHẤN ĐỘNG (AOE)', color: 'bg-yellow-950 text-yellow-400 border-yellow-600' },
    desc: 'Mở khóa Chiến Thú Khổng Lồ (Behemoth Titan) 650 HP, đòn chùy dậm đất gây chấn động nứt toác diện rộng 45px.',
  },
  // Tiểu nhánh: Bọ bầy đàn
  {
    id: 'acid_banelings',
    name: 'Tuyến Dịch Axit Cảm Tử',
    subBranch: 'BỌ BẦY ĐÀN',
    majorBranch: 'monster',
    minerals: 160,
    energy: 90,
    time: 14,
    damageTypeBadge: { label: '💥 NỔ AXIT SIÊU CẤP (AOE)', color: 'bg-lime-950 text-lime-400 border-lime-600' },
    desc: 'Mở khóa Bọ Nổ Cảm Tử (Acid Baneling). Tốc độ 160 lao thẳng vào đội hình địch tự kích nổ sát thương Nổ 65px cực mạnh.',
  },
  {
    id: 'hive_proliferation',
    name: 'Sinh Sôi Siêu Tốc',
    subBranch: 'BỌ BẦY ĐÀN',
    majorBranch: 'monster',
    minerals: 260,
    energy: 160,
    time: 20,
    requires: 'acid_banelings',
    desc: 'Tế bào bầy đàn sinh sôi chóng mặt: Giảm 35% chi phí và thời gian huấn luyện toàn quân, tăng +25% tốc độ di chuyển.',
  },

  // ==========================================
  // 4. KINH TẾ & HẬU CẦN (LOGISTICS)
  // ==========================================
  {
    id: 'optical_drills',
    name: 'Mũi Khoan Quang Học',
    subBranch: 'KINH TẾ',
    majorBranch: 'logistics',
    minerals: 100,
    energy: 50,
    time: 10,
    desc: 'Kỹ sư khai thác khoáng sản nhanh hơn +30%, mang thêm +5 khoáng sản mỗi chuyến.',
  },
  {
    id: 'overcharged_reactors',
    name: 'Lò Phản Ứng Quá Tải',
    subBranch: 'KINH TẾ',
    majorBranch: 'logistics',
    minerals: 150,
    energy: 100,
    time: 15,
    requires: 'optical_drills',
    desc: 'Tăng tốc độ sản sinh Năng Lượng từ trạm Power Reactor thêm +50%.',
  },
  {
    id: 'fortified_blueprints',
    name: 'Bản Vẽ Gia Cố Kiến Trúc',
    subBranch: 'KIẾN TRÚC',
    majorBranch: 'logistics',
    minerals: 200,
    energy: 150,
    time: 20,
    requires: 'overcharged_reactors',
    desc: 'Tăng +30% lượng Máu tối đa và độ bền cho toàn bộ Công trình & Tháp pháo.',
  },
  {
    id: 'magnetic_rounds',
    name: 'Đạn Gia Tốc Từ Trường',
    subBranch: 'VŨ TRANG',
    majorBranch: 'logistics',
    minerals: 200,
    energy: 150,
    time: 15,
    damageTypeBadge: { label: '⚔️ VẬT LÝ CAO CẤP', color: 'bg-slate-900 text-slate-300 border-slate-600' },
    desc: 'Tăng +25% Sát thương Vật lý và mở rộng tầm bắn cho Xạ Thủ Súng Trường.',
  },
  {
    id: 'exo_boosters',
    name: 'Bộ Phản Lực Chân Exo-Boost',
    subBranch: 'VŨ TRANG',
    majorBranch: 'logistics',
    minerals: 250,
    energy: 200,
    time: 20,
    requires: 'magnetic_rounds',
    desc: 'Tăng +20% Tốc độ di chuyển chiến thuật cho toàn bộ Binh chủng Người que.',
  },
  {
    id: 'overload_strike',
    name: 'Tụ Điện Quá Tải Hủy Diệt',
    subBranch: 'VŨ TRANG',
    majorBranch: 'logistics',
    minerals: 400,
    energy: 350,
    time: 35,
    desc: 'Tăng +40% tốc độ bắn cho Tháp Pháo Phòng Thủ và Pháo Thủ Laser.',
  },
];

// =========================================================================
// SUBCOMPONENT 1: Memoized Resource Badge (Chỉ re-render khi số nguyên thay đổi)
// =========================================================================
interface ResourceBadgeProps {
  minerals: number;
  energy: number;
}

const ResourceBadge = React.memo<ResourceBadgeProps>(
  ({ minerals, energy }) => {
    return (
      <div className="flex items-center gap-3 bg-slate-950 px-3.5 py-1.5 rounded-lg border border-cyan-900/70 font-mono text-xs">
        <span className="text-cyan-400 font-bold">💎 {minerals.toFixed(0)}</span>
        <span className="text-purple-400 font-bold">⚡ {energy.toFixed(0)}</span>
      </div>
    );
  },
  (prev, next) => {
    return (
      Math.floor(prev.minerals) === Math.floor(next.minerals) &&
      Math.floor(prev.energy) === Math.floor(next.energy)
    );
  }
);

// =========================================================================
// SUBCOMPONENT 2: Memoized Tech Card (Tối ưu hóa DOM khi cập nhật liên tục)
// =========================================================================
interface TechCardProps {
  node: TechNode;
  status: 'unlocked' | 'researching' | 'available' | 'locked';
  progress: number;
  isOtherResearching: boolean;
  canAfford: boolean;
  prerequisiteName?: string;
  onResearch: (techId: string) => void;
}

const TechCard = React.memo<TechCardProps>(
  ({
    node,
    status,
    progress,
    isOtherResearching,
    canAfford,
    prerequisiteName,
    onResearch,
  }) => {
    const isUnlocked = status === 'unlocked';
    const isResearching = status === 'researching';
    const isLocked = status === 'locked';
    const isAvailable = status === 'available';

    return (
      <div
        className={`p-3.5 rounded-lg border transition-all ${
          isUnlocked
            ? 'bg-cyan-950/40 border-cyan-500/70 shadow-[0_0_14px_rgba(0,229,255,0.25)]'
            : isResearching
            ? 'bg-amber-950/40 border-amber-500/80 ring-1 ring-amber-500/50 shadow-[0_0_14px_rgba(245,158,11,0.25)]'
            : isLocked
            ? 'bg-slate-950/60 border-slate-800 opacity-60'
            : 'bg-slate-800/40 border-slate-700 hover:border-cyan-400/60'
        }`}
      >
        <div className="flex items-start justify-between">
          <div className="font-semibold text-sm text-slate-100 flex items-center gap-1.5 font-mono">
            {isUnlocked ? (
              <CheckCircle2 className="w-4 h-4 text-cyan-400 shrink-0" />
            ) : isResearching ? (
              <Loader2 className="w-4 h-4 text-amber-400 animate-spin shrink-0" />
            ) : isLocked ? (
              <Lock className="w-4 h-4 text-slate-500 shrink-0" />
            ) : null}
            <span>{node.name}</span>
          </div>
        </div>

        {/* Badges Phân loại Sát thương & Kháng */}
        <div className="flex flex-wrap gap-1.5 mt-2">
          {node.damageTypeBadge && (
            <span
              className={`text-[9px] font-mono px-2 py-0.5 rounded border ${node.damageTypeBadge.color}`}
            >
              {node.damageTypeBadge.label}
            </span>
          )}
          {node.resistanceBadge && (
            <span
              className={`text-[9px] font-mono px-2 py-0.5 rounded border ${node.resistanceBadge.color}`}
            >
              {node.resistanceBadge.label}
            </span>
          )}
        </div>

        <p className="text-xs text-slate-300 mt-2 leading-relaxed">{node.desc}</p>

        {isLocked && prerequisiteName && (
          <div className="mt-2 text-[10px] text-amber-400/90 font-mono flex items-center gap-1">
            <Lock className="w-3 h-3" />
            <span>Yêu cầu trước: {prerequisiteName}</span>
          </div>
        )}

        {/* Thanh tiến trình nghiên cứu thời gian thực */}
        {isResearching && (
          <div className="mt-2.5">
            <div className="flex justify-between items-center text-[10px] font-mono text-amber-400 font-bold mb-1">
              <span>ĐANG TIẾN HÀNH</span>
              <span>{(progress * 100).toFixed(0)}%</span>
            </div>
            <div className="w-full bg-slate-900 rounded-full h-2 overflow-hidden border border-amber-500/40">
              <div
                className="bg-gradient-to-r from-amber-500 via-amber-400 to-yellow-300 h-full transition-all duration-150 ease-linear rounded-full"
                style={{ width: `${Math.min(100, Math.max(0, progress * 100))}%` }}
              />
            </div>
          </div>
        )}

        <div className="flex items-center justify-between mt-3 pt-2.5 border-t border-slate-700/60 text-xs">
          <div className="flex items-center gap-2.5 font-mono text-xs">
            <span className={`font-bold ${canAfford ? 'text-cyan-400' : 'text-rose-400'}`}>
              💎 {node.minerals}
            </span>
            <span className={`font-bold ${canAfford ? 'text-purple-400' : 'text-rose-400'}`}>
              ⚡ {node.energy}
            </span>
            <span className="text-slate-400">⏱ {node.time}s</span>
          </div>

          {isUnlocked ? (
            <span className="text-xs font-mono text-cyan-400 font-bold tracking-wider flex items-center gap-1">
              <CheckCircle2 className="w-3.5 h-3.5" /> ĐÃ MỞ KHÓA
            </span>
          ) : isResearching ? (
            <div className="flex items-center gap-1 text-xs text-amber-400 font-mono font-bold">
              <span>{(progress * 100).toFixed(0)}%</span>
            </div>
          ) : (
            <button
              onClick={() => onResearch(node.id)}
              disabled={!isAvailable}
              className={`px-3 py-1 rounded text-xs font-mono font-bold transition-all ${
                isAvailable
                  ? 'bg-cyan-600 hover:bg-cyan-500 text-white shadow-md cursor-pointer active:scale-95'
                  : 'bg-slate-800 text-slate-500 cursor-not-allowed'
              }`}
            >
              {isLocked
                ? 'CHƯA ĐỦ ĐIỀU KIỆN'
                : isOtherResearching
                ? 'ĐANG BẬN NGHIÊN CỨU'
                : !canAfford
                ? 'THIẾU TÀI NGUYÊN'
                : 'NGHIÊN CỨU'}
            </button>
          )}
        </div>
      </div>
    );
  },
  (prev, next) => {
    // 1. Nếu trạng thái thay đổi (ví dụ từ locked -> available hoặc researching -> unlocked) => re-render
    if (prev.status !== next.status) return false;
    // 2. Nếu đang nghiên cứu, chỉ re-render khi % tiến trình nguyên (0..100) thay đổi
    if (next.status === 'researching') {
      return Math.floor(prev.progress * 100) === Math.floor(next.progress * 100);
    }
    // 3. Nếu chưa unlocked, kiểm tra xem khả năng chi trả tài nguyên có đảo chiều hay không
    if (next.status !== 'unlocked') {
      if (prev.canAfford !== next.canAfford) return false;
      if (prev.isOtherResearching !== next.isOtherResearching) return false;
      if (prev.prerequisiteName !== next.prerequisiteName) return false;
    }
    // 4. Các trường hợp còn lại (đặc biệt là 'unlocked'): BỎ QUA RE-RENDER HOÀN TOÀN
    return true;
  }
);

// =========================================================================
// MAIN COMPONENT: TechTreeModal
// =========================================================================
export const TechTreeModal: React.FC<TechTreeModalProps> = ({
  isOpen,
  onClose,
  techState,
  minerals,
  energy,
  onResearch,
  autoPause = true,
  onToggleAutoPause,
}) => {
  const [activeTab, setActiveTab] = useState<MajorBranch>('science');

  const isUnlocked = useCallback(
    (id: string): boolean => {
      if (!techState) return false;
      return !!(techState as any)[id];
    },
    [techState]
  );

  const isResearching = useCallback(
    (id: string): boolean => {
      return techState?.current_research === id;
    },
    [techState?.current_research]
  );

  const currentNodes = useMemo(
    () => TECH_NODES.filter((n) => n.majorBranch === activeTab),
    [activeTab]
  );

  // Nhóm theo tiểu nhánh
  const subBranches = useMemo(
    () => Array.from(new Set(currentNodes.map((n) => n.subBranch))),
    [currentNodes]
  );

  if (!isOpen) return null;

  const getTabHeader = () => {
    switch (activeTab) {
      case 'science':
        return {
          title: 'HỌC THUYẾT KHOA HỌC // SCIENCE MATRIX',
          desc: 'Công nghệ tương lai (Tia laser, khiên photon) & Hắc khoa kỹ (Rocket hạt nhân AoE, khung titan giáp dày).',
          accent: 'text-cyan-400 border-cyan-500',
        };
      case 'magic':
        return {
          title: 'HỌC PHÁI MA THUẬT // ARCANE ORDER',
          desc: 'Tiên pháp (Phi kiếm chân khí, linh quang hồi phục) & Pháp thuật bóng tối (Hắc ám chú ấn, bộc phá hư không).',
          accent: 'text-purple-400 border-purple-500',
        };
      case 'monster':
        return {
          title: 'ĐỘT BIẾN QUÁI VẬT // BIOMASS EVOLUTION',
          desc: 'Khổng lồ (Chiến thú Titan 650 HP, dậm chấn động) & Bọ bầy đàn (Bọ nổ cảm tử axit AoE, sinh sôi siêu tốc).',
          accent: 'text-amber-400 border-amber-500',
        };
      case 'logistics':
        return {
          title: 'HẬU CẦN & CÔNG NGHỆ CƠ BẢN // BASE LOGISTICS',
          desc: 'Khai khoáng mũi khoan quang học, lò phản ứng hạt nhân, gia cố kiến trúc và gia tốc động cơ.',
          accent: 'text-emerald-400 border-emerald-500',
        };
    }
  };

  const headerInfo = getTabHeader();
  const isAnyResearching = !!techState?.current_research;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/85 backdrop-blur-md p-4 select-none">
      <div className="bg-[#030d1a] border-2 border-cyan-500/80 rounded-xl w-full max-w-5xl shadow-[0_0_50px_rgba(0,229,255,0.3)] flex flex-col max-h-[92vh]">
        {/* Header Modal */}
        <div className="flex items-center justify-between px-6 py-3.5 border-b border-cyan-900/60 bg-slate-900/50">
          <div className="flex items-center gap-3">
            <Atom className="w-6 h-6 text-cyan-400 animate-spin-slow" />
            <div>
              <h2 className="text-base font-bold font-mono text-cyan-300 tracking-wider">
                TACTICAL EVOLUTION & COMBAT TECH // CÂY TIẾN HÓA CHIẾN THUẬT
              </h2>
              <p className="text-xs text-slate-400">Chọn nhánh tiến hóa và nghiên cứu nâng cấp (Phím tắt: T / ESC)</p>
            </div>
          </div>
          <div className="flex items-center gap-4">
            {/* Tùy chọn Tạm dừng Trận đấu Ngay trong Header */}
            {onToggleAutoPause && (
              <label className="flex items-center gap-2 cursor-pointer select-none text-[11px] font-mono text-slate-300 hover:text-cyan-300 transition-colors bg-slate-950 px-3 py-1.5 rounded-lg border border-slate-700/80">
                <input
                  type="checkbox"
                  checked={autoPause}
                  onChange={(e) => onToggleAutoPause(e.target.checked)}
                  className="w-3.5 h-3.5 accent-cyan-400 cursor-pointer"
                />
                <span className="flex items-center gap-1">
                  {autoPause ? <Pause className="w-3 h-3 text-cyan-400" /> : <Play className="w-3 h-3 text-emerald-400" />}
                  {autoPause ? 'Dừng thời gian khi mở' : 'Thời gian tiếp tục chạy'}
                </span>
              </label>
            )}

            <ResourceBadge minerals={minerals} energy={energy} />

            <button
              onClick={onClose}
              className="p-1.5 text-slate-400 hover:text-white rounded-lg hover:bg-slate-800 transition-colors"
            >
              <X className="w-5 h-5" />
            </button>
          </div>
        </div>

        {/* 4 Tabs Chuyển Nhánh Tiến Hóa */}
        <div className="grid grid-cols-4 border-b border-cyan-900/60 bg-slate-950/60 font-mono text-xs font-bold">
          <button
            onClick={() => setActiveTab('science')}
            className={`py-3 px-4 flex items-center justify-center gap-2 border-b-2 transition-all ${
              activeTab === 'science'
                ? 'border-cyan-400 text-cyan-300 bg-cyan-950/40 shadow-[inset_0_-2px_10px_rgba(0,229,255,0.2)]'
                : 'border-transparent text-slate-400 hover:text-slate-200 hover:bg-slate-900/50'
            }`}
          >
            <Atom className="w-4 h-4 text-cyan-400" />
            <span>🔬 KHOA HỌC (SCIENCE)</span>
          </button>

          <button
            onClick={() => setActiveTab('magic')}
            className={`py-3 px-4 flex items-center justify-center gap-2 border-b-2 transition-all ${
              activeTab === 'magic'
                ? 'border-purple-400 text-purple-300 bg-purple-950/40 shadow-[inset_0_-2px_10px_rgba(168,85,247,0.2)]'
                : 'border-transparent text-slate-400 hover:text-slate-200 hover:bg-slate-900/50'
            }`}
          >
            <Sparkles className="w-4 h-4 text-purple-400" />
            <span>🔮 MA THUẬT (MAGIC)</span>
          </button>

          <button
            onClick={() => setActiveTab('monster')}
            className={`py-3 px-4 flex items-center justify-center gap-2 border-b-2 transition-all ${
              activeTab === 'monster'
                ? 'border-amber-400 text-amber-300 bg-amber-950/40 shadow-[inset_0_-2px_10px_rgba(245,158,11,0.2)]'
                : 'border-transparent text-slate-400 hover:text-slate-200 hover:bg-slate-900/50'
            }`}
          >
            <Bug className="w-4 h-4 text-amber-400" />
            <span>🧬 QUÁI VẬT (MONSTERS)</span>
          </button>

          <button
            onClick={() => setActiveTab('logistics')}
            className={`py-3 px-4 flex items-center justify-center gap-2 border-b-2 transition-all ${
              activeTab === 'logistics'
                ? 'border-emerald-400 text-emerald-300 bg-emerald-950/40 shadow-[inset_0_-2px_10px_rgba(16,185,129,0.2)]'
                : 'border-transparent text-slate-400 hover:text-slate-200 hover:bg-slate-900/50'
            }`}
          >
            <Zap className="w-4 h-4 text-emerald-400" />
            <span>⚙️ HẬU CẦN CƠ BẢN</span>
          </button>
        </div>

        {/* Banner Miêu Tả Nhánh Đang Chọn */}
        <div className="px-6 py-2.5 bg-slate-900/40 border-b border-cyan-900/30 flex items-center justify-between">
          <div>
            <div className={`text-xs font-mono font-bold ${headerInfo.accent}`}>
              {headerInfo.title}
            </div>
            <div className="text-[11px] text-slate-400 mt-0.5">{headerInfo.desc}</div>
          </div>
          <div className="flex items-center gap-2 text-[10px] font-mono text-slate-400">
            <span className="flex items-center gap-1">
              <span className="w-2 h-2 rounded-full bg-slate-200 inline-block"></span> Vật lý
            </span>
            <span className="flex items-center gap-1">
              <span className="w-2 h-2 rounded-full bg-cyan-400 inline-block"></span> Năng lượng
            </span>
            <span className="flex items-center gap-1">
              <span className="w-2 h-2 rounded-full bg-orange-400 inline-block"></span> Nổ AoE
            </span>
            <span className="flex items-center gap-1">
              <span className="w-2 h-2 rounded-full bg-purple-400 inline-block"></span> Ma thuật
            </span>
          </div>
        </div>

        {/* Danh Sách Các Tiểu Nhánh & Node Công Nghệ */}
        <div className="flex-1 overflow-y-auto p-6 flex flex-col md:flex-row gap-5">
          {subBranches.map((subName) => {
            const nodes = currentNodes.filter((n) => n.subBranch === subName);
            return (
              <div
                key={subName}
                className="flex-1 flex flex-col gap-4 bg-slate-900/50 p-4 rounded-xl border border-slate-700/60 shadow-inner"
              >
                <div className="flex items-center justify-between pb-2 border-b border-slate-700/80 font-mono text-xs font-bold text-cyan-300">
                  <span>{subName}</span>
                  <span className="text-[10px] text-slate-400">{nodes.length} Nâng Cấp</span>
                </div>

                <div className="flex flex-col gap-3.5">
                  {nodes.map((node) => {
                    const unlocked = isUnlocked(node.id);
                    const researching = isResearching(node.id);
                    const lockedByReq = Boolean(node.requires && !isUnlocked(node.requires));
                    const canAfford = minerals >= node.minerals && energy >= node.energy;

                    let status: 'unlocked' | 'researching' | 'available' | 'locked' = 'available';
                    if (unlocked) {
                      status = 'unlocked';
                    } else if (researching) {
                      status = 'researching';
                    } else if (lockedByReq) {
                      status = 'locked';
                    } else if (isAnyResearching || !canAfford) {
                      status = 'available'; // nút sẽ disabled nếu đang bận hoặc thiếu tiền
                    }

                    const reqNode = node.requires ? TECH_NODES.find((n) => n.id === node.requires) : undefined;
                    const prerequisiteName = reqNode ? reqNode.name : node.requires;

                    return (
                      <TechCard
                        key={node.id}
                        node={node}
                        status={status}
                        progress={techState?.research_progress || 0}
                        isOtherResearching={isAnyResearching && !researching}
                        canAfford={canAfford}
                        prerequisiteName={prerequisiteName}
                        onResearch={onResearch}
                      />
                    );
                  })}
                </div>
              </div>
            );
          })}
        </div>

        {/* Footer */}
        <div className="px-6 py-3 border-t border-cyan-900/40 bg-slate-950/80 flex justify-between items-center text-xs text-slate-400 font-mono">
          <div className="flex items-center gap-3">
            <span className="flex items-center gap-1.5">
              <span
                className={`w-2 h-2 rounded-full ${
                  autoPause ? 'bg-cyan-400 animate-pulse' : 'bg-emerald-400 animate-ping'
                }`}
              />
              <span className={autoPause ? 'text-cyan-300' : 'text-emerald-300 font-semibold'}>
                {autoPause
                  ? 'Trận đấu đang tạm dừng (Chiến trường đóng băng)'
                  : 'Trận đấu đang tiếp tục (Cập nhật thời gian thực)'}
              </span>
            </span>
          </div>
          <button
            onClick={onClose}
            className="px-4 py-1.5 bg-cyan-600 hover:bg-cyan-500 text-white rounded font-bold transition-all"
          >
            TIẾP TỤC TRẬN ĐẤU [ESC]
          </button>
        </div>
      </div>
    </div>
  );
};
