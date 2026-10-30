import React, { useState } from 'react';
import { Play, Shield, Cpu, Zap, Crosshair, Sliders } from 'lucide-react';

interface MainMenuProps {
  onStartGame: (difficulty: string) => void;
  onOpenConfig?: () => void;
}

export const MainMenu: React.FC<MainMenuProps> = ({ onStartGame, onOpenConfig }) => {
  const [difficulty, setDifficulty] = useState<string>('normal');

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-[#030d1a] text-slate-100 font-mono p-4">
      {/* Blueprint Grid Background Pattern */}
      <div className="absolute inset-0 bg-[linear-gradient(to_right,#0a223e_1px,transparent_1px),linear-gradient(to_bottom,#0a223e_1px,transparent_1px)] bg-[size:4rem_4rem] [mask-image:radial-gradient(ellipse_60%_50%_at_50%_50%,#000_70%,transparent_100%)] opacity-70 pointer-events-none" />

      <div className="relative z-10 max-w-xl w-full flex flex-col items-center text-center p-8 border-2 border-cyan-500/60 rounded-2xl bg-slate-950/80 shadow-[0_0_60px_rgba(0,229,255,0.2)]">
        {/* Logo Badge */}
        <div className="flex items-center gap-2 px-3 py-1 rounded-full bg-cyan-950/80 border border-cyan-500 text-cyan-400 text-xs font-bold tracking-widest uppercase mb-4 shadow-sm">
          <Crosshair className="w-3.5 h-3.5 animate-spin" /> Tactical RTS Engine // Wasm + React
        </div>

        <h1 className="text-4xl md:text-5xl font-black tracking-tight text-white mb-2 drop-shadow-[0_0_20px_rgba(0,229,255,0.6)]">
          BATTLEFIELD <span className="text-cyan-400">BLUEPRINT</span>
        </h1>
        <p className="text-sm text-slate-400 max-w-md mb-8 leading-relaxed">
          Chỉ huy đội quân Người Que viễn tưởng trên bản vẽ chiến thuật. Khai thác tài nguyên, xây dựng công trình, nghiên cứu công nghệ và tiêu diệt căn cứ AI đối phương.
        </p>

        {/* Feature Highlights */}
        <div className="grid grid-cols-3 gap-3 w-full mb-8 text-left">
          <div className="p-3 bg-slate-900/60 rounded border border-slate-800 flex flex-col gap-1.5">
            <Zap className="w-4 h-4 text-cyan-400" />
            <span className="text-xs font-bold text-slate-200">Rust Wasm Core</span>
            <span className="text-[11px] text-slate-400">Toàn bộ mô phỏng lính, sương mù & HUD do Wasm vẽ</span>
          </div>
          <div className="p-3 bg-slate-900/60 rounded border border-slate-800 flex flex-col gap-1.5">
            <Shield className="w-4 h-4 text-emerald-400" />
            <span className="text-xs font-bold text-slate-200">5 Binh chủng</span>
            <span className="text-[11px] text-slate-400">Kỹ sư, Kiếm sĩ, Xạ thủ, Khiên chắn, Pháo Laser</span>
          </div>
          <div className="p-3 bg-slate-900/60 rounded border border-slate-800 flex flex-col gap-1.5">
            <Cpu className="w-4 h-4 text-purple-400" />
            <span className="text-xs font-bold text-slate-200">Cây Công Nghệ</span>
            <span className="text-[11px] text-slate-400">Nâng cấp 3 nhánh trực quan bằng React [Phím T]</span>
          </div>
        </div>

        {/* Difficulty Selection */}
        <div className="w-full mb-8">
          <label className="text-xs font-semibold text-slate-400 uppercase tracking-wider block mb-2 text-left">
            Độ khó AI Đối thủ:
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

        {/* Actions */}
        <div className="flex flex-col gap-3 w-full">
          <button
            onClick={() => onStartGame(difficulty)}
            className="flex items-center justify-center gap-2 w-full py-4 rounded-xl bg-cyan-500 hover:bg-cyan-400 text-slate-950 font-black text-base tracking-wider uppercase shadow-[0_0_30px_rgba(0,229,255,0.4)] transition-all transform hover:scale-[1.02] active:scale-[0.98] cursor-pointer"
          >
            <Play className="w-5 h-5 fill-current" /> XUẤT QUÂN // BẮT ĐẦU TRẬN ĐẤU
          </button>

          {onOpenConfig && (
            <button
              onClick={onOpenConfig}
              className="flex items-center justify-center gap-2 w-full py-2.5 rounded-lg bg-slate-900/80 hover:bg-slate-800 border border-slate-700 hover:border-cyan-500/50 text-cyan-400 font-bold text-xs tracking-wider uppercase transition-all cursor-pointer"
            >
              <Sliders className="w-4 h-4" /> CẤU HÌNH THÔNG SỐ GAME // GAME CONFIG
            </button>
          )}
        </div>
      </div>
    </div>
  );
};

