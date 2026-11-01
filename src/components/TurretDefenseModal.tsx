import React, { useRef, useState, useEffect, useCallback } from 'react';
import { X, Shield, RotateCcw, Check, Sparkles, MapPin, Sliders, Layers } from 'lucide-react';
import { TurretDefenseLine } from '../bridge/wasmBridge';

interface TurretDefenseModalProps {
  isOpen: boolean;
  onClose: () => void;
  defenseLine: TurretDefenseLine;
  onSave: (newLine: TurretDefenseLine) => void;
}

const MAP_SIZE = 4800.0;
const CANVAS_SIZE = 480; // Kích thước mini-map 480x480px

export const TurretDefenseModal: React.FC<TurretDefenseModalProps> = ({
  isOpen,
  onClose,
  defenseLine,
  onSave,
}) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const [lineState, setLineState] = useState<TurretDefenseLine>(defenseLine);
  const [selectedPointMode, setSelectedPointMode] = useState<'p1' | 'p2' | null>(null);
  const [clickStep, setClickStep] = useState<1 | 2>(1);
  const [dragMode, setDragMode] = useState<'p1' | 'p2' | 'draw' | null>(null);
  const [hoveredPoint, setHoveredPoint] = useState<'p1' | 'p2' | null>(null);

  const prevIsOpenRef = useRef(false);
  const isPointerDownRef = useRef(false);
  const dragTargetRef = useRef<'p1' | 'p2' | 'new_line' | null>(null);
  const pointerStartWorldRef = useRef<[number, number] | null>(null);
  const hasMovedRef = useRef<boolean>(false);

  // Giới hạn tọa độ trong phạm vi an toàn của bản đồ 4800x4800
  const clampWorld = useCallback((val: number): number => {
    return Math.round(Math.max(80, Math.min(MAP_SIZE - 80, val)));
  }, []);

  // Chỉ đồng bộ cấu hình ban đầu khi modal được mở lên (tránh bị reset bởi vòng lặp sync state)
  useEffect(() => {
    if (isOpen && !prevIsOpenRef.current) {
      setLineState(defenseLine);
      setSelectedPointMode(null);
      setClickStep(1);
      setDragMode(null);
    }
    prevIsOpenRef.current = isOpen;
  }, [isOpen, defenseLine]);

  // Chuyển đổi tọa độ World (0..4800) sang Canvas Screen (0..480)
  const worldToCanvas = useCallback((wx: number, wy: number): [number, number] => {
    return [(wx / MAP_SIZE) * CANVAS_SIZE, (wy / MAP_SIZE) * CANVAS_SIZE];
  }, []);

  // Chuyển đổi tọa độ Canvas Screen sang World (0..4800)
  const canvasToWorld = useCallback((cx: number, cy: number): [number, number] => {
    return [(cx / CANVAS_SIZE) * MAP_SIZE, (cy / CANVAS_SIZE) * MAP_SIZE];
  }, []);

  // Tính toán danh sách vị trí tháp dự kiến
  const computePreviewSpots = useCallback((): Array<[number, number]> => {
    const spots: Array<[number, number]> = [];
    const [x1, y1] = lineState.p1;
    const [x2, y2] = lineState.p2;

    const dx = x2 - x1;
    const dy = y2 - y1;
    const len = Math.sqrt(dx * dx + dy * dy);

    if (len < 10) return [[x1, y1]];

    const ux = dx / len;
    const uy = dy / len;
    const nx = -uy;
    const ny = ux;

    const spacing = Math.max(45, lineState.spacing);
    const count = Math.floor(len / spacing) + 1;
    const rowSpacing = Math.max(40, lineState.row_spacing);
    const rows = Math.max(1, lineState.rows);

    for (let r = 0; r < rows; r++) {
      const rowOffX = r * rowSpacing * nx;
      const rowOffY = r * rowSpacing * ny;
      for (let i = 0; i < count; i++) {
        const t = count > 1 ? i * spacing : len * 0.5;
        if (t <= len + 5) {
          spots.push([x1 + t * ux + rowOffX, y1 + t * uy + rowOffY]);
        }
      }
    }
    return spots;
  }, [lineState]);

  const previewSpots = computePreviewSpots();

  // Vẽ Canvas Mini-Map
  useEffect(() => {
    if (!isOpen) return;
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // 1. Nền bản đồ tối Cyberpunk
    ctx.fillStyle = '#030d1a';
    ctx.fillRect(0, 0, CANVAS_SIZE, CANVAS_SIZE);

    // Lưới tọa độ
    ctx.strokeStyle = '#0a223e';
    ctx.lineWidth = 1;
    const step = CANVAS_SIZE / 12; // mỗi ô ~200px world
    for (let i = 0; i <= CANVAS_SIZE; i += step) {
      ctx.beginPath();
      ctx.moveTo(i, 0);
      ctx.lineTo(i, CANVAS_SIZE);
      ctx.stroke();
      ctx.beginPath();
      ctx.moveTo(0, i);
      ctx.lineTo(CANVAS_SIZE, i);
      ctx.stroke();
    }

    // 2. Căn cứ Player HQ (Tây-Nam: 550, 4250)
    const [pccX, pccY] = worldToCanvas(550, 4250);
    ctx.fillStyle = 'rgba(0, 229, 255, 0.2)';
    ctx.strokeStyle = '#00e5ff';
    ctx.lineWidth = 1.5;
    ctx.strokeRect(pccX - 16, pccY - 16, 32, 32);
    ctx.fillRect(pccX - 16, pccY - 16, 32, 32);
    ctx.fillStyle = '#00e5ff';
    ctx.font = 'bold 9px monospace';
    ctx.fillText('PLAYER HQ', pccX - 24, pccY + 25);

    // Mỏ khoáng sản Player
    const minerals = [
      [500, 4050],
      [550, 4020],
      [600, 4050],
    ];
    minerals.forEach(([mx, my]) => {
      const [mcx, mcy] = worldToCanvas(mx, my);
      ctx.fillStyle = '#38bdf8';
      ctx.beginPath();
      ctx.arc(mcx, mcy, 3.5, 0, Math.PI * 2);
      ctx.fill();
    });

    // 3. Căn cứ Enemy HQ (Đông-Bắc: 4250, 550)
    const [eccX, eccY] = worldToCanvas(4250, 550);
    ctx.fillStyle = 'rgba(244, 63, 94, 0.2)';
    ctx.strokeStyle = '#f43f5e';
    ctx.lineWidth = 1.5;
    ctx.strokeRect(eccX - 16, eccY - 16, 32, 32);
    ctx.fillRect(eccX - 16, eccY - 16, 32, 32);
    ctx.fillStyle = '#f43f5e';
    ctx.fillText('ENEMY HQ', eccX - 22, eccY + 25);

    // Hướng tiến công địch (mũi tên đứt nét đỏ)
    ctx.strokeStyle = 'rgba(244, 63, 94, 0.3)';
    ctx.setLineDash([4, 4]);
    ctx.beginPath();
    ctx.moveTo(eccX, eccY);
    ctx.lineTo(pccX, pccY);
    ctx.stroke();
    ctx.setLineDash([]);

    // 4. Tuyến Phòng Thủ
    const [p1X, p1Y] = worldToCanvas(lineState.p1[0], lineState.p1[1]);
    const [p2X, p2Y] = worldToCanvas(lineState.p2[0], lineState.p2[1]);

    if (lineState.enabled) {
      // Đường nối hàng rào chính (màu hổ phách sáng nét đứt)
      ctx.strokeStyle = '#f59e0b';
      ctx.lineWidth = 2.5;
      ctx.setLineDash([6, 4]);
      ctx.beginPath();
      ctx.moveTo(p1X, p1Y);
      ctx.lineTo(p2X, p2Y);
      ctx.stroke();
      ctx.setLineDash([]);

      // Các tháp dự kiến
      previewSpots.forEach(([wx, wy], idx) => {
        const [sx, sy] = worldToCanvas(wx, wy);

        // Vòng tròn tầm bắn tháp (~220px world)
        ctx.strokeStyle = 'rgba(251, 191, 36, 0.18)';
        ctx.lineWidth = 1;
        ctx.beginPath();
        ctx.arc(sx, sy, (220 / MAP_SIZE) * CANVAS_SIZE, 0, Math.PI * 2);
        ctx.stroke();

        // Chấm tháp Turret
        ctx.fillStyle = '#f59e0b';
        ctx.strokeStyle = '#ffffff';
        ctx.lineWidth = 1.2;
        ctx.beginPath();
        ctx.arc(sx, sy, 5, 0, Math.PI * 2);
        ctx.fill();
        ctx.stroke();

        ctx.fillStyle = '#ffffff';
        ctx.font = 'bold 8px monospace';
        ctx.fillText(`${idx + 1}`, sx - 2.5, sy + 2.5);
      });
    }

    // 5. Điểm Neo A (Cyan)
    ctx.fillStyle = '#06b6d4';
    ctx.strokeStyle = '#ffffff';
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.arc(p1X, p1Y, 8, 0, Math.PI * 2);
    ctx.fill();
    ctx.stroke();

    // Vòng phát sáng nếu đang chọn, hover hoặc kéo Điểm A
    if (selectedPointMode === 'p1' || dragMode === 'p1' || hoveredPoint === 'p1') {
      ctx.strokeStyle = '#22d3ee';
      ctx.lineWidth = 2;
      ctx.setLineDash([3, 3]);
      ctx.beginPath();
      ctx.arc(p1X, p1Y, 15, 0, Math.PI * 2);
      ctx.stroke();
      ctx.setLineDash([]);
    }

    ctx.fillStyle = '#ffffff';
    ctx.font = 'bold 9px monospace';
    ctx.fillText('A', p1X - 3.5, p1Y + 3);

    // 6. Điểm Neo B (Purple)
    ctx.fillStyle = '#a855f7';
    ctx.strokeStyle = '#ffffff';
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.arc(p2X, p2Y, 8, 0, Math.PI * 2);
    ctx.fill();
    ctx.stroke();

    if (selectedPointMode === 'p2' || dragMode === 'p2' || hoveredPoint === 'p2' || dragMode === 'draw') {
      ctx.strokeStyle = '#c084fc';
      ctx.lineWidth = 2;
      ctx.setLineDash([3, 3]);
      ctx.beginPath();
      ctx.arc(p2X, p2Y, 15, 0, Math.PI * 2);
      ctx.stroke();
      ctx.setLineDash([]);
    }

    ctx.fillStyle = '#ffffff';
    ctx.fillText('B', p2X - 3.5, p2Y + 3);
  }, [isOpen, lineState, previewSpots, selectedPointMode, dragMode, hoveredPoint, worldToCanvas]);

  // Tính tọa độ canvas chính xác từ MouseEvent
  const getCanvasCoords = (e: React.MouseEvent<HTMLCanvasElement> | MouseEvent) => {
    const canvas = canvasRef.current;
    if (!canvas) return null;
    const rect = canvas.getBoundingClientRect();
    const cx = ((e.clientX - rect.left) / rect.width) * CANVAS_SIZE;
    const cy = ((e.clientY - rect.top) / rect.height) * CANVAS_SIZE;
    return { cx, cy };
  };

  // Hover chuột trên canvas để đổi con trỏ
  const handleCanvasMouseMove = (e: React.MouseEvent<HTMLCanvasElement>) => {
    if (dragMode) return;
    const coords = getCanvasCoords(e);
    if (!coords) return;
    const { cx, cy } = coords;

    const [p1X, p1Y] = worldToCanvas(lineState.p1[0], lineState.p1[1]);
    const [p2X, p2Y] = worldToCanvas(lineState.p2[0], lineState.p2[1]);

    const distA = Math.hypot(cx - p1X, cy - p1Y);
    const distB = Math.hypot(cx - p2X, cy - p2Y);

    if (distA <= 22) {
      setHoveredPoint('p1');
    } else if (distB <= 22) {
      setHoveredPoint('p2');
    } else {
      setHoveredPoint(null);
    }
  };

  // MouseDown trên canvas: Bắt đầu kéo thả hoặc click đặt điểm
  const handleCanvasMouseDown = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const coords = getCanvasCoords(e);
    if (!coords) return;
    const { cx, cy } = coords;
    const [rawWx, rawWy] = canvasToWorld(cx, cy);
    const wx = clampWorld(rawWx);
    const wy = clampWorld(rawWy);

    const [p1X, p1Y] = worldToCanvas(lineState.p1[0], lineState.p1[1]);
    const [p2X, p2Y] = worldToCanvas(lineState.p2[0], lineState.p2[1]);

    const distA = Math.hypot(cx - p1X, cy - p1Y);
    const distB = Math.hypot(cx - p2X, cy - p2Y);

    // 1. Nếu đang bật chế độ chọn thủ công Điểm A qua nút "CHỌN TRÊN MAP"
    if (selectedPointMode === 'p1') {
      setLineState((prev) => ({
        ...prev,
        p1: [wx, wy],
      }));
      setSelectedPointMode('p2'); // Tự động chuyển sang chờ chọn Điểm B
      return;
    }

    // 2. Nếu đang bật chế độ chọn thủ công Điểm B qua nút "CHỌN TRÊN MAP"
    if (selectedPointMode === 'p2') {
      setLineState((prev) => ({
        ...prev,
        p2: [wx, wy],
      }));
      setSelectedPointMode(null);
      return;
    }

    // 3. Nhấp trúng điểm A hoặc B -> Kéo thả trực tiếp điểm đó
    if (distA <= 22) {
      isPointerDownRef.current = true;
      dragTargetRef.current = 'p1';
      pointerStartWorldRef.current = [wx, wy];
      hasMovedRef.current = false;
      setDragMode('p1');
      return;
    }
    if (distB <= 22) {
      isPointerDownRef.current = true;
      dragTargetRef.current = 'p2';
      pointerStartWorldRef.current = [wx, wy];
      hasMovedRef.current = false;
      setDragMode('p2');
      return;
    }

    // 4. Nhấn trên bản đồ trống: Chuẩn bị kéo vẽ đường mới hoặc click đặt điểm
    isPointerDownRef.current = true;
    dragTargetRef.current = 'new_line';
    pointerStartWorldRef.current = [wx, wy];
    hasMovedRef.current = false;
  };

  // Lắng nghe chuột toàn màn hình khi đang nhấn chuột / kéo
  useEffect(() => {
    const handleWindowMouseMove = (e: MouseEvent) => {
      if (!isPointerDownRef.current || !pointerStartWorldRef.current) return;

      const coords = getCanvasCoords(e);
      if (!coords) return;
      const [rawWx, rawWy] = canvasToWorld(coords.cx, coords.cy);
      const wx = clampWorld(rawWx);
      const wy = clampWorld(rawWy);

      const moveDist = Math.hypot(
        wx - pointerStartWorldRef.current[0],
        wy - pointerStartWorldRef.current[1]
      );
      if (moveDist > 15) {
        hasMovedRef.current = true;
      }

      if (dragTargetRef.current === 'p1') {
        setLineState((prev) => ({
          ...prev,
          p1: [wx, wy],
        }));
      } else if (dragTargetRef.current === 'p2') {
        setLineState((prev) => ({
          ...prev,
          p2: [wx, wy],
        }));
      } else if (dragTargetRef.current === 'new_line') {
        // Đang kéo vẽ tuyến phòng thủ mới từ A (vị trí ấn chuột) tới B (vị trí hiện tại)
        setDragMode('draw');
        setLineState((prev) => ({
          ...prev,
          p1: pointerStartWorldRef.current!,
          p2: [wx, wy],
        }));
      }
    };

    const handleWindowMouseUp = () => {
      if (!isPointerDownRef.current) return;
      isPointerDownRef.current = false;
      const target = dragTargetRef.current;
      const startPos = pointerStartWorldRef.current;
      const didMove = hasMovedRef.current;

      setDragMode(null);
      dragTargetRef.current = null;
      pointerStartWorldRef.current = null;
      hasMovedRef.current = false;

      // Nếu người chơi kéo thả (Drag)
      if (didMove) {
        if (target === 'new_line') {
          setClickStep(1);
          setSelectedPointMode(null);
        }
        return;
      }

      // Nếu người chơi chỉ Click chuột tại chỗ (không kéo)
      if (target === 'p1') {
        setSelectedPointMode('p1');
      } else if (target === 'p2') {
        setSelectedPointMode('p2');
      } else if (target === 'new_line' && startPos) {
        // Click từng điểm tuần tự: Lần 1 đặt A, Lần 2 đặt B
        if (clickStep === 1) {
          setLineState((prev) => ({
            ...prev,
            p1: startPos,
          }));
          setClickStep(2);
        } else {
          setLineState((prev) => ({
            ...prev,
            p2: startPos,
          }));
          setClickStep(1);
        }
      }
    };

    window.addEventListener('mousemove', handleWindowMouseMove);
    window.addEventListener('mouseup', handleWindowMouseUp);

    return () => {
      window.removeEventListener('mousemove', handleWindowMouseMove);
      window.removeEventListener('mouseup', handleWindowMouseUp);
    };
  }, [canvasToWorld, clampWorld, clickStep]);

  // Đặt lại vị trí phòng thủ mặc định
  const handleResetDefault = () => {
    setLineState({
      enabled: true,
      p1: [850, 3800],
      p2: [1300, 4250],
      spacing: 90,
      rows: 1,
      row_spacing: 80,
    });
    setSelectedPointMode(null);
    setClickStep(1);
    setDragMode(null);
  };

  // Lưu và áp dụng
  const handleSave = () => {
    onSave(lineState);
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/85 backdrop-blur-md p-4 select-none"
      onClick={(e) => {
        if (e.target === e.currentTarget) {
          onClose();
        }
      }}
    >
      <div
        className="bg-[#030d1a] border-2 border-amber-500/80 rounded-xl w-full max-w-4xl shadow-[0_0_50px_rgba(245,158,11,0.3)] flex flex-col max-h-[95vh] overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header Modal */}
        <div className="flex items-center justify-between px-6 py-3 border-b border-amber-900/60 bg-slate-900/50">
          <div className="flex items-center gap-2.5">
            <Shield className="w-5 h-5 text-amber-400" />
            <div>
              <h2 className="text-base font-bold font-mono text-amber-300">
                QUY HOẠCH HÀNG RÀO PHÒNG THỦ TURRET // TACTICAL DEFENSE LINE
              </h2>
              <p className="text-xs text-slate-400">
                Nhấn giữ và kéo trực tiếp trên bản đồ để định hình tuyến A-B, hoặc kéo thả từng điểm neo
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-1.5 text-slate-400 hover:text-white rounded-lg hover:bg-slate-800 transition-colors cursor-pointer"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Nội Dung: Mini-map bên trái, Bảng Tùy chỉnh bên phải */}
        <div className="flex-1 overflow-y-auto p-6 flex flex-col md:flex-row gap-6 items-start">
          {/* Mini-map Canvas */}
          <div className="flex flex-col items-center gap-2.5 w-full md:w-auto">
            <div className="relative border-2 border-amber-500/60 rounded-xl overflow-hidden shadow-[0_0_20px_rgba(245,158,11,0.2)]">
              <canvas
                ref={canvasRef}
                width={CANVAS_SIZE}
                height={CANVAS_SIZE}
                onMouseDown={handleCanvasMouseDown}
                onMouseMove={handleCanvasMouseMove}
                style={{
                  cursor: dragMode ? 'grabbing' : hoveredPoint ? 'grab' : 'crosshair',
                }}
                className="block max-w-full h-auto"
              />
            </div>
            <div className="text-[11px] font-mono text-slate-400 flex items-center justify-between w-full px-2">
              <span className="flex items-center gap-1.5 text-cyan-400 font-bold">
                <span className="w-2.5 h-2.5 rounded-full bg-cyan-400 inline-block shadow-[0_0_6px_#22d3ee]" />
                Điểm A: ({lineState.p1[0]}, {lineState.p1[1]})
              </span>
              <span className="flex items-center gap-1.5 text-purple-400 font-bold">
                <span className="w-2.5 h-2.5 rounded-full bg-purple-400 inline-block shadow-[0_0_6px_#c084fc]" />
                Điểm B: ({lineState.p2[0]}, {lineState.p2[1]})
              </span>
            </div>
            <div className="text-[10px] text-amber-400/90 font-mono text-center w-full bg-slate-900/60 py-1 px-2.5 rounded-lg border border-amber-500/30">
              💡 Nhấn giữ & kéo chuột để vạch hàng rào A-B, hoặc kéo từng điểm neo
            </div>
          </div>

          {/* Bảng thông số tùy chỉnh bên phải */}
          <div className="flex-1 flex flex-col justify-between w-full h-full gap-4 font-mono text-xs">
            <div className="space-y-3.5">
              {/* Bật/Tắt Hàng Rào */}
              <div className="bg-slate-900/60 p-3 rounded-xl border border-slate-800 flex items-center justify-between">
                <div>
                  <div className="font-bold text-slate-200">KÍCH HOẠT HÀNG RÀO PHÒNG THỦ</div>
                  <div className="text-[11px] text-slate-400">
                    Khi xây TurretTower, tự động cắm tháp theo hàng rào đã vạch
                  </div>
                </div>
                <button
                  onClick={() =>
                    setLineState((prev) => ({ ...prev, enabled: !prev.enabled }))
                  }
                  className={`px-3 py-1.5 rounded font-bold text-xs transition-all cursor-pointer ${
                    lineState.enabled
                      ? 'bg-amber-500 text-slate-950 font-bold shadow-[0_0_10px_rgba(245,158,11,0.4)]'
                      : 'bg-slate-800 text-slate-400 hover:text-white'
                  }`}
                >
                  {lineState.enabled ? 'ĐANG BẬT' : 'ĐANG TẮT'}
                </button>
              </div>

              {/* Tùy chỉnh tọa độ Điểm A & B */}
              <div className="grid grid-cols-2 gap-3">
                {/* Khối Điểm A */}
                <div
                  className={`p-2.5 rounded-xl border transition-all ${
                    selectedPointMode === 'p1'
                      ? 'bg-cyan-950/40 border-cyan-400 shadow-[0_0_10px_rgba(6,182,212,0.3)]'
                      : 'bg-slate-900/60 border-slate-800'
                  }`}
                >
                  <div className="flex items-center justify-between mb-2">
                    <span className="font-bold text-cyan-400 flex items-center gap-1">
                      <MapPin className="w-3.5 h-3.5" /> Điểm A (Đầu)
                    </span>
                    <button
                      onClick={() =>
                        setSelectedPointMode((prev) => (prev === 'p1' ? null : 'p1'))
                      }
                      className={`px-2 py-0.5 rounded text-[10px] font-bold cursor-pointer ${
                        selectedPointMode === 'p1'
                          ? 'bg-cyan-500 text-slate-950'
                          : 'bg-slate-800 text-cyan-300 hover:bg-slate-700'
                      }`}
                    >
                      {selectedPointMode === 'p1' ? 'ĐANG CHỌN...' : 'CHỌN TRÊN MAP'}
                    </button>
                  </div>
                  <div className="flex items-center gap-2">
                    <div className="flex-1 flex items-center gap-1 bg-slate-950 px-2 py-1 rounded border border-slate-800">
                      <span className="text-slate-500 text-[10px]">X:</span>
                      <input
                        type="number"
                        min="50"
                        max="4750"
                        step="10"
                        value={lineState.p1[0]}
                        onChange={(e) => {
                          const val = Math.max(50, Math.min(4750, parseInt(e.target.value) || 0));
                          setLineState((prev) => ({
                            ...prev,
                            p1: [val, prev.p1[1]],
                          }));
                        }}
                        className="w-full bg-transparent text-cyan-300 font-bold text-center focus:outline-none"
                      />
                    </div>
                    <div className="flex-1 flex items-center gap-1 bg-slate-950 px-2 py-1 rounded border border-slate-800">
                      <span className="text-slate-500 text-[10px]">Y:</span>
                      <input
                        type="number"
                        min="50"
                        max="4750"
                        step="10"
                        value={lineState.p1[1]}
                        onChange={(e) => {
                          const val = Math.max(50, Math.min(4750, parseInt(e.target.value) || 0));
                          setLineState((prev) => ({
                            ...prev,
                            p1: [prev.p1[0], val],
                          }));
                        }}
                        className="w-full bg-transparent text-cyan-300 font-bold text-center focus:outline-none"
                      />
                    </div>
                  </div>
                </div>

                {/* Khối Điểm B */}
                <div
                  className={`p-2.5 rounded-xl border transition-all ${
                    selectedPointMode === 'p2'
                      ? 'bg-purple-950/40 border-purple-400 shadow-[0_0_10px_rgba(168,85,247,0.3)]'
                      : 'bg-slate-900/60 border-slate-800'
                  }`}
                >
                  <div className="flex items-center justify-between mb-2">
                    <span className="font-bold text-purple-400 flex items-center gap-1">
                      <MapPin className="w-3.5 h-3.5" /> Điểm B (Cuối)
                    </span>
                    <button
                      onClick={() =>
                        setSelectedPointMode((prev) => (prev === 'p2' ? null : 'p2'))
                      }
                      className={`px-2 py-0.5 rounded text-[10px] font-bold cursor-pointer ${
                        selectedPointMode === 'p2'
                          ? 'bg-purple-500 text-slate-950'
                          : 'bg-slate-800 text-purple-300 hover:bg-slate-700'
                      }`}
                    >
                      {selectedPointMode === 'p2' ? 'ĐANG CHỌN...' : 'CHỌN TRÊN MAP'}
                    </button>
                  </div>
                  <div className="flex items-center gap-2">
                    <div className="flex-1 flex items-center gap-1 bg-slate-950 px-2 py-1 rounded border border-slate-800">
                      <span className="text-slate-500 text-[10px]">X:</span>
                      <input
                        type="number"
                        min="50"
                        max="4750"
                        step="10"
                        value={lineState.p2[0]}
                        onChange={(e) => {
                          const val = Math.max(50, Math.min(4750, parseInt(e.target.value) || 0));
                          setLineState((prev) => ({
                            ...prev,
                            p2: [val, prev.p2[1]],
                          }));
                        }}
                        className="w-full bg-transparent text-purple-300 font-bold text-center focus:outline-none"
                      />
                    </div>
                    <div className="flex-1 flex items-center gap-1 bg-slate-950 px-2 py-1 rounded border border-slate-800">
                      <span className="text-slate-500 text-[10px]">Y:</span>
                      <input
                        type="number"
                        min="50"
                        max="4750"
                        step="10"
                        value={lineState.p2[1]}
                        onChange={(e) => {
                          const val = Math.max(50, Math.min(4750, parseInt(e.target.value) || 0));
                          setLineState((prev) => ({
                            ...prev,
                            p2: [prev.p2[0], val],
                          }));
                        }}
                        className="w-full bg-transparent text-purple-300 font-bold text-center focus:outline-none"
                      />
                    </div>
                  </div>
                </div>
              </div>

              {/* Khoảng cách mỗi tháp (Spacing) */}
              <div className="bg-slate-900/60 p-3 rounded-xl border border-slate-800 space-y-2">
                <div className="flex justify-between items-center">
                  <span className="text-slate-300 font-bold flex items-center gap-1.5">
                    <Sliders className="w-3.5 h-3.5 text-amber-400" />
                    Khoảng cách mỗi tháp:
                  </span>
                  <div className="flex items-center gap-2">
                    <span className="text-amber-400 font-bold text-sm">
                      {lineState.spacing} px
                    </span>
                    <div className="flex gap-1 text-[10px]">
                      {[
                        { label: 'Dày', val: 65 },
                        { label: 'Chuẩn', val: 90 },
                        { label: 'Thưa', val: 130 },
                      ].map((preset) => (
                        <button
                          key={preset.val}
                          onClick={() =>
                            setLineState((prev) => ({ ...prev, spacing: preset.val }))
                          }
                          className={`px-1.5 py-0.5 rounded cursor-pointer ${
                            lineState.spacing === preset.val
                              ? 'bg-amber-500 text-slate-950 font-bold'
                              : 'bg-slate-800 text-slate-400 hover:text-white'
                          }`}
                        >
                          {preset.label}
                        </button>
                      ))}
                    </div>
                  </div>
                </div>
                <input
                  type="range"
                  min="55"
                  max="160"
                  step="5"
                  value={lineState.spacing}
                  onChange={(e) =>
                    setLineState((prev) => ({
                      ...prev,
                      spacing: parseFloat(e.target.value) || 90,
                    }))
                  }
                  className="w-full accent-amber-400 cursor-pointer"
                />
              </div>

              {/* Số hàng tháp phòng thủ (Rows) */}
              <div className="bg-slate-900/60 p-3 rounded-xl border border-slate-800 space-y-2">
                <div className="flex justify-between items-center">
                  <span className="text-slate-300 font-bold flex items-center gap-1.5">
                    <Layers className="w-3.5 h-3.5 text-amber-400" />
                    Số hàng tháp phòng ngự:
                  </span>
                  <div className="flex items-center gap-2">
                    {[1, 2, 3].map((r) => (
                      <button
                        key={r}
                        onClick={() => setLineState((prev) => ({ ...prev, rows: r }))}
                        className={`w-8 h-7 rounded font-bold transition-all cursor-pointer ${
                          lineState.rows === r
                            ? 'bg-amber-500 text-slate-950 shadow-[0_0_8px_rgba(245,158,11,0.5)]'
                            : 'bg-slate-800 text-slate-400 hover:text-white'
                        }`}
                      >
                        {r}
                      </button>
                    ))}
                  </div>
                </div>

                {lineState.rows > 1 && (
                  <div className="pt-2 border-t border-slate-800 flex justify-between items-center text-[11px]">
                    <span className="text-slate-400">Khoảng cách giữa các hàng:</span>
                    <div className="flex items-center gap-2">
                      <input
                        type="range"
                        min="50"
                        max="140"
                        step="5"
                        value={lineState.row_spacing}
                        onChange={(e) =>
                          setLineState((prev) => ({
                            ...prev,
                            row_spacing: parseFloat(e.target.value) || 80,
                          }))
                        }
                        className="w-24 accent-amber-400 cursor-pointer"
                      />
                      <span className="text-amber-300 font-bold text-xs w-12 text-right">
                        {lineState.row_spacing} px
                      </span>
                    </div>
                  </div>
                )}
              </div>

              {/* Dự tính tổng số tháp */}
              <div className="bg-amber-950/20 p-2.5 rounded-xl border border-amber-500/40 flex justify-between items-center text-xs">
                <span className="text-slate-300">Tổng số tháp quy hoạch:</span>
                <span className="text-amber-400 font-bold text-sm flex items-center gap-1">
                  <Sparkles className="w-4 h-4" />
                  {previewSpots.length} Tháp TurretTower
                </span>
              </div>
            </div>

            {/* Các Nút Lưu / Reset */}
            <div className="flex items-center justify-between pt-3 border-t border-slate-800">
              <button
                onClick={handleResetDefault}
                className="px-3 py-1.5 rounded-lg border border-slate-700 text-slate-400 hover:text-amber-300 hover:border-amber-400 font-mono text-xs flex items-center gap-1.5 transition-colors cursor-pointer"
                title="Đặt lại vị trí góc xéo đón đầu quân địch giữa Player HQ và Enemy HQ"
              >
                <RotateCcw className="w-3.5 h-3.5" />
                <span>Vị Trí Mặc Định</span>
              </button>

              <div className="flex items-center gap-2">
                <button
                  onClick={onClose}
                  className="px-4 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 font-mono text-xs transition-colors cursor-pointer"
                >
                  HỦY
                </button>
                <button
                  onClick={handleSave}
                  className="px-5 py-1.5 rounded-lg bg-amber-500 hover:bg-amber-400 text-slate-950 font-mono text-xs font-bold flex items-center gap-1.5 shadow-[0_0_12px_rgba(245,158,11,0.4)] transition-all cursor-pointer"
                >
                  <Check className="w-4 h-4" />
                  <span>ÁP DỤNG TUYẾN PHÒNG THỦ</span>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
