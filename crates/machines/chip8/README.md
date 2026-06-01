# chip8-machine

CHIP-8 / SUPER-CHIP / XO-CHIP emulator with configurable quirks.

| Feature | Support |
|---------|---------|
| CHIP-8  | Full    |
| SCHIP   | Full    |
| XO-CHIP | Planned |

## Public API

```rust
use chip8_machine::Emulator;
let mut emu = Emulator::new();
emu.load_rom(&rom_data);
emu.tick(); // fetch + execute one opcode
```

Supports dynamic reconfiguration via `load_config` / `get_config_json` (quirks, display, audio).

## Display

`cpu-display` pixel buffer, 64×32 (CHIP-8) or 128×64 (SCHIP). Scheme flag controls color mode.

## Tests

```
cargo test -p chip8-machine --lib
```

Located in `src/tests/{lib,config,memory}.rs`.

## Dependencies

`chip8-cpu`, `cpu-bus`, `cpu-display`, `cpu-keyboard`, `cpu-memory`, `serde`, `wasm-bindgen`.
