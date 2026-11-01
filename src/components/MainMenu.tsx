import React, { useState, useMemo } from 'react';
import { Play, Users, Crosshair, ChevronDown, Check, Swords } from 'lucide-react';
import { PlayerSlotConfig } from '../bridge/wasmBridge';

interface MainMenuProps {
  onStartGame: (difficulty: string, slots: PlayerSlotConfig[]) => void;
}

type MatchPreset = '1v1' | '2v2' | '3v3' | '4v4' | '3factions' | 'ffa8' | 'custom';

interface SlotUIData {
  index: number;
  name: string;
  location: string;
  active: boolean;
  faction: number;
  isHuman: boolean;
  color: string;
}

const SLOT_LOCATIONS = [
  'Tây-Nam (Gốc dưới trái)',
  'Đông-Bắc (Gốc trên phải)',
  'Tây-Bắc (Gốc trên trái)',
  'Đông-Nam (Gốc dưới phải)',
  'Bắc (Cạnh trên giữa)',
  'Nam (Cạnh dưới giữa)',
  'Tây (Cạnh trái giữa)',
  'Đông (Cạnh phải giữa)',
];

const FACTION_COLORS = [
  '#00e5ff', // Phe 0: Player / Đồng minh (Cyan neon)
  '#f43f5e', // Phe 1: Kẻ địch chính (Rose / Đỏ)
  '#a855f7', // Phe 2: Tím (Arcane)
  '#f59e0b', // Phe 3: Vàng hổ phách
  '#10b981', // Phe 4: Lục bảo
  '#ec4899', // Phe 5: Hồng Magenta
  '#06b6d4', // Phe 6: Xanh biển
  '#eab308', // Phe 7: Vàng Gold
];

export const MainMenu: React.FC<MainMenuProps> = ({ onStartGame }) => {
  const [difficulty, setDifficulty] = useState<string>('normal');
  const [preset, setPreset] = useState<MatchPreset>('4v4');

  // Khởi tạo trạng thái 8 slots
  const [slots, setSlots] = useState<SlotUIData[]>([
    { index: 0, name: 'Chỉ Huy (Bạn)', location: SLOT_LOCATIONS[0], active: true, faction: 0, isHuman: true, color: FACTION_COLORS[0] },
    { index: 1, name: 'AI Đối Thủ 1', location: SLOT_LOCATIONS[1], active: true, faction: 1, isHuman: false, color: FACTION_COLORS[1] },
    { index: 2, name: 'AI Đồng Minh 1', location: SLOT_LOCATIONS[2], active: true, faction: 0, isHuman: false, color: FACTION_COLORS[0] },
    { index: 3, name: 'AI Đối Thủ 2', location: SLOT_LOCATIONS[3], active: true, faction: 1, isHuman: false, color: FACTION_COLORS[1] },
    { index: 4, name: 'AI Đồng Minh 2', location: SLOT_LOCATIONS[4], active: true, faction: 0, isHuman: false, color: FACTION_COLORS[0] },
    { index: 5, name: 'AI Đối Thủ 3', location: SLOT_LOCATIONS[5], active: true, faction: 1, isHuman: false, color: FACTION_COLORS[1] },
    { index: 6, name: 'AI Đồng Minh 3', location: SLOT_LOCATIONS[6], active: true, faction: 0, isHuman: false, color: FACTION_COLORS[0] },
    { index: 7, name: 'AI Đối Thủ 4', location: SLOT_LOCATIONS[7], active: true, faction: 1, isHuman: false, color: FACTION_COLORS[1] },
  ]);

  // Áp dụng Preset nhanh
  const applyPreset = (p: MatchPreset) => {
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
            next[i].faction = (i % 2 === 0) ? 0 : 1;
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
          // Giữ nguyên hiện trạng
          break;
      }
      return next;
    });
  };

  // Toggle active slot
  const toggleSlotActive = (index: number) => {
    if (index === 0) return; // Slot 0 của Player luôn bật
    setPreset('custom');
    setSlots((prev) =>
      prev.map((s) => (s.index === index ? { ...s, active: !s.active } : s))
    );
  };

  // Thay đổi phe của slot
  const cycleSlotFaction = (index: number) => {
    if (index === 0) return; // Slot 0 luôn là Phe 0
    setPreset('custom');
    setSlots((prev) =>
      prev.map((s) =>
        s.index === index ? { ...s, faction: (s.faction + 1) % 8 } : s
      )
    );
  };

  // Thống kê tóm tắt
  const summary = useMemo(() => {
    const activeSlots = slots.filter((s) => s.active);
    const allies = activeSlots.filter((s) => s.index !== 0 && s.faction === 0).length;
    const enemies = activeSlots.filter((s) => s.faction !== 0).length;
    const factionsCount = new Set(activeSlots.map((s) => s.faction)).size;
    return { total: activeSlots.length, allies, enemies, factionsCount };
  }, [slots]);

  const handleStart = () => {
    const payloadSlots: PlayerSlotConfig[] = slots.map((s) => ({
      slot_index: s.index,
      active: s.active,
      faction: s.faction,
      is_human: s.isHuman,
    }));
    onStartGame(difficulty, payloadSlots);
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-[#030d1a] text-slate-100 font-mono p-4 overflow-y-auto">
      {/* Background blueprint grid */}
      <div className="fixed inset-0 bg-[linear-gradient(to_right,#0a223e_1px,transparent_1px),linear-gradient(to_bottom,#0a223e_1px,transparent_1px)] bg-[size:4rem_4rem] [mask-image:radial-gradient(ellipse_60%_50%_at_50%_50%,#000_70%,transparent_100%)] opacity-70 pointer-events-none" />

      <div className="relative z-10 max-w-4xl w-full flex flex-col items-center text-center p-6 md:p-8 border-2 border-cyan-500/60 rounded-2xl bg-slate-950/90 shadow-[0_0_60px_rgba(0,229,255,0.25)] my-auto">
        {/* Badge header */}
        <div className="flex items-center gap-2 px-3.5 py-1 rounded-full bg-cyan-950/80 border border-cyan-500 text-cyan-400 text-xs font-bold tracking-widest uppercase mb-3 shadow-sm">
          <Crosshair className="w-3.5 h-3.5 animate-spin" /> Battlefield Blueprint // Grand Tactical RTS
        </div>

        <h1 className="text-3xl md:text-5xl font-black tracking-tight text-white mb-2 drop-shadow-[0_0_20px_rgba(0,229,255,0.6)]">
          BATTLEFIELD <span className="text-cyan-400">BLUEPRINT</span>
        </h1>
        <p className="text-xs md:text-sm text-slate-400 max-w-2xl mb-6 leading-relaxed">
          Bản đồ mở rộng quy mô lớn <span className="text-cyan-300 font-bold">4800 x 4800</span>. Hỗ trợ tối đa{' '}
          <span className="text-amber-300 font-bold">8 Người chơi</span> với tùy chọn Đồng minh, Kẻ địch và Chia phe chiến thuật đa dạng!
        </p>

        {/* Cụm tùy chọn Kịch bản nhanh (Presets) */}
        <div className="w-full mb-6 text-left">
          <div className="flex items-center justify-between mb-2">
            <label className="text-xs font-bold text-slate-300 uppercase tracking-wider flex items-center gap-2">
              <Swords className="w-4 h-4 text-cyan-400" /> Chế độ chơi & Thiết lập trận đấu:
            </label>
            <span className="text-[11px] text-cyan-400 bg-cyan-950/60 border border-cyan-800/80 px-2 py-0.5 rounded">
              {summary.total} Người chơi • {summary.allies} Đồng minh • {summary.enemies} Kẻ địch • {summary.factionsCount} Phe
            </span>
          </div>

          <div className="grid grid-cols-3 md:grid-cols-6 gap-2">
            {[
              { id: '1v1', label: '1 vs 1', desc: 'Đối đầu' },
              { id: '2v2', label: '2 vs 2', desc: 'Song đấu' },
              { id: '3v3', label: '3 vs 3', desc: 'Chiến đoàn' },
              { id: '4v4', label: '4 vs 4', desc: 'Đại chiến 8P' },
              { id: '3factions', label: '2v2v2', desc: 'Tam giác phe' },
              { id: 'ffa8', label: '8-FFA', desc: 'Hỗn chiến 8 phe' },
            ].map((p) => (
              <button
                key={p.id}
                onClick={() => applyPreset(p.id as MatchPreset)}
                className={`py-2 px-2.5 rounded-xl border text-center transition-all cursor-pointer flex flex-col items-center justify-center ${
                  preset === p.id
                    ? 'bg-cyan-950/90 border-cyan-400 text-cyan-300 shadow-[0_0_12px_rgba(0,229,255,0.35)] scale-[1.02]'
                    : 'bg-slate-900/60 border-slate-800 text-slate-400 hover:border-slate-700 hover:text-slate-200'
                }`}
              >
                <span className="text-xs font-bold">{p.label}</span>
                <span className="text-[10px] text-slate-500">{p.desc}</span>
              </button>
            ))}
          </div>
        </div>

        {/* Danh sách 8 Slots Roster */}
        <div className="w-full bg-slate-900/50 p-3.5 rounded-xl border border-slate-800/80 mb-6 text-left">
          <div className="flex items-center justify-between mb-2.5">
            <span className="text-xs font-bold text-slate-300 flex items-center gap-1.5">
              <Users className="w-4 h-4 text-amber-400" /> Bảng phân bố 8 Căn cứ trên Bản đồ:
            </span>
            <span className="text-[10px] text-slate-400 italic">
              * Click để bật/tắt slot hoặc click đổi Phe của AI
            </span>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-2 text-xs">
            {slots.map((slot) => {
              const isPlayer = slot.index === 0;
              const isAlly = !isPlayer && slot.faction === 0;
              const factionColor = FACTION_COLORS[slot.faction % FACTION_COLORS.length];

              return (
                <div
                  key={slot.index}
                  className={`p-2.5 rounded-lg border flex items-center justify-between transition-all ${
                    !slot.active
                      ? 'bg-slate-950/40 border-slate-900 opacity-40'
                      : isPlayer
                      ? 'bg-cyan-950/40 border-cyan-500/70 shadow-[0_0_8px_rgba(0,229,255,0.2)]'
                      : isAlly
                      ? 'bg-emerald-950/40 border-emerald-500/60'
                      : 'bg-slate-950/70 border-slate-800'
                  }`}
                >
                  <div className="flex items-center gap-2.5 min-w-0">
                    {/* Checkbox toggle slot */}
                    <button
                      onClick={() => toggleSlotActive(slot.index)}
                      disabled={isPlayer}
                      className={`w-5 h-5 rounded flex items-center justify-center border transition-all ${
                        isPlayer
                          ? 'bg-cyan-500 border-cyan-400 text-slate-950 cursor-not-allowed'
                          : slot.active
                          ? 'bg-cyan-600 border-cyan-400 text-white cursor-pointer'
                          : 'bg-slate-900 border-slate-700 cursor-pointer'
                      }`}
                    >
                      {slot.active && <Check className="w-3.5 h-3.5 stroke-[3]" />}
                    </button>

                    <div className="truncate">
                      <div className="flex items-center gap-2">
                        <span className="font-bold text-slate-200">
                          Slot {slot.index + 1}: {slot.name}
                        </span>
                        {isPlayer && (
                          <span className="bg-cyan-500 text-slate-950 text-[9px] px-1.5 py-0.2 rounded font-black">
                            BẠN
                          </span>
                        )}
                        {isAlly && slot.active && (
                          <span className="bg-emerald-500 text-slate-950 text-[9px] px-1.5 py-0.2 rounded font-black">
                            ĐỒNG MINH
                          </span>
                        )}
                      </div>
                      <div className="text-[10px] text-slate-500">{slot.location}</div>
                    </div>
                  </div>

                  {/* Faction selector badge */}
                  {slot.active ? (
                    <button
                      onClick={() => cycleSlotFaction(slot.index)}
                      disabled={isPlayer}
                      className="px-2 py-1 rounded text-[11px] font-bold border transition-all cursor-pointer flex items-center gap-1.5 shrink-0"
                      style={{
                        borderColor: factionColor,
                        backgroundColor: `${factionColor}22`,
                        color: factionColor,
                      }}
                      title={isPlayer ? 'Phe của bạn cố định là Phe 1 (Allies)' : 'Click để đổi Phe'}
                    >
                      <span
                        className="w-2 h-2 rounded-full inline-block"
                        style={{ backgroundColor: factionColor }}
                      />
                      Phe {slot.faction + 1}
                      {!isPlayer && <ChevronDown className="w-3 h-3 opacity-70" />}
                    </button>
                  ) : (
                    <span className="text-[10px] text-slate-600 italic">Trống</span>
                  )}
                </div>
              );
            })}
          </div>
        </div>

        {/* Cấu hình độ khó AI */}
        <div className="w-full mb-6 text-left">
          <label className="text-xs font-semibold text-slate-400 uppercase tracking-wider block mb-2">
            Độ khó trí tuệ nhân tạo (AI Difficulty):
          </label>
          <div className="grid grid-cols-3 gap-2">
            {[
              { id: 'easy', label: 'TÂN BINH (EASY)' },
              { id: 'normal', label: 'TIÊU CHUẨN (NORMAL)' },
              { id: 'hard', label: 'CHUYÊN GIA (HARD)' },
            ].map((diff) => (
              <button
                key={diff.id}
                onClick={() => setDifficulty(diff.id)}
                className={`py-2 px-3 rounded text-xs font-bold transition-all border cursor-pointer ${
                  difficulty === diff.id
                    ? 'bg-cyan-950/80 border-cyan-400 text-cyan-300 shadow-[0_0_10px_rgba(0,229,255,0.3)]'
                    : 'bg-slate-900/40 border-slate-800 text-slate-400 hover:border-slate-700'
                }`}
              >
                {diff.label}
              </button>
            ))}
          </div>
        </div>

        {/* Nút Bắt đầu trận đấu */}
        <button
          onClick={handleStart}
          className="flex items-center justify-center gap-2.5 w-full py-4 rounded-xl bg-cyan-500 hover:bg-cyan-400 text-slate-950 font-black text-base tracking-wider uppercase shadow-[0_0_30px_rgba(0,229,255,0.4)] transition-all transform hover:scale-[1.01] active:scale-[0.99] cursor-pointer"
        >
          <Play className="w-5 h-5 fill-current" /> XUẤT QUÂN // BẮT ĐẦU CHIẾN TRƯỜNG ĐA PHE
        </button>
      </div>
    </div>
  );
};
