import React, { useEffect, useRef, useState, useCallback } from 'react';
import { wasmBridge, EngineSyncState, AutoPlayConfig, PlayerSlotConfig, GameConfigData } from './bridge/wasmBridge';
import { MainMenu } from './components/MainMenu';
import { TechTreeModal } from './components/TechTreeModal';
import { SettingsModal, FpsLimit } from './components/SettingsModal';
import { GameOverModal } from './components/GameOverModal';
import { AutoPlayModal } from './components/AutoPlayModal';
import { GuideModal } from './components/GuideModal';
import { soundEngine } from './audio/soundEngine';
import { Bot } from 'lucide-react';

const defaultAutoPlay: AutoPlayConfig = {
  enabled: false,
  priority_techs: [],
  turret_defense: {
    enabled: true,
    p1: [850, 3800],
    p2: [1300, 4250],
    spacing: 90,
    rows: 1,
    row_spacing: 80,
  },
  loops: [
    {
      id: 'loop_production',
      name: 'Sản Xuất',
      enabled: true,
      current_card_index: 0,
      current_card_progress: 0,
      card_timer: 0,
      cards: [
        {
          id: 'card_prod_worker',
          name: 'Mua 2 Thợ Mỏ (Engineer)',
          enabled: true,
          conditions: [{ type: 'UnitCountLessThan', unit_type: 'Engineer', count: 7 }],
          action: { type: 'TrainUnit', unit_type: 'Engineer', count: 2 },
        },
        {
          id: 'card_prod_supply',
          name: 'Xây SupplyDepot tránh kẹt',
          enabled: true,
          conditions: [{ type: 'SupplyNearCap' }],
          action: { type: 'BuildStructure', building_type: 'SupplyDepot', count: 1 },
        },
        {
          id: 'card_prod_reactor',
          name: 'Xây 2 PowerReactor',
          enabled: true,
          conditions: [{ type: 'Always' }],
          action: { type: 'BuildStructure', building_type: 'PowerReactor', count: 2 },
        },
        {
          id: 'card_prod_turret',
          name: 'Xây 2 TurretTower',
          enabled: true,
          conditions: [{ type: 'Always' }],
          action: { type: 'BuildStructure', building_type: 'TurretTower', count: 2 },
        },
        {
          id: 'card_prod_tech',
          name: 'Nâng cấp Cây Công Nghệ',
          enabled: true,
          conditions: [{ type: 'Always' }],
          action: {
            type: 'ResearchTech',
            tech_id: 'dark_matter_rockets',
            use_priority_queue: true,
          },
        },
        {
          id: 'card_prod_barracks',
          name: 'Xây 1 Barracks',
          enabled: true,
          conditions: [{ type: 'Always' }],
          action: { type: 'BuildStructure', building_type: 'Barracks', count: 1 },
        },
        {
          id: 'card_prod_converter',
          name: 'Xây EnergyConverter',
          enabled: true,
          conditions: [{ type: 'ReactorPerConverterRatio' }],
          action: { type: 'BuildStructure', building_type: 'EnergyConverter', count: 1 },
        },
        {
          id: 'card_prod_shock',
          name: 'Mua ShockTrooper (>150 Khoáng)',
          enabled: true,
          conditions: [{ type: 'MineralsGreaterThan', amount: 150 }],
          action: { type: 'TrainUnit', unit_type: 'ShockTrooper', count: 1 },
        },
        {
          id: 'card_prod_rifle',
          name: 'Mua Rifleman (>150 Khoáng)',
          enabled: true,
          conditions: [{ type: 'MineralsGreaterThan', amount: 150 }],
          action: { type: 'TrainUnit', unit_type: 'Rifleman', count: 1 },
        },
        {
          id: 'card_prod_delay',
          name: 'Tạm nghỉ sản xuất (Delay 2s)',
          enabled: true,
          conditions: [{ type: 'Always' }],
          action: { type: 'Delay', seconds: 2.0 },
        },
      ],
    },
    {
      id: 'loop_military',
      name: 'Quân Sự',
      enabled: true,
      current_card_index: 0,
      current_card_progress: 0,
      card_timer: 0,
      cards: [
        {
          id: 'card_mil_attack',
          name: 'Tự động tấn công (>40 lính)',
          enabled: true,
          conditions: [{ type: 'UnitCountGreaterThan', unit_type: 'Combat', count: 40 }],
          action: { type: 'AttackWave' },
        },
        {
          id: 'card_mil_delay',
          name: 'Nghỉ lệnh hành quân (Delay 2s)',
          enabled: true,
          conditions: [{ type: 'Always' }],
          action: { type: 'Delay', seconds: 2.0 },
        },
      ],
    },
  ],
};

const initialGameState: EngineSyncState = {
  player_minerals: 200,
  player_energy: 50,
  player_supply: 4,
  player_max_supply: 10,
  tech_state: {
    focused_laser: false,
    energy_shielding: false,
    dark_matter_rockets: false,
    titan_alloy_plating: false,
    celestial_flying_swords: false,
    golden_bell_shield: false,
    corrosive_curse: false,
    void_cataclysm: false,
    primeval_scales: false,
    behemoth_mutation: false,
    acid_banelings: false,
    hive_proliferation: false,
    optical_drills: false,
    overcharged_reactors: false,
    fortified_blueprints: false,
    magnetic_rounds: false,
    exo_boosters: false,
    superheated_blades: false,
    overload_strike: false,
    active_researches: [],
    current_research: null,
    research_progress: 0,
    research_total_time: 0,
  },
  autoplay_state: defaultAutoPlay,
  game_time: 0,
  is_paused: false,
};

const SETTINGS_STORAGE_KEY = 'battlefield_blueprint_settings';

interface StoredSettings {
  soundVolume: number;
  cameraSpeed: number;
  autoPause: boolean;
  showGrid: boolean;
  fpsLimit: FpsLimit;
}

function loadStoredSettings(): StoredSettings {
  const defaults: StoredSettings = {
    soundVolume: 0.7,
    cameraSpeed: 1.0,
    autoPause: true,
    showGrid: true,
    fpsLimit: 'vsync',
  };
  try {
    const raw = localStorage.getItem(SETTINGS_STORAGE_KEY);
    if (raw) {
      return {
        ...defaults,
        ...JSON.parse(raw),
      };
    }
  } catch (e) {
    // ignore
  }
  return defaults;
}

export const HUD_SIZES = {
  MINIMAP: 220,
  SELECTION_W: 480,
  SELECTION_H: 180,
  COMMAND_CARD_W: 285,
  COMMAND_CARD_H: 180,
};

export const App: React.FC = () => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const topBarCanvasRef = useRef<HTMLCanvasElement | null>(null);
  const minimapCanvasRef = useRef<HTMLCanvasElement | null>(null);
  const selectionCanvasRef = useRef<HTMLCanvasElement | null>(null);
  const commandCardCanvasRef = useRef<HTMLCanvasElement | null>(null);
  const overlayCanvasRef = useRef<HTMLCanvasElement | null>(null);
  const [isInGame, setIsInGame] = useState<boolean>(false);
  const [isTechTreeOpen, setIsTechTreeOpen] = useState<boolean>(false);
  const [isSettingsOpen, setIsSettingsOpen] = useState<boolean>(false);
  const [isAutoPlayOpen, setIsAutoPlayOpen] = useState<boolean>(false);
  const [isGuideOpen, setIsGuideOpen] = useState<boolean>(false);
  const [gameConfig, setGameConfig] = useState<GameConfigData | null>(null);
  const [gameOverState, setGameOverState] = useState<{
    isOpen: boolean;
    playerWon: boolean;
  }>({ isOpen: false, playerWon: false });

  const [gameState, setGameState] = useState<EngineSyncState>(initialGameState);
  const [settings, setSettings] = useState<StoredSettings>(loadStoredSettings);
  const fpsLimitRef = useRef<FpsLimit>(settings.fpsLimit);
  fpsLimitRef.current = settings.fpsLimit;

  // Khởi tạo Wasm và 2 vòng lặp tách biệt: Logic 60Hz cố định & Render độc lập
  useEffect(() => {
    let isRunning = true;
    let simTimerId: number | null = null;
    let animationFrameId: number | null = null;
    let renderTimerId: number | null = null;
    let lastRenderTime = performance.now();

    const initEngine = async () => {
      if (!canvasRef.current) return;
      await wasmBridge.initialize(
        canvasRef.current,
        null,
        topBarCanvasRef.current,
        minimapCanvasRef.current,
        selectionCanvasRef.current,
        commandCardCanvasRef.current,
        overlayCanvasRef.current
      );

      const resize = () => {
        if (!canvasRef.current) return;
        const width = window.innerWidth;
        const height = window.innerHeight;
        const dpr = window.devicePixelRatio || 1;
        const pxW = Math.round(width * dpr);
        const pxH = Math.round(height * dpr);

        canvasRef.current.width = pxW;
        canvasRef.current.height = pxH;
        canvasRef.current.style.width = `${width}px`;
        canvasRef.current.style.height = `${height}px`;

        if (topBarCanvasRef.current) {
          topBarCanvasRef.current.width = pxW;
          topBarCanvasRef.current.height = Math.round(48 * dpr);
          topBarCanvasRef.current.style.width = `${width}px`;
          topBarCanvasRef.current.style.height = '48px';
        }

        if (minimapCanvasRef.current) {
          minimapCanvasRef.current.width = Math.round(HUD_SIZES.MINIMAP * dpr);
          minimapCanvasRef.current.height = Math.round(HUD_SIZES.MINIMAP * dpr);
          minimapCanvasRef.current.style.width = `${HUD_SIZES.MINIMAP}px`;
          minimapCanvasRef.current.style.height = `${HUD_SIZES.MINIMAP}px`;
        }

        if (selectionCanvasRef.current) {
          selectionCanvasRef.current.width = Math.round(HUD_SIZES.SELECTION_W * dpr);
          selectionCanvasRef.current.height = Math.round(HUD_SIZES.SELECTION_H * dpr);
          selectionCanvasRef.current.style.width = `${HUD_SIZES.SELECTION_W}px`;
          selectionCanvasRef.current.style.height = `${HUD_SIZES.SELECTION_H}px`;
        }

        if (commandCardCanvasRef.current) {
          commandCardCanvasRef.current.width = Math.round(HUD_SIZES.COMMAND_CARD_W * dpr);
          commandCardCanvasRef.current.height = Math.round(HUD_SIZES.COMMAND_CARD_H * dpr);
          commandCardCanvasRef.current.style.width = `${HUD_SIZES.COMMAND_CARD_W}px`;
          commandCardCanvasRef.current.style.height = `${HUD_SIZES.COMMAND_CARD_H}px`;
        }

        if (overlayCanvasRef.current) {
          overlayCanvasRef.current.width = pxW;
          overlayCanvasRef.current.height = pxH;
          overlayCanvasRef.current.style.width = `${width}px`;
          overlayCanvasRef.current.style.height = `${height}px`;
        }

        wasmBridge.resize(width, height, dpr);
      };

      window.addEventListener('resize', resize);
      resize();

      // Đồng bộ settings ban đầu xuống Wasm
      wasmBridge.sendCommand({
        type: 'UpdateSettings',
        sound_volume: settings.soundVolume,
        camera_speed: settings.cameraSpeed,
        auto_pause: settings.autoPause,
        show_grid: settings.showGrid,
        fps_limit: String(settings.fpsLimit),
      });

      const cfg = wasmBridge.getGameConfig();
      if (cfg) {
        setGameConfig(cfg);
      }

      // ============================================================================
      // VÒNG LẶP 1: GAME LOGIC / SIMULATION LOOP (Cố định 60Hz - Chuẩn 1x thực tế)
      // Hoàn toàn tách biệt khỏi vòng lặp Render, không bị ảnh hưởng bởi FPS.
      // ============================================================================
      wasmBridge.simStep(performance.now());
      simTimerId = window.setInterval(() => {
        if (!isRunning) return;
        wasmBridge.simStep(performance.now());
      }, 8);

      // ============================================================================
      // VÒNG LẶP 2: RENDER LOOP (Độc lập theo giới hạn FPS: V-Sync / 60..240 / Max)
      // Chỉ gọi xuống Wasm khi thực sự vẽ khung hình, dùng alpha nội suy mượt mà.
      // ============================================================================
      const channel = new MessageChannel();
      channel.port1.onmessage = () => {
        renderLoop();
      };

      const scheduleNextRender = () => {
        if (!isRunning) return;
        const limit = fpsLimitRef.current;

        if (limit === 'vsync') {
          animationFrameId = requestAnimationFrame(() => renderLoop());
        } else if (limit === 'unlimited') {
          channel.port2.postMessage(null);
        } else {
          const interval = 1000 / limit;
          const now = performance.now();
          const elapsed = now - lastRenderTime;
          const remaining = interval - elapsed;

          if (limit <= 60) {
            animationFrameId = requestAnimationFrame(() => renderLoop());
          } else if (remaining <= 3.5) {
            channel.port2.postMessage(null);
          } else {
            renderTimerId = window.setTimeout(() => {
              if (isRunning) {
                channel.port2.postMessage(null);
              }
            }, Math.floor(remaining - 2.5));
          }
        }
      };

      const renderLoop = () => {
        if (!isRunning) return;

        // Luôn dùng duy nhất đồng hồ đơn điệu performance.now()
        const now = performance.now();
        const limit = fpsLimitRef.current;

        if (limit === 'vsync' || limit === 'unlimited') {
          lastRenderTime = now;
          wasmBridge.renderFrame(now);
        } else {
          const interval = 1000 / limit;
          const elapsed = now - lastRenderTime;
          if (elapsed >= interval - 0.5) {
            lastRenderTime = now - (elapsed % interval);
            wasmBridge.renderFrame(now);
          }
        }

        scheduleNextRender();
      };

      scheduleNextRender();
    };

    initEngine();

    return () => {
      isRunning = false;
      if (simTimerId !== null) clearInterval(simTimerId);
      if (animationFrameId !== null) cancelAnimationFrame(animationFrameId);
      if (renderTimerId !== null) clearTimeout(renderTimerId);
    };
  }, []);

  // Lắng nghe các sự kiện từ Rust Wasm Engine
  useEffect(() => {
    const unsubscribe = wasmBridge.subscribe((type, payload) => {
      switch (type) {
        case 'SyncState': {
          let parsedTech = null;
          if (payload.tech_state && payload.tech_state.length > 0) {
            try {
              parsedTech = JSON.parse(payload.tech_state);
            } catch (e) {
              // ignore
            }
          }

          let parsedAutoPlay = null;
          if (payload.autoplay_state && payload.autoplay_state.length > 0) {
            try {
              parsedAutoPlay = JSON.parse(payload.autoplay_state);
            } catch (e) {
              // ignore
            }
          }

          setGameState((prev) => ({
            ...prev,
            player_minerals: payload.player_minerals,
            player_energy: payload.player_energy,
            player_supply: payload.player_supply,
            player_max_supply: payload.player_max_supply,
            tech_state: parsedTech || prev.tech_state,
            autoplay_state: parsedAutoPlay || prev.autoplay_state,
            game_time: payload.game_time,
            is_paused: payload.is_paused,
          }));
          break;
        }
        case 'OpenTechTreeModal': {
          setIsTechTreeOpen(true);
          break;
        }
        case 'OpenSettingsModal': {
          setIsSettingsOpen(true);
          break;
        }
        case 'OpenAutoPlayModal': {
          setIsAutoPlayOpen(true);
          break;
        }
        case 'OpenGuideModal': {
          setIsGuideOpen(true);
          break;
        }
        case 'GameOver': {
          setGameOverState({
            isOpen: true,
            playerWon: payload.player_won,
          });
          break;
        }
      }
    });

    return () => {
      unsubscribe();
    };
  }, []);

  // Điều khiển chuột & phím gắn vào Canvas
  const handleMouseDown = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    wasmBridge.onMouseDown(e.button, e.clientX, e.clientY);
  }, []);

  const handleMouseUp = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    wasmBridge.onMouseUp(e.button, e.clientX, e.clientY);
  }, []);

  const handleMouseMove = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    wasmBridge.onMouseMove(e.clientX, e.clientY);
  }, []);

  const handleWheel = useCallback((e: React.WheelEvent<HTMLCanvasElement>) => {
    wasmBridge.onMouseWheel(e.deltaY);
  }, []);

  const handleContextMenu = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    e.preventDefault(); // Ngăn menu chuột phải mặc định của trình duyệt
  }, []);

  const handleCloseTechTree = useCallback(() => {
    setIsTechTreeOpen(false);
  }, []);

  // Tự động tạm dừng game khi mở bất kỳ bảng nào (TechTree, Guide, Settings) theo cài đặt autoPause
  const isAnyPauseModalOpen = isTechTreeOpen || isGuideOpen || isSettingsOpen;
  const wasAutoPausedRef = useRef(false);

  useEffect(() => {
    if (!isInGame) return;

    if (isAnyPauseModalOpen) {
      if (settings.autoPause) {
        wasmBridge.sendCommand({ type: 'SetPaused', paused: true });
        wasAutoPausedRef.current = true;
      } else if (wasAutoPausedRef.current) {
        // Tắt autoPause trong khi modal vẫn mở -> tiếp tục trận đấu
        wasmBridge.sendCommand({ type: 'SetPaused', paused: false });
        wasAutoPausedRef.current = false;
      }
    } else {
      // Khi tất cả bảng đóng -> bỏ tạm dừng nếu trước đó do autoPause kích hoạt
      if (wasAutoPausedRef.current) {
        wasmBridge.sendCommand({ type: 'SetPaused', paused: false });
        wasAutoPausedRef.current = false;
      }
    }
  }, [isInGame, isAnyPauseModalOpen, settings.autoPause]);

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.code === 'KeyT') {
        setIsTechTreeOpen((prev) => !prev);
        return;
      } else if (e.code === 'KeyB') {
        setIsAutoPlayOpen((prev) => !prev);
        return;
      } else if (e.code === 'KeyH') {
        setIsGuideOpen((prev) => !prev);
        return;
      } else if (e.code === 'Escape') {
        if (isGuideOpen) {
          setIsGuideOpen(false);
          return;
        } else if (isTechTreeOpen) {
          setIsTechTreeOpen(false);
          return;
        } else if (isAutoPlayOpen) {
          setIsAutoPlayOpen(false);
          return;
        } else {
          setIsSettingsOpen((prev) => !prev);
          return;
        }
      }
      wasmBridge.onKeyDown(e.code);
    };

    const onKeyUp = (e: KeyboardEvent) => {
      wasmBridge.onKeyUp(e.code);
    };

    window.addEventListener('keydown', onKeyDown);
    window.addEventListener('keyup', onKeyUp);

    return () => {
      window.removeEventListener('keydown', onKeyDown);
      window.removeEventListener('keyup', onKeyUp);
    };
  }, [isTechTreeOpen, isGuideOpen, isSettingsOpen, isAutoPlayOpen]);

  const handleStartGame = (difficulty: string, slots?: PlayerSlotConfig[]) => {
    setIsInGame(true);
    setGameOverState({ isOpen: false, playerWon: false });
    setIsTechTreeOpen(false);
    setIsSettingsOpen(false);
    setIsGuideOpen(false);
    wasAutoPausedRef.current = false;
    setGameState(initialGameState);
    wasmBridge.sendCommand({
      type: 'StartGame',
      difficulty,
      slots,
    });
    soundEngine.playSound('order');
  };

  const handleResearchTech = (techId: string) => {
    wasmBridge.sendCommand({
      type: 'UpgradeTech',
      tech_id: techId,
    });
  };

  const handleUpdateSettings = useCallback((newSettings: StoredSettings) => {
    setSettings(newSettings);
    fpsLimitRef.current = newSettings.fpsLimit;
    try {
      localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(newSettings));
    } catch (e) {
      // ignore
    }
    wasmBridge.sendCommand({
      type: 'UpdateSettings',
      sound_volume: newSettings.soundVolume,
      camera_speed: newSettings.cameraSpeed,
      auto_pause: newSettings.autoPause,
      show_grid: newSettings.showGrid,
      fps_limit: String(newSettings.fpsLimit),
    });
  }, []);

  const handleRestartGame = () => {
    setIsInGame(true);
    setGameOverState({ isOpen: false, playerWon: false });
    setIsTechTreeOpen(false);
    setIsSettingsOpen(false);
    setIsGuideOpen(false);
    wasAutoPausedRef.current = false;
    setGameState(initialGameState);
    wasmBridge.sendCommand({ type: 'RestartGame' });
    soundEngine.playSound('order');
  };

  const handleBackToMenu = () => {
    setIsInGame(false);
    setGameOverState({ isOpen: false, playerWon: false });
    setIsTechTreeOpen(false);
    setIsSettingsOpen(false);
    setIsGuideOpen(false);
    wasAutoPausedRef.current = false;
    wasmBridge.sendCommand({ type: 'SetPaused', paused: true });
  };

  const handleTogglePriorityTech = (techId: string) => {
    wasmBridge.sendCommand({
      type: 'TogglePriorityTech',
      tech_id: techId,
    });
  };

  const handleUpdateAutoPlayConfig = (newCfg: AutoPlayConfig) => {
    setGameState((prev) => ({
      ...prev,
      autoplay_state: newCfg,
    }));
    wasmBridge.sendCommand({
      type: 'UpdateAutoPlayConfig',
      config_json: JSON.stringify(newCfg),
    });
  };

  return (
    <div className="relative w-full h-full overflow-hidden select-none bg-[#030d1a]">
      {/* Canvas Game World (WebGPU) */}
      <canvas
        ref={canvasRef}
        onMouseDown={handleMouseDown}
        onMouseUp={handleMouseUp}
        onMouseMove={handleMouseMove}
        onWheel={handleWheel}
        onContextMenu={handleContextMenu}
        className="block w-full h-full cursor-crosshair"
      />

      {/* Multi-Canvas HUD Layering */}
      {/* 1. Top Bar Canvas: 100vw x 48px */}
      <canvas
        ref={topBarCanvasRef}
        className="absolute top-0 left-0 w-full h-[48px] pointer-events-none block z-10"
      />

      {/* 2. Minimap Canvas: 220 x 220px cố định góc dưới trái */}
      <canvas
        ref={minimapCanvasRef}
        className="absolute bottom-[10px] left-[10px] w-[220px] h-[220px] pointer-events-none block z-10"
      />

      {/* 3. Selection Info Panel Canvas: 480 x 180px đáy giữa màn hình */}
      <canvas
        ref={selectionCanvasRef}
        className="absolute bottom-[10px] left-1/2 -translate-x-1/2 w-[480px] h-[180px] pointer-events-none block z-10"
      />

      {/* 4. Command Card Canvas: 285 x 180px đáy góc phải màn hình */}
      <canvas
        ref={commandCardCanvasRef}
        className="absolute bottom-[10px] right-[10px] w-[285px] h-[180px] pointer-events-none block z-10"
      />

      {/* 5. Dynamic Overlay Canvas: Box select, Attack-move cursor, Floating texts */}
      <canvas
        ref={overlayCanvasRef}
        className="absolute inset-0 pointer-events-none block w-full h-full z-20"
      />

      {/* UI Ngoài vòng lặp do React quản lý */}
      {!isInGame && (
        <MainMenu
          onStartGame={handleStartGame}
          onOpenGuide={() => setIsGuideOpen(true)}
          onOpenSettings={() => setIsSettingsOpen(true)}
        />
      )}

      <TechTreeModal
        isOpen={isTechTreeOpen}
        onClose={handleCloseTechTree}
        techState={gameState.tech_state}
        minerals={gameState.player_minerals}
        energy={gameState.player_energy}
        onResearch={handleResearchTech}
        priorityTechs={gameState.autoplay_state.priority_techs}
        onTogglePriorityTech={handleTogglePriorityTech}
        autoPause={settings.autoPause}
      />

      <AutoPlayModal
        isOpen={isAutoPlayOpen}
        onClose={() => setIsAutoPlayOpen(false)}
        autoPlayConfig={gameState.autoplay_state}
        techState={gameState.tech_state}
        onUpdateConfig={handleUpdateAutoPlayConfig}
      />

      {/* Mini status badge khi đang trong trận */}
      {isInGame && gameState.autoplay_state.enabled && (
        <div
          onClick={() => setIsAutoPlayOpen(true)}
          className="fixed top-14 right-4 z-40 bg-slate-950/90 border border-emerald-500/80 rounded-xl px-3 py-1.5 shadow-[0_0_15px_rgba(16,185,129,0.3)] flex items-center gap-2.5 font-mono text-xs cursor-pointer hover:bg-slate-900 transition-colors"
        >
          <Bot className="w-4 h-4 text-emerald-400 animate-spin-slow" />
          <span className="text-emerald-300 font-bold">AUTO RUNNING</span>
          <div className="flex items-center gap-1.5 text-[10px] text-slate-300 border-l border-slate-700 pl-2">
            {gameState.autoplay_state.loops.map((l, i) => (
              <span
                key={l.id}
                className={`px-1.5 py-0.5 rounded ${
                  l.enabled
                    ? 'bg-emerald-950 text-emerald-300 border border-emerald-600'
                    : 'bg-slate-900 text-slate-500'
                }`}
              >
                {i === 0 ? 'SX' : i === 1 ? 'QS' : `L${i + 1}`}: #{l.current_card_index + 1}
              </span>
            ))}
          </div>
        </div>
      )}

      <SettingsModal
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
        soundVolume={settings.soundVolume}
        cameraSpeed={settings.cameraSpeed}
        autoPause={settings.autoPause}
        showGrid={settings.showGrid}
        fpsLimit={settings.fpsLimit}
        isInGame={isInGame}
        onUpdateSettings={handleUpdateSettings}
        onRestartGame={handleRestartGame}
        onBackToMenu={handleBackToMenu}
      />

      <GuideModal
        isOpen={isGuideOpen}
        onClose={() => setIsGuideOpen(false)}
        gameConfig={gameConfig}
        autoPause={settings.autoPause}
        isInGame={isInGame}
      />

      <GameOverModal
        isOpen={gameOverState.isOpen}
        playerWon={gameOverState.playerWon}
        gameTime={gameState.game_time}
        onRestart={handleRestartGame}
        onBackToMenu={handleBackToMenu}
      />
    </div>
  );
};

