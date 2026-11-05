import React, { useState, useEffect } from 'react';
import {
  X,
  Volume2,
  Compass,
  Play,
  RotateCcw,
  Grid,
  Home,
  Gauge,
  Sliders,
  Pause,
  Keyboard,
} from 'lucide-react';
import { soundEngine } from '../audio/soundEngine';
import { ConfirmModal } from './ConfirmModal';

export type FpsLimit = 'vsync' | 60 | 90 | 120 | 240 | 'unlimited';

const FPS_OPTIONS: { label: string; value: FpsLimit; desc: string }[] = [
  { label: 'V-Sync', value: 'vsync', desc: 'Đồng bộ theo tần số quét thực tế của màn hình (tự động quy đổi FPS ngầm), chống xé hình và ổn định khung hình.' },
  { label: '60 FPS', value: 60, desc: 'Khóa tối đa 60 FPS để tiết kiệm pin và giảm tải GPU.' },
  { label: '90 FPS', value: 90, desc: 'Mức khung hình cân bằng, mượt mà cho màn hình 90Hz+.' },
  { label: '120 FPS', value: 120, desc: 'Tốc độ khung hình cao, mượt mà dành cho màn hình 120Hz/144Hz.' },
  { label: '240 FPS', value: 240, desc: 'Tối ưu cho màn hình gaming tần số quét siêu cao 240Hz+.' },
  { label: 'Bỏ giới hạn', value: 'unlimited', desc: 'Không giới hạn FPS, dựng hình tối đa công suất phần cứng.' },
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

  const handleClose = () => {
    setConfirmAction(null);
    onClose();
  };

  // Hỗ trợ phím tắt ESC để đóng bảng cài đặt
  useEffect(() => {
    if (!isOpen) return;
    const handleKeyDown = (e: KeyboardEvent) => {
      if (confirmAction !== null) return; // Nhường phím cho ConfirmModal
      if (e.key === 'Escape') {
        e.preventDefault();
        soundEngine.playSound('click');
        handleClose();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, confirmAction]);

  if (!isOpen) return null;

  const currentFpsDesc =
    FPS_OPTIONS.find((opt) => opt.value === fpsLimit)?.desc ||
    'Đồng bộ tần số quét màn hình.';

  return (
    <>
      <div className="fixed inset-0 z-[60] flex items-center justify-center bg-black/85 backdrop-blur-md p-3 sm:p-5 md:p-6 overflow-y-auto font-mono text-slate-200 select-none">
        {/* Main Widescreen Settings Dialog */}
        <div className="relative bg-[#020813] border-2 border-cyan-500/70 rounded-2xl w-[96vw] max-w-5xl max-h-[92vh] shadow-[0_0_70px_rgba(0,229,255,0.25)] flex flex-col overflow-hidden my-auto">
          {/* =========================================================================
              HEADER BAR: Title, Subtitle, Status & Close
          ========================================================================= */}
          <div className="flex items-center justify-between px-5 sm:px-6 py-3.5 border-b border-cyan-900/70 bg-slate-950/90 shrink-0">
            <div className="flex items-center gap-3">
              <div className="p-2 rounded-xl bg-cyan-950/90 border border-cyan-500/80 text-cyan-400 shadow-[0_0_12px_rgba(0,229,255,0.3)] shrink-0">
                <Sliders className="w-5 h-5 text-cyan-400" />
              </div>
              <div>
                <h2 className="text-base sm:text-lg font-black text-cyan-300 tracking-wider flex items-center gap-2">
                  GAME SETTINGS <span className="text-slate-400 font-normal">// CÀI ĐẶT HỆ THỐNG</span>
                </h2>
                <p className="text-[13px] sm:text-sm text-slate-300 hidden sm:block">
                  Cấu hình hiệu năng GPU, giới hạn FPS, âm lượng, tốc độ camera và cơ chế trận đấu
                </p>
              </div>
            </div>

            <div className="flex items-center gap-3">
              {isInGame && (
                <div className="hidden sm:flex items-center gap-2 bg-slate-900/90 border border-slate-700/80 px-3 py-1.5 rounded-lg text-[13px] sm:text-sm font-mono">
                  <span
                    className={`w-2 h-2 rounded-full ${
                      autoPause ? 'bg-cyan-400 animate-pulse' : 'bg-emerald-400 animate-ping'
                    }`}
                  />
                  <span className={autoPause ? 'text-cyan-300 font-bold' : 'text-emerald-300 font-bold'}>
                    {autoPause ? 'TRẬN ĐẤU TẠM DỪNG' : 'THỜI GIAN ĐANG CHẠY'}
                  </span>
                </div>
              )}

              <button
                type="button"
                onClick={() => {
                  soundEngine.playSound('click');
                  handleClose();
                }}
                className="p-1.5 hover:bg-slate-800 rounded-lg border border-slate-700 hover:border-cyan-400 text-slate-400 hover:text-white transition-all cursor-pointer"
                title="Đóng bảng cài đặt [ESC]"
              >
                <X className="w-5 h-5" />
              </button>
            </div>
          </div>

          {/* =========================================================================
              2-COLUMN HORIZONTAL GRID: Left (Performance & Audio) | Right (Game & Keymap)
          ========================================================================= */}
          <div className="grid grid-cols-1 md:grid-cols-12 gap-4 sm:gap-5 p-4 sm:p-6 flex-1 min-h-0 overflow-y-auto">
            {/* ---------------------------------------------------------------------
                LEFT COLUMN (6 Cols): Audio Controls, Camera Speed & FPS Settings
            --------------------------------------------------------------------- */}
            <div className="md:col-span-6 flex flex-col justify-between gap-4 min-h-0">
              {/* Box 1: Âm lượng & Camera */}
              <div className="bg-slate-900/60 p-4 rounded-xl border border-slate-800/90 space-y-4">
                <div className="text-sm font-bold text-cyan-400 uppercase tracking-wider flex items-center gap-1.5 pb-2 border-b border-slate-800">
                  <Volume2 className="w-4 h-4 text-cyan-400" /> Điều Khiển Âm Thanh & Camera:
                </div>

                {/* Âm lượng hiệu ứng */}
                <div>
                  <div className="flex items-center justify-between mb-1.5">
                    <span className="text-sm text-slate-300 flex items-center gap-1.5">
                      Âm lượng âm thanh tổng:
                    </span>
                    <span className="text-cyan-300 font-bold text-sm bg-cyan-950/80 border border-cyan-800 px-2.5 py-0.5 rounded">
                      {Math.round(soundVolume * 100)}%
                    </span>
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
                    className="w-full accent-cyan-400 cursor-pointer h-2 bg-slate-950 rounded-lg"
                  />
                  {/* Quick Preset Buttons */}
                  <div className="grid grid-cols-4 gap-1.5 mt-2">
                    {[
                      { label: 'MUTE', val: 0 },
                      { label: '30%', val: 0.3 },
                      { label: '70%', val: 0.7 },
                      { label: '100%', val: 1.0 },
                    ].map((btn) => (
                      <button
                        key={btn.label}
                        type="button"
                        onClick={() => {
                          soundEngine.volume = btn.val;
                          soundEngine.playSound('click');
                          onUpdateSettings({ soundVolume: btn.val, cameraSpeed, autoPause, showGrid, fpsLimit });
                        }}
                        className={`py-1.5 text-[13px] font-bold rounded border transition-all ${
                          Math.abs(soundVolume - btn.val) < 0.05
                            ? 'bg-cyan-950 border-cyan-400 text-cyan-300'
                            : 'bg-slate-950/60 border-slate-800 text-slate-300 hover:text-white hover:border-slate-700'
                        }`}
                      >
                        {btn.label}
                      </button>
                    ))}
                  </div>
                </div>

                {/* Tốc độ Camera */}
                <div className="pt-2 border-t border-slate-800/80">
                  <div className="flex items-center justify-between mb-1.5">
                    <span className="text-sm text-slate-300 flex items-center gap-1.5">
                      <Compass className="w-4 h-4 text-cyan-400" /> Tốc độ di chuyển Camera:
                    </span>
                    <span className="text-cyan-300 font-bold text-sm bg-cyan-950/80 border border-cyan-800 px-2.5 py-0.5 rounded">
                      {cameraSpeed.toFixed(1)}x
                    </span>
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
                    className="w-full accent-cyan-400 cursor-pointer h-2 bg-slate-950 rounded-lg"
                  />
                  {/* Camera Speed Presets */}
                  <div className="grid grid-cols-4 gap-1.5 mt-2">
                    {[
                      { label: '0.8x Chậm', val: 0.8 },
                      { label: '1.0x Chuẩn', val: 1.0 },
                      { label: '1.5x Nhanh', val: 1.5 },
                      { label: '2.0x Tốc độ', val: 2.0 },
                    ].map((btn) => (
                      <button
                        key={btn.label}
                        type="button"
                        onClick={() => {
                          soundEngine.playSound('click');
                          onUpdateSettings({
                            soundVolume,
                            cameraSpeed: btn.val,
                            autoPause,
                            showGrid,
                            fpsLimit,
                          });
                        }}
                        className={`py-1.5 px-1 text-[13px] font-bold rounded border transition-all text-center leading-tight ${
                          Math.abs(cameraSpeed - btn.val) < 0.05
                            ? 'bg-cyan-950 border-cyan-400 text-cyan-300'
                            : 'bg-slate-950/60 border-slate-800 text-slate-300 hover:text-white hover:border-slate-700'
                        }`}
                      >
                        {btn.label}
                      </button>
                    ))}
                  </div>
                </div>
              </div>

              {/* Box 2: Giới hạn FPS */}
              <div className="bg-slate-900/60 p-4 rounded-xl border border-slate-800/90 space-y-2.5">
                <div className="flex items-center justify-between pb-2 border-b border-slate-800">
                  <span className="text-sm font-bold text-cyan-400 uppercase tracking-wider flex items-center gap-1.5">
                    <Gauge className="w-4 h-4 text-cyan-400" /> Tần Số Quét & Giới Hạn FPS:
                  </span>
                  <span className="text-cyan-300 font-bold text-sm bg-cyan-950/90 border border-cyan-800 px-2.5 py-0.5 rounded uppercase">
                    {fpsLimit === 'vsync'
                      ? 'V-Sync (Auto)'
                      : fpsLimit === 'unlimited'
                      ? 'Không giới hạn'
                      : `${fpsLimit} FPS`}
                  </span>
                </div>

                <div className="grid grid-cols-3 gap-2">
                  {FPS_OPTIONS.map((opt) => {
                    const isActive = fpsLimit === opt.value;
                    return (
                      <button
                        key={String(opt.value)}
                        type="button"
                        onClick={() => {
                          soundEngine.playSound('click');
                          onUpdateSettings({
                            soundVolume,
                            cameraSpeed,
                            autoPause,
                            showGrid,
                            fpsLimit: opt.value,
                          });
                        }}
                        className={`py-2 px-2 rounded-lg text-sm font-bold transition-all border text-center cursor-pointer flex flex-col items-center justify-center ${
                          isActive
                            ? 'bg-cyan-950/90 border-cyan-400 text-cyan-300 shadow-[0_0_12px_rgba(6,182,212,0.35)] scale-[1.02]'
                            : 'bg-slate-950/60 border-slate-800 text-slate-300 hover:text-white hover:border-slate-700'
                        }`}
                      >
                        <span>{opt.label}</span>
                      </button>
                    );
                  })}
                </div>

                <div className="p-3 rounded-lg bg-slate-950/80 border border-slate-800/80 text-[13px] text-slate-300 leading-relaxed">
                  <span className="text-cyan-400 font-semibold">ℹ️ Chi tiết chế độ: </span>
                  {currentFpsDesc}
                </div>
              </div>
            </div>

            {/* ---------------------------------------------------------------------
                RIGHT COLUMN (6 Cols): Gameplay Switches & Keymap Matrix
            --------------------------------------------------------------------- */}
            <div className="md:col-span-6 flex flex-col justify-between gap-4 min-h-0">
              {/* Box 1: Gameplay Toggles */}
              <div className="bg-slate-900/60 p-4 rounded-xl border border-slate-800/90 space-y-3">
                <div className="text-sm font-bold text-cyan-400 uppercase tracking-wider flex items-center gap-1.5 pb-2 border-b border-slate-800">
                  <Pause className="w-4 h-4 text-cyan-400" /> Cơ Chế Trận Đấu & Hiển Thị:
                </div>

                {/* Switch 1: Tự động tạm dừng khi mở bảng */}
                <div
                  onClick={() => {
                    soundEngine.playSound('click');
                    onUpdateSettings({
                      soundVolume,
                      cameraSpeed,
                      autoPause: !autoPause,
                      showGrid,
                      fpsLimit,
                    });
                  }}
                  className={`p-3.5 rounded-xl border flex items-center justify-between gap-3 cursor-pointer transition-all ${
                    autoPause
                      ? 'bg-cyan-950/30 border-cyan-500/60'
                      : 'bg-slate-950/50 border-slate-800 hover:border-slate-700'
                  }`}
                >
                  <div className="min-w-0">
                    <span className="text-sm font-bold text-slate-100 block">
                      Tự động đóng băng trận đấu (Auto Pause)
                    </span>
                    <span className="text-[13px] text-slate-300 block mt-0.5">
                      {autoPause
                        ? 'Đóng băng thời gian khi mở Cây công nghệ, Cài đặt hoặc Hướng dẫn.'
                        : 'Trận đấu tiếp tục diễn ra theo thời gian thực khi mở bảng modal.'}
                    </span>
                  </div>

                  <div
                    className={`w-12 h-6 rounded-full transition-colors relative flex items-center px-1 shrink-0 ${
                      autoPause ? 'bg-cyan-500' : 'bg-slate-800'
                    }`}
                  >
                    <div
                      className={`w-4 h-4 rounded-full bg-white transition-transform ${
                        autoPause ? 'translate-x-6' : 'translate-x-0'
                      }`}
                    />
                  </div>
                </div>

                {/* Switch 2: Hiển thị lưới Blueprint */}
                <div
                  onClick={() => {
                    soundEngine.playSound('click');
                    onUpdateSettings({
                      soundVolume,
                      cameraSpeed,
                      autoPause,
                      showGrid: !showGrid,
                      fpsLimit,
                    });
                  }}
                  className={`p-3.5 rounded-xl border flex items-center justify-between gap-3 cursor-pointer transition-all ${
                    showGrid
                      ? 'bg-cyan-950/30 border-cyan-500/60'
                      : 'bg-slate-950/50 border-slate-800 hover:border-slate-700'
                  }`}
                >
                  <div className="min-w-0">
                    <span className="text-sm font-bold text-slate-100 flex items-center gap-1.5">
                      <Grid className="w-4 h-4 text-cyan-400" /> Hiển thị lưới tọa độ Blueprint
                    </span>
                    <span className="text-[13px] text-slate-300 block mt-0.5">
                      {showGrid
                        ? 'Lưới ô vuông 100x100 phong cách bản vẽ thiết kế quân sự đang BẬT.'
                        : 'Lưới ô vuông tọa độ đang ẨN.'}
                    </span>
                  </div>

                  <div
                    className={`w-12 h-6 rounded-full transition-colors relative flex items-center px-1 shrink-0 ${
                      showGrid ? 'bg-cyan-500' : 'bg-slate-800'
                    }`}
                  >
                    <div
                      className={`w-4 h-4 rounded-full bg-white transition-transform ${
                        showGrid ? 'translate-x-6' : 'translate-x-0'
                      }`}
                    />
                  </div>
                </div>
              </div>

              {/* Box 2: Bảng phím tắt tác chiến */}
              <div className="bg-slate-900/60 p-4 rounded-xl border border-slate-800/90 flex flex-col justify-between flex-1 min-h-0">
                <div className="text-sm font-bold text-cyan-400 uppercase tracking-wider flex items-center gap-1.5 pb-2 border-b border-slate-800 mb-2.5">
                  <Keyboard className="w-4 h-4 text-cyan-400" /> Bảng Thao Tác Chuột & Phím Tắt:
                </div>

                <div className="grid grid-cols-2 gap-2 text-sm">
                  {[
                    { key: 'WASD / Mũi tên', desc: 'Di chuyển Camera' },
                    { key: 'Cuộn chuột', desc: 'Zoom phóng to/thu nhỏ' },
                    { key: 'Kéo chuột trái', desc: 'Quét chọn nhiều lính' },
                    { key: 'Chuột phải', desc: 'Lệnh Di chuyển/Đánh' },
                    { key: 'Phím T', desc: 'Mở Cây Công Nghệ' },
                    { key: 'Phím B', desc: 'Bảng Tự Động Hóa' },
                    { key: 'Phím Space', desc: 'Tạm dừng/Tiếp tục' },
                    { key: 'Phím Esc', desc: 'Đóng bảng / Thoát' },
                  ].map((hotkey) => (
                    <div
                      key={hotkey.key}
                      className="p-2.5 rounded-lg bg-slate-950/70 border border-slate-800/80 flex flex-col justify-center"
                    >
                      <span className="text-cyan-300 font-bold font-mono text-[13px] truncate">
                        {hotkey.key}
                      </span>
                      <span className="text-[13px] text-slate-300 truncate">{hotkey.desc}</span>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </div>

          {/* =========================================================================
              FOOTER ACTION BAR: Match Actions & Close Button
          ========================================================================= */}
          <div className="px-5 sm:px-6 py-3.5 border-t border-cyan-900/70 bg-slate-950/95 flex items-center justify-between gap-3 shrink-0 flex-wrap">
            <div className="flex items-center gap-2">
              {isInGame && (
                <button
                  type="button"
                  onClick={() => {
                    soundEngine.playSound('click');
                    setConfirmAction('restart');
                  }}
                  className="group flex items-center gap-2 px-4 py-2.5 bg-gradient-to-r from-rose-950/90 via-rose-900/70 to-rose-950/90 hover:from-rose-900 hover:to-rose-800 active:scale-95 border border-rose-700/80 hover:border-rose-400 text-rose-300 hover:text-white rounded-xl text-sm font-bold tracking-wide transition-all shadow-[0_0_12px_rgba(244,63,94,0.2)] hover:shadow-[0_0_20px_rgba(244,63,94,0.45)] cursor-pointer"
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
                  className="group flex items-center gap-2 px-4 py-2.5 bg-gradient-to-r from-slate-900/90 via-slate-800/80 to-slate-900/90 hover:from-slate-800 hover:to-slate-700 active:scale-95 border border-slate-700/90 hover:border-cyan-500/80 text-slate-300 hover:text-cyan-200 rounded-xl text-sm font-bold tracking-wide transition-all shadow-[0_0_12px_rgba(15,23,42,0.4)] hover:shadow-[0_0_18px_rgba(6,182,212,0.3)] cursor-pointer"
                >
                  <Home className="w-4 h-4 text-slate-400 group-hover:text-cyan-300 transition-transform duration-200 group-hover:scale-110" />
                  <span>VỀ MENU CHÍNH</span>
                </button>
              )}
            </div>

            <button
              type="button"
              onClick={() => {
                soundEngine.playSound('click');
                handleClose();
              }}
              className="flex items-center gap-2 px-5 py-2.5 bg-gradient-to-r from-cyan-600 to-sky-500 hover:from-cyan-500 hover:to-sky-400 active:scale-95 text-slate-950 font-black rounded-xl text-sm tracking-wider uppercase transition-all shadow-[0_0_20px_rgba(6,182,212,0.4)] hover:shadow-[0_0_25px_rgba(6,182,212,0.6)] cursor-pointer"
            >
              {isInGame ? (
                <>
                  <Play className="w-4 h-4 fill-current" />
                  <span>TIẾP TỤC TRẬN ĐẤU</span>
                  <span className="text-[13px] bg-cyan-300/60 text-slate-950 px-2 py-0.5 rounded font-mono font-bold">
                    [ESC]
                  </span>
                </>
              ) : (
                <>
                  <span>LƯU & ĐÓNG</span>
                  <span className="text-[13px] bg-cyan-300/60 text-slate-950 px-2 py-0.5 rounded font-mono font-bold">
                    [ESC]
                  </span>
                </>
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
