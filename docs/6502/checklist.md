# 6502 Implementation Checklist

## Core CPU
- [x] BCD mode: ADC/SBC sprawdzają `cpu.sr.d()`, korygują wynik
- [x] KIL/JAM (02,12,22,32,42,52,62,72,92,B2,D2,F2) → `cpu.halted = true`
- [x] STP (DB) → `cpu.stopped = true` (65C02)
- [x] WAI (CB) → `cpu.waiting = true` (65C02)
- [x] Wszystkie 56 dokumentowanych opcodów
- [x] Wszystkie 13 trybów adresowania

## Illegal opcodes
- [x] *SLO/ASO (03,07,0F,13,17,1B,1F) — ASL+ORA
- [x] *RLA (23,27,2F,33,37,3B,3F) — ROL+AND
- [x] *SRE/LSE (43,47,4F,53,57,5B,5F) — LSR+EOR
- [x] *RRA (63,67,6F,73,77,7B,7F) — ROR+ADC
- [x] *DCP/DCM (C3,C7,CF,D3,D7,DB,DF) — DEC+CMP
- [x] *ISC/INS (E3,E7,EF,F3,F7,FB,FF) — INC+SBC
- [x] *ANC (0B,2B) — AND+bit7→C
- [x] *ALR (4B) — AND+LSR
- [x] *ARR (6B) — AND+ROR (V=bit7^bit6, C=bit7)
- [x] *XAA (8B) — TXA+AND#imm
- [x] *OAL (AB) — ORA#$EE+AND#imm
- [x] *SAX (CB) — A&X - #imm → X
- [x] *LAS (BB) — mem&SP→A,X,SP
- [x] *TAS (9B), *SAY (9C), *XAS (9E), *AXA (9F,93)
- [x] *NOP z readem operandu (~20 opcodów: 04,0C,14,1C,34,3C,44,54,5C,64,74,7C,80,82,89,D4,DC,E2,F4,FC,1A,3A,5A,7A,DA,FA)

## Quirks & warianty
- [x] `undocumented_ops` quirk blokuje wszystkie *-prefix illegal opcody gdy false
- [x] `bcd_available` quirk blokuje BCD gdy false (Ricoh 2A03)
- [x] RMW behavior: `RmwBehavior::Cmos` — dummy read; Nmos — dummy write
- [x] JMP indirect bug tylko dla NMOS (CMOS ma fix)

## 65C02/CMOS extensions
- [x] STP (0xDB)
- [x] WAI (0xCB)
- [x] BRA (0x80)
- [x] PHX (0xDA) / PHY (0x5A)
- [x] PLX (0xFA) / PLY (0x7A)
- [x] BIT #imm (0x89)
- [x] JMP (abs,X) (0x7C)

## Emulator
- [ ] `step()` cycle-accurate (instrukcja-level, cycle-by-cycle przyszłość)
- [x] `reset()` nie niszczy wektorów w pamięci
- [x] Apple 1 PIA — `is_apple1` w MachineConfig, preset `apple1()`
- [x] Martwe feature flags: usunięte z Cargo.toml

## Testy (do rozszerzenia)
- [ ] execute() testy dla każdego opcodu (obecnie ~28/56)
- [x] Memory::Apple1Pia, Bus impl
- [ ] Config::R65C02, Nmos6510/8502/6507

## Drobne (zrobione)
- [x] Zdublowany `MemoryAccess` trait — usunięty z 6502/memory, używa cpu_memory
- [x] Zero warningów przy `cargo build`
