use std::path::PathBuf;
use std::time::Duration;

use minifb::{Key, Window, WindowOptions};

const W: usize = 280; // 40 cols × 7 px
const H: usize = 192; // 24 rows × 8 px
const SCALE: usize = 3;
const TICK_BATCH: u32 = 5000;
const KEY_MAP: &[(Key, u8)] = &[
    (Key::Key0, b'0'), (Key::Key1, b'1'), (Key::Key2, b'2'),
    (Key::Key3, b'3'), (Key::Key4, b'4'), (Key::Key5, b'5'),
    (Key::Key6, b'6'), (Key::Key7, b'7'), (Key::Key8, b'8'), (Key::Key9, b'9'),
    (Key::A, b'A'), (Key::B, b'B'), (Key::C, b'C'), (Key::D, b'D'),
    (Key::E, b'E'), (Key::F, b'F'), (Key::G, b'G'), (Key::H, b'H'),
    (Key::I, b'I'), (Key::J, b'J'), (Key::K, b'K'), (Key::L, b'L'),
    (Key::M, b'M'), (Key::N, b'N'), (Key::O, b'O'), (Key::P, b'P'),
    (Key::Q, b'Q'), (Key::R, b'R'), (Key::S, b'S'), (Key::T, b'T'),
    (Key::U, b'U'), (Key::V, b'V'), (Key::W, b'W'), (Key::X, b'X'),
    (Key::Y, b'Y'), (Key::Z, b'Z'),
    (Key::Space, b' '), (Key::Enter, b'\r'), (Key::Backspace, 0x7F),
    (Key::Minus, b'-'), (Key::Equal, b'='),
    (Key::Slash, b'/'), (Key::Period, b'.'), (Key::Comma, b','),
    (Key::Semicolon, b';'), (Key::Apostrophe, b'\''),
    (Key::LeftBracket, b'['), (Key::RightBracket, b']'),
    (Key::Backslash, b'\\'), (Key::Backquote, b'`'),
];

fn main() {
    // ── 1. Font ──
    eprintln!("[apple1-window] Font: loading built-in ASCII 8×8...");
    let font = cpu_display::Font::ascii_8x8();
    eprintln!("[apple1-window] Font: {} chars (${:02X}-${:02X}), {}×{} px",
        font.count, font.first, font.last, font.char_width, font.char_height);

    // ── 2. Display init ──
    eprintln!("[apple1-window] Display: creating 40×24 text mode...");
    let cfg = cpu_display::DisplayConfig::apple1();
    let mut display = cpu_display::Display::new_text(
        cfg.clone(), 40, 24, font, cpu_display::FontMapping::Direct,
    );
    eprintln!("[apple1-window] Display: {}×{} px, {} colours, aspect {}:{}",
        cfg.width, cfg.height, cfg.palette.len(), cfg.pixel_aspect.0, cfg.pixel_aspect.1);

    // ── 3. ROMs ──
    let rom_dir = std::env::args().nth(1).unwrap_or_else(|| {
        "crates/machines/apple1/roms/apple-1".to_string()
    });
    let dir = PathBuf::from(&rom_dir);
    eprintln!("[apple1-window] ROMs: loading from {}", dir.display());
    let basic = std::fs::read(dir.join("basic.bin")).expect("basic.bin not found");
    let wozmon = std::fs::read(dir.join("wozmon.bin")).expect("wozmon.bin not found");
    eprintln!("[apple1-window] ROMs: basic={}B, wozmon={}B", basic.len(), wozmon.len());

    // ── 4. Emulator ──
    eprintln!("[apple1-window] Emulator: creating Apple 1...");
    let mut emu = apple1_core::Apple1Emulator::new();
    emu.load_roms(&basic, &wozmon);
    eprintln!("[apple1-window] Emulator: PC=${:04X}, SP=${:02X}", emu.get_pc(), emu.get_sp());

    // ── 5. Warm-up boot ──
    eprintln!("[apple1-window] Boot: running {} instructions...", 100_000);
    emu.run(100_000);
    eprintln!("[apple1-window] Boot: PC=${:04X}, A=${:02X}, X=${:02X}, Y=${:02X}",
        emu.get_pc(), emu.get_a(), emu.get_x(), emu.get_y());

    // ── 6. Test display output ──
    let test = emu.take_display();
    eprintln!("[apple1-window] Display output: {} ASCII bytes drained", test.len());
    if !test.is_empty() {
        let preview: String = test.iter().take(80).map(|b| {
            if *b >= 0x20 && *b <= 0x7E { *b as char } else { '?' }
        }).collect();
        eprintln!("[apple1-window] Output preview (first 80): {}", preview);
    }
    // Feed display to our local copy
    for &b in &test {
        display.put_char(b);
    }
    let rgba = display.render();
    let non_black = rgba.chunks(4).filter(|p| p[0] != 0 || p[1] != 0 || p[2] != 0).count();
    eprintln!("[apple1-window] RGBA: {}×{} = {} px, {} non-black pixels",
        cfg.width, cfg.height, rgba.len() / 4, non_black);

    // ── 7. Window ──
    eprintln!("[apple1-window] Window: creating {}×{} ({}× scaled)...",
        W * SCALE, H * SCALE, SCALE);
    let mut win = Window::new(
        "Apple 1  280×192",
        W * SCALE,
        H * SCALE,
        WindowOptions { resize: true, scale: minifb::Scale::X1, ..Default::default() },
    ).unwrap();
    win.limit_update_rate(Some(Duration::from_micros(16_666)));
    eprintln!("[apple1-window] Ready. ESC=quit");

    // ── 8. Main loop ──
    let mut frame_count: u64 = 0;
    while win.is_open() && !win.is_key_down(Key::Escape) {
        // Keyboard
        for (key, ascii) in KEY_MAP {
            if win.is_key_down(*key) {
                emu.press_key(*ascii);
            }
        }

        // Tick
        emu.run(TICK_BATCH);

        // Drain ASCII display into our local RGBA renderer
        for b in emu.take_display() {
            display.put_char(b);
        }

        // Render RGBA
        let rgba = display.render();
        if rgba.len() >= W * H * 4 {
            let mut fb = vec![0u32; W * SCALE * H * SCALE];
            for sy in 0..(H * SCALE) {
                let py = sy / SCALE;
                for sx in 0..(W * SCALE) {
                    let px = sx / SCALE;
                    let src = (py * W + px) * 4;
                    let r = rgba[src] as u32;
                    let g = rgba[src + 1] as u32;
                    let b = rgba[src + 2] as u32;
                    fb[sx + sy * (W * SCALE)] = (r << 16) | (g << 8) | b;
                }
            }
            win.update_with_buffer(&fb, W * SCALE, H * SCALE).unwrap();
        }

        frame_count += 1;
        if frame_count % 60 == 0 {
            eprintln!("[apple1-window] Frame {} PC=${:04X}",
                frame_count, emu.get_pc());
        }
    }
}
