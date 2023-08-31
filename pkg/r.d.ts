/* tslint:disable */
/* eslint-disable */
/**
* @returns {string}
*/
export function wasm_session_header(): string;
/**
* @returns {any}
*/
export function wasm_env(): any;
/**
* @param {string} input
* @returns {boolean}
*/
export function wasm_parses_successfully(input: string): boolean;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly wasm_session_header: (a: number) => void;
  readonly wasm_parses_successfully: (a: number, b: number) => number;
  readonly wasm_env: () => number;
  readonly __wbindgen_export_0: WebAssembly.Table;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly wasm_bindgen__convert__closures__invoke1__h5b77997aa44f29a4: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
