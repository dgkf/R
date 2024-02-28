/* tslint:disable */
/* eslint-disable */
/**
* @param {any} js
* @returns {Cli}
*/
export function wasm_args(js: any): Cli;
/**
* @param {any} args
* @returns {string}
*/
export function wasm_session_header(args: any): string;
/**
* @param {any} args
* @returns {any}
*/
export function wasm_runtime(args: any): any;
/**
* @param {any} args
* @param {string} input
* @returns {boolean}
*/
export function wasm_parses_successfully(args: any, input: string): boolean;
/**
* returns a stream of strings. Each pair represents a style and text
* @param {any} args
* @param {string} input
* @returns {any[]}
*/
export function wasm_highlight(args: any, input: string): any[];
/**
*/
export enum Localization {
  En = 0,
  Es = 1,
  Cn = 2,
  Pirate = 3,
  Emoji = 4,
}
/**
*/
export enum Experiment {
  TailCalls = 0,
  RestArgs = 1,
}
/**
* Run the R REPL
*/
export class Cli {
  free(): void;
/**
* Enable experimental language features
*/
  experiments: any[];
/**
* Localization to use for runtime
*/
  locale: Localization;
/**
* Show the extended warranty information at startup
*/
  warranty: boolean;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly wasm_args: (a: number) => number;
  readonly wasm_session_header: (a: number, b: number) => void;
  readonly wasm_runtime: (a: number) => number;
  readonly wasm_parses_successfully: (a: number, b: number, c: number) => number;
  readonly wasm_highlight: (a: number, b: number, c: number, d: number) => void;
  readonly __wbg_cli_free: (a: number) => void;
  readonly __wbg_get_cli_locale: (a: number) => number;
  readonly __wbg_set_cli_locale: (a: number, b: number) => void;
  readonly __wbg_get_cli_warranty: (a: number) => number;
  readonly __wbg_set_cli_warranty: (a: number, b: number) => void;
  readonly __wbg_get_cli_experiments: (a: number, b: number) => void;
  readonly __wbg_set_cli_experiments: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly wasm_bindgen__convert__closures__invoke1__hb65858ceeaa6016b: (a: number, b: number, c: number, d: number, e: number) => void;
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
