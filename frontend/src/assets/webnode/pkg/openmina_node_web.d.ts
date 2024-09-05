/* tslint:disable */
/* eslint-disable */
/**
* Automatically run after wasm is loaded.
*/
export function main(): void;
/**
* @returns {Promise<RpcSender>}
*/
export function run(): Promise<RpcSender>;
/**
* Entry point for web workers
* @param {number} ptr
*/
export function wasm_thread_entry_point(ptr: number): void;
/**
*/
export class RpcSender {
  free(): void;
/**
* @returns {State}
*/
  state(): State;
/**
* @returns {Stats}
*/
  stats(): Stats;
/**
* @returns {Promise<any>}
*/
  status(): Promise<any>;
}
/**
*/
export class State {
  free(): void;
/**
* @returns {Promise<any>}
*/
  peers(): Promise<any>;
/**
* @returns {Promise<any>}
*/
  message_progress(): Promise<any>;
}
/**
*/
export class Stats {
  free(): void;
/**
* @param {number | undefined} [limit]
* @returns {Promise<any>}
*/
  sync(limit?: number): Promise<any>;
/**
* @returns {Promise<any>}
*/
  block_producer(): Promise<any>;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly main: () => void;
  readonly run: () => number;
  readonly stats_sync: (a: number, b: number, c: number) => number;
  readonly stats_block_producer: (a: number) => number;
  readonly __wbg_state_free: (a: number, b: number) => void;
  readonly state_peers: (a: number) => number;
  readonly state_message_progress: (a: number) => number;
  readonly __wbg_stats_free: (a: number, b: number) => void;
  readonly __wbg_rpcsender_free: (a: number, b: number) => void;
  readonly rpcsender_state: (a: number) => number;
  readonly rpcsender_status: (a: number) => number;
  readonly rpcsender_stats: (a: number) => number;
  readonly wasm_thread_entry_point: (a: number) => void;
  readonly memory: WebAssembly.Memory;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_3: WebAssembly.Table;
  readonly _dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hfc3cd25b53215e84: (a: number, b: number) => void;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h45df3e6947fcd651: (a: number, b: number, c: number) => void;
  readonly _dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hd4a56f8647b4502b: (a: number, b: number) => void;
  readonly _dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h92f8e55d458a8e39: (a: number, b: number) => void;
  readonly _dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h90985823836ef819: (a: number, b: number, c: number) => void;
  readonly _dyn_core__ops__function__FnMut___A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hff3e249b51682479: (a: number, b: number, c: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly wasm_bindgen__convert__closures__invoke2_mut__h808c9af25a9c04f4: (a: number, b: number, c: number, d: number) => void;
  readonly __wbindgen_thread_destroy: (a?: number, b?: number, c?: number) => void;
  readonly __wbindgen_start: (a: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput, memory?: WebAssembly.Memory, thread_stack_size?: number }} module - Passing `SyncInitInput` directly is deprecated.
* @param {WebAssembly.Memory} memory - Deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput, memory?: WebAssembly.Memory, thread_stack_size?: number } | SyncInitInput, memory?: WebAssembly.Memory): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput>, memory?: WebAssembly.Memory, thread_stack_size?: number }} module_or_path - Passing `InitInput` directly is deprecated.
* @param {WebAssembly.Memory} memory - Deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput>, memory?: WebAssembly.Memory, thread_stack_size?: number } | InitInput | Promise<InitInput>, memory?: WebAssembly.Memory): Promise<InitOutput>;
