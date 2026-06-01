# cpu-rust

Emulatory procesorów i maszyn w Rust + WASM.

## Struktura

```
crates/
  cpu/               ← uniwersalne komponenty procesorów
    bus/             → cpu-bus: trait Bus { read, write, read_u16, write_u16 }
    chip8/           → chip8-cpu: Cpu + execute (generic: &mut impl Bus)
    display/         → cpu-display: bufor pikseli (width, height, get/set pixel)
    keyboard/        → cpu-keyboard: 16-klawiszowy stan
    memory/          → cpu-memory: płaska pamięć Vec<u8> + MemoryAccess trait
    segment/         → cpu-segment: 7-segment LED patterns + pixel rendering

  6502/
    config/          → mos6502-config: MachineConfig, CpuFamily, CpuQuirks
    core/            → mos6502-core: Cpu, StatusRegister, instruction::execute, Emulator
    memory/          → mos6502-memory: Memory + bank switching + Apple1Pia + wektory

  machines/
    apple1/          → apple1-core: Apple 1 (6502 + PIA + terminal + WASM)
    chip8/           → chip8-machine: CHIP-8/SCHIP/XO-CHIP (chip8-cpu + display + keyboard + WASM)
    pet/             → pet-core: PET 2001 (6502 + PIA + VIA + WASM)

  chips/
    pia-6520/        → pia-6520: PIA 6821
    via-6522/        → via-6522: VIA 6522

  other/
    z80/             → z80-core: Z80 CPU (WIP)
```

## Komendy

```bash
cargo build              # build workspace
cargo test --lib         # testy biblioteczne (bez integracyjnych)
cargo test --lib -p <crate>  # testy jednego crate'a
```

## Konwencje

- `cpu-*` — uniwersalne, bez zależności platformowych
- `mos6502-*` — rodzina 6502
- `chip8-machine` → używa `cpu-bus`, `cpu-display`, `cpu-keyboard`, `cpu-memory` + WASM
- `apple1-core`, `pet-core` → używają `cpu-bus`, `mos6502-core` + WASM
- testy w `src/tests/` (wydzielone `#[path]`)
- WASM: `#[wasm_bindgen]` tylko w `machines/*`
- `Bus` — jedyny trait uniwersalny; CPU/instrukcje przyjmują `&mut impl Bus`
