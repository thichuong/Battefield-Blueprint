import init, {
  init_engine,
  init_engine_webgpu,
  engine_frame,
  engine_sim_step,
  engine_render_frame,
  engine_step,
  engine_tick,
  engine_render,
  send_command_to_engine,
  poll_events_from_engine,
  resize_canvas,
  handle_mouse_down,
  handle_mouse_up,
  handle_mouse_wheel,
  handle_key_down,
  handle_key_up,
  get_game_config_json,
} from '../wasm-pkg/rust_engine.js';
import { soundEngine } from '../audio/soundEngine';

export interface ResistancesConfig {
  physical: number;
  energy: number;
  explosive: number;
  arcane: number;
}

export interface UnitStatsConfig {
  name: string;
  short_name: string;
  description: string;
  hp: number;
  speed: number;
  attack_range: number;
  attack_damage: number;
  attack_cooldown: number;
  damage_type: 'Physical' | 'Energy' | 'Explosive' | 'Arcane';
  aoe_radius: number;
  resistances: ResistancesConfig;
  cost_minerals: number;
  cost_energy: number;
  training_time: number;
  percent_max_hp_dps?: number;
}

export interface BuildingStatsConfig {
  name: string;
  short_name: string;
  description: string;
  hp: number;
  cost_minerals: number;
  cost_energy: number;
  width: number;
  height: number;
  build_time: number;
  resistances: ResistancesConfig;
  supply_bonus: number;
  energy_per_sec: number;
  turret_range: number;
  turret_damage: number;
  turret_cooldown: number;
  turret_damage_type: 'Physical' | 'Energy' | 'Explosive' | 'Arcane';
  turret_projectile_speed: number;
}

export interface EconomyConfig {
  starting_minerals: number;
  starting_energy: number;
  starting_supply: number;
  base_max_supply: number;
  base_energy_per_sec: number;
  worker_carry_capacity: number;
  worker_mining_time: number;
  worker_carry_capacity_upgraded: number;
  worker_mining_time_upgraded: number;
  converter_threshold: number;
  converter_energy_per_sec: number;
  converter_minerals_per_100_energy: number;
}

export interface TechItemConfig {
  name: string;
  branch: string;
  cost_minerals: number;
  cost_energy: number;
  research_time: number;
}

export interface GameConfigData {
  units: Record<string, UnitStatsConfig>;
  buildings: Record<string, BuildingStatsConfig>;
  economy: EconomyConfig;
  techs: Record<string, TechItemConfig>;
}

export interface TechState {
  // Khoa học
  focused_laser: boolean;
  energy_shielding: boolean;
  dark_matter_rockets: boolean;
  titan_alloy_plating: boolean;

  // Ma thuật
  celestial_flying_swords: boolean;
  golden_bell_shield: boolean;
  corrosive_curse: boolean;
  void_cataclysm: boolean;

  // Quái vật
  primeval_scales: boolean;
  behemoth_mutation: boolean;
  acid_banelings: boolean;
  hive_proliferation: boolean;

  // Kinh tế & phụ trợ
  optical_drills: boolean;
  overcharged_reactors: boolean;
  fortified_blueprints: boolean;
  magnetic_rounds: boolean;
  exo_boosters: boolean;
  superheated_blades: boolean;
  overload_strike: boolean;

  active_researches?: ActiveResearch[];
  current_research: string | null;
  research_progress: number;
  research_total_time: number;
}

export interface ActiveResearch {
  tech_id: string;
  branch: string;
  progress: number;
  total_time: number;
}

export type FormationType = 'Line' | 'Wedge' | 'Box' | 'Scatter' | 'Circle' | 'Column';

export interface AutoCondition {
  type:
    | 'Always'
    | 'SupplyNearCap'
    | 'UnitCountLessThan'
    | 'UnitCountGreaterThan'
    | 'MineralsGreaterThan'
    | 'EnergyGreaterThan'
    | 'ReactorPerConverterRatio'
    | 'BuildingCountLessThan';
  unit_type?: string;
  count?: number;
  amount?: number;
  building_type?: string;
}

export interface AutoAction {
  type: 'TrainUnit' | 'BuildStructure' | 'ResearchTech' | 'AttackWave' | 'Delay';
  unit_type?: string;
  building_type?: string;
  tech_id?: string;
  use_priority_queue?: boolean;
  count?: number;
  seconds?: number;
  formation?: FormationType | string;
}

export interface AutoCard {
  id: string;
  name: string;
  enabled: boolean;
  conditions: AutoCondition[];
  action: AutoAction;
}

export interface AutoLoop {
  id: string;
  name: string;
  enabled: boolean;
  current_card_index: number;
  current_card_progress: number;
  card_timer?: number;
  cards: AutoCard[];
}

export interface TurretDefenseLine {
  enabled: boolean;
  p1: [number, number];
  p2: [number, number];
  spacing: number;
  rows: number;
  row_spacing: number;
}

export interface AutoPlayConfig {
  enabled: boolean;
  priority_techs: string[];
  preferred_formation?: FormationType | string;
  turret_defense: TurretDefenseLine;
  loops: AutoLoop[];
}

export interface PlayerSlotConfig {
  slot_index: number;
  active: boolean;
  faction: number;
  is_human: boolean;
}

export interface EngineSyncState {
  player_minerals: number;
  player_energy: number;
  player_supply: number;
  player_max_supply: number;
  player_formation?: FormationType | string;
  tech_state: TechState;
  autoplay_state: AutoPlayConfig;
  game_time: number;
  is_paused: boolean;
}

export type EngineEventCallback = (
  type: string,
  payload?: any
) => void;

class WasmBridge {
  private isInitialized = false;
  private eventListeners: EngineEventCallback[] = [];
  public mouseX = 0;
  public mouseY = 0;
  public isWebGpu = false;

  public async initialize(
    canvas: HTMLCanvasElement,
    uiCanvas?: HTMLCanvasElement | null,
    topBarCanvas?: HTMLCanvasElement | null,
    minimapCanvas?: HTMLCanvasElement | null,
    selectionCanvas?: HTMLCanvasElement | null,
    commandCardCanvas?: HTMLCanvasElement | null,
    overlayCanvas?: HTMLCanvasElement | null
  ): Promise<boolean> {
    if (this.isInitialized) return this.isWebGpu;
    await init();

    try {
      this.isWebGpu = await init_engine_webgpu(
        canvas,
        uiCanvas,
        topBarCanvas,
        minimapCanvas,
        selectionCanvas,
        commandCardCanvas,
        overlayCanvas
      );
      if (this.isWebGpu) {
        console.log(
          '%c[WGPU] Engine initialized with Multi-Canvas GPU acceleration!',
          'color: #10b981; font-weight: bold;'
        );
      }
    } catch (e) {
      console.warn('init_engine_webgpu error, fallback to 2d:', e);
      init_engine(canvas);
      this.isWebGpu = false;
    }
    this.isInitialized = true;
    return this.isWebGpu;
  }

  /**
   * Vòng lặp 1 (Độc lập): Cập nhật logic mô phỏng cố định 60Hz theo thời gian thực.
   */
  public simStep(nowMs: number) {
    if (!this.isInitialized) return;
    const flags = engine_sim_step(nowMs);
    if (flags !== 0) {
      this.pollEvents();
    }
  }

  /**
   * Vòng lặp 2 (Độc lập): Chỉ dựng hình (Render) và nội suy vị trí theo FPS.
   */
  public renderFrame(nowMs: number) {
    if (!this.isInitialized) return;
    const flags = engine_render_frame(nowMs, this.mouseX, this.mouseY);
    if (flags !== 0) {
      this.pollEvents();
    }
  }

  /**
   * Vòng lặp kết hợp (giữ tương thích ngược)
   */
  public frame(nowMs: number, shouldRender: boolean) {
    if (!this.isInitialized) return;
    const flags = engine_frame(nowMs, this.mouseX, this.mouseY, shouldRender);
    if (flags !== 0) {
      this.pollEvents();
    }
  }

  public tick(dt: number) {
    if (!this.isInitialized) return;
    engine_tick(dt);
    this.pollEvents();
  }

  public render(alpha?: number) {
    if (!this.isInitialized) return;
    engine_render(alpha ?? 1.0);
  }

  public pollEvents() {
    const rawEvents = poll_events_from_engine();
    if (rawEvents && rawEvents !== '[]') {
      try {
        const events = JSON.parse(rawEvents);
        for (const ev of events) {
          if (ev.type === 'PlaySound') {
            soundEngine.playSound(ev.sound);
          }
          this.notifyListeners(ev.type, ev);
        }
      } catch (err) {
        console.error('Failed to parse events from Wasm:', err);
      }
    }
  }

  public step(dt: number) {
    if (!this.isInitialized) return;
    engine_step(dt);
    this.pollEvents();
  }

  public sendCommand(cmd: any) {
    if (!this.isInitialized) return;
    send_command_to_engine(JSON.stringify(cmd));
  }

  public getGameConfig(): GameConfigData | null {
    if (!this.isInitialized) return null;
    try {
      const raw = get_game_config_json();
      if (raw) {
        return JSON.parse(raw);
      }
    } catch (e) {
      console.error('Failed to get game config:', e);
    }
    return null;
  }

  public setFormation(formation: FormationType | string) {
    this.sendCommand({
      type: 'SetFormation',
      formation,
    });
  }

  public subscribe(cb: EngineEventCallback) {
    this.eventListeners.push(cb);
    return () => {
      this.eventListeners = this.eventListeners.filter((l) => l !== cb);
    };
  }

  private notifyListeners(type: string, payload: any) {
    this.eventListeners.forEach((l) => l(type, payload));
  }

  public resize(w: number, h: number, dpr?: number) {
    if (!this.isInitialized) return;
    resize_canvas(w, h, dpr ?? window.devicePixelRatio ?? 1.0);
  }

  public onMouseDown(button: number, x: number, y: number) {
    if (!this.isInitialized) return;
    handle_mouse_down(button, x, y);
  }

  public onMouseUp(button: number, x: number, y: number) {
    if (!this.isInitialized) return;
    handle_mouse_up(button, x, y);
  }

  public onMouseMove(x: number, y: number) {
    this.mouseX = x;
    this.mouseY = y;
  }

  public onMouseWheel(deltaY: number) {
    if (!this.isInitialized) return;
    handle_mouse_wheel(deltaY);
  }

  public onKeyDown(key: string) {
    if (!this.isInitialized) return;
    handle_key_down(key);
  }

  public onKeyUp(key: string) {
    if (!this.isInitialized) return;
    handle_key_up(key);
  }
}

export const wasmBridge = new WasmBridge();

