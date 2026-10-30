/* tslint:disable */
/* eslint-disable */

export enum BuildingType {
    CommandCenter = 0,
    SupplyDepot = 1,
    PowerReactor = 2,
    Barracks = 3,
    TurretTower = 4,
}

export enum DamageType {
    Physical = 0,
    Energy = 1,
    Explosive = 2,
    Arcane = 3,
}

export enum Team {
    Player = 0,
    Enemy = 1,
}

export enum UnitType {
    Engineer = 0,
    ShockTrooper = 1,
    Rifleman = 2,
    Vanguard = 3,
    LaserMarksman = 4,
    DarkRocketeer = 5,
    TaoistSwordmaster = 6,
    VoidWarlock = 7,
    BehemothTitan = 8,
    AcidBaneling = 9,
}

/**
 * Vòng lặp 2: Vẽ khung hình lên Canvas (chạy theo requestAnimationFrame / VSync)
 */
export function engine_render(): void;

/**
 * Giữ lại hàm cũ để đảm bảo tính tương thích ngược
 */
export function engine_step(dt: number): void;

/**
 * Vòng lặp 1: Cập nhật mô phỏng logic (AI, kinh tế, di chuyển, va chạm, đạn)
 */
export function engine_tick(dt: number): void;

export function handle_key_down(key: string): void;

export function handle_key_up(key: string): void;

export function handle_mouse_down(button: number, client_x: number, client_y: number): void;

export function handle_mouse_move(client_x: number, client_y: number): void;

export function handle_mouse_up(button: number, client_x: number, client_y: number): void;

export function handle_mouse_wheel(delta_y: number): void;

export function init_engine(canvas: HTMLCanvasElement): void;

export function poll_events_from_engine(): string;

export function resize_canvas(width: number, height: number): void;

export function send_command_to_engine(cmd_json: string): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly handle_key_down: (a: number, b: number) => void;
    readonly handle_key_up: (a: number, b: number) => void;
    readonly init_engine: (a: any) => [number, number];
    readonly poll_events_from_engine: () => [number, number];
    readonly send_command_to_engine: (a: number, b: number) => void;
    readonly engine_step: (a: number) => void;
    readonly engine_tick: (a: number) => void;
    readonly handle_mouse_down: (a: number, b: number, c: number) => void;
    readonly handle_mouse_up: (a: number, b: number, c: number) => void;
    readonly resize_canvas: (a: number, b: number) => void;
    readonly handle_mouse_wheel: (a: number) => void;
    readonly handle_mouse_move: (a: number, b: number) => void;
    readonly engine_render: () => void;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
