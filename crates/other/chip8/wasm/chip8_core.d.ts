/* tslint:disable */
/* eslint-disable */

export class Emulator {
    free(): void;
    [Symbol.dispose](): void;
    get_display(): Uint8Array;
    get_sound(): boolean;
    key_down(key: number): void;
    key_up(key: number): void;
    load_rom(data: Uint8Array): void;
    constructor();
    reset(): void;
    tick(): void;
    tick_timers(): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_emulator_free: (a: number, b: number) => void;
    readonly emulator_get_display: (a: number, b: number) => void;
    readonly emulator_get_sound: (a: number) => number;
    readonly emulator_key_down: (a: number, b: number) => void;
    readonly emulator_key_up: (a: number, b: number) => void;
    readonly emulator_load_rom: (a: number, b: number, c: number) => void;
    readonly emulator_new: () => number;
    readonly emulator_reset: (a: number) => void;
    readonly emulator_tick: (a: number) => void;
    readonly emulator_tick_timers: (a: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export2: (a: number, b: number) => number;
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
