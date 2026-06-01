# 6502 Emulator — TODO

## Status: 56 documented opcodes ✅, 105 illegal ✅, BCD ✅

---

## 🔴 Krytyczne (wszystkie naprawione)

---

## 🟡 Pozostałe do zrobienia

### CMOS extensions
- [x] **Zero-page indirect (zpg)** — 8 opcodów dla 65C02: ORA/AND/EOR/ADC/STA/LDA/CMP/SBC
- [x] **TSB** — Test and Set Bit (R65C02, 0x04=zp, 0x0C=abs)
- [x] **TRB** — Test and Reset Bit (0x14 zp, 0x1C abs)

### Różnice wariantów
- [x] **Nmos6510/8502/6507 quirks** — własne `CpuQuirks` presety.

### Cycle-accurate
- [x] **`step()` cycle-by-cycle** — zaimplementowany przez state machine w Emulator. Pierwszy cykl = execute(), pozostałe = idle aż do completion.

### Testy
- [x] execute() testy dla każdego opcodu (102 testy)
- [x] Config::R65C02, Nmos6510/8502/6507

---

## 📊 Podsumowanie

| Kategoria | Status |
|-----------|--------|
| Dokumentowane opcody (56) | ✅ Wszystkie w TABLE + execute match |
| Addressing modes (13) | ✅ Wszystkie z page-cross penalty |
| Cykle | ✅ Poprawne (base + page-cross + branch) |
| Flagi | ✅ Wszystkie poprawne dla dokumentowanych opcodów |
| BCD | ✅ Zaimplementowane (NMOS + Ricoh blokada) |
| Illegal opcody (105) | ✅ Wszystkie 105 zaimplementowane |
| CMOS extensions | ✅ STP/WAI/BRA/PHX/Y/PLX/Y/BIT#imm/JMP(abs,X)/zpg/TSB/TRB |
| Cycle-accurate | ✅ cycle-by-cycle przez state machine |
| RMW behavior | ✅ Cmos dummy read, Nmos dummy write |
| Apple 1 PIA | ✅ Przez `is_apple1` flagę + `apple1()` preset |
| Zero warnings | ✅ cargo build — 0 warningów |
| nestest.log | ✅ Przechodzi ~5800 kroków |
| functional_test.bin | ✅ Przechodzi 20M cykli |
