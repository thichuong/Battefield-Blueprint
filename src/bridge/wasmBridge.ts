import init, {
  init_engine,
  engine_step,
  engine_tick,
  engine_render,
  send_command_to_engine,
  poll_events_from_engine,
  resize_canvas,
  handle_mouse_down,
  handle_mouse_up,
  handle_mouse_move,
  handle_mouse_wheel,
  handle_key_down,
  handle_key_up,
  get_game_config,
  set_game_config,
  reset_game_config,
} from '../wasm-pkg/rust_engine.js';
import { soundEngine } from '../audio/soundEngine';

export interface Resistances {
  physical: number;
  energy: number;
  explosive: number;
  arcane: number;
}

export type DamageType = 'Physical' | 'Energy' | 'Explosive' | 'Arcane';

export interface UnitStatsConfig {
  name: string;
  hp: number;
  speed: number;
  attack_range: number;
  attack_damage: number;
  attack_cooldown: number;
  damage_type: DamageType;
  aoe_radius: number;
  resistances: Resistances;
  cost_minerals: number;
  cost_energy: number;
  training_time: number;
}

export interface BuildingStatsConfig {
  name: string;
  hp: number;
  cost_minerals: number;
  cost_energy: number;
  width: number;
  height: number;
  build_time: number;
  resistances: Resistances;
  supply_bonus: number;
  energy_per_sec: number;
  turret_range: number;
  turret_damage: number;
  turret_cooldown: number;
  turret_damage_type: DamageType;
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
  converter_gold_per_100_energy: number;
}

export interface TechItemConfig {
  name: string;
  cost_minerals: number;
  cost_energy: number;
  research_time: number;
}

export interface GameConfig {
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

  current_research: string | null;
  research_progress: number;
  research_total_time: number;
}

export interface EngineSyncState {
  player_minerals: number;
  player_energy: number;
  player_supply: number;
  player_max_supply: number;
  tech_state: TechState;
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

  public async initialize(canvas: HTMLCanvasElement): Promise<void> {
    if (this.isInitialized) return;
    await init();
    init_engine(canvas);
    this.isInitialized = true;
  }

  public tick(dt: number) {
    if (!this.isInitialized) return;
    engine_tick(dt);
    this.pollEvents();
  }

  public render() {
    if (!this.isInitialized) return;
    engine_render();
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

  public subscribe(cb: EngineEventCallback) {
    this.eventListeners.push(cb);
    return () => {
      this.eventListeners = this.eventListeners.filter((l) => l !== cb);
    };
  }

  private notifyListeners(type: string, payload: any) {
    this.eventListeners.forEach((l) => l(type, payload));
  }

  public resize(w: number, h: number) {
    if (!this.isInitialized) return;
    resize_canvas(w, h);
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
    if (!this.isInitialized) return;
    handle_mouse_move(x, y);
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

  public getGameConfig(): GameConfig | null {
    if (!this.isInitialized) return null;
    try {
      const json = get_game_config();
      return JSON.parse(json);
    } catch (e) {
      console.error('Failed to get game config:', e);
      return null;
    }
  }

  public setGameConfig(config: GameConfig | string): boolean {
    if (!this.isInitialized) return false;
    const jsonStr = typeof config === 'string' ? config : JSON.stringify(config);
    return set_game_config(jsonStr);
  }

  public resetGameConfig(): GameConfig | null {
    if (!this.isInitialized) return null;
    try {
      const json = reset_game_config();
      return JSON.parse(json);
    } catch (e) {
      console.error('Failed to reset game config:', e);
      return null;
    }
  }
}

export const wasmBridge = new WasmBridge();

