use std::path::PathBuf;
use std::time::Duration;

use minifb::{Key, Window, WindowOptions};

const W: usize = 280;  // 40×7
const H: usize = 192;  // 24×8
const SCALE: usize = 3;
const TICK_BATCH: u32 = 2000;
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
    (Key::Space, b' '),
    (Key::Enter, b'\r'),
    (Key::Backspace, 0x7F),
    (Key::Minus, b'-'), (Key::Equal, b'='),
    (Key::Slash, b'/'), (Key::Period, b'.'), (Key::Comma, b','),
    (Key::Semicolon, b';'), (Key::Apostrophe, b'\''),
    (Key::LeftBracket, b'['), (Key::RightBracket, b']'),
    (Key::Backslash, b'\\'),
    (Key::Backquote, b'`'),
];

fn key_to_apple1(k: Key) -> Option<u8> {
    KEY_MAP.iter().find(|(key, _)| *key == k).map(|(_, c)| *c)
}

fn main() {
    let rom_dir = std::env::args().nth(1).unwrap_or_else(|| {
        "crates/machines/apple1/roms/apple-1".to_string()
    });
    let dir = PathBuf::from(&rom_dir);
    let basic = std::fs::read(dir.join("basic.bin")).expect("basic.bin not found");
    let wozmon = std::fs::read(dir.join("wozmon.bin")).expect("wozmon.bin not found");

    let mut emu = apple1_core::Apple1Emulator::new();
    emu.load_roms(&basic, &wozmon);

    let mut win = Window::new(
        "Apple 1  280×192",
        W * SCALE,
        H * SCALE,
        WindowOptions { resize: true, scale: minifb::Scale::X1, ..Default::default() },
    ).unwrap();
    win.limit_update_rate(Some(Duration::from_micros(16_666)));

    while win.is_open() && !win.is_key_down(Key::Escape) {
        // Keyboard input
        for key in KEY_MAP.iter().filter(|(k, _)| win.is_key_down(*k)) {
            emu.press_key(key.1);
        }

        // Tick CPU
        emu.run(TICK_BATCH);

        // Get RGBA buffer and scale to window
        let gfx = emu.take_gfx();
        if gfx.len() >= W * H * 4 {
            let mut fb = vec![0u32; W * SCALE * H * SCALE];
            for sy in 0..(H * SCALE) {
                let py = sy / SCALE;
                for sx in 0..(W * SCALE) {
                    let px = sx / SCALE;
                    let src = (py * W + px) * 4;
                    let r = gfx[src];
                    let g = gfx[src + 1];
                    let b = gfx[src + 2];
                    fb[sx + sy * (W * SCALE)] = (r as u32) << 16 | (g as u32) << 8 | b as u32;
                }
            }
            let _ = win.update_with_buffer(&fb, W * SCALE, H * SCALE);
        }
    }
}
