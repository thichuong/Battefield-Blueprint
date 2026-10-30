import React from 'react';
import { Trophy, Skull, RotateCcw } from 'lucide-react';

interface GameOverModalProps {
  isOpen: boolean;
  playerWon: boolean;
  gameTime: number;
  onRestart: () => void;
}

export const GameOverModal: React.FC<GameOverModalProps> = ({
  isOpen,
  playerWon,
  gameTime,
  onRestart,
}) => {
  if (!isOpen) return null;

  const minutes = Math.floor(gameTime / 60);
  const seconds = Math.floor(gameTime % 60);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/85 backdrop-blur-md p-4">
      <div
        className={`bg-[#030d1a] border-2 rounded-xl w-full max-w-md p-8 shadow-2xl flex flex-col items-center text-center font-mono ${
          playerWon
            ? 'border-cyan-400 shadow-[0_0_50px_rgba(0,229,255,0.35)]'
            : 'border-rose-500 shadow-[0_0_50px_rgba(244,63,94,0.35)]'
        }`}
      >
        {playerWon ? (
          <>
            <div className="w-16 h-16 rounded-full bg-cyan-950/80 border-2 border-cyan-400 flex items-center justify-center mb-4 text-cyan-300">
              <Trophy className="w-8 h-8" />
            </div>
            <h2 className="text-2xl font-black text-cyan-300 tracking-wider mb-2">CHIẾN THẮNG TUYỆT ĐỐI</h2>
            <p className="text-sm text-slate-300 mb-6">
              Bạn đã tiêu diệt hoàn toàn Căn cứ Chỉ huy của AI đối phương và làm chủ chiến trường Blueprint!
            </p>
          </>
        ) : (
          <>
            <div className="w-16 h-16 rounded-full bg-rose-950/80 border-2 border-rose-500 flex items-center justify-center mb-4 text-rose-400">
              <Skull className="w-8 h-8" />
            </div>
            <h2 className="text-2xl font-black text-rose-500 tracking-wider mb-2">CĂN CỨ BỊ HỦY DIỆT</h2>
            <p className="text-sm text-slate-300 mb-6">
              Trung tâm chỉ huy của bạn đã sụp đổ trước các đợt càn quét của quân địch.
            </p>
          </>
        )}

        <div className="w-full bg-slate-900/80 p-3.5 rounded border border-slate-800 mb-6 text-xs text-slate-400 flex justify-around">
          <div>
            <span>THỜI GIAN TRẬN ĐẤU:</span>
            <div className="text-slate-100 font-bold text-sm mt-0.5">
              {minutes}m {seconds}s
            </div>
          </div>
          <div>
            <span>KẾT QUẢ TÁC CHIẾN:</span>
            <div className={`font-bold text-sm mt-0.5 ${playerWon ? 'text-cyan-400' : 'text-rose-400'}`}>
              {playerWon ? 'VICTORY' : 'DEFEAT'}
            </div>
          </div>
        </div>

        <button
          onClick={onRestart}
          className={`flex items-center justify-center gap-2 w-full py-3 rounded-lg font-bold text-sm transition-all cursor-pointer ${
            playerWon
              ? 'bg-cyan-500 hover:bg-cyan-400 text-slate-950 shadow-lg shadow-cyan-500/30'
              : 'bg-rose-600 hover:bg-rose-500 text-white shadow-lg shadow-rose-600/30'
          }`}
        >
          <RotateCcw className="w-4 h-4" /> CHƠI TRẬN MỚI
        </button>
      </div>
    </div>
  );
};

