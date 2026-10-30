import React from 'react';
import { X, Volume2, Compass, Play, RotateCcw, Grid, Home } from 'lucide-react';
import { soundEngine } from '../audio/soundEngine';

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  soundVolume: number;
  cameraSpeed: number;
  autoPause: boolean;
  showGrid: boolean;
  onUpdateSettings: (settings: {
    soundVolume: number;
    cameraSpeed: number;
    autoPause: boolean;
    showGrid: boolean;
  }) => void;
  onRestartGame: () => void;
  onBackToMenu?: () => void;
}

export const SettingsModal: React.FC<SettingsModalProps> = ({
  isOpen,
  onClose,
  soundVolume,
  cameraSpeed,
  autoPause,
  showGrid,
  onUpdateSettings,
  onRestartGame,
  onBackToMenu,
}) => {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm p-4">
      <div className="bg-[#030d1a] border-2 border-slate-700 rounded-xl w-full max-w-md shadow-2xl flex flex-col font-mono text-slate-200">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-slate-800 bg-slate-900/40">
          <h2 className="text-base font-bold text-cyan-400 tracking-wider">GAME SETTINGS // CÀI ĐẶT</h2>
          <button onClick={onClose} className="p-1 hover:bg-slate-800 rounded text-slate-400 hover:text-white">
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6 flex flex-col gap-5 text-sm">
          {/* Âm lượng */}
          <div>
            <div className="flex items-center justify-between mb-2">
              <span className="flex items-center gap-2 text-slate-300">
                <Volume2 className="w-4 h-4 text-cyan-400" /> Âm lượng hiệu ứng:
              </span>
              <span className="text-cyan-400 font-bold">{Math.round(soundVolume * 100)}%</span>
            </div>
            <input
              type="range"
              min="0"
              max="1"
              step="0.05"
              value={soundVolume}
              onChange={(e) => {
                const val = parseFloat(e.target.value);
                soundEngine.volume = val;
                onUpdateSettings({ soundVolume: val, cameraSpeed, autoPause, showGrid });
              }}
              className="w-full accent-cyan-400 cursor-pointer"
            />
          </div>

          {/* Tốc độ Camera */}
          <div>
            <div className="flex items-center justify-between mb-2">
              <span className="flex items-center gap-2 text-slate-300">
                <Compass className="w-4 h-4 text-cyan-400" /> Tốc độ di chuyển Camera:
              </span>
              <span className="text-cyan-400 font-bold">{cameraSpeed.toFixed(1)}x</span>
            </div>
            <input
              type="range"
              min="0.5"
              max="2.5"
              step="0.1"
              value={cameraSpeed}
              onChange={(e) => {
                onUpdateSettings({
                  soundVolume,
                  cameraSpeed: parseFloat(e.target.value),
                  autoPause,
                  showGrid,
                });
              }}
              className="w-full accent-cyan-400 cursor-pointer"
            />
          </div>

          {/* Tự động tạm dừng khi xem Tech Tree */}
          <div className="flex items-center justify-between pt-2 border-t border-slate-800">
            <div>
              <span className="text-slate-300 block">Tạm dừng trận đấu khi xem Cây Công Nghệ [T]:</span>
              <span className="text-[11px] text-slate-500">
                {autoPause
                  ? 'Đang bật: Trận đấu đóng băng để suy nghĩ chiến thuật.'
                  : 'Đang tắt: Trận đấu & nghiên cứu cập nhật liên tục thời gian thực.'}
              </span>
            </div>
            <input
              type="checkbox"
              checked={autoPause}
              onChange={(e) => {
                onUpdateSettings({
                  soundVolume,
                  cameraSpeed,
                  autoPause: e.target.checked,
                  showGrid,
                });
              }}
              className="w-4 h-4 accent-cyan-400 cursor-pointer shrink-0 ml-3"
            />
          </div>

          {/* Hiển thị lưới Blueprint */}
          <div className="flex items-center justify-between pt-2 border-t border-slate-800">
            <span className="flex items-center gap-2 text-slate-300">
              <Grid className="w-4 h-4 text-cyan-400" /> Hiển thị lưới tọa độ Blueprint:
            </span>
            <input
              type="checkbox"
              checked={showGrid}
              onChange={(e) => {
                onUpdateSettings({
                  soundVolume,
                  cameraSpeed,
                  autoPause,
                  showGrid: e.target.checked,
                });
              }}
              className="w-4 h-4 accent-cyan-400 cursor-pointer"
            />
          </div>

          {/* Hướng dẫn phím tắt */}
          <div className="bg-slate-900/60 p-3 rounded border border-slate-800 text-xs text-slate-400 leading-relaxed">
            <p className="text-cyan-300 font-semibold mb-1">🎮 PHÍM TẮT CHIẾN THUẬT:</p>
            <p>• <span className="text-slate-200">WASD / Mũi tên:</span> Di chuyển góc nhìn Camera</p>
            <p>• <span className="text-slate-200">Cuộn chuột:</span> Phóng to / Thu nhỏ chiến trường</p>
            <p>• <span className="text-slate-200">Kéo chuột trái:</span> Quét chọn nhiều đơn vị lính</p>
            <p>• <span className="text-slate-200">Chuột phải:</span> Di chuyển / Khai mỏ / Tấn công mục tiêu</p>
            <p>• <span className="text-slate-200">Phím T:</span> Bật sơ đồ Cây Công Nghệ</p>
            <p>• <span className="text-slate-200">Phím Space:</span> Tạm dừng / Tiếp tục trận đấu</p>
          </div>
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-slate-800 bg-slate-950 flex items-center justify-between gap-2 flex-wrap">
          <div className="flex items-center gap-2">
            <button
              onClick={() => {
                if (confirm('Bạn có chắc muốn chơi lại từ đầu trận này?')) {
                  onRestartGame();
                  onClose();
                }
              }}
              className="flex items-center gap-1.5 px-3 py-2 bg-rose-950/60 hover:bg-rose-900 border border-rose-800 text-rose-300 rounded text-xs font-bold transition-colors cursor-pointer"
            >
              <RotateCcw className="w-4 h-4" /> CHƠI LẠI TRẬN
            </button>
            {onBackToMenu && (
              <button
                onClick={() => {
                  if (confirm('Bạn có muốn thoát về Menu chính không?')) {
                    onBackToMenu();
                    onClose();
                  }
                }}
                className="flex items-center gap-1.5 px-3 py-2 bg-slate-900 hover:bg-slate-800 border border-slate-700 text-slate-300 rounded text-xs font-bold transition-colors cursor-pointer"
              >
                <Home className="w-4 h-4" /> VỀ MENU
              </button>
            )}
          </div>
          <button
            onClick={onClose}
            className="flex items-center gap-1.5 px-4 py-2 bg-cyan-600 hover:bg-cyan-500 text-white rounded text-xs font-bold transition-colors cursor-pointer"
          >
            <Play className="w-4 h-4" /> TIẾP TỤC
          </button>
        </div>
      </div>
    </div>
  );
};

