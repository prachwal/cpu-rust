# pet-core

Commodore PET 2001 emulator.

| Component | Address          | Notes |
|-----------|------------------|-------|
| RAM       | `$0000`-$$7FFF  | 32 KB |
| Screen    | `$8000`-$$8FFF  | 4 KB  |
| PIA 1     | `$E810`          | Keyboard matrix |
| PIA 2     | `$E820`          | IEEE-488 |
| VIA 6522  | `$E840`          | User port + VBLANK |
| ROMs      | `$C000`-$$FFFF   | BASIC + Editor + Kernal |

## Public API

```rust
use pet_core::PetBus; // implements Bus
```

WASM wrapper (`PetEmulator`) exposed via `#[wasm_bindgen]`.

## Tests

```
cargo test -p pet-core --lib
```

## Dependencies

`mos6502-core`, `pia-6520`, `via-6522`, `cpu-bus`, `wasm-bindgen`.
