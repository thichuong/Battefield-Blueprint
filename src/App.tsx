import React, { useEffect, useRef, useState, useCallback } from 'react';
import { wasmBridge, EngineSyncState } from './bridge/wasmBridge';
import { MainMenu } from './components/MainMenu';
import { TechTreeModal } from './components/TechTreeModal';
import { SettingsModal } from './components/SettingsModal';
import { GameOverModal } from './components/GameOverModal';
import { soundEngine } from './audio/soundEngine';

export const App: React.FC = () => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const [isInGame, setIsInGame] = useState<boolean>(false);
  const [isTechTreeOpen, setIsTechTreeOpen] = useState<boolean>(false);
  const [isSettingsOpen, setIsSettingsOpen] = useState<boolean>(false);
  const [gameOverState, setGameOverState] = useState<{
    isOpen: boolean;
    playerWon: boolean;
  }>({ isOpen: false, playerWon: false });

  const [gameState, setGameState] = useState<EngineSyncState>({
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
      current_research: null,
      research_progress: 0,
      research_total_time: 0,
    },
    game_time: 0,
    is_paused: false,
  });

  const [settings, setSettings] = useState({
    soundVolume: 0.7,
    cameraSpeed: 1.0,
    autoPause: true,
    showGrid: true,
  });

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

      // Main Game Loop chạy Wasm Step & Render
      const loop = (time: number) => {
        const dt = (time - lastTime) / 1000;
        lastTime = time;

        wasmBridge.step(dt);
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

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.code === 'KeyT') {
        setIsTechTreeOpen((prev) => !prev);
      } else if (e.code === 'Escape') {
        if (isTechTreeOpen) {
          setIsTechTreeOpen(false);
        } else {
          setIsSettingsOpen((prev) => !prev);
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
  }, [isTechTreeOpen]);

  const handleStartGame = (difficulty: string) => {
    setIsInGame(true);
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

  const handleUpdateSettings = (newSettings: typeof settings) => {
    setSettings(newSettings);
    wasmBridge.sendCommand({
      type: 'UpdateSettings',
      sound_volume: newSettings.soundVolume,
      camera_speed: newSettings.cameraSpeed,
      auto_pause: newSettings.autoPause,
      show_grid: newSettings.showGrid,
    });
  };

  const handleRestartGame = () => {
    setGameOverState({ isOpen: false, playerWon: false });
    setIsTechTreeOpen(false);
    setIsSettingsOpen(false);
    wasmBridge.sendCommand({ type: 'RestartGame' });
    wasmBridge.sendCommand({ type: 'StartGame', difficulty: 'normal' });
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
        onClose={() => {
          setIsTechTreeOpen(false);
          if (settings.autoPause) {
            wasmBridge.sendCommand({ type: 'SetPaused', paused: false });
          }
        }}
        techState={gameState.tech_state}
        minerals={gameState.player_minerals}
        energy={gameState.player_energy}
        onResearch={handleResearchTech}
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
      />

      <GameOverModal
        isOpen={gameOverState.isOpen}
        playerWon={gameOverState.playerWon}
        gameTime={gameState.game_time}
        onRestart={handleRestartGame}
      />
    </div>
  );
};

