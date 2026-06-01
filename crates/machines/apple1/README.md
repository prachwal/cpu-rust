# apple1-core

Apple 1 emulator — 6502 + 4 KB RAM + WozMon + BASIC + 6821 PIA (keyboard + display).

Built for WASM (`#[wasm_bindgen]`). Exposes `Apple1Emulator::run(count)` for batch execution.

## Components

| Component | Source |
|-----------|--------|
| CPU       | `mos6502-core` |
| PIA 6821  | `pia-6520` |
| Keyboard  | `Apple1Pia` (8-byte FIFO) |
| Display   | 40×24 ASCII buffer via PIA port |
| ROMs      | WozMon (`$FF00`) + BASIC (`$E000`) |

## Public API

```rust
let mut apple1 = apple1_core::Apple1Emulator::new();
apple1.run(1000); // execute instructions
apple1.apple1_press_key(b'A');
let output = apple1.apple1_take_output();
```

## WASM

```
cd wasm/
wasm-pack build --target web
```

## Tests

None (integration test lives in `mos6502-core/tests/apple1_basic.rs`).

## Dependencies

`mos6502-core`, `pia-6520`, `cpu-bus`, `wasm-bindgen`.
