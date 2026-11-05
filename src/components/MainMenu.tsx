import React, { useState, useMemo, useEffect } from 'react';
import {
  Play,
  Users,
  Crosshair,
  ChevronDown,
  Check,
  Swords,
  BookOpen,
  Settings,
  Zap,
  Shield,
  X,
} from 'lucide-react';
import { PlayerSlotConfig } from '../bridge/wasmBridge';
import { soundEngine } from '../audio/soundEngine';

interface MainMenuProps {
  onStartGame: (difficulty: string, slots: PlayerSlotConfig[]) => void;
  onOpenGuide: () => void;
  onOpenSettings: () => void;
}

type MatchPreset = '1v1' | '2v2' | '3v3' | '4v4' | '3factions' | 'ffa8' | 'custom';

interface SlotUIData {
  index: number;
  name: string;
  location: string;
  shortLoc: string;
  radarX: number; // Tỉ lệ % trên radar 0..100
  radarY: number; // Tỉ lệ % trên radar 0..100
  active: boolean;
  faction: number;
  isHuman: boolean;
  color: string;
}

export const SLOT_CONFIGS = [
  { name: 'Chỉ Huy (Bạn)', location: 'Tây-Nam (Gốc dưới trái)', shortLoc: 'Tây-Nam', x: 22, y: 78 },
  { name: 'AI Đối Thủ 1', location: 'Đông-Bắc (Gốc trên phải)', shortLoc: 'Đông-Bắc', x: 78, y: 22 },
  { name: 'AI Đồng Minh 1', location: 'Tây-Bắc (Gốc trên trái)', shortLoc: 'Tây-Bắc', x: 22, y: 22 },
  { name: 'AI Đối Thủ 2', location: 'Đông-Nam (Gốc dưới phải)', shortLoc: 'Đông-Nam', x: 78, y: 78 },
  { name: 'AI Đồng Minh 2', location: 'Bắc (Cạnh trên giữa)', shortLoc: 'Bắc', x: 50, y: 16 },
  { name: 'AI Đối Thủ 3', location: 'Nam (Cạnh dưới giữa)', shortLoc: 'Nam', x: 50, y: 84 },
  { name: 'AI Đồng Minh 3', location: 'Tây (Cạnh trái giữa)', shortLoc: 'Tây', x: 16, y: 50 },
  { name: 'AI Đối Thủ 4', location: 'Đông (Cạnh phải giữa)', shortLoc: 'Đông', x: 84, y: 50 },
];

export const PLAYER_COLORS = [
  '#00e5ff', // Slot 0: Cyan Neon (Chỉ Huy / Bạn)
  '#f43f5e', // Slot 1: Crimson / Đỏ hồng
  '#a855f7', // Slot 2: Arcane Purple / Tím
  '#f59e0b', // Slot 3: Amber / Vàng hổ phách
  '#10b981', // Slot 4: Emerald Green / Lục bảo
  '#ec4899', // Slot 5: Hot Pink / Hồng đậm
  '#3b82f6', // Slot 6: Royal Blue / Xanh lam
  '#eab308', // Slot 7: Gold / Vàng tươi
];

export const PLAYER_COLOR_NAMES = [
  'Cyan Neon',
  'Đỏ Crimson',
  'Tím Arcane',
  'Vàng Hổ Phách',
  'Lục Bảo',
  'Hồng Magenta',
  'Xanh Lam',
  'Vàng Gold',
];

export const FACTION_NAMES = [
  'Phe 1 (Liên Minh)',
  'Phe 2 (Đối Lập)',
  'Phe 3 (Độc Lập)',
  'Phe 4 (Kháng Chiến)',
  'Phe 5 (Lực Lượng 5)',
  'Phe 6 (Lực Lượng 6)',
  'Phe 7 (Lực Lượng 7)',
  'Phe 8 (Lực Lượng 8)',
];

export const FACTION_SHORT_LABELS = [
  'Liên Minh',
  'Đối Lập',
  'Độc Lập',
  'Kháng Chiến',
  'Lực Lượng 5',
  'Lực Lượng 6',
  'Lực Lượng 7',
  'Lực Lượng 8',
];

export const MainMenu: React.FC<MainMenuProps> = ({ onStartGame, onOpenGuide, onOpenSettings }) => {
  const [difficulty, setDifficulty] = useState<string>('normal');
  const [preset, setPreset] = useState<MatchPreset>('4v4');
  const [hoveredSlot, setHoveredSlot] = useState<number | null>(null);
  const [openFactionDropdown, setOpenFactionDropdown] = useState<{
    slotIndex: number;
    top: number;
    left: number;
  } | null>(null);

  // Khởi tạo trạng thái 8 slots
  const [slots, setSlots] = useState<SlotUIData[]>(() =>
    SLOT_CONFIGS.map((cfg, idx) => ({
      index: idx,
      name: cfg.name,
      location: cfg.location,
      shortLoc: cfg.shortLoc,
      radarX: cfg.x,
      radarY: cfg.y,
      active: true,
      faction: idx % 2 === 0 ? 0 : 1,
      isHuman: idx === 0,
      color: PLAYER_COLORS[idx],
    }))
  );

  // Áp dụng Preset nhanh
  const applyPreset = (p: MatchPreset) => {
    soundEngine.playSound('click');
    setOpenFactionDropdown(null);
    setPreset(p);
    setSlots((prev) => {
      const next = prev.map((s) => ({ ...s, isHuman: s.index === 0 }));
      switch (p) {
        case '1v1':
          next[0] = { ...next[0], active: true, faction: 0 };
          next[1] = { ...next[1], active: true, faction: 1 };
          for (let i = 2; i < 8; i++) next[i].active = false;
          break;
        case '2v2':
          next[0] = { ...next[0], active: true, faction: 0 };
          next[2] = { ...next[2], active: true, faction: 0 };
          next[1] = { ...next[1], active: true, faction: 1 };
          next[3] = { ...next[3], active: true, faction: 1 };
          for (let i = 4; i < 8; i++) next[i].active = false;
          break;
        case '3v3':
          next[0] = { ...next[0], active: true, faction: 0 };
          next[2] = { ...next[2], active: true, faction: 0 };
          next[4] = { ...next[4], active: true, faction: 0 };
          next[1] = { ...next[1], active: true, faction: 1 };
          next[3] = { ...next[3], active: true, faction: 1 };
          next[5] = { ...next[5], active: true, faction: 1 };
          next[6].active = false;
          next[7].active = false;
          break;
        case '4v4':
          for (let i = 0; i < 8; i++) {
            next[i].active = true;
            next[i].faction = i % 2 === 0 ? 0 : 1;
          }
          break;
        case '3factions':
          // 3 phe: Phe 0 (0, 3), Phe 1 (1, 4), Phe 2 (2, 5)
          next[0] = { ...next[0], active: true, faction: 0 };
          next[3] = { ...next[3], active: true, faction: 0 };
          next[1] = { ...next[1], active: true, faction: 1 };
          next[4] = { ...next[4], active: true, faction: 1 };
          next[2] = { ...next[2], active: true, faction: 2 };
          next[5] = { ...next[5], active: true, faction: 2 };
          next[6].active = false;
          next[7].active = false;
          break;
        case 'ffa8':
          for (let i = 0; i < 8; i++) {
            next[i].active = true;
            next[i].faction = i;
          }
          break;
        case 'custom':
          break;
      }
      return next;
    });
  };

  // Toggle active slot
  const toggleSlotActive = (index: number) => {
    if (index === 0) return; // Slot 0 luôn bật
    soundEngine.playSound('click');
    setOpenFactionDropdown(null);
    setPreset('custom');
    setSlots((prev) =>
      prev.map((s) => (s.index === index ? { ...s, active: !s.active } : s))
    );
  };

  // Thay đổi phe của slot bằng cách chọn trực tiếp từ danh sách 8 phe
  const selectSlotFaction = (index: number, faction: number) => {
    soundEngine.playSound('order');
    setPreset('custom');
    setSlots((prev) =>
      prev.map((s) =>
        s.index === index ? { ...s, faction } : s
      )
    );
    setOpenFactionDropdown(null);
  };

  // Mở/Đóng danh sách lựa chọn phe dạng popover nằm đè lên mọi component
  const handleFactionButtonClick = (
    index: number,
    e: React.MouseEvent<HTMLButtonElement>
  ) => {
    e.stopPropagation();
    soundEngine.playSound('click');

    if (openFactionDropdown?.slotIndex === index) {
      setOpenFactionDropdown(null);
      return;
    }

    const rect = e.currentTarget.getBoundingClientRect();
    const dropdownHeight = 330;
    const dropdownWidth = 340;

    const spaceBelow = window.innerHeight - rect.bottom;
    const showAbove = spaceBelow < dropdownHeight && rect.top > dropdownHeight;
    const top = showAbove
      ? Math.max(12, rect.top - dropdownHeight - 6)
      : Math.min(window.innerHeight - dropdownHeight - 12, rect.bottom + 6);

    let left = rect.right - dropdownWidth;
    if (left < 12) left = 12;
    if (left + dropdownWidth > window.innerWidth - 12) {
      left = window.innerWidth - dropdownWidth - 12;
    }

    setOpenFactionDropdown({
      slotIndex: index,
      top,
      left,
    });
  };

  // Đóng dropdown khi click ra ngoài, cuộn trang hoặc bấm ESC
  useEffect(() => {
    if (!openFactionDropdown) return;

    const handleClickOutside = () => {
      setOpenFactionDropdown(null);
    };

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setOpenFactionDropdown(null);
      }
    };

    const handleScroll = () => {
      setOpenFactionDropdown(null);
    };

    window.addEventListener('click', handleClickOutside);
    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('scroll', handleScroll, true);
    window.addEventListener('resize', handleClickOutside);

    return () => {
      window.removeEventListener('click', handleClickOutside);
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('scroll', handleScroll, true);
      window.removeEventListener('resize', handleClickOutside);
    };
  }, [openFactionDropdown]);

  // Thống kê tóm tắt
  const summary = useMemo(() => {
    const activeSlots = slots.filter((s) => s.active);
    const playerFaction = slots[0]?.faction ?? 0;
    const allies = activeSlots.filter((s) => s.index !== 0 && s.faction === playerFaction).length;
    const enemies = activeSlots.filter((s) => s.faction !== playerFaction).length;
    const factionsCount = new Set(activeSlots.map((s) => s.faction)).size;
    return { total: activeSlots.length, allies, enemies, factionsCount };
  }, [slots]);

  const handleStart = () => {
    soundEngine.playSound('order');
    const payloadSlots: PlayerSlotConfig[] = slots.map((s) => ({
      slot_index: s.index,
      active: s.active,
      faction: s.faction,
      is_human: s.isHuman,
    }));
    onStartGame(difficulty, payloadSlots);
  };

  // Lắng nghe phím tắt điều khiển nhanh từ màn hình chính
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Tránh bắt phím khi đang gõ vào input
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        handleStart();
      } else if (e.key === 'g' || e.key === 'G') {
        e.preventDefault();
        onOpenGuide();
      } else if (e.key === 's' || e.key === 'S') {
        e.preventDefault();
        onOpenSettings();
      } else if (e.key === '1') {
        applyPreset('1v1');
      } else if (e.key === '2') {
        applyPreset('2v2');
      } else if (e.key === '3') {
        applyPreset('3v3');
      } else if (e.key === '4') {
        applyPreset('4v4');
      } else if (e.key === '5') {
        applyPreset('3factions');
      } else if (e.key === '6') {
        applyPreset('ffa8');
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [difficulty, slots, onOpenGuide, onOpenSettings]);


  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-[#020813]/95 text-slate-100 font-mono p-2 sm:p-4 md:p-6 overflow-y-auto select-none backdrop-blur-sm">
      {/* Background blueprint grid */}
      <div className="fixed inset-0 bg-[linear-gradient(to_right,#071e3d_1px,transparent_1px),linear-gradient(to_bottom,#071e3d_1px,transparent_1px)] bg-[size:3.5rem_3.5rem] [mask-image:radial-gradient(ellipse_75%_65%_at_50%_50%,#000_70%,transparent_100%)] opacity-80 pointer-events-none" />

      {/* Main Tactical Command Console (Landscape 16:9 / Widescreen Optimized) */}
      <div className="relative z-10 w-[1280px] max-w-[96vw] h-[750px] max-h-[95vh] min-h-[580px] flex flex-col shrink-0 border-2 border-cyan-500/70 rounded-2xl bg-slate-950/95 shadow-[0_0_80px_rgba(0,229,255,0.28)] p-3 sm:p-5 md:p-6 my-auto overflow-hidden">
        {/* =========================================================================
            TOP BAR HUD: Title, Status Pill & Quick Action Buttons
        ========================================================================= */}
        <header className="flex items-center justify-between gap-3 pb-3 border-b border-cyan-900/70 shrink-0">
          <div className="flex items-center gap-3 min-w-0">
            <div className="p-2 sm:p-2.5 rounded-xl bg-cyan-950/80 border border-cyan-500/80 text-cyan-400 shadow-[0_0_15px_rgba(0,229,255,0.35)] shrink-0">
              <Crosshair className="w-5 h-5 sm:w-6 sm:h-6 animate-spin-slow" />
            </div>
            <div className="min-w-0">
              <div className="flex items-center gap-2 flex-wrap">
                <h1 className="text-lg sm:text-2xl md:text-3xl font-black tracking-tight text-white drop-shadow-[0_0_18px_rgba(0,229,255,0.6)] truncate">
                  BATTLEFIELD <span className="text-cyan-400">BLUEPRINT</span>
                </h1>
                <span className="hidden md:inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-cyan-950/90 border border-cyan-500/70 text-cyan-300 text-[13px] font-bold tracking-widest uppercase shadow-sm">
                  <span className="w-1.5 h-1.5 rounded-full bg-cyan-400 animate-ping inline-block" />
                  RTS 4800x4800 // RUST WASM
                </span>
              </div>
              <p className="text-xs sm:text-sm text-slate-300 mt-0.5 sm:mt-1 block whitespace-normal break-words leading-snug">
                Trung tâm chỉ huy chiến thuật đa phe • Mô phỏng thời gian thực • Deep RL Bot & AutoPlay
              </p>
            </div>
          </div>

          <div className="flex items-center gap-2.5 shrink-0">
            {/* Quick Status Pill */}
            <div className="hidden lg:flex items-center gap-2 bg-slate-900/90 border border-cyan-900/80 px-3.5 py-1.5 rounded-xl text-sm text-slate-200 shadow-inner">
              <span className="text-cyan-400 font-bold">{summary.total}/8</span> Căn cứ •{' '}
              <span className="text-emerald-400 font-bold">{summary.allies}</span> Đồng minh •{' '}
              <span className="text-rose-400 font-bold">{summary.enemies}</span> Đối thủ •{' '}
              <span className="text-amber-400 font-bold">{summary.factionsCount}</span> Phe
            </div>

            {/* Guide Button */}
            <button
              onClick={() => {
                soundEngine.playSound('click');
                onOpenGuide();
              }}
              className="flex items-center gap-1.5 px-3 py-2 rounded-xl bg-slate-900/90 hover:bg-cyan-950/80 border border-slate-700 hover:border-cyan-400 text-cyan-300 font-bold text-sm uppercase tracking-wider transition-all cursor-pointer shadow-sm hover:shadow-[0_0_12px_rgba(0,229,255,0.25)]"
              title="Xem cẩm nang chiến thuật [Phím tắt: G]"
            >
              <BookOpen className="w-4 h-4 text-cyan-400" />
              <span className="hidden sm:inline">HƯỚNG DẪN</span>
              <span className="text-[13px] text-cyan-400 font-medium">[G]</span>
            </button>

            {/* Settings Button */}
            <button
              onClick={() => {
                soundEngine.playSound('click');
                onOpenSettings();
              }}
              className="flex items-center gap-1.5 px-3 py-2 rounded-xl bg-slate-900/90 hover:bg-slate-800 border border-slate-700 hover:border-slate-500 text-slate-200 hover:text-white font-bold text-sm uppercase tracking-wider transition-all cursor-pointer shadow-sm"
              title="Cài đặt game [Phím tắt: S]"
            >
              <Settings className="w-4 h-4 text-slate-400" />
              <span className="hidden sm:inline">CÀI ĐẶT</span>
              <span className="text-[13px] text-slate-400 font-medium">[S]</span>
            </button>
          </div>
        </header>

        {/* =========================================================================
            MAIN 2-COLUMN WAR ROOM LAYOUT (Optimized for Horizontal Displays)
        ========================================================================= */}
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-3 sm:gap-4 md:gap-5 flex-1 min-h-0 pt-3 overflow-y-auto">
          {/* -----------------------------------------------------------------------
              LEFT COLUMN (5 Cols): Presets & AI Difficulty
          ----------------------------------------------------------------------- */}
          <div className="lg:col-span-5 flex flex-col justify-start gap-3.5 min-h-0">
            {/* 1. Presets Grid */}
            <div className="bg-slate-900/60 p-3 sm:p-3.5 rounded-xl border border-slate-800/80 text-left shrink-0">
              <div className="flex items-center justify-between mb-2.5">
                <label className="text-sm font-bold text-slate-200 uppercase tracking-wider flex items-center gap-1.5">
                  <Swords className="w-4 h-4 text-cyan-400" /> Kịch Bản & Chế Độ Chiến Đấu:
                </label>
                <span className="text-[13px] text-cyan-300 bg-cyan-950/70 border border-cyan-800/80 px-2.5 py-0.5 rounded font-mono font-semibold">
                  Preset: {preset.toUpperCase()}
                </span>
              </div>

              <div className="grid grid-cols-3 sm:grid-cols-6 lg:grid-cols-3 gap-2">
                {[
                  { id: '1v1', label: '1 vs 1', desc: 'Đối đầu', key: '1' },
                  { id: '2v2', label: '2 vs 2', desc: 'Song đấu', key: '2' },
                  { id: '3v3', label: '3 vs 3', desc: 'Chiến đoàn', key: '3' },
                  { id: '4v4', label: '4 vs 4', desc: 'Đại chiến 8P', key: '4' },
                  { id: '3factions', label: '2v2v2', desc: 'Tam giác phe', key: '5' },
                  { id: 'ffa8', label: '8-FFA', desc: 'Hỗn chiến 8 phe', key: '6' },
                ].map((p) => (
                  <button
                    key={p.id}
                    onClick={() => applyPreset(p.id as MatchPreset)}
                    className={`py-2 px-2.5 rounded-lg border text-center transition-all cursor-pointer flex flex-col items-center justify-center relative overflow-hidden ${
                      preset === p.id
                        ? 'bg-cyan-950/90 border-cyan-400 text-cyan-300 shadow-[0_0_12px_rgba(0,229,255,0.35)] scale-[1.02]'
                        : 'bg-slate-950/60 border-slate-800 text-slate-300 hover:border-slate-700 hover:text-white'
                    }`}
                  >
                    <div className="flex items-center gap-1">
                      <span className="text-sm font-bold">{p.label}</span>
                      <span className="text-[13px] text-cyan-400 font-mono font-bold">[{p.key}]</span>
                    </div>
                    <span className="text-[13px] text-slate-300 truncate w-full mt-0.5">{p.desc}</span>
                  </button>
                ))}
              </div>
            </div>

            {/* 2. AI Difficulty Selector */}
            <div className="bg-slate-900/60 p-3 sm:p-3.5 rounded-xl border border-slate-800/80 text-left shrink-0">
              <label className="text-sm font-bold text-slate-200 uppercase tracking-wider block mb-2 flex items-center gap-1.5">
                <Zap className="w-4 h-4 text-amber-400" /> Độ Khó Trí Tuệ Nhân Tạo (AI Difficulty):
              </label>
              <div className="grid grid-cols-3 gap-2">
                {[
                  { id: 'easy', label: 'TÂN BINH', desc: '-20% T/Nguyên', badgeColor: 'text-sky-300' },
                  { id: 'normal', label: 'TIÊU CHUẨN', desc: 'Chuẩn 100%', badgeColor: 'text-emerald-300' },
                  { id: 'hard', label: 'CHUYÊN GIA', desc: '+20% T/Nguyên', badgeColor: 'text-rose-300' },
                ].map((diff) => (
                  <button
                    key={diff.id}
                    onClick={() => {
                      soundEngine.playSound('click');
                      setDifficulty(diff.id);
                    }}
                    className={`py-2 px-2.5 rounded-lg text-center transition-all border cursor-pointer flex flex-col items-center justify-center ${
                      difficulty === diff.id
                        ? 'bg-cyan-950/90 border-cyan-400 text-cyan-300 shadow-[0_0_10px_rgba(0,229,255,0.3)]'
                        : 'bg-slate-950/50 border-slate-800 text-slate-300 hover:border-slate-700 hover:text-white'
                    }`}
                  >
                    <span className="text-sm font-bold">{diff.label}</span>
                    <span className={`text-[13px] font-semibold mt-0.5 ${diff.badgeColor}`}>{diff.desc}</span>
                  </button>
                ))}
              </div>
            </div>
          </div>

          {/* -----------------------------------------------------------------------
              RIGHT COLUMN (7 Cols): 8 Base Roster Slots Grid & Launch Button
          ----------------------------------------------------------------------- */}
          <div className="lg:col-span-7 flex flex-col justify-between gap-3 min-h-0">
            {/* 1. 8 Slots Roster Grid */}
            <div className="bg-slate-900/60 p-3 sm:p-3.5 rounded-xl border border-slate-800/80 flex flex-col flex-1 min-h-0 text-left">
              <div className="flex items-center justify-between mb-2 shrink-0">
                <span className="text-sm font-bold text-slate-100 flex items-center gap-1.5">
                  <Users className="w-4 h-4 text-amber-400" /> Phân Bố 8 Căn Cứ Chiến Lược:
                </span>
                <span className="text-[13px] text-slate-300 italic hidden sm:inline">
                  * Checkbox để bật/tắt • Click huy hiệu Phe để đổi liên minh
                </span>
              </div>

              {/* Grid 4 rows x 2 cols (Compact & Sleek for landscape) */}
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 text-sm flex-1 min-h-0 overflow-y-auto pr-0.5">
                {slots.map((slot) => {
                  const playerFaction = slots[0]?.faction ?? 0;
                  const isPlayer = slot.index === 0;
                  const isAlly = !isPlayer && slot.faction === playerFaction;
                  const playerColor = PLAYER_COLORS[slot.index];
                  const factionColor = PLAYER_COLORS[slot.faction % PLAYER_COLORS.length];
                  const isHovered = hoveredSlot === slot.index;

                  return (
                    <div
                      key={slot.index}
                      onMouseEnter={() => setHoveredSlot(slot.index)}
                      onMouseLeave={() => setHoveredSlot(null)}
                      className={`relative py-1.5 px-2.5 rounded-lg border transition-all flex items-center justify-between gap-2 overflow-hidden ${
                        !slot.active
                          ? 'bg-slate-950/40 border-slate-900 opacity-40'
                          : isPlayer
                          ? 'bg-cyan-950/40 border-cyan-500/70 shadow-[0_0_10px_rgba(0,229,255,0.18)]'
                          : isAlly
                          ? 'bg-slate-950/90 border-emerald-900/60'
                          : 'bg-slate-950/80 border-slate-800/90'
                      } ${isHovered && slot.active ? 'ring-1 ring-cyan-400/50 bg-slate-900/80' : ''}`}
                      style={
                        slot.active
                          ? {
                              borderLeftColor: playerColor,
                              borderLeftWidth: '3.5px',
                            }
                          : { borderLeftWidth: '3.5px', borderLeftColor: '#334155' }
                      }
                    >
                      <div className="flex items-center gap-2 min-w-0">
                        {/* Checkbox toggle slot */}
                        <button
                          type="button"
                          onClick={() => toggleSlotActive(slot.index)}
                          disabled={isPlayer}
                          className={`w-5 h-5 rounded flex items-center justify-center border transition-all shrink-0 ${
                            isPlayer
                              ? 'bg-cyan-500 border-cyan-400 text-slate-950 cursor-not-allowed shadow-[0_0_6px_rgba(0,229,255,0.4)]'
                              : slot.active
                              ? 'bg-cyan-600 border-cyan-400 text-white cursor-pointer hover:bg-cyan-500'
                              : 'bg-slate-900 border-slate-700 cursor-pointer hover:border-slate-500'
                          }`}
                          title={isPlayer ? 'Chỉ Huy luôn tham chiến' : 'Bật/Tắt Căn cứ này'}
                        >
                          {slot.active && <Check className="w-3.5 h-3.5 stroke-[3]" />}
                        </button>

                        <div className="min-w-0 flex flex-col justify-center">
                          <div className="flex items-center gap-1.5 leading-tight">
                            <span className="font-mono text-[11px] font-bold text-slate-400">
                              #{slot.index + 1}
                            </span>
                            <span className="font-bold text-slate-100 truncate text-xs sm:text-[13px]">
                              {slot.name}
                            </span>

                            {isPlayer && (
                              <span className="bg-cyan-500/20 border border-cyan-400/50 text-cyan-300 text-[10px] px-1.5 py-0.2 rounded font-black tracking-wide shrink-0">
                                BẠN
                              </span>
                            )}
                            {isAlly && slot.active && (
                              <span className="bg-emerald-500/20 border border-emerald-400/50 text-emerald-300 text-[10px] px-1.5 py-0.2 rounded font-bold tracking-wide shrink-0">
                                MINH
                              </span>
                            )}
                            {!isPlayer && !isAlly && slot.active && (
                              <span className="bg-rose-500/20 border border-rose-500/50 text-rose-300 text-[10px] px-1.5 py-0.2 rounded font-bold tracking-wide shrink-0">
                                ĐỊCH
                              </span>
                            )}
                          </div>

                          <div className="text-[11px] text-slate-400 flex items-center gap-1 mt-0.5 leading-none truncate">
                            <span
                              className="w-2 h-2 rounded-full inline-block shrink-0"
                              style={{
                                backgroundColor: playerColor,
                                boxShadow: slot.active ? `0 0 6px ${playerColor}` : 'none',
                              }}
                            />
                            <span style={{ color: playerColor }} className="font-semibold text-[11px] truncate">
                              {PLAYER_COLOR_NAMES[slot.index]}
                            </span>
                            <span className="text-slate-600">•</span>
                            <span className="text-slate-400 text-[11px] truncate">{slot.shortLoc}</span>
                          </div>
                        </div>
                      </div>

                      {/* Faction selector badge */}
                      {slot.active ? (
                        <button
                          type="button"
                          onClick={(e) => handleFactionButtonClick(slot.index, e)}
                          className={`h-7 px-2.5 rounded-md text-xs font-mono font-bold border transition-all flex items-center gap-1.5 shrink-0 select-none cursor-pointer hover:brightness-125 hover:scale-105 active:scale-95 ${
                            openFactionDropdown?.slotIndex === slot.index
                              ? 'ring-2 ring-cyan-400 shadow-[0_0_12px_rgba(0,229,255,0.6)] brightness-125'
                              : ''
                          }`}
                          style={{
                            borderColor: factionColor,
                            backgroundColor: `${factionColor}22`,
                            color: factionColor,
                            boxShadow: `0 0 8px ${factionColor}33`,
                          }}
                          title={`Click để chọn trong 8 Phe (Hiện tại: Phe ${slot.faction + 1})`}
                        >
                          <span
                            className="w-2.5 h-2.5 rounded-full inline-block shrink-0"
                            style={{ backgroundColor: factionColor, boxShadow: `0 0 6px ${factionColor}` }}
                          />
                          <span>Phe {slot.faction + 1}</span>
                          <ChevronDown
                            className={`w-3.5 h-3.5 opacity-80 transition-transform duration-200 ${
                              openFactionDropdown?.slotIndex === slot.index ? 'rotate-180 text-white' : ''
                            }`}
                          />
                        </button>
                      ) : (
                        <span className="text-[11px] font-mono text-slate-500 italic shrink-0 px-2 py-0.5">
                          Tắt
                        </span>
                      )}
                    </div>
                  );
                })}
              </div>
            </div>

            {/* 2. Tactical Launch Action Bar */}
            <div className="bg-slate-900/60 p-3 sm:p-3.5 rounded-xl border border-slate-800/80 shrink-0">
              <button
                type="button"
                onClick={handleStart}
                className="group relative flex items-center justify-center gap-3 w-full py-3.5 sm:py-4 px-6 rounded-xl bg-gradient-to-r from-cyan-500 via-sky-400 to-cyan-500 hover:from-cyan-400 hover:via-sky-300 hover:to-cyan-400 text-slate-950 font-black text-sm sm:text-base tracking-wider uppercase shadow-[0_0_35px_rgba(0,229,255,0.45)] transition-all transform hover:scale-[1.01] active:scale-[0.99] cursor-pointer overflow-hidden border border-cyan-300"
              >
                {/* Glowing light streak */}
                <div className="absolute inset-0 bg-white/20 -translate-x-full group-hover:translate-x-full transition-transform duration-1000 pointer-events-none" />

                <Play className="w-5 h-5 fill-current shrink-0" />
                <span className="truncate">XUẤT QUÂN // BẮT ĐẦU CHIẾN TRƯỜNG ĐA PHE</span>
                <span className="hidden sm:inline-block text-sm font-mono font-bold text-slate-950 bg-cyan-300/80 px-2 py-0.5 rounded ml-1">
                  [ENTER]
                </span>
              </button>

              <div className="flex items-center justify-between text-[13px] text-slate-300 mt-2 px-1">
                <span>⚡ Phím tắt nhanh: [Enter] Xuất quân • [G] Hướng dẫn • [S] Cài đặt • [1..6] Chọn Kịch bản</span>
                <span className="text-cyan-400 font-bold hidden sm:inline">Mô phỏng 60 FPS • Đồ họa Vector WebGPU</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* =========================================================================
          POPOVER: Floating Faction Selection Grid (Guaranteed all 8 factions visible)
      ========================================================================= */}
      {openFactionDropdown && (
        <div
          className="fixed z-[9999] p-3.5 bg-slate-950/98 border-2 border-cyan-500/80 rounded-2xl shadow-[0_20px_60px_rgba(0,0,0,0.95),0_0_35px_rgba(0,229,255,0.35)] backdrop-blur-xl w-[340px] select-none animate-in fade-in zoom-in-95 duration-150"
          style={{
            top: `${openFactionDropdown.top}px`,
            left: `${openFactionDropdown.left}px`,
          }}
          onClick={(e) => e.stopPropagation()}
        >
          {/* Header */}
          <div className="flex items-center justify-between pb-2 mb-2.5 border-b border-cyan-900/70">
            <div className="flex items-center gap-2">
              <Shield className="w-4 h-4 text-cyan-400" />
              <span className="text-xs font-black tracking-wider text-cyan-300 uppercase">
                CHỌN PHE TÁC CHIẾN
              </span>
            </div>
            <div className="flex items-center gap-1.5">
              <span className="text-[10px] font-mono font-bold px-2 py-0.5 rounded bg-cyan-950 border border-cyan-800 text-cyan-400">
                #{openFactionDropdown.slotIndex + 1}
              </span>
              <button
                type="button"
                onClick={() => setOpenFactionDropdown(null)}
                className="w-5 h-5 rounded-md flex items-center justify-center text-slate-400 hover:text-white hover:bg-slate-800 transition-colors cursor-pointer"
                title="Đóng"
              >
                <X className="w-3.5 h-3.5" />
              </button>
            </div>
          </div>

          {/* Subheader info: Target base location & name */}
          <div className="flex items-center justify-between px-2.5 py-1.5 mb-2.5 rounded-lg bg-cyan-950/40 border border-cyan-900/50 text-xs">
            <div className="flex items-center gap-2 min-w-0">
              <span
                className="w-2.5 h-2.5 rounded-full shrink-0"
                style={{
                  backgroundColor: PLAYER_COLORS[openFactionDropdown.slotIndex],
                  boxShadow: `0 0 6px ${PLAYER_COLORS[openFactionDropdown.slotIndex]}`,
                }}
              />
              <span className="font-bold text-slate-100 truncate">
                {slots[openFactionDropdown.slotIndex]?.name}
              </span>
            </div>
            <span className="text-cyan-400 font-mono text-[11px] shrink-0 font-semibold">
              {slots[openFactionDropdown.slotIndex]?.shortLoc}
            </span>
          </div>

          {/* 8 Factions Grid - 2 columns x 4 rows, EVERY SINGLE FACTION 100% VISIBLE */}
          <div className="grid grid-cols-2 gap-2">
            {[0, 1, 2, 3, 4, 5, 6, 7].map((fIndex) => {
              const fColor = PLAYER_COLORS[fIndex];
              const isSelected = slots[openFactionDropdown.slotIndex]?.faction === fIndex;
              const isPlayerSlot = openFactionDropdown.slotIndex === 0;
              const playerFaction = slots[0]?.faction ?? 0;
              const isAlliedWithPlayer = isPlayerSlot ? true : fIndex === playerFaction;

              // Đếm số lượng căn cứ đang thuộc phe này
              const basesInFaction = slots.filter((s) => s.active && s.faction === fIndex).length;

              return (
                <button
                  key={fIndex}
                  type="button"
                  onClick={() => selectSlotFaction(openFactionDropdown.slotIndex, fIndex)}
                  className={`group relative p-2 rounded-xl border text-left transition-all cursor-pointer flex flex-col justify-between overflow-hidden ${
                    isSelected
                      ? 'border-2 shadow-[0_0_14px_rgba(0,229,255,0.35)] scale-[1.02]'
                      : 'border-slate-800/90 hover:border-slate-700 bg-slate-900/60 hover:bg-slate-900/90 hover:scale-[1.01]'
                  }`}
                  style={{
                    borderColor: isSelected ? fColor : undefined,
                    backgroundColor: isSelected ? `${fColor}20` : undefined,
                  }}
                >
                  <div className="flex items-center justify-between gap-1 mb-1">
                    <div className="flex items-center gap-1.5 min-w-0">
                      <span
                        className="w-3 h-3 rounded-full shrink-0 transition-transform group-hover:scale-110"
                        style={{
                          backgroundColor: fColor,
                          boxShadow: `0 0 8px ${fColor}`,
                        }}
                      />
                      <span
                        className="font-black text-xs truncate"
                        style={{ color: isSelected ? '#ffffff' : fColor }}
                      >
                        Phe {fIndex + 1}
                      </span>
                    </div>
                    {isSelected && (
                      <span
                        className="p-0.5 rounded-full text-slate-950 flex items-center justify-center shrink-0"
                        style={{ backgroundColor: fColor }}
                      >
                        <Check className="w-2.5 h-2.5 stroke-[3.5]" />
                      </span>
                    )}
                  </div>

                  <div className="flex items-center justify-between text-[10px] mt-0.5">
                    <span className="text-slate-400 truncate font-medium">
                      {FACTION_SHORT_LABELS[fIndex]}
                    </span>
                    <span
                      className={`px-1 py-0.2 rounded font-bold text-[9px] shrink-0 ${
                        isPlayerSlot
                          ? 'bg-cyan-950/80 border border-cyan-500/60 text-cyan-300'
                          : isAlliedWithPlayer
                          ? 'bg-emerald-950/80 border border-emerald-600/60 text-emerald-300'
                          : 'bg-rose-950/80 border border-rose-600/60 text-rose-300'
                      }`}
                    >
                      {isPlayerSlot ? 'BẠN' : isAlliedWithPlayer ? 'MINH' : 'ĐỊCH'}
                    </span>
                  </div>

                  <div className="text-[9px] text-slate-400 font-mono mt-1 pt-1 border-t border-slate-800/60 flex items-center justify-between">
                    <span>Đang có:</span>
                    <span className={`font-bold ${basesInFaction > 0 ? 'text-slate-200' : 'text-slate-500'}`}>
                      {basesInFaction} căn cứ
                    </span>
                  </div>
                </button>
              );
            })}
          </div>

          {/* Tactical Tip / Helper */}
          <div className="mt-2.5 pt-2 border-t border-slate-800/80 text-[10px] text-slate-400 flex items-center justify-between px-1">
            <span className="text-cyan-400/90 font-mono">⚡ Đủ 8 Phe độc lập</span>
            <span className="text-slate-400 font-mono">Cùng Phe = Đồng minh</span>
          </div>
        </div>
      )}
    </div>
  );
};
