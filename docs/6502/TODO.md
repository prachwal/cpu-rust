# 6502 Emulator — TODO

## Status: 56 documented opcodes ✅, 105 illegal ✅, BCD ✅

---

## 🔴 Krytyczne (wszystkie naprawione)

---

## 🟡 Pozostałe do zrobienia

### CMOS extensions
- [ ] **LDA/STA/ADC/… (zpg)** — 0x12, 0x32, 0x52, 0x72, 0x92, 0xB2, 0xD2, 0xF2 to nowe tryby w 65C02
- [ ] **TSB/TRB** — Test and set/reset bit (R65C02)

### Różnice wariantów
- [ ] **Nmos6510/8502/6507 quirks** — wszystkie używają `CpuQuirks::nmos()` zamiast własnych presetów.

### Cycle-accurate
- [ ] **`step()` cycle-by-cycle** — obecnie instruction-level, cycle-by-cycle wymaga refaktoryzacji execute()

### Testy
- [ ] execute() testy dla każdego opcodu (obecnie ~28/56)
- [ ] Config::R65C02, Nmos6510/8502/6507

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
| CMOS extensions | ✅ STP/WAI/BRA/PHX/Y/PLX/Y/BIT#imm/JMP(abs,X), ❌ TSB/TRB |
| Cycle-accurate | ⏳ Instruction-level, cycle-by-cycle przyszłość |
| RMW behavior | ✅ Cmos dummy read, Nmos dummy write |
| Apple 1 PIA | ✅ Przez `is_apple1` flagę + `apple1()` preset |
| Zero warnings | ✅ cargo build — 0 warningów |
| nestest.log | ✅ Przechodzi ~5800 kroków |
| functional_test.bin | ✅ Przechodzi 20M cykli |
