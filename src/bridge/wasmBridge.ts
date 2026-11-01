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
} from '../wasm-pkg/rust_engine.js';
import { soundEngine } from '../audio/soundEngine';

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
}

export const wasmBridge = new WasmBridge();

