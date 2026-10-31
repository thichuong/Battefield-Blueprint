import React, { useEffect, useRef, useState, useCallback } from 'react';
import { wasmBridge, EngineSyncState } from './bridge/wasmBridge';
import { MainMenu } from './components/MainMenu';
import { TechTreeModal } from './components/TechTreeModal';
import { SettingsModal } from './components/SettingsModal';
import { GameOverModal } from './components/GameOverModal';
import { soundEngine } from './audio/soundEngine';

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
  game_time: 0,
  is_paused: false,
};

const SETTINGS_STORAGE_KEY = 'battlefield_blueprint_settings';

function loadStoredSettings() {
  try {
    const raw = localStorage.getItem(SETTINGS_STORAGE_KEY);
    if (raw) {
      return {
        soundVolume: 0.7,
        cameraSpeed: 1.0,
        autoPause: true,
        showGrid: true,
        ...JSON.parse(raw),
      };
    }
  } catch (e) {
    // ignore
  }
  return {
    soundVolume: 0.7,
    cameraSpeed: 1.0,
    autoPause: true,
    showGrid: true,
  };
}

export const App: React.FC = () => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const [isInGame, setIsInGame] = useState<boolean>(false);
  const [isTechTreeOpen, setIsTechTreeOpen] = useState<boolean>(false);
  const [isSettingsOpen, setIsSettingsOpen] = useState<boolean>(false);
  const [gameOverState, setGameOverState] = useState<{
    isOpen: boolean;
    playerWon: boolean;
  }>({ isOpen: false, playerWon: false });

  const [gameState, setGameState] = useState<EngineSyncState>(initialGameState);
  const [settings, setSettings] = useState(loadStoredSettings);

  // Khởi tạo Wasm và Canvas
  useEffect(() => {
    let animationFrameId: number;
    let lastTime = performance.now();

    const initEngine = async () => {
      if (!canvasRef.current) return;
      await wasmBridge.initialize(canvasRef.current);

      const resize = () => {
        if (!canvasRef.current) return;
        const width = window.innerWidth;
        const height = window.innerHeight;
        canvasRef.current.width = width;
        canvasRef.current.height = height;
        wasmBridge.resize(width, height);
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
      });

      // Tách 2 vòng lặp:
      // Vòng lặp 1: Fixed Timestep Accumulator cho Logic Mô phỏng (60 Ticks/giây)
      // Vòng lặp 2: Render theo tần số quét màn hình (requestAnimationFrame / VSync)
      const FIXED_TIMESTEP = 1 / 60;
      const MAX_SUB_STEPS = 5;
      let accumulator = 0;

      const loop = (time: number) => {
        let frameTime = (time - lastTime) / 1000;
        lastTime = time;

        // Giới hạn delta time tối đa 0.25s khi tab bị ẩn hoặc lag đột ngột (tránh Spiral of Death)
        if (frameTime > 0.25) {
          frameTime = 0.25;
        }

        accumulator += frameTime;

        // VÒNG LẶP 1: LOGIC / SIMULATION (Fixed 60 Hz)
        let subSteps = 0;
        while (accumulator >= FIXED_TIMESTEP && subSteps < MAX_SUB_STEPS) {
          wasmBridge.tick(FIXED_TIMESTEP);
          accumulator -= FIXED_TIMESTEP;
          subSteps++;
        }

        // VÒNG LẶP 2: RENDER (Chạy theo tần số màn hình)
        wasmBridge.render();

        animationFrameId = requestAnimationFrame(loop);
      };
      animationFrameId = requestAnimationFrame(loop);
    };

    initEngine();

    return () => {
      cancelAnimationFrame(animationFrameId);
    };
  }, []);

  // Lắng nghe các sự kiện từ Rust Wasm Engine
  useEffect(() => {
    const unsubscribe = wasmBridge.subscribe((type, payload) => {
      switch (type) {
        case 'SyncState': {
          let parsedTech = null;
          try {
            parsedTech = JSON.parse(payload.tech_state);
          } catch (e) {
            // ignore
          }

          setGameState((prev) => ({
            ...prev,
            player_minerals: payload.player_minerals,
            player_energy: payload.player_energy,
            player_supply: payload.player_supply,
            player_max_supply: payload.player_max_supply,
            tech_state: parsedTech || prev.tech_state,
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

  const handleOpenTechTree = useCallback(() => {
    setIsTechTreeOpen(true);
    if (settings.autoPause) {
      wasmBridge.sendCommand({ type: 'SetPaused', paused: true });
    }
  }, [settings.autoPause]);

  const handleCloseTechTree = useCallback(() => {
    setIsTechTreeOpen(false);
    if (settings.autoPause) {
      wasmBridge.sendCommand({ type: 'SetPaused', paused: false });
    }
  }, [settings.autoPause]);

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.code === 'KeyT') {
        if (isTechTreeOpen) {
          handleCloseTechTree();
        } else {
          handleOpenTechTree();
        }
        return;
      } else if (e.code === 'Escape') {
        if (isTechTreeOpen) {
          handleCloseTechTree();
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
  }, [isTechTreeOpen, handleCloseTechTree, handleOpenTechTree]);

  const handleStartGame = (difficulty: string) => {
    setIsInGame(true);
    setGameOverState({ isOpen: false, playerWon: false });
    setIsTechTreeOpen(false);
    setIsSettingsOpen(false);
    setGameState(initialGameState);
    wasmBridge.sendCommand({
      type: 'StartGame',
      difficulty,
    });
    soundEngine.playSound('order');
  };

  const handleResearchTech = (techId: string) => {
    wasmBridge.sendCommand({
      type: 'UpgradeTech',
      tech_id: techId,
    });
  };

  const handleUpdateSettings = useCallback((newSettings: typeof settings) => {
    setSettings(newSettings);
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
    });
  }, []);

  const handleRestartGame = () => {
    setIsInGame(true);
    setGameOverState({ isOpen: false, playerWon: false });
    setIsTechTreeOpen(false);
    setIsSettingsOpen(false);
    setGameState(initialGameState);
    wasmBridge.sendCommand({ type: 'RestartGame' });
    soundEngine.playSound('order');
  };

  const handleBackToMenu = () => {
    setIsInGame(false);
    setGameOverState({ isOpen: false, playerWon: false });
    setIsTechTreeOpen(false);
    setIsSettingsOpen(false);
    wasmBridge.sendCommand({ type: 'SetPaused', paused: true });
  };

  return (
    <div className="relative w-full h-full overflow-hidden select-none bg-[#030d1a]">
      {/* Canvas Game hoàn toàn do Wasm vẽ */}
      <canvas
        ref={canvasRef}
        onMouseDown={handleMouseDown}
        onMouseUp={handleMouseUp}
        onMouseMove={handleMouseMove}
        onWheel={handleWheel}
        onContextMenu={handleContextMenu}
        className="block w-full h-full cursor-crosshair"
      />

      {/* UI Ngoài vòng lặp do React quản lý */}
      {!isInGame && <MainMenu onStartGame={handleStartGame} />}

      <TechTreeModal
        isOpen={isTechTreeOpen}
        onClose={handleCloseTechTree}
        techState={gameState.tech_state}
        minerals={gameState.player_minerals}
        energy={gameState.player_energy}
        onResearch={handleResearchTech}
        autoPause={settings.autoPause}
        onToggleAutoPause={(val) => {
          const newSettings = { ...settings, autoPause: val };
          handleUpdateSettings(newSettings);
          if (isTechTreeOpen) {
            wasmBridge.sendCommand({ type: 'SetPaused', paused: val });
          }
        }}
      />

      <SettingsModal
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
        soundVolume={settings.soundVolume}
        cameraSpeed={settings.cameraSpeed}
        autoPause={settings.autoPause}
        showGrid={settings.showGrid}
        onUpdateSettings={handleUpdateSettings}
        onRestartGame={handleRestartGame}
        onBackToMenu={handleBackToMenu}
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

