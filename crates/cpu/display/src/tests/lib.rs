use super::*;

// ── Legacy API (backward compat) ──

#[test]
fn test_new_default() {
    let d = Display::new(64, 32);
    assert_eq!(d.width(), 64);
    assert_eq!(d.height(), 32);
    assert_eq!(d.buffer_len(), 2048);
    assert_eq!(d.mode(), DisplayMode::Graphics);
}

#[test]
fn test_clear() {
    let mut d = Display::new(64, 32);
    d.set_pixel(0, 0, 1);
    d.clear();
    assert_eq!(d.get_pixel(0, 0), 0);
}

#[test]
fn test_set_get_pixel() {
    let mut d = Display::new(64, 32);
    d.set_pixel(10, 5, 1);
    assert_eq!(d.get_pixel(10, 5), 1);
    assert_eq!(d.get_pixel(11, 5), 0);
}

#[test]
fn test_out_of_bounds() {
    let mut d = Display::new(64, 32);
    d.set_pixel(100, 100, 1);
    assert_eq!(d.get_pixel(100, 100), 0);
}

#[test]
fn test_get_buffer() {
    let d = Display::new(64, 32);
    assert_eq!(d.get_buffer().len(), 2048);
}

// ── DisplayConfig ──

#[test]
fn test_config_default() {
    let cfg = DisplayConfig::default();
    assert_eq!(cfg.width, 64);
    assert_eq!(cfg.height, 32);
    assert_eq!(cfg.palette.len(), 2);
    assert_eq!(cfg.pixel_aspect, (1, 1));
}

#[test]
fn test_config_builtins() {
    let vic = DisplayConfig::vic20_ntsc();
    assert_eq!(vic.width, 176);
    assert_eq!(vic.height, 184);
    assert_eq!(vic.palette.len(), 16);
    assert_eq!(vic.pixel_aspect, (8, 7));

    let chip = DisplayConfig::chip8();
    assert_eq!(chip.width, 64);
    assert_eq!(chip.height, 32);
    assert_eq!(chip.palette.len(), 2);
}

#[test]
fn test_palette_index() {
    let cfg = DisplayConfig::new(10, 10);
    assert_eq!(cfg.palette_index(0), &[0, 0, 0, 255]);
    assert_eq!(cfg.palette_index(1), &[255, 255, 255, 255]);
    // Out of range: returns first
    assert_eq!(cfg.palette_index(99), &[0, 0, 0, 255]);
}

// ── Font ──

fn make_test_font_8x8() -> Font {
    // 2 characters: 'A' and 'B', 8 bytes each, 8x8 pixels
    // row[0] = top, bit 7 = leftmost pixel
    let data: Vec<u8> = vec![
        // 'A'
        0b00111100, // ░░████░░  row 0
        0b01100110, // ░██░░██░  row 1
        0b01100110, // ░██░░██░  row 2
        0b01111110, // ░██████░  row 3
        0b01100110, // ░██░░██░  row 4
        0b01100110, // ░██░░██░  row 5
        0b01100110, // ░██░░██░  row 6
        0b00000000, // ░░░░░░░░  row 7
        // 'B'
        0b01111100, // ░████░░░  row 0
        0b01100110, // ░██░░██░  row 1
        0b01111100, // ░████░░░  row 2
        0b01100110, // ░██░░██░  row 3
        0b01100110, // ░██░░██░  row 4
        0b01111100, // ░████░░░  row 5
        0b00000000, // ░░░░░░░░  row 6
        0b00000000, // ░░░░░░░░  row 7
    ];
    Font::load_c64(&data, 2)
}

#[test]
fn test_font_load_c64() {
    let font = make_test_font_8x8();
    assert_eq!(font.char_width, 8);
    assert_eq!(font.char_height, 8);
    assert_eq!(font.count, 2);
    assert_eq!(font.first, 0);
    assert_eq!(font.last, 1);
}

#[test]
fn test_font_pixel() {
    let font = make_test_font_8x8();
    // 'A' (index 0): row 0 = 0b00111100 → pixels 2-5 are set
    assert!(!font.pixel(0, 0, 0));
    assert!(!font.pixel(0, 1, 0));
    assert!(font.pixel(0, 2, 0));
    assert!(font.pixel(0, 3, 0));
    assert!(font.pixel(0, 4, 0));
    assert!(font.pixel(0, 5, 0));
    assert!(!font.pixel(0, 6, 0));
    assert!(!font.pixel(0, 7, 0));
    // row 1 = 0b01100110
    assert!(!font.pixel(0, 0, 1));
    assert!(font.pixel(0, 1, 1));
    assert!(font.pixel(0, 2, 1));
    assert!(!font.pixel(0, 3, 1));
    assert!(!font.pixel(0, 4, 1));
    assert!(font.pixel(0, 5, 1));
    assert!(font.pixel(0, 6, 1));
    assert!(!font.pixel(0, 7, 1));
}

#[test]
fn test_font_out_of_range_char() {
    let font = make_test_font_8x8();
    // char 255 should clamp to last (1 = 'B')
    assert!(font.pixel(255, 1, 0)); // B row 0 has pixel at x=1
}

#[test]
fn test_font_row_bits() {
    let font = make_test_font_8x8();
    assert_eq!(font.row_bits(0, 0), 0b00111100);
    assert_eq!(font.row_bits(0, 1), 0b01100110);
    assert_eq!(font.row_bits(1, 0), 0b01111100);
}

#[test]
fn test_font_load_pet() {
    let data = vec![0; 512 * 8];
    let font = Font::load_pet(&data);
    assert_eq!(font.count, 512);
}

#[test]
fn test_font_load_raw() {
    // 2 chars, 2x4 pixels, raw format, MSB-first (bit 7 = leftmost)
    let data = vec![
        // Char 'A' (0x41): 2x4, MSB-first
        0x80, // █░  (bit 7=1, bit 6=0)
        0xC0, // ██  (bit 7=1, bit 6=1)
        0x80, // █░
        0x00,
        // Char 'B' (0x42):
        0xC0, // ██
        0x80, // █░
        0xC0, // ██
        0x00,
    ];
    let font = Font::load_raw(&data, 2, 4, 0x41, 0x42);
    assert_eq!(font.char_width, 2);
    assert_eq!(font.char_height, 4);
    assert_eq!(font.first, 0x41);
    assert_eq!(font.last, 0x42);
    // Test 'A' (0x41 → index 0)
    assert!(font.pixel(0x41, 0, 0)); // █
    assert!(!font.pixel(0x41, 1, 0)); // ░
    assert!(font.pixel(0x41, 0, 1));
    assert!(font.pixel(0x41, 1, 1));
}

// ── FontMapping ──

#[test]
fn test_font_mapping_direct() {
    let m = FontMapping::Direct;
    assert_eq!(m.map(0x41), 0x41);
    assert_eq!(m.map(0x00), 0x00);
}

#[test]
fn test_font_mapping_petascii() {
    let m = FontMapping::PetAscii;
    // PETSCII: lowercase 'a' (0x41) → font index 0x21 (lowercase a in font)
    assert_eq!(m.map(0x41), 0x21);
    // Space (0x20) → 0x20
    assert_eq!(m.map(0x20), 0x20);
    // Uppercase 'A' (0x01 on C64) → 0x81 (shifted)
    assert_eq!(m.map(0x01), 0x81);
}

#[test]
fn test_font_mapping_apple1() {
    let m = FontMapping::Apple1;
    // 'A' (0x41) → 0x01
    assert_eq!(m.map(b'A'), 0x01);
    // space
    assert_eq!(m.map(0x20), 0);
}

// ── Display — Graphics Mode ──

#[test]
fn test_display_graphics_config() {
    let cfg = DisplayConfig::vic20_ntsc();
    let d = Display::from_config(cfg.clone());
    assert_eq!(d.width(), 176);
    assert_eq!(d.height(), 184);
    assert_eq!(d.mode(), DisplayMode::Graphics);
}

#[test]
fn test_display_graphics_render() {
    let mut d = Display::new(4, 4);
    d.set_pixel(0, 0, 1);
    d.set_pixel(3, 3, 1);
    let rgba = d.render().to_vec();
    assert_eq!(rgba.len(), 4 * 4 * 4); // width * height * 4

    // pixel (0,0) = white (index 1)
    assert_eq!(&rgba[0..4], &[255, 255, 255, 255]);
    // pixel (1,0) = black (index 0)
    assert_eq!(&rgba[4..8], &[0, 0, 0, 255]);
    // pixel (3,3) = white
    let last = 3 * 4 * 4 + 3 * 4;
    assert_eq!(&rgba[last..last + 4], &[255, 255, 255, 255]);
}

#[test]
fn test_display_graphics_pixels_mut() {
    let mut d = Display::new(4, 4);
    if let Some(pix) = d.pixels_mut() {
        pix[0] = 1;
        pix[15] = 1;
    }
    let rgba = d.render().to_vec();
    assert_eq!(&rgba[0..4], &[255, 255, 255, 255]);
    let last = 15 * 4;
    assert_eq!(&rgba[last..last + 4], &[255, 255, 255, 255]);
}

// ── Display — Text Mode ──

fn make_ascii_font() -> Font {
    // 96 ASCII chars (0x20-0x7F), 8x8 pixels, C64 format
    let count = 96;
    let mut data = vec![0u8; count as usize * 8];
    // Fill with simple patterns: space = blank, others = solid
    for i in 1..count {
        let base = i as usize * 8;
        data[base] = 0xFF;
        data[base + 1] = 0x81;
        data[base + 2] = 0x81;
        data[base + 3] = 0x81;
        data[base + 4] = 0x81;
        data[base + 5] = 0x81;
        data[base + 6] = 0xFF;
        data[base + 7] = 0x00;
    }
    Font::load_c64(&data, count)
}

#[test]
fn test_text_mode_basic() {
    let cfg = DisplayConfig::new(80, 24 * 8); // 80 columns, 24 rows × 8px
    let font = make_ascii_font();
    let mut d = Display::new_text(cfg, 80, 24, font, FontMapping::Direct);

    assert_eq!(d.mode(), DisplayMode::Text(80, 24));
    assert_eq!(d.cols(), 80);
    assert_eq!(d.rows(), 24);

    d.set_char(0, 0, b'H');
    d.set_char(1, 0, b'i');
    assert_eq!(d.get_char(0, 0), b'H');
    assert_eq!(d.get_char(1, 0), b'i');
}

#[test]
fn test_text_mode_colors() {
    let cfg = DisplayConfig::new(80, 24 * 8);
    let font = make_ascii_font();
    let mut d = Display::new_text(cfg, 80, 24, font, FontMapping::Direct);

    d.set_fg(0, 0, 2); // red
    d.set_bg(0, 0, 3); // cyan
    assert_eq!(d.char_fg(0, 0), 2);
    assert_eq!(d.char_bg(0, 0), 3);
}

#[test]
fn test_text_mode_render() {
    let cfg = DisplayConfig::new(8 * 2, 8); // 2 columns × 1 row
    let font = make_ascii_font();
    let mut d = Display::new_text(cfg, 2, 1, font, FontMapping::Direct);

    d.set_char(0, 0, b'A');
    d.set_char(1, 0, b'B');
    let rgba = d.render().to_vec();
    assert_eq!(rgba.len(), 16 * 8 * 4);

    // 'A' at col 0: row 0 = 0xFF (all pixels white)
    let row0_col0 = 0; // pixel (0, 0)
    assert_eq!(&rgba[row0_col0 * 4..row0_col0 * 4 + 4], &[255, 255, 255, 255]);

    // 'B' at col 1: row 0 = 0xFF
    let row0_col1 = 8 * 4; // pixel (8, 0)
    assert_eq!(&rgba[row0_col1 * 4..row0_col1 * 4 + 4], &[255, 255, 255, 255]);
}

#[test]
fn test_text_mapped_chars() {
    let cfg = DisplayConfig::new(8, 8);
    let font = make_ascii_font(); // font has chars 0-95 (for ASCII 0x20-0x7F)
    let mut d = Display::new_text(cfg, 1, 1, font, FontMapping::Apple1);

    // Apple1 'A' (0x41) → font index 0x01
    d.set_char(0, 0, b'A');
    d.render(); // should not panic

    // PETSCII: lowercase a (0x41) → uppercase A (0x01)
    let cfg2 = DisplayConfig::new(8, 8);
    let mut d2 = Display::new_text(cfg2, 1, 1, make_ascii_font(), FontMapping::PetAscii);
    d2.set_char(0, 0, 0x41);
    d2.render(); // should not panic
}

#[test]
fn test_clear_text() {
    let cfg = DisplayConfig::new(80, 24 * 8);
    let font = make_ascii_font();
    let mut d = Display::new_text(cfg, 80, 24, font, FontMapping::Direct);

    d.set_char(5, 3, b'X');
    d.clear();
    assert_eq!(d.get_char(5, 3), 0x20); // back to space
}
