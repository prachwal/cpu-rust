# 6502 Nieudokumentowane opcody

Źródło: [Adam Vardy, "Extra Instructions Of The 65XX Series CPU"](https://raw.githubusercontent.com/spacerace/6502/master/doc/6502-asm-doc/extra_instructions.txt)

Z 256 możliwych opcodów, 151 to "legalne". Pozostałe 105 to nieudokumentowane opcody,
zwane też *illegal*, *undocumented*, *unofficial* lub *extra instructions*.

Nazewnictwo pochodzi z "The Complete Commodore Inner Space Anthology" (CCISA) oraz "C=Hacking".

## Lista nieudokumentowanych opcodów

### ASO (SLO) — ASL + ORA
ASL pamięci, potem OR z akumulatorem.

| Tryb | Opc | Bajty | Cykle |
|------|-----|-------|-------|
| absolute | 0F | 3 | 6 |
| absolute,X | 1F | 3 | 7 |
| absolute,Y | 1B | 3 | 7 |
| zeropage | 07 | 2 | 5 |
| zeropage,X | 17 | 2 | 6 |
| (indirect,X) | 03 | 2 | 8 |
| (indirect),Y | 13 | 2 | 8 |

### RLA — ROL + AND
ROL pamięci, potem AND z akumulatorem.

| Tryb | Opc | Bajty | Cykle |
|------|-----|-------|-------|
| absolute | 2F | 3 | 6 |
| absolute,X | 3F | 3 | 7 |
| absolute,Y | 3B | 3 | 7 |
| zeropage | 27 | 2 | 5 |
| zeropage,X | 37 | 2 | 6 |
| (indirect,X) | 23 | 2 | 8 |
| (indirect),Y | 33 | 2 | 8 |

### LSE (SRE) — LSR + EOR
LSR pamięci, potem EOR z akumulatorem.

| Tryb | Opc | Bajty | Cykle |
|------|-----|-------|-------|
| absolute | 4F | 3 | 6 |
| absolute,X | 5F | 3 | 7 |
| absolute,Y | 5B | 3 | 7 |
| zeropage | 47 | 2 | 5 |
| zeropage,X | 57 | 2 | 6 |
| (indirect,X) | 43 | 2 | 8 |
| (indirect),Y | 53 | 2 | 8 |

### RRA — ROR + ADC
ROR pamięci, potem ADC z akumulatorem.

| Tryb | Opc | Bajty | Cykle |
|------|-----|-------|-------|
| absolute | 6F | 3 | 6 |
| absolute,X | 7F | 3 | 7 |
| absolute,Y | 7B | 3 | 7 |
| zeropage | 67 | 2 | 5 |
| zeropage,X | 77 | 2 | 6 |
| (indirect,X) | 63 | 2 | 8 |
| (indirect),Y | 73 | 2 | 8 |

### AXS (SAX) — A & X → memory
AND A i X, store do pamięci. Nie zmienia flag.

| Tryb | Opc | Bajty | Cykle |
|------|-----|-------|-------|
| absolute | 8F | 3 | 4 |
| zeropage | 87 | 2 | 3 |
| zeropage,Y | 97 | 2 | 4 |
| (indirect,X) | 83 | 2 | 6 |

### LAX — LDA + LDX
Ładuje zarówno A jak i X z pamięci.

| Tryb | Opc | Bajty | Cykle |
|------|-----|-------|-------|
| absolute | AF | 3 | 4 |
| absolute,Y | BF | 3 | 4\* |
| zeropage | A7 | 2 | 3 |
| zeropage,Y | B7 | 2 | 4 |
| (indirect,X) | A3 | 2 | 6 |
| (indirect),Y | B3 | 2 | 5\* |

\* +1 cykl jeśli page boundary crossed

### DCM (DCP) — DEC + CMP
DEC pamięci, potem CMP z A.

| Tryb | Opc | Bajty | Cykle |
|------|-----|-------|-------|
| absolute | CF | 3 | 6 |
| absolute,X | DF | 3 | 7 |
| absolute,Y | DB | 3 | 7 |
| zeropage | C7 | 2 | 5 |
| zeropage,X | D7 | 2 | 6 |
| (indirect,X) | C3 | 2 | 8 |
| (indirect),Y | D3 | 2 | 8 |

### INS (ISC) — INC + SBC
INC pamięci, potem SBC z A.

| Tryb | Opc | Bajty | Cykle |
|------|-----|-------|-------|
| absolute | EF | 3 | 6 |
| absolute,X | FF | 3 | 7 |
| absolute,Y | FB | 3 | 7 |
| zeropage | E7 | 2 | 5 |
| zeropage,X | F7 | 2 | 6 |
| (indirect,X) | E3 | 2 | 8 |
| (indirect),Y | F3 | 2 | 8 |

### ALR — AND + LSR
AND z immediate, potem LSR A.

| Tryb | Opc | Bajty | Cykle |
|------|-----|-------|-------|
| immediate | 4B | 2 | 2 |

### ARR — AND + ROR
AND z immediate, potem ROR A.
Działa bardziej złożone niż AND+ROR — wpływa na V flag (XOR bit 7 z bit 6).

| Tryb | Opc | Bajty | Cykle |
|------|-----|-------|-------|
| immediate | 6B | 2 | 2 |

### XAA — X → A AND #imm
Transfer X do A, potem AND z immediate.

| Tryb | Opc | Bajty | Cykle |
|------|-----|-------|-------|
| immediate | 8B | 2 | 2 |

### OAL — ORA #$EE AND #imm
OR A z $EE, AND z immediate, store w A i X.

| Tryb | Opc | Bajty | Cykle |
|------|-----|-------|-------|
| immediate | AB | 2 | 2 |

### SAX — AND + SBC
AND A i X, subtract immediate, store w X. Niezależne od Carry.

| Tryb | Opc | Bajty | Cykle |
|------|-----|-------|-------|
| immediate | CB | 2 | 2 |

### NOP — No Operation
Dodatkowe NOP-y: 1A, 3A, 5A, 7A, DA, FA (2 cykle).

### SKB — Skip Next Byte
Pomija następny bajt. Opc: 80, 82, C2, E2, 04, 14, 34, 44, 54, 64, 74, D4, F4.
2-4 cykle.

### SKW — Skip Next Word
Pomija następne dwa bajty. Opc: 0C, 1C, 3C, 5C, 7C, DC, FC.
4 cykle (+1 jeśli page boundary).

### HLT (JAM/KIL) — Halt
Zatrzymuje CPU. Tylko reset restartuje.
Opc: 02, 12, 22, 32, 42, 52, 62, 72, 92, B2, D2, F2.

### TAS — A & X → SP, AND z adresem → memory
| Tryb | Opc | Bajty | Cykle |
|------|-----|-------|-------|
| absolute,Y | 9B | 3 | 5 |

### SAY — Y & adres → memory
| Tryb | Opc | Bajty | Cykle |
|------|-----|-------|-------|
| absolute,X | 9C | 3 | 5 |

### XAS — X & adres → memory
| Tryb | Opc | Bajty | Cykle |
|------|-----|-------|-------|
| absolute,Y | 9E | 3 | 5 |

### AXA — A & X & adres → memory
| Tryb | Opc | Bajty | Cykle |
|------|-----|-------|-------|
| absolute,Y | 9F | 3 | 5 |
| (indirect),Y | 93 | 2 | 6 |

### ANC — AND + bit 7 → Carry
AND z immediate, potem bit 7 A → Carry.
Opc: 2B, 0B (2 cykle).

### LAS — memory & SP → A, X, SP
| Tryb | Opc | Bajty | Cykle |
|------|-----|-------|-------|
| absolute,Y | BB | 3 | 4\* |

### Opcode 89 — SKB (2 cykle)
### Opcode EB — SBC #immediate (2 cykle)

## Uwagi

- Opcode 89 to kolejny SKB
- Opcode EB działa jak SBC #immediate
- ARR ma złożone zachowanie (V flag, carry, bit 0 gubiony)
- SKB 82, C2, E2 mogą być HLT na niektórych maszynach
- XAA ($8B) i OAL ($AB) są wysoce zmienne między maszynami
- LAS jest podejrzany — potencjalnie zawodny
- TAS, SAY, XAS, AXA mogą mieć bugi przy page boundary crossing
