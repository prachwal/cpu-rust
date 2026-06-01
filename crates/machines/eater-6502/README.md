# eater-6502

Ben Eater-style 6502 computer ([YouTube series](https://eater.net/6502)).

| Component | Address   | Size  |
|-----------|-----------|-------|
| RAM       | `$0000`   | 16 KB |
| ACIA 6551 | `$6000`   | 4 reg |
| ROM       | `$8000`   | 32 KB |

## Public API

```rust
let rom = eater_6502::rom::generate_monitor();
let mut m = eater_6502::Eater6502::new(rom);
m.tick();
```

Also implements `cpu_machine::Machine` and `cpu_machine::SerialMachine` — usable with `serial-term`.

## Tests

```
cargo test -p eater-6502 --lib
```

5 tests: boot, memory, single echo, CR→CR+LF echo, multi-char echo.

## Dependencies

`cpu-bus`, `cpu-machine`, `acia-6551`, `mos6502-core`, `mos6502-config`.
