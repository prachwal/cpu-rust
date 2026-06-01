# Poprawki emulatora PET 2001 — znak `@` i mrugający kursor

## Kontekst

Emulator Commodore PET 2001 napisany w Rust (`pet-window`, `pet-core`, `cpu-display`).
Problemy ujawnione podczas testowania interaktywnego.

---

## Problem 1: Cyfra `0` wyświetlana jako `@`

### Objaw

Wpisanie `PRINT 10` i naciśnięcie Enter dawało na ekranie `PRINT 1@` zamiast `PRINT 10`.
Polecenie `10 PRINT 10` / `LIST` wyświetlało `10 PRINT 1@`.

### Diagnoza

W PET 2001 obowiązują dwa różne systemy kodowania znaków:

- **PETSCII** — kod używany w buforze klawiatury i komunikacji z ROM-em (cyfra `0` = `$30`)
- **Screen codes** — kod zapisywany bezpośrednio do Video RAM (`$8000–$83E7`)

W PET screen codes cyfry nie odpowiadają bezpośrednio ASCII. ROM edytora, obsługując
wyjście znakowe (`CHROUT`), zapisuje do VRAM screen code `$00` dla cyfry `0`.
Tymczasem w pliku `chargen.bin` pod indeksem `$00` znajdował się glyf `@`, nie `0`.

Analiza `chargen.bin` potwierdziła:
- Indeks `$00` → glyf `@` (kształt charakterystyczny dla małpki)
- Indeks `$30` → glyf `0` (cyfra zero)

Ale dump VRAM po wykonaniu `PRINT 10` pokazał `$31 $00` — tzn. ROM zapisał `$00` dla cyfry `0`.

### Pierwotna błędna implementacja

```rust
// pet-window/src/main.rs
fn patch_readable_zero(chargen: &mut [u8]) {
    const ZERO: [u8; 8] = [0x3C, 0x42, 0x42, 0x42, 0x42, 0x42, 0x3C, 0x00];
    let start = 0x30 * 8;  // ← BŁĄD: patch pod indeksem 0x30
    chargen[start..start + ZERO.len()].copy_from_slice(&ZERO);
}
```

Funkcja naprawiała glyf pod indeksem `0x30`, podczas gdy ROM zapisuje screen code `$00`
dla cyfry `0` — a więc renderer szuka glyfu pod indeksem `0x00`.

### Co było źle zaprojektowane

Zakładano że screen codes PET = ASCII, co jest błędem. W oryginalnym PET 2001:

| Znak | PETSCII | Screen code |
|------|---------|-------------|
| `@`  | `$40`   | `$00`       |
| `A`  | `$41`   | `$01`       |
| `0`  | `$30`   | `$00` ← tak, `0` i `@` mają ten sam screen code! |

W rzeczywistości ROM edytora PET 2001 używa screen code `$00` dla cyfry `0`,
a `$01`–`$1A` dla liter A–Z. Nie ma tu sprzeczności bo `@` nie pojawia się
jako wynik PRINT — ale glyf pod indeksem `$00` w chargen **musi** wyglądać jak `0`.

### Poprawka

```rust
// pet-window/src/main.rs
fn patch_readable_zero(chargen: &mut [u8]) {
    // ROM PET 2001 zapisuje screen code 0x00 dla cyfry '0'.
    // Patch chargen pod indeksem 0x00 — tam szuka renderer.
    const ZERO: [u8; 8] = [0x3C, 0x42, 0x42, 0x42, 0x42, 0x42, 0x3C, 0x00];
    chargen[0..ZERO.len()].copy_from_slice(&ZERO);  // ← indeks 0x00
}
```

**Commit:** `9d4460c pet-window: fix @ vs 0 - patch chargen index 0x00 not 0x30`

---

## Problem 2: Brak mrugającego kursora

### Objaw

Ekran wyświetlał tekst poprawnie, ale nie było widać kursora — ani stałego,
ani mrugającego. Oryginalne PET 2001 miało mrugający prostokąt w miejscu kursora.

### Diagnoza

#### 2a. `sync_display` nie rysował kursora

Funkcja `sync_display` w `pet-core/src/lib.rs` odczytywała pozycję kursora
z RAM (`$00C4`–`$00C6`) i zamieniała screen code `$00` na `$20` (spację):

```rust
// BŁĘDNA implementacja
fn sync_display(&self) {
    let cursor_idx = self.cursor_screen_index();
    for row in 0..rows {
        for col in 0..cols {
            let mut ch = self.screen[idx];
            if cursor_idx == Some(idx) && ch == 0x00 {
                ch = 0x20;  // zamiana @→spacja, ale NIE rysuje kursora
            }
            disp.set_char(col, row, ch);
        }
    }
}
```

Kod poprawnie lokalizował kursor, ale zamiast go narysować — tylko chował `@`.
Nie było logiki mrugania.

**Poprawka (commit `5936af6`):** Dodano pole `blink_on: bool` w strukturze `PetBus`
przełączane co 30 VBLANK-ów (~2Hz). W `sync_display` na pozycji kursora:

```rust
if cursor_idx == Some(idx) && self.blink_on {
    ch |= 0x80;  // ustaw bit inwersji
}
```

#### 2b. Renderer nie obsługiwał inwersji (bit 7)

Ustawienie `ch |= 0x80` nie dawało efektu, bo `cpu-display/src/lib.rs`
używał `FontMapping::Direct` — screen code przekazywany wprost jako indeks
do chargen. Kod `$A0` (spacja z bitem 7) szukał glyfu pod indeksem 160,
zamiast narysować odwrócony kolor.

```rust
// BŁĘDNA implementacja renderera
let mapped = self.mapping.map(ch);  // ch=0xA0 → idx=160 (losowy glyf)
for cy in 0..char_dy {
    let pixel_set = (row_bits >> (7 - cx)) & 1 != 0;
    let color_idx = if pixel_set { fg } else { bg };  // brak inwersji
}
```

**Poprawka (commit `e1be12b`):**

```rust
let inverse = ch & 0x80 != 0;
let mapped = self.mapping.map(ch & 0x7F);  // strip bit 7 przed lookup
for cy in 0..char_dy {
    let pixel_set = (row_bits >> (7 - cx)) & 1 != 0;
    let pixel_set = if inverse { !pixel_set } else { pixel_set };  // inwersja fg/bg
    let color_idx = if pixel_set { fg } else { bg };
}
```

### Co było źle zaprojektowane

1. `sync_display` miał wyłącznie logikę "nie pokazuj `@` na kursorze" — bez faktycznego rysowania kursora. Brak kursora nie był oczywistym błędem podczas implementacji bo ekran działał poprawnie dla tekstu.

2. `cpu-display` nie miał obsługi inverse video, choć jest to fundamentalna cecha PET (i ogólnie komputerów 8-bitowych). Bit 7 w screen code to standardowy mechanizm inwersji w PET.

---

## Problem 3: Gubienie pierwszego znaku w buforze klawiatury

### Objaw

`PRINT 11` zamiast `PRINT 10` — cyfra `0` była gubiona. Dump VRAM pokazywał
`$31 $30` (= `10`) na ekranie, ale BASIC wykonywał jakby wpisano `1`.

### Diagnoza

```rust
// pet-core/src/lib.rs — BŁĘDNA implementacja
fn type_ascii_byte(&mut self, byte: u8) {
    let count = self.ram[0x009E] as usize;
    self.ram[0x026F + count] = code;  // ← BŁĄD: off-by-one
    self.ram[0x009E] = (count + 1) as u8;
}
```

Bufor klawiatury w PET 2001:
- `$009E` — liczba znaków w buforze
- `$0270`–`$0279` — dane bufora (10 slotów)

Kod pisał pierwszy znak pod `$026F + 0 = $026F`, ale ROM edytora czyta dane
od adresu `$0270`. Adres `$026F` jest przez ROM interpretowany jako **długość bufora**,
nie jako dane. W efekcie:

- znak `1` → zapisany pod `$026F` → ignorowany przez ROM (traktowany jako długość)  
- znak `0` → zapisany pod `$0270` → poprawnie odczytany przez ROM  

Tylko ostatni wpisany znak (przed CR) był faktycznie przetwarzany.

### Poprawka

```rust
self.ram[0x0270 + count] = code;  // ← poprawny adres początku bufora
```

**Commit:** `008a50c pet-core: fix keyboard buffer address 0x026F->0x0270`

### Co było źle zaprojektowane

Adres bufora klawiatury `$026F` zamiast `$0270` to błąd off-by-one prawdopodobnie
wynikający z pomylenia adresu `$026F` (który w dokumentacji PET pojawia się jako
"keyboard buffer –1" lub wskaźnik) z faktycznym początkiem danych.

---

## Podsumowanie zmian

| Commit | Plik | Zmiana |
|--------|------|--------|
| `9d4460c` | `pet-window/src/main.rs` | `patch_readable_zero`: indeks `0x30` → `0x00` |
| `5936af6` | `pet-core/src/lib.rs` | `sync_display`: dodano mruganie kursora przez `blink_on` |
| `e1be12b` | `cpu-display/src/lib.rs` | `render`: obsługa inverse video (bit 7) |
| `008a50c` | `pet-core/src/lib.rs` | `type_ascii_byte`: bufor `0x026F` → `0x0270` |
| `12dda73` | `pet-window/src/main.rs` | `TICK_BATCH`: `5000` → `16666` (1MHz/60Hz) |
| `813092f` | `pet-window/src/main.rs` | Input feeding: jeden bajt per klatka przed `pet.run()` |
