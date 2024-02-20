/* tslint:disable */
/* eslint-disable */
/**
* @param {boolean} warranty
* @param {string | undefined} locale
* @returns {Cli}
*/
export function wasm_cli_args(warranty: boolean, locale?: string): Cli;
/**
* @param {Cli} args
* @returns {string}
*/
export function wasm_session_header(args: Cli): string;
/**
* @param {Cli} args
* @returns {any}
*/
export function wasm_runtime(args: Cli): any;
/**
* @param {Cli} args
* @param {string} input
* @returns {boolean}
*/
export function wasm_parses_successfully(args: Cli, input: string): boolean;
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
* Run the R REPL
*/
export class Cli {
  free(): void;
/**
* Localization to use for runtime
*/
  locale: number;
/**
* Show the extended warranty information at startup
*/
  warranty: boolean;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_cli_free: (a: number) => void;
  readonly __wbg_get_cli_locale: (a: number) => number;
  readonly __wbg_set_cli_locale: (a: number, b: number) => void;
  readonly __wbg_get_cli_warranty: (a: number) => number;
  readonly __wbg_set_cli_warranty: (a: number, b: number) => void;
  readonly wasm_cli_args: (a: number, b: number, c: number) => number;
  readonly wasm_session_header: (a: number, b: number) => void;
  readonly wasm_runtime: (a: number) => number;
  readonly wasm_parses_successfully: (a: number, b: number, c: number) => number;
  readonly __wbindgen_export_0: WebAssembly.Table;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly wasm_bindgen__convert__closures__invoke1__h5b85ada496addd6e: (a: number, b: number, c: number, d: number, e: number) => void;
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
