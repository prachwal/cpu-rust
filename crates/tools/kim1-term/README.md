# KIM-1 Terminal Emulator

Emulator KIM-1 z 7-segment LED display i hex keypad.

## Uruchomienie

```bash
# Interaktywny KIM-1 monitor
cargo run -p kim1-term

# Z załadowanym programem
cargo run -p kim1-term -- --load program.bin 0x0200

# Z ustawionym PC
cargo run -p kim1-term -- --addr 0x1C4F
```

## Klawisze

| PC | KIM-1 | Funkcja |
|----|-------|---------|
| `0-9 A-F` | hex | Wpisz cyfrę/hex |
| `Enter` | AD | Address |
| `Tab` | DA | Data |
| `+` | + | Advance |
| `G` | GO | Uruchom |
| `S` | ST | Stop (NMI) |
| `R` | RS | Reset |
| `Esc` | — | Wyjście |

## Użycie monitora KIM-1

1. Uruchom: `cargo run -p kim1-term`
2. Zobaczysz 6 pustych cyfr 7-segment (start monitora)
3. Wpisz: `AD 0 2 0 0` (Enter, 0, 2, 0, 0 — ustaw adres $0200)
4. Wpisz: `DA A 9` (Tab, A, 9 — wpisz $A9 do $0200)
5. Wpisz: `+` — następny adres ($0201)
6. Wpisz: `DA 0 0` — wpisz $00
7. Wpisz: `+` — $0202  
8. Wpisz: `DA 0 0` — wpisz $00 (dla 3-bajtowej instrukcji JMP)
9. Wpisz: `AD 0 2 0 0` — wróć do $0200
10. Wpisz: `GO` — uruchom program

Ten program pokazuje "A9 00" na wyświetlaczu i zapętla się (NOP + JMP $0200).

## Ładowanie programu z pliku

```bash
# Asemblacja z cc65:
ca65 -o program.o program.s
ld65 -C kim1.cfg -o program.bin program.o

# Uruchom w emulatorze:
cargo run -p kim1-term -- --load program.bin 0x0200
```

Plik konfiguracyjny linkera (`kim1.cfg`):
```
MEMORY {
    RAM: start = $0200, size = $0200, type = rw;
}
SEGMENTS {
    CODE: load = RAM, type = rw;
    DATA: load = RAM, type = rw;
    RODATA: load = RAM, type = rw;
}
```

## Testowy program (ledtest)

W `examples/ledtest.s` — program który wyświetla licznik na LED-ach KIM-1.

```asm
; KIM-1 LED Test — wyświetla licznik na 7-segment display
    .org $0200
    ldx #$00
loop:
    stx $02          ; data na prawych 2 cyfrach
    lda #$02
    sta $00
    lda #$00
    sta $01          ; adres na lewych 4 cyfrach = $0200
    jsr $1F1F        ; SCANDS — refresh display
    inx
    jmp loop
```

Asemblacja i uruchomienie:
```bash
cd crates/tools/kim1-term/examples
ca65 -o ledtest.o ledtest.s
ld65 -C kim1.cfg -o ledtest.bin ledtest.o
cargo run -p kim1-term -- --load ledtest.bin 0x0200
```
