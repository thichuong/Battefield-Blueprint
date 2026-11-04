import React, { useState } from 'react';
import { X, Volume2, Compass, Play, RotateCcw, Grid, Home, Gauge } from 'lucide-react';
import { soundEngine } from '../audio/soundEngine';
import { ConfirmModal } from './ConfirmModal';

export type FpsLimit = 'vsync' | 60 | 90 | 120 | 240 | 'unlimited';

const FPS_OPTIONS: { label: string; value: FpsLimit }[] = [
  { label: 'V-Sync', value: 'vsync' },
  { label: '60 FPS', value: 60 },
  { label: '90 FPS', value: 90 },
  { label: '120 FPS', value: 120 },
  { label: '240 FPS', value: 240 },
  { label: 'Bỏ giới hạn', value: 'unlimited' },
];

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  soundVolume: number;
  cameraSpeed: number;
  autoPause: boolean;
  showGrid: boolean;
  fpsLimit?: FpsLimit;
  isInGame?: boolean;
  onUpdateSettings: (settings: {
    soundVolume: number;
    cameraSpeed: number;
    autoPause: boolean;
    showGrid: boolean;
    fpsLimit: FpsLimit;
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
  fpsLimit = 'vsync',
  isInGame = true,
  onUpdateSettings,
  onRestartGame,
  onBackToMenu,
}) => {
  const [confirmAction, setConfirmAction] = useState<'restart' | 'menu' | null>(null);

  if (!isOpen) return null;

  const handleClose = () => {
    setConfirmAction(null);
    onClose();
  };

  return (
    <>
      <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm p-4">
        <div className="bg-[#030d1a] border-2 border-slate-700 rounded-xl w-full max-w-md shadow-2xl flex flex-col font-mono text-slate-200">
          {/* Header */}
          <div className="flex items-center justify-between px-6 py-4 border-b border-slate-800 bg-slate-900/40">
            <h2 className="text-base font-bold text-cyan-400 tracking-wider">GAME SETTINGS // CÀI ĐẶT</h2>
            <button
              onClick={() => {
                soundEngine.playSound('click');
                handleClose();
              }}
              className="p-1 hover:bg-slate-800 rounded text-slate-400 hover:text-white transition-colors cursor-pointer"
            >
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
                onUpdateSettings({ soundVolume: val, cameraSpeed, autoPause, showGrid, fpsLimit });
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
                  fpsLimit,
                });
              }}
              className="w-full accent-cyan-400 cursor-pointer"
            />
          </div>

          {/* Giới hạn FPS */}
          <div className="pt-2 border-t border-slate-800">
            <div className="flex items-center justify-between mb-2">
              <span className="flex items-center gap-2 text-slate-300">
                <Gauge className="w-4 h-4 text-cyan-400" /> Giới hạn FPS:
              </span>
              <span className="text-cyan-400 font-bold text-xs uppercase">
                {fpsLimit === 'vsync'
                  ? 'V-Sync'
                  : fpsLimit === 'unlimited'
                  ? 'Bỏ giới hạn'
                  : `${fpsLimit} FPS`}
              </span>
            </div>
            <div className="grid grid-cols-3 gap-1.5 sm:grid-cols-6">
              {FPS_OPTIONS.map((opt) => {
                const isActive = fpsLimit === opt.value;
                return (
                  <button
                    key={String(opt.value)}
                    type="button"
                    onClick={() => {
                      onUpdateSettings({
                        soundVolume,
                        cameraSpeed,
                        autoPause,
                        showGrid,
                        fpsLimit: opt.value,
                      });
                    }}
                    className={`py-1.5 px-1 rounded text-[11px] font-bold transition-all border text-center ${
                      isActive
                        ? 'bg-cyan-500/25 border-cyan-400 text-cyan-300 shadow-[0_0_10px_rgba(6,182,212,0.35)]'
                        : 'bg-slate-900/60 border-slate-700/80 text-slate-400 hover:text-slate-200 hover:border-slate-500'
                    }`}
                  >
                    {opt.label}
                  </button>
                );
              })}
            </div>
            <p className="text-[11px] text-slate-500 mt-1.5 leading-tight">
              {fpsLimit === 'vsync' && 'Đồng bộ trực tiếp theo tần số quét màn hình, chống xé hình.'}
              {fpsLimit === 60 && 'Khóa tối đa 60 FPS để tiết kiệm pin và giảm tải GPU.'}
              {fpsLimit === 90 && 'Mức khung hình cân bằng, mượt mà cho màn hình 90Hz+.'}
              {fpsLimit === 120 && 'Tốc độ khung hình cao, mượt mà dành cho màn hình 120Hz/144Hz.'}
              {fpsLimit === 240 && 'Tối ưu cho màn hình gaming tần số quét siêu cao 240Hz+.'}
              {fpsLimit === 'unlimited' && 'Không giới hạn FPS, dựng hình tối đa công suất phần cứng.'}
            </p>
          </div>

          {/* Tự động tạm dừng khi mở bảng */}
          <div className="flex items-center justify-between pt-2 border-t border-slate-800">
            <div>
              <span className="text-slate-200 font-medium block">Tạm dừng game khi mở bảng</span>
              <span className="text-[11px] text-slate-500">
                {autoPause
                  ? 'Tự động đóng băng trận đấu khi mở Cây công nghệ, Cài đặt hoặc Hướng dẫn.'
                  : 'Trận đấu tiếp tục diễn ra theo thời gian thực.'}
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
                  fpsLimit,
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
                  fpsLimit,
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
            {isInGame && (
              <span className="flex items-center gap-2 mr-3 font-mono text-xs">
                <span
                  className={`w-2.5 h-2.5 rounded-full ${
                    autoPause ? 'bg-cyan-400 animate-pulse' : 'bg-emerald-400 animate-ping'
                  }`}
                />
                <span className={autoPause ? 'text-cyan-300 font-semibold' : 'text-emerald-300 font-semibold'}>
                  {autoPause ? 'Trận đấu đang tạm dừng' : 'Thời gian đang chạy'}
                </span>
              </span>
            )}
            {isInGame && (
              <button
                type="button"
                onClick={() => {
                  soundEngine.playSound('click');
                  setConfirmAction('restart');
                }}
                className="group flex items-center gap-2 px-3.5 py-2 bg-gradient-to-r from-rose-950/90 via-rose-900/70 to-rose-950/90 hover:from-rose-900 hover:to-rose-800 active:scale-95 border border-rose-700/80 hover:border-rose-400 text-rose-300 hover:text-white rounded-lg text-xs font-bold tracking-wide transition-all shadow-[0_0_12px_rgba(244,63,94,0.2)] hover:shadow-[0_0_20px_rgba(244,63,94,0.45)] cursor-pointer"
              >
                <RotateCcw className="w-4 h-4 text-rose-400 group-hover:text-rose-200 transition-transform duration-300 group-hover:-rotate-90" />
                <span>CHƠI LẠI TRẬN</span>
              </button>
            )}
            {isInGame && onBackToMenu && (
              <button
                type="button"
                onClick={() => {
                  soundEngine.playSound('click');
                  setConfirmAction('menu');
                }}
                className="group flex items-center gap-2 px-3.5 py-2 bg-gradient-to-r from-slate-900/90 via-slate-800/80 to-slate-900/90 hover:from-slate-800 hover:to-slate-700 active:scale-95 border border-slate-700/90 hover:border-cyan-500/80 text-slate-300 hover:text-cyan-200 rounded-lg text-xs font-bold tracking-wide transition-all shadow-[0_0_12px_rgba(15,23,42,0.4)] hover:shadow-[0_0_18px_rgba(6,182,212,0.3)] cursor-pointer"
              >
                <Home className="w-4 h-4 text-slate-400 group-hover:text-cyan-300 transition-transform duration-200 group-hover:scale-110" />
                <span>VỀ MENU</span>
              </button>
            )}
          </div>
          <button
            type="button"
            onClick={() => {
              soundEngine.playSound('click');
              handleClose();
            }}
            className="flex items-center gap-1.5 px-4 py-2 bg-cyan-600 hover:bg-cyan-500 active:scale-95 text-white rounded-lg text-xs font-bold transition-all shadow-[0_0_12px_rgba(6,182,212,0.3)] hover:shadow-[0_0_18px_rgba(6,182,212,0.5)] cursor-pointer"
          >
            {isInGame ? (
              <>
                <Play className="w-4 h-4" /> TIẾP TỤC
              </>
            ) : (
              'ĐÓNG'
            )}
          </button>
        </div>
      </div>
    </div>

    {/* Bảng thông báo xác nhận tái sử dụng cho cả 2 thao tác */}
    <ConfirmModal
      isOpen={confirmAction !== null}
      title={
        confirmAction === 'restart'
          ? 'XÁC NHẬN CHƠI LẠI TRẬN'
          : 'XÁC NHẬN VỀ MENU CHÍNH'
      }
      subtitle={
        confirmAction === 'restart'
          ? 'TACTICAL RESET // KHỞI ĐỘNG LẠI'
          : 'ABORT MISSION // RỜI CHIẾN TRƯỜNG'
      }
      message={
        confirmAction === 'restart'
          ? 'Bạn có chắc chắn muốn chơi lại ván này từ đầu không?'
          : 'Bạn có chắc chắn muốn kết thúc trận đấu và quay về Menu chính không?'
      }
      detailNote={
        confirmAction === 'restart'
          ? 'Toàn bộ tài nguyên, công trình, lực lượng quân đội và thời gian trận đấu sẽ được đặt lại về trạng thái ban đầu.'
          : 'Tiến trình tác chiến hiện tại sẽ không được lưu. Trận đấu sẽ kết thúc ngay lập tức.'
      }
      confirmText={confirmAction === 'restart' ? 'CHƠI LẠI NGAY' : 'VỀ MENU CHÍNH'}
      cancelText={confirmAction === 'restart' ? 'TIẾP TỤC TRẬN' : 'Ở LẠI TRẬN ĐẤU'}
      variant={confirmAction === 'restart' ? 'danger' : 'warning'}
      iconType={confirmAction === 'restart' ? 'restart' : 'home'}
      onConfirm={() => {
        if (confirmAction === 'restart') {
          setConfirmAction(null);
          onRestartGame();
          onClose();
        } else if (confirmAction === 'menu') {
          setConfirmAction(null);
          onBackToMenu?.();
          onClose();
        }
      }}
      onCancel={() => {
        setConfirmAction(null);
      }}
    />
  </>
);
};

