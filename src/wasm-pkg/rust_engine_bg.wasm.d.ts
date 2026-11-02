/* tslint:disable */
/* eslint-disable */
export const memory: WebAssembly.Memory;
export const engine_render: (a: number) => void;
export const engine_tick: (a: number) => void;
export const handle_key_down: (a: number, b: number) => void;
export const handle_key_up: (a: number, b: number) => void;
export const init_engine: (a: any) => [number, number];
export const poll_events_from_engine: () => [number, number];
export const resize_canvas: (a: number, b: number, c: number) => void;
export const send_command_to_engine: (a: number, b: number) => void;
export const engine_step: (a: number) => void;
export const handle_mouse_down: (a: number, b: number, c: number) => void;
export const handle_mouse_up: (a: number, b: number, c: number) => void;
export const handle_mouse_wheel: (a: number) => void;
export const handle_mouse_move: (a: number, b: number) => void;
export const __wbindgen_exn_store: (a: number) => void;
export const __externref_table_alloc: () => number;
export const __wbindgen_externrefs: WebAssembly.Table;
export const __wbindgen_malloc: (a: number, b: number) => number;
export const __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
export const __externref_table_dealloc: (a: number) => void;
export const __wbindgen_free: (a: number, b: number, c: number) => void;
export const __wbindgen_start: () => void;
