# kim1

KIM-1 microcomputer emulator ([MCS6500](https://en.wikipedia.org/wiki/KIM-1)).

| Component | Address   | Notes |
|-----------|-----------|-------|
| RAM       | `$0000`   | 1 KB  |
| RIOT 6530 | `$1700`   | ROM 1 (1 KB) + I/O + timer |
| RIOT 6530 | `$1C00`   | ROM 2 (1 KB) + I/O + timer |
| Display   | 6× 7-seg  | `cpu-segment` rendering |
| Keypad    | Hex 4×4   | `cpu-keyboard` matrix |

## Public API

```rust
let (rom2, rom3) = load_roms();
let mut kim = kim1::Kim1::new(rom2, rom3);
kim.tick();
```

Interactive runner: `cargo run -p kim1-term`.

## Tests

```
cargo test -p kim1 --lib
cargo test -p kim1 --test ledtest
```

## Dependencies

`mos6502-core`, `mos6502-config`, `cpu-bus`, `cpu-display`, `cpu-keyboard`, `cpu-segment`.
