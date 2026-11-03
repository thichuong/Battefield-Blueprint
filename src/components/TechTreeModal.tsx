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
} from 'lucide-react';
import { TechState } from '../bridge/wasmBridge';

interface TechTreeModalProps {
  isOpen: boolean;
  onClose: () => void;
  techState: TechState | null;
  minerals: number;
  energy: number;
  onResearch: (techId: string) => void;
  priorityTechs?: string[];
  onTogglePriorityTech?: (techId: string) => void;
  autoPause?: boolean;
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
    desc: 'Mở khóa SIEGE // LASER MARKSMAN & tăng +25% sát thương Energy toàn quân.',
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
    desc: 'Mở khóa VANGUARD // SHIELDER & tăng +40% Kháng Năng Lượng toàn quân.',
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
    desc: 'Mở khóa DARK TECH // ROCKETEER & đòn bắn TURRET TOWER chuyển sang Nổ AoE 40px.',
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
    desc: 'Tăng +35% Kháng Vật Lý toàn quân và +30% Máu toàn bộ Công Trình.',
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
    desc: 'Mở khóa CELESTIAL // SWORDMASTER (7 phi kiếm Arcane xuyên giáp).',
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
    desc: 'Tăng +40% Kháng Ma Thuật/Năng Lượng và tự hồi phục +3 HP/giây.',
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
    desc: 'Đòn đánh toàn quân làm tan chảy vĩnh viễn mọi chỉ số Kháng của kẻ địch.',
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
    desc: 'Mở khóa VOID // SHADOW WARLOCK (Triệu hồi pháp trận đốt 10% Max HP/giây).',
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
    resistanceBadge: { label: '🛡️ +35% VẬT LÝ & +20% HP', color: 'bg-amber-950 text-amber-400 border-amber-600' },
    desc: 'Tăng +35% Kháng Vật Lý và +20% Lượng Máu tối đa của toàn quân.',
  },
  {
    id: 'behemoth_mutation',
    name: 'Biến Dị Khổng Lồ',
    subBranch: 'KHỔNG LỒ',
    majorBranch: 'monster',
    minerals: 320,
    energy: 180,
    time: 26,
    requires: 'primeval_scales',
    damageTypeBadge: { label: '🔨 DẬM CHẤN ĐỘNG (AOE)', color: 'bg-yellow-950 text-yellow-400 border-yellow-600' },
    desc: 'Mở khóa BEHEMOTH // TITAN (650 HP, dậm chấn động diện rộng).',
  },
  // Tiểu nhánh: Bọ bầy đàn
  {
    id: 'acid_banelings',
    name: 'Bọ Độc Axit',
    subBranch: 'BỌ BẦY ĐÀN',
    majorBranch: 'monster',
    minerals: 160,
    energy: 90,
    time: 14,
    damageTypeBadge: { label: '💥 NỔ AXIT SIÊU CẤP (AOE)', color: 'bg-lime-950 text-lime-400 border-lime-600' },
    desc: 'Mở khóa SWARM // ACID BANELING (Tự nổ AoE 70px x2 sát thương công trình).',
  },
  {
    id: 'hive_proliferation',
    name: 'Bầy Đàn Phân Hạch',
    subBranch: 'BỌ BẦY ĐÀN',
    majorBranch: 'monster',
    minerals: 260,
    energy: 160,
    time: 20,
    requires: 'acid_banelings',
    desc: 'Giảm -35% Chi Phí và Thời Gian sản xuất toàn bộ binh chủng!',
  },

  // ==========================================
  // 4. KINH TẾ & HẬU CẦN (LOGISTICS)
  // ==========================================
  {
    id: 'optical_drills',
    name: 'Khoan Quang Học',
    subBranch: 'KINH TẾ',
    majorBranch: 'logistics',
    minerals: 100,
    energy: 50,
    time: 10,
    desc: 'ENGINEER // WORKER đào nhanh hơn và tăng sức chứa quặng (10 ➔ 15).',
  },
  {
    id: 'overcharged_reactors',
    name: 'Lò Phản Ứng Siêu Nạp',
    subBranch: 'KINH TẾ',
    majorBranch: 'logistics',
    minerals: 150,
    energy: 100,
    time: 15,
    requires: 'optical_drills',
    desc: 'Tăng +50% sản lượng năng lượng của POWER REACTOR.',
  },
  {
    id: 'fortified_blueprints',
    name: 'Bản Vẽ Gia Cố',
    subBranch: 'KIẾN TRÚC',
    majorBranch: 'logistics',
    minerals: 200,
    energy: 150,
    time: 20,
    requires: 'overcharged_reactors',
    desc: 'Tăng +30% Máu công trình và tốc độ thi công.',
  },
  {
    id: 'overload_strike',
    name: 'Kích Nạp Quá Tải',
    subBranch: 'KIẾN TRÚC',
    majorBranch: 'logistics',
    minerals: 400,
    energy: 350,
    time: 35,
    requires: 'fortified_blueprints',
    desc: 'Tăng +66% tốc độ bắn cho TURRET TOWER & HQ COMMAND CENTER.',
  },
  {
    id: 'magnetic_rounds',
    name: 'Đạn Từ Tính',
    subBranch: 'VŨ TRANG',
    majorBranch: 'logistics',
    minerals: 200,
    energy: 150,
    time: 15,
    damageTypeBadge: { label: '⚔️ VẬT LÝ CAO CẤP', color: 'bg-slate-900 text-slate-300 border-slate-600' },
    desc: 'Tăng tầm bắn cho MARKSMAN // RIFLEMAN và TURRET TOWER.',
  },
  {
    id: 'exo_boosters',
    name: 'Động Cơ Tăng Tốc',
    subBranch: 'VŨ TRANG',
    majorBranch: 'logistics',
    minerals: 250,
    energy: 200,
    time: 20,
    requires: 'magnetic_rounds',
    desc: 'Toàn quân di chuyển nhanh hơn +20%.',
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
      <div className="flex items-center gap-3.5 bg-slate-950 px-4 py-2 rounded-xl border border-cyan-500/50 font-mono text-sm shadow-inner">
        <span className="text-cyan-400 font-black">💎 {minerals.toFixed(0)}</span>
        <span className="text-purple-400 font-black">⚡ {energy.toFixed(0)}</span>
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
  isPriority?: boolean;
  priorityIndex?: number;
  onResearch: (techId: string) => void;
  onTogglePriority?: (techId: string) => void;
}

const TechCard = React.memo<TechCardProps>(
  ({
    node,
    status,
    progress,
    isOtherResearching,
    canAfford,
    prerequisiteName,
    isPriority,
    priorityIndex,
    onResearch,
    onTogglePriority,
  }) => {
    const isUnlocked = status === 'unlocked';
    const isResearching = status === 'researching';
    const isLocked = status === 'locked';
    const isAvailable = status === 'available';

    return (
      <div
        className={`w-full min-w-0 box-border p-4 rounded-xl border transition-all ${
          isUnlocked
            ? 'bg-cyan-950/40 border-cyan-500/70 shadow-[0_0_14px_rgba(0,229,255,0.25)]'
            : isResearching
            ? 'bg-amber-950/40 border-amber-500/80 ring-1 ring-amber-500/50 shadow-[0_0_14px_rgba(245,158,11,0.25)]'
            : isPriority
            ? 'bg-amber-950/30 border-amber-400/90 shadow-[0_0_12px_rgba(245,158,11,0.2)]'
            : isLocked
            ? 'bg-slate-950/60 border-slate-800 opacity-60'
            : 'bg-slate-800/40 border-slate-700 hover:border-cyan-400/60'
        }`}
      >
        <div className="flex items-start justify-between gap-2 min-w-0">
          <div className="font-bold text-sm md:text-base text-slate-100 flex items-center gap-2 font-mono min-w-0 flex-1">
            {isUnlocked ? (
              <CheckCircle2 className="w-5 h-5 text-cyan-400 shrink-0" />
            ) : isResearching ? (
              <Loader2 className="w-5 h-5 text-amber-400 animate-spin shrink-0" />
            ) : isPriority ? (
              <span className="text-amber-400 shrink-0 text-base">⭐</span>
            ) : isLocked ? (
              <Lock className="w-5 h-5 text-slate-500 shrink-0" />
            ) : null}
            <span className="break-words leading-snug">{node.name}</span>
          </div>

          {!isUnlocked && onTogglePriority && (
            <button
              onClick={() => onTogglePriority(node.id)}
              className={`shrink-0 px-2 py-0.5 rounded text-[11px] font-mono font-bold flex items-center gap-1 transition-all cursor-pointer ${
                isPriority
                  ? 'bg-amber-500 text-slate-950 shadow-[0_0_8px_rgba(245,158,11,0.5)]'
                  : 'bg-slate-900/80 border border-slate-700 text-slate-400 hover:text-amber-300 hover:border-amber-400'
              }`}
              title="Đánh dấu ưu tiên tự động nâng cấp"
            >
              ⭐ {isPriority ? `ƯU TIÊN #${(priorityIndex ?? 0) + 1}` : 'ƯU TIÊN'}
            </button>
          )}
        </div>

        {/* Badges Phân loại Sát thương & Kháng */}
        <div className="flex flex-wrap gap-1.5 mt-2.5">
          {node.damageTypeBadge && (
            <span
              className={`text-[11px] font-mono font-bold px-2 py-0.5 rounded border leading-tight ${node.damageTypeBadge.color}`}
            >
              {node.damageTypeBadge.label}
            </span>
          )}
          {node.resistanceBadge && (
            <span
              className={`text-[11px] font-mono font-bold px-2 py-0.5 rounded border leading-tight ${node.resistanceBadge.color}`}
            >
              {node.resistanceBadge.label}
            </span>
          )}
        </div>

        <p className="text-xs md:text-sm text-slate-200 mt-2.5 leading-relaxed break-words">{node.desc}</p>

        {isLocked && prerequisiteName && (
          <div className="mt-2.5 text-xs text-amber-400 font-mono flex items-center gap-1.5 break-words">
            <Lock className="w-3.5 h-3.5 shrink-0" />
            <span>Yêu cầu trước: {prerequisiteName}</span>
          </div>
        )}

        {/* Thanh tiến trình nghiên cứu thời gian thực */}
        {isResearching && (
          <div className="mt-3">
            <div className="flex justify-between items-center text-xs font-mono text-amber-400 font-bold mb-1">
              <span>ĐANG TIẾN HÀNH</span>
              <span>{(progress * 100).toFixed(0)}%</span>
            </div>
            <div className="w-full bg-slate-900 rounded-full h-2.5 overflow-hidden border border-amber-500/40">
              <div
                className="bg-gradient-to-r from-amber-500 via-amber-400 to-yellow-300 h-full transition-all duration-150 ease-linear rounded-full"
                style={{ width: `${Math.min(100, Math.max(0, progress * 100))}%` }}
              />
            </div>
          </div>
        )}

        <div className="flex flex-wrap items-center justify-between gap-2.5 mt-3.5 pt-2.5 border-t border-slate-700/60 text-xs md:text-sm">
          <div className="flex items-center gap-2.5 font-mono text-xs md:text-sm flex-wrap">
            <span className={`font-bold ${canAfford ? 'text-cyan-400' : 'text-rose-400'}`}>
              💎 {node.minerals}
            </span>
            <span className={`font-bold ${canAfford ? 'text-purple-400' : 'text-rose-400'}`}>
              ⚡ {node.energy}
            </span>
            <span className="text-slate-400">⏱ {node.time}s</span>
          </div>

          {isUnlocked ? (
            <span className="text-xs md:text-sm font-mono text-cyan-400 font-bold tracking-wider flex items-center gap-1.5 shrink-0">
              <CheckCircle2 className="w-4 h-4" /> ĐÃ MỞ KHÓA
            </span>
          ) : isResearching ? (
            <div className="flex items-center gap-1 text-xs md:text-sm text-amber-400 font-mono font-bold shrink-0">
              <span>{(progress * 100).toFixed(0)}%</span>
            </div>
          ) : (
            <button
              onClick={() => onResearch(node.id)}
              disabled={!isAvailable}
              className={`px-3 py-1.5 rounded text-xs font-mono font-bold transition-all shrink-0 ${
                isAvailable
                  ? 'bg-cyan-600 hover:bg-cyan-500 text-white shadow-md cursor-pointer active:scale-95'
                  : 'bg-slate-800 text-slate-500 cursor-not-allowed'
              }`}
            >
              {isLocked
                ? 'CHƯA ĐỦ Đ/K'
                : isOtherResearching
                ? 'NHÁNH BẬN'
                : !canAfford
                ? 'THIẾU T/N'
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
    if (prev.isPriority !== next.isPriority) return false;
    if (prev.priorityIndex !== next.priorityIndex) return false;
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
  priorityTechs = [],
  onTogglePriorityTech,
  autoPause = true,
}) => {
  const [activeTab, setActiveTab] = useState<MajorBranch>('science');

  const isUnlocked = useCallback(
    (id: string): boolean => {
      if (!techState) return false;
      return !!(techState as any)[id];
    },
    [techState]
  );

  const getResearchItem = useCallback(
    (id: string) => {
      if (techState?.active_researches && techState.active_researches.length > 0) {
        return techState.active_researches.find((r) => r.tech_id === id);
      }
      if (techState?.current_research === id) {
        return {
          tech_id: id,
          branch: '',
          progress: techState.research_progress,
          total_time: techState.research_total_time,
        };
      }
      return undefined;
    },
    [techState?.active_researches, techState?.current_research, techState?.research_progress, techState?.research_total_time]
  );

  const isBranchResearching = useCallback(
    (branch: MajorBranch, excludeTechId?: string) => {
      if (techState?.active_researches && techState.active_researches.length > 0) {
        return techState.active_researches.some(
          (r) => r.branch === branch && r.tech_id !== excludeTechId
        );
      }
      return false;
    },
    [techState?.active_researches]
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

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/85 backdrop-blur-md p-3 md:p-6 select-none">
      <div className="bg-[#030d1a] border-2 border-cyan-500/80 rounded-2xl w-[96vw] max-w-6xl h-[88vh] min-h-[640px] max-h-[920px] shadow-[0_0_50px_rgba(0,229,255,0.3)] flex flex-col overflow-hidden">
        {/* Header Modal */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-cyan-900/60 bg-slate-900/50 shrink-0">
          <div className="flex items-center gap-3">
            <Atom className="w-7 h-7 text-cyan-400 animate-spin-slow shrink-0" />
            <div>
              <h2 className="text-base md:text-lg font-black font-mono text-cyan-300 tracking-wider">
                TACTICAL EVOLUTION & COMBAT TECH // CÂY TIẾN HÓA CHIẾN THUẬT
              </h2>
              <p className="text-xs text-slate-400">Chọn nhánh tiến hóa và nghiên cứu nâng cấp (Phím tắt: T / ESC)</p>
            </div>
          </div>
          <div className="flex items-center gap-4">
            <ResourceBadge minerals={minerals} energy={energy} />

            <button
              onClick={onClose}
              className="p-1.5 text-slate-400 hover:text-white rounded-lg hover:bg-slate-800 transition-colors cursor-pointer"
            >
              <X className="w-6 h-6" />
            </button>
          </div>
        </div>

        {/* 4 Tabs Chuyển Nhánh Tiến Hóa */}
        <div className="grid grid-cols-4 border-b border-cyan-900/60 bg-slate-950/60 font-mono text-sm font-bold shrink-0">
          <button
            onClick={() => setActiveTab('science')}
            className={`py-3.5 px-4 flex items-center justify-center gap-2 border-b-2 transition-all cursor-pointer ${
              activeTab === 'science'
                ? 'border-cyan-400 text-cyan-300 bg-cyan-950/40 shadow-[inset_0_-2px_10px_rgba(0,229,255,0.2)]'
                : 'border-transparent text-slate-400 hover:text-slate-200 hover:bg-slate-900/50'
            }`}
          >
            <Atom className="w-5 h-5 text-cyan-400" />
            <span>🔬 KHOA HỌC (SCIENCE)</span>
          </button>

          <button
            onClick={() => setActiveTab('magic')}
            className={`py-3.5 px-4 flex items-center justify-center gap-2 border-b-2 transition-all cursor-pointer ${
              activeTab === 'magic'
                ? 'border-purple-400 text-purple-300 bg-purple-950/40 shadow-[inset_0_-2px_10px_rgba(168,85,247,0.2)]'
                : 'border-transparent text-slate-400 hover:text-slate-200 hover:bg-slate-900/50'
            }`}
          >
            <Sparkles className="w-5 h-5 text-purple-400" />
            <span>🔮 MA THUẬT (MAGIC)</span>
          </button>

          <button
            onClick={() => setActiveTab('monster')}
            className={`py-3.5 px-4 flex items-center justify-center gap-2 border-b-2 transition-all cursor-pointer ${
              activeTab === 'monster'
                ? 'border-amber-400 text-amber-300 bg-amber-950/40 shadow-[inset_0_-2px_10px_rgba(245,158,11,0.2)]'
                : 'border-transparent text-slate-400 hover:text-slate-200 hover:bg-slate-900/50'
            }`}
          >
            <Bug className="w-5 h-5 text-amber-400" />
            <span>🧬 QUÁI VẬT (MONSTERS)</span>
          </button>

          <button
            onClick={() => setActiveTab('logistics')}
            className={`py-3.5 px-4 flex items-center justify-center gap-2 border-b-2 transition-all cursor-pointer ${
              activeTab === 'logistics'
                ? 'border-emerald-400 text-emerald-300 bg-emerald-950/40 shadow-[inset_0_-2px_10px_rgba(16,185,129,0.2)]'
                : 'border-transparent text-slate-400 hover:text-slate-200 hover:bg-slate-900/50'
            }`}
          >
            <Zap className="w-5 h-5 text-emerald-400" />
            <span>⚙️ HẬU CẦN CƠ BẢN</span>
          </button>
        </div>

        {/* Banner Miêu Tả Nhánh Đang Chọn */}
        <div className="px-6 py-3 bg-slate-900/40 border-b border-cyan-900/30 flex items-center justify-between shrink-0">
          <div>
            <div className={`text-sm font-mono font-bold ${headerInfo.accent}`}>
              {headerInfo.title}
            </div>
            <div className="text-xs text-slate-300 mt-0.5">{headerInfo.desc}</div>
          </div>
          <div className="flex items-center gap-4 text-xs font-mono">
            {priorityTechs.length > 0 && (
              <span className="text-amber-400 bg-amber-950/60 border border-amber-500/50 px-2.5 py-1 rounded font-bold">
                ⭐ {priorityTechs.length} Tech Ưu Tiên Tự Động
              </span>
            )}
            <div className="flex items-center gap-2.5 text-slate-400">
              <span className="flex items-center gap-1">
                <span className="w-2.5 h-2.5 rounded-full bg-slate-200 inline-block"></span> Vật lý
              </span>
              <span className="flex items-center gap-1">
                <span className="w-2.5 h-2.5 rounded-full bg-cyan-400 inline-block"></span> Năng lượng
              </span>
              <span className="flex items-center gap-1">
                <span className="w-2.5 h-2.5 rounded-full bg-orange-400 inline-block"></span> Nổ AoE
              </span>
              <span className="flex items-center gap-1">
                <span className="w-2.5 h-2.5 rounded-full bg-purple-400 inline-block"></span> Ma thuật
              </span>
            </div>
          </div>
        </div>

        {/* Danh Sách Các Tiểu Nhánh & Node Công Nghệ */}
        <div className="flex-1 overflow-y-auto p-5 md:p-6 flex flex-col md:flex-row gap-5 min-h-0 items-start">
          {subBranches.map((subName) => {
            const nodes = currentNodes.filter((n) => n.subBranch === subName);
            return (
              <div
                key={subName}
                className="w-full flex-1 min-w-0 flex flex-col gap-4 bg-slate-900/60 p-4 rounded-xl border border-slate-700/70 shadow-inner h-fit"
              >
                <div className="flex items-center justify-between pb-2.5 border-b border-slate-700/80 font-mono text-sm font-bold text-cyan-300">
                  <span>{subName}</span>
                  <span className="text-xs text-slate-400">{nodes.length} Nâng Cấp</span>
                </div>

                <div className="flex flex-col gap-3.5 w-full min-w-0">
                  {nodes.map((node) => {
                    const unlocked = isUnlocked(node.id);
                    const researchItem = getResearchItem(node.id);
                    const researching = !!researchItem;
                    const progress = researchItem ? researchItem.progress : 0;
                    const isOtherResearchingInBranch = isBranchResearching(node.majorBranch, node.id);
                    const lockedByReq = Boolean(node.requires && !isUnlocked(node.requires));
                    const canAfford = minerals >= node.minerals && energy >= node.energy;

                    let status: 'unlocked' | 'researching' | 'available' | 'locked' = 'available';
                    if (unlocked) {
                      status = 'unlocked';
                    } else if (researching) {
                      status = 'researching';
                    } else if (lockedByReq) {
                      status = 'locked';
                    } else if (isOtherResearchingInBranch || !canAfford) {
                      status = 'available'; // nút sẽ disabled nếu cùng nhánh đang bận hoặc thiếu tiền
                    }

                    const reqNode = node.requires ? TECH_NODES.find((n) => n.id === node.requires) : undefined;
                    const prerequisiteName = reqNode ? reqNode.name : node.requires;
                    const pIndex = priorityTechs.indexOf(node.id);
                    const isPriority = pIndex !== -1;

                    return (
                      <TechCard
                        key={node.id}
                        node={node}
                        status={status}
                        progress={progress}
                        isOtherResearching={isOtherResearchingInBranch}
                        canAfford={canAfford}
                        prerequisiteName={prerequisiteName}
                        isPriority={isPriority}
                        priorityIndex={pIndex}
                        onResearch={onResearch}
                        onTogglePriority={onTogglePriorityTech}
                      />
                    );
                  })}
                </div>
              </div>
            );
          })}
        </div>

        {/* Footer */}
        <div className="px-6 py-3.5 border-t border-cyan-900/40 bg-slate-950/80 flex justify-between items-center text-xs md:text-sm text-slate-400 font-mono shrink-0">
          <div className="flex items-center gap-3">
            <span className="flex items-center gap-2">
              <span
                className={`w-2.5 h-2.5 rounded-full ${
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
            className="px-5 py-2 bg-cyan-600 hover:bg-cyan-500 text-white rounded font-bold transition-all cursor-pointer text-sm"
          >
            TIẾP TỤC TRẬN ĐẤU [ESC]
          </button>
        </div>
      </div>
    </div>
  );
};
