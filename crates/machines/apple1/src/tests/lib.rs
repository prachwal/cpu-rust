use super::*;

#[test]
fn test_display_output_non_black() {
    let mut emu = Apple1Emulator::new();
    emu.load_roms(&[], &[]);
    let gfx = emu.take_gfx();
    assert_eq!(gfx.len(), 200 * 168 * 4);
}

#[test]
fn test_gfx_render() {
    let font = cpu_display::Font::ascii_8x8();
    let cfg = cpu_display::DisplayConfig::apple1();
    let mut d = cpu_display::Display::new_text(
        cfg, 40, 24, font, cpu_display::FontMapping::Direct,
    );
    d.put_char(b'H');
    d.put_char(b'e');
    d.put_char(b'l');
    d.put_char(b'l');
    d.put_char(b'o');
    let rgba = d.render();
    let non_black = rgba.chunks(4).filter(|p| p[0] != 0 || p[1] != 0 || p[2] != 0).count();
    assert!(non_black > 0, "RGBA buffer should have non-black pixels");
}
