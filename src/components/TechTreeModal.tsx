import React from 'react';
import { X, Cpu, Shield, Zap, CheckCircle2, Loader2, Lock } from 'lucide-react';
import { TechState } from '../bridge/wasmBridge';

interface TechTreeModalProps {
  isOpen: boolean;
  onClose: () => void;
  techState: TechState | null;
  minerals: number;
  energy: number;
  onResearch: (techId: string) => void;
}

interface TechNode {
  id: string;
  name: string;
  category: 'eco' | 'infantry' | 'advanced';
  minerals: number;
  energy: number;
  time: number;
  desc: string;
  requires?: string;
}

const TECH_NODES: TechNode[] = [
  // Nhánh Kinh tế
  {
    id: 'optical_drills',
    name: 'Mũi Khoan Quang Học',
    category: 'eco',
    minerals: 100,
    energy: 50,
    time: 10,
    desc: 'Kỹ sư khai mỏ nhanh hơn +30%, mang thêm +5 khoáng sản mỗi chuyến.',
  },
  {
    id: 'overcharged_reactors',
    name: 'Lò Phản Ứng Quá Tải',
    category: 'eco',
    minerals: 150,
    energy: 100,
    time: 15,
    requires: 'optical_drills',
    desc: 'Tăng tốc độ sản sinh Năng Lượng từ trạm Power Reactor thêm +50%.',
  },
  {
    id: 'fortified_blueprints',
    name: 'Bản Vẽ Gia Cố Kiến Trúc',
    category: 'eco',
    minerals: 200,
    energy: 150,
    time: 20,
    requires: 'overcharged_reactors',
    desc: 'Tăng +30% lượng Máu tối đa và độ bền cho toàn bộ Công trình & Tháp pháo.',
  },

  // Nhánh Bộ binh
  {
    id: 'energy_shielding',
    name: 'Khiên Năng Lượng Tác Chiến',
    category: 'infantry',
    minerals: 150,
    energy: 100,
    time: 12,
    desc: 'Mở khóa đơn vị Vệ Binh Khiên Chắn (Vanguard) và tăng +20% HP cho Kiếm Sĩ.',
  },
  {
    id: 'magnetic_rounds',
    name: 'Đạn Gia Tốc Từ Trường',
    category: 'infantry',
    minerals: 200,
    energy: 150,
    time: 15,
    requires: 'energy_shielding',
    desc: 'Tăng +25% Sát thương và mở rộng tầm bắn cho Xạ Thủ Súng Trường.',
  },
  {
    id: 'exo_boosters',
    name: 'Bộ Phản Lực Chân Exo-Boost',
    category: 'infantry',
    minerals: 250,
    energy: 200,
    time: 20,
    requires: 'magnetic_rounds',
    desc: 'Tăng +20% Tốc độ di chuyển chiến thuật cho toàn bộ Binh chủng Người que.',
  },

  // Nhánh Công nghệ cao
  {
    id: 'focused_laser',
    name: 'Tia Laser Hội Tụ Quang Năng',
    category: 'advanced',
    minerals: 300,
    energy: 250,
    time: 25,
    desc: 'Mở khóa đơn vị Pháo Thủ Laser Tầm Xa (Laser Marksman) với hỏa lực cực mạnh.',
  },
  {
    id: 'superheated_blades',
    name: 'Lưỡi Kiếm Plasma Quá Nhiệt',
    category: 'advanced',
    minerals: 350,
    energy: 300,
    time: 30,
    requires: 'focused_laser',
    desc: 'Đòn chém của Kiếm Sĩ Xung Kích phá vỡ giáp đối phương, tăng +35% sát thương.',
  },
  {
    id: 'overload_strike',
    name: 'Tụ Điện Quá Tải Hủy Diệt',
    category: 'advanced',
    minerals: 400,
    energy: 350,
    time: 35,
    requires: 'superheated_blades',
    desc: 'Tăng +40% tốc độ bắn cho Tháp Pháo Phòng Thủ và Pháo Thủ Laser.',
  },
];

export const TechTreeModal: React.FC<TechTreeModalProps> = ({
  isOpen,
  onClose,
  techState,
  minerals,
  energy,
  onResearch,
}) => {
  if (!isOpen) return null;

  const isUnlocked = (id: string): boolean => {
    if (!techState) return false;
    return !!(techState as any)[id];
  };

  const isResearching = (id: string): boolean => {
    return techState?.current_research === id;
  };

  const canResearch = (node: TechNode): boolean => {
    if (isUnlocked(node.id)) return false;
    if (techState?.current_research) return false;
    if (node.requires && !isUnlocked(node.requires)) return false;
    if (minerals < node.minerals || energy < node.energy) return false;
    return true;
  };

  const renderColumn = (category: 'eco' | 'infantry' | 'advanced', title: string, icon: any, color: string) => {
    const nodes = TECH_NODES.filter((n) => n.category === category);

    return (
      <div className="flex-1 flex flex-col gap-4 bg-slate-900/60 p-4 rounded-lg border border-slate-700/60">
        <div className={`flex items-center gap-2 pb-2 border-b border-slate-700 font-bold ${color}`}>
          {icon}
          <span>{title}</span>
        </div>

        <div className="flex flex-col gap-4">
          {nodes.map((node) => {
            const unlocked = isUnlocked(node.id);
            const researching = isResearching(node.id);
            const available = canResearch(node);
            const lockedByReq = node.requires && !isUnlocked(node.requires);

            return (
              <div
                key={node.id}
                className={`p-3.5 rounded border transition-all ${
                  unlocked
                    ? 'bg-cyan-950/40 border-cyan-500/60 shadow-[0_0_12px_rgba(0,229,255,0.2)]'
                    : researching
                    ? 'bg-amber-950/40 border-amber-500/80 animate-pulse'
                    : lockedByReq
                    ? 'bg-slate-950/50 border-slate-800 opacity-50'
                    : 'bg-slate-800/50 border-slate-700 hover:border-cyan-400/50'
                }`}
              >
                <div className="flex items-start justify-between">
                  <div className="font-semibold text-sm text-slate-100 flex items-center gap-1.5">
                    {unlocked ? (
                      <CheckCircle2 className="w-4 h-4 text-cyan-400" />
                    ) : researching ? (
                      <Loader2 className="w-4 h-4 text-amber-400 animate-spin" />
                    ) : lockedByReq ? (
                      <Lock className="w-4 h-4 text-slate-500" />
                    ) : null}
                    <span>{node.name}</span>
                  </div>
                </div>

                <p className="text-xs text-slate-400 mt-1.5 leading-relaxed">{node.desc}</p>

                <div className="flex items-center justify-between mt-3 pt-2 border-t border-slate-700/50 text-xs">
                  <div className="flex items-center gap-2 font-mono">
                    <span className="text-cyan-400">💎 {node.minerals}</span>
                    <span className="text-purple-400">⚡ {node.energy}</span>
                    <span className="text-slate-400">⏱ {node.time}s</span>
                  </div>

                  {unlocked ? (
                    <span className="text-xs font-mono text-cyan-400 font-bold tracking-wider">ĐÃ NÂNG CẤP</span>
                  ) : researching ? (
                    <div className="flex items-center gap-1.5 text-xs text-amber-400 font-mono">
                      <span>{( (techState?.research_progress || 0) * 100 ).toFixed(0)}%</span>
                    </div>
                  ) : (
                    <button
                      onClick={() => onResearch(node.id)}
                      disabled={!available}
                      className={`px-3 py-1 rounded text-xs font-mono font-bold transition-all ${
                        available
                          ? 'bg-cyan-600 hover:bg-cyan-500 text-white shadow-md cursor-pointer'
                          : 'bg-slate-800 text-slate-500 cursor-not-allowed'
                      }`}
                    >
                      NGHIÊN CỨU
                    </button>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      </div>
    );
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm p-4">
      <div className="bg-[#030d1a] border-2 border-cyan-500/80 rounded-xl w-full max-w-5xl shadow-[0_0_40px_rgba(0,229,255,0.25)] flex flex-col max-h-[90vh]">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-cyan-900/60 bg-slate-900/40">
          <div className="flex items-center gap-3">
            <Cpu className="w-6 h-6 text-cyan-400" />
            <div>
              <h2 className="text-lg font-bold font-mono text-cyan-300 tracking-wider">
                TACTICAL TECH MATRIX // CÂY CÔNG NGHỆ CHIẾN THUẬT
              </h2>
              <p className="text-xs text-slate-400">Mở khóa nâng cấp trực tiếp trong trận đấu (Phím tắt: T)</p>
            </div>
          </div>
          <div className="flex items-center gap-4">
            <div className="flex items-center gap-3 bg-slate-950 px-3 py-1.5 rounded border border-slate-800 font-mono text-xs">
              <span className="text-cyan-400 font-bold">💎 {minerals.toFixed(0)}</span>
              <span className="text-purple-400 font-bold">⚡ {energy.toFixed(0)}</span>
            </div>
            <button
              onClick={onClose}
              className="p-1.5 text-slate-400 hover:text-white rounded hover:bg-slate-800 transition-colors"
            >
              <X className="w-5 h-5" />
            </button>
          </div>
        </div>

        {/* 3 Cột Công Nghệ */}
        <div className="flex-1 overflow-y-auto p-6 flex flex-col md:flex-row gap-5">
          {renderColumn('eco', 'KINH TẾ & CÔNG TRÌNH', <Zap className="w-4 h-4 text-emerald-400" />, 'text-emerald-400')}
          {renderColumn('infantry', 'BỘ BINH & VŨ TRANG', <Shield className="w-4 h-4 text-cyan-400" />, 'text-cyan-400')}
          {renderColumn('advanced', 'CÔNG NGHỆ CAO & LASER', <Cpu className="w-4 h-4 text-purple-400" />, 'text-purple-400')}
        </div>

        {/* Footer */}
        <div className="px-6 py-3 border-t border-cyan-900/40 bg-slate-950/80 flex justify-between items-center text-xs text-slate-400 font-mono">
          <span>* Các nâng cấp có hiệu lực tức thì lên toàn bộ đơn vị và công trình tương ứng.</span>
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

