# MOS 6502 — Dokumentacja z projektów zewnętrznych

Zebrane z innych emulatorów 6502 w Rust.

## Projekty referencyjne

| Projekt | Opis | Testy |
|---------|------|-------|
| [mre/mos6502](https://github.com/mre/mos6502) | no_std, przechodzi Klaus Dormann functional test | solid65, 30M instr |
| [KaiWalter/rust6502](https://github.com/KaiWalter/rust6502) | Apple 1 + WASM, component-based design | functional test |
| [amw-zero/6502-rs](https://github.com/amw-zero/6502-rs) | Oryginalny fork mre/mos6502 | — |
| [GarettCooper/emulator_6502](https://github.com/GarettCooper/emulator_6502) | NES-oriented | — |
| [seasalim/retro6502](https://github.com/seasalim/retro6502) | CLI + monitor debug | — |
| [mortenjc/6502sim](https://github.com/mortenjc/6502sim) | Przechodzi Klaus Dormann tests | functional test |

## Testy walidacyjne

- **[Klaus Dormann 6502/65C02 functional tests](https://github.com/Klaus2m5/6502_65C02_functional_tests)** — ~30M instr, testuje wszystkie opcody we wszystkich trybach adresowania w binarnym i BCD
- **[solid65](https://github.com/omarandlorraine/solid65)** — porównuje output różnych emulatorów 6502 między sobą

## Zasoby online

- [masswerk.at 6502 Instruction Set](https://masswerk.at/6502/6502_instruction_set.html) — pełna dokumentacja opcodów
- [Visual 6502](http://visual6502.org/) — symulacja na poziomie tranzystorów
- [6502.org](http://6502.org/) — community resources
- [Easy 6502](https://skilldrick.github.io/easy6502/) — interaktywny tutorial asemblera
- [masswerk.at 6502 Disassembler](https://masswerk.at/6502/disassembler.html)
- [masswerk.at 6502 Assembler](https://masswerk.at/6502/assembler.html)

## Uwagi z implementacji (KaiWalter/rust6502)

### Unsigned overflow w Rust

Rust nie akceptuje przepełnień na unsigned int. W Go (`uint16`) overflow automatycznie wrapuje.
W Rust trzeba użyć `wrapping_sub`:

```rust
// Zamiast:
let temp = cpu.r.a as u16 - fetched;  // panic na overflow

// Użyj:
let temp = cpu.r.a.wrapping_sub(fetched);
```

### Operator precedence

W Go `&` (bitwise AND) ma wyższy priorytet niż `+`. W Rust odwrotnie — `+` ma wyższy priorytet.
Przy konwersji kodu z Go na Rust trzeba dodawać nawiasy:

```rust
// Go: temp & 0x0f + val    → (temp & 0x0f) + val
// Rust: temp & 0x0f + val  → temp & (0x0f + val) ← BŁĄD!
// Rust: (temp & 0x0f) + val ← POPRAWNIE
```

### Zalecenia architektoniczne

- `HashMap` dla adresacji jest wolny — lepiej użyć `Vec` indeksowanego blokami
- Komponenty (RAM, ROM, PIA) powinny być osobnymi bytami połączonymi przez `trait`
- ROM-y powinny mieć osobną przestrzeń adresową, nie być ładowane do jednej dużej 64kB pamięci

## Zastosowania 6502

- Apple II (1977) — Steve Wozniak
- Atari 2600 (1977) — 6507 (13-bit address bus)
- Commodore 64 (1982) — 6510 (z I/O port)
- Nintendo NES (1983) — Ricoh 2A03 (bez BCD)
- KIM-1 — development board
- Sterowanie przemysłowe — niski koszt, niskie zużycie
