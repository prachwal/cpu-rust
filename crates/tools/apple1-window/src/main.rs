use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use minifb::{Key, Window, WindowOptions};

const SCREEN_W: usize = 200; // 40 cols × 5 px
const SCREEN_H: usize = 192; // 24 rows × 8 px
const BORDER: usize = 8; // logical pixels around the emulated screen
const WINDOW_W: usize = SCREEN_W + BORDER * 2;
const WINDOW_H: usize = SCREEN_H + BORDER * 2;
const SCALE: usize = 3;
const TICK_BATCH: u32 = 5000;
const INPUT_TICK_BATCH: u32 = 500;

const CONTROL_KEYS: &[(Key, u8, &str)] = &[
    (Key::Enter, b'\r', "Enter"),
    (Key::Backspace, 0x7F, "Backspace"),
];

struct TextInput {
    chars: Rc<RefCell<Vec<char>>>,
}

impl minifb::InputCallback for TextInput {
    fn add_char(&mut self, uni_char: u32) {
        if let Some(ch) = char::from_u32(uni_char) {
            self.chars.borrow_mut().push(ch);
        }
    }
}

struct Args {
    rom_dir: String,
    smoke: bool,
}

impl Args {
    fn parse() -> Self {
        let mut rom_dir = "crates/machines/apple1/roms/apple-1".to_string();
        let mut smoke = false;

        for arg in std::env::args().skip(1) {
            if arg == "--smoke" {
                smoke = true;
            } else {
                rom_dir = arg;
            }
        }

        Self { rom_dir, smoke }
    }
}

/// Generic text terminal frontend backed by `cpu_display::Display`.
///
/// Machines expose very different display devices: Apple 1 PIA bytes, ACIA
/// serial streams, PET screen RAM, etc. This adapter keeps the window code
/// independent from that hardware detail. It accepts a byte stream, applies the
/// small set of terminal controls that are common to simple monitors, and writes
/// printable ASCII into the shared text-mode display renderer.
struct GraphicalTerminal {
    display: cpu_display::Display,
}

impl GraphicalTerminal {
    fn new(display: cpu_display::Display) -> Self {
        Self { display }
    }

    fn write_bytes<I>(&mut self, bytes: I)
    where
        I: IntoIterator<Item = u8>,
    {
        for byte in bytes {
            self.write_byte(byte);
        }
    }

    fn write_byte(&mut self, byte: u8) {
        match byte & 0x7F {
            // Apple 1 firmware writes 0x7F while configuring the display PIA.
            // It is not terminal output; rendering it produces a garbage glyph.
            0x7F => {}
            b'\r' | b'\n' => self.display.put_char(b'\r'),
            0x08 => self.display.put_char(0x08),
            ch if (0x20..=0x5F).contains(&ch) => self.display.put_char(ch),
            // Keep the renderer deterministic for monitor/control bytes that
            // are not part of the visible Apple 1 character set.
            _ => {}
        }
    }

    fn render(&mut self) -> &[u8] {
        self.display.render()
    }
}

fn ascii_label(byte: u8) -> String {
    match byte {
        b'\r' => "CR".to_string(),
        0x7F => "DEL".to_string(),
        b' ' => "SPACE".to_string(),
        b if b.is_ascii_graphic() => (b as char).to_string(),
        _ => format!("0x{byte:02X}"),
    }
}

fn apple1_text_byte(ch: char) -> Option<u8> {
    if !ch.is_ascii() {
        return None;
    }

    match ch {
        '\r' | '\n' => Some(b'\r'),
        '\u{8}' | '\u{7F}' => Some(0x7F),
        ch if ch.is_ascii_graphic() || ch == ' ' => Some(ch.to_ascii_uppercase() as u8),
        _ => None,
    }
}

fn main() {
    let args = Args::parse();

    // ── 1. Font ──
    eprintln!("[apple1-window] Font: loading Apple 1 5×7...");
    let font = cpu_display::Font::apple1_5x7();
    eprintln!(
        "[apple1-window] Font: {} chars (${:02X}-${:02X}), {}×{} px",
        font.count, font.first, font.last, font.char_width, font.char_height
    );

    // ── 2. Display init ──
    eprintln!("[apple1-window] Display: creating 40×24 text mode...");
    let cfg = cpu_display::DisplayConfig::apple1();
    let display =
        cpu_display::Display::new_text(cfg.clone(), 40, 24, font, cpu_display::FontMapping::Direct);
    let mut terminal = GraphicalTerminal::new(display);
    eprintln!(
        "[apple1-window] Display: {}×{} px, {} colours, aspect {}:{}",
        cfg.width,
        cfg.height,
        cfg.palette.len(),
        cfg.pixel_aspect.0,
        cfg.pixel_aspect.1
    );

    // ── 3. ROMs ──
    let dir = PathBuf::from(&args.rom_dir);
    eprintln!("[apple1-window] ROMs: loading from {}", dir.display());
    let basic = std::fs::read(dir.join("basic.bin")).expect("basic.bin not found");
    let wozmon = std::fs::read(dir.join("wozmon.bin")).expect("wozmon.bin not found");
    eprintln!(
        "[apple1-window] ROMs: basic={}B, wozmon={}B",
        basic.len(),
        wozmon.len()
    );

    // ── 4. Emulator ──
    eprintln!("[apple1-window] Emulator: creating Apple 1...");
    let mut emu = apple1_core::Apple1Emulator::new();
    emu.load_roms(&basic, &wozmon);
    eprintln!(
        "[apple1-window] Emulator: PC=${:04X}, SP=${:02X}",
        emu.get_pc(),
        emu.get_sp()
    );

    // ── 5. Warm-up boot ──
    eprintln!("[apple1-window] Boot: running {} instructions...", 100_000);
    emu.run(100_000);
    eprintln!(
        "[apple1-window] Boot: PC=${:04X}, A=${:02X}, X=${:02X}, Y=${:02X}",
        emu.get_pc(),
        emu.get_a(),
        emu.get_x(),
        emu.get_y()
    );

    // ── 6. Test display output ──
    let test = emu.take_display();
    eprintln!(
        "[apple1-window] Display output: {} ASCII bytes drained",
        test.len()
    );
    if !test.is_empty() {
        let preview: String = test
            .iter()
            .take(80)
            .map(|b| {
                if *b >= 0x20 && *b <= 0x7E {
                    *b as char
                } else {
                    '?'
                }
            })
            .collect();
        eprintln!("[apple1-window] Output preview (first 80): {}", preview);
    }
    terminal.write_bytes(test.iter().copied());
    let rgba = terminal.render();
    let non_black = rgba
        .chunks(4)
        .filter(|p| p[0] != 0 || p[1] != 0 || p[2] != 0)
        .count();
    eprintln!(
        "[apple1-window] RGBA: {}×{} = {} px, {} non-black pixels",
        cfg.width,
        cfg.height,
        rgba.len() / 4,
        non_black
    );
    if args.smoke {
        assert!(non_black > 0, "smoke render produced a blank frame");
        eprintln!("[apple1-window] Smoke: render ok");
        return;
    }

    // ── 7. Window ──
    eprintln!(
        "[apple1-window] Window: creating {}×{} ({}× scaled)...",
        WINDOW_W * SCALE,
        WINDOW_H * SCALE,
        SCALE
    );
    let mut win = Window::new(
        "Apple 1  280×192",
        WINDOW_W * SCALE,
        WINDOW_H * SCALE,
        WindowOptions {
            resize: true,
            scale: minifb::Scale::X1,
            ..Default::default()
        },
    )
    .unwrap();
    win.limit_update_rate(Some(Duration::from_micros(16_666)));
    let typed_chars = Rc::new(RefCell::new(Vec::new()));
    win.set_input_callback(Box::new(TextInput {
        chars: Rc::clone(&typed_chars),
    }));
    eprintln!("[apple1-window] Ready. ESC=quit");

    // ── 8. Main loop ──
    let mut control_down = vec![false; CONTROL_KEYS.len()];
    let mut input_queue = VecDeque::new();
    while win.is_open() && !win.is_key_down(Key::Escape) {
        // Text input comes from minifb's character callback rather than
        // physical `Key::A`/`Key::E` states. That path respects the active
        // keyboard layout and avoids backend-specific missing letter events.
        for ch in typed_chars.borrow_mut().drain(..) {
            if let Some(ascii) = apple1_text_byte(ch) {
                eprintln!(
                    "[apple1-window] Text: {:?} -> {} (${:02X})",
                    ch,
                    ascii_label(ascii),
                    ascii
                );
                input_queue.push_back(ascii);
            }
        }

        // Non-text controls still use edge-triggered physical key state.
        for (idx, (key, ascii, name)) in CONTROL_KEYS.iter().enumerate() {
            let down = win.is_key_down(*key);
            if down && !control_down[idx] {
                eprintln!(
                    "[apple1-window] Key: {} -> {} (${:02X})",
                    name,
                    ascii_label(*ascii),
                    ascii
                );
                input_queue.push_back(*ascii);
            }
            control_down[idx] = down;
        }

        // Feed queued input gradually. Some monitors poll a one-character
        // keyboard device; pacing input between CPU runs avoids dropped
        // characters when the host receives a whole word in one GUI frame.
        let mut remaining = TICK_BATCH;
        while remaining > 0 {
            if let Some(ascii) = input_queue.pop_front() {
                emu.press_key(ascii);
            }

            let batch = remaining.min(INPUT_TICK_BATCH);
            emu.run(batch);
            remaining -= batch;
        }

        // Drain ASCII display into our local RGBA renderer
        terminal.write_bytes(emu.take_display());

        // Render RGBA
        let rgba = terminal.render();
        if rgba.len() >= SCREEN_W * SCREEN_H * 4 {
            let mut fb = vec![0u32; WINDOW_W * SCALE * WINDOW_H * SCALE];

            // Copy the emulated display into a slightly larger framebuffer.
            // The margin is part of the frontend, not the machine display:
            // it prevents the first/last glyph pixels from being hidden by
            // window borders, scaling artifacts, or screenshot crop edges.
            for sy in 0..(SCREEN_H * SCALE) {
                let py = sy / SCALE;
                let dst_y = sy + BORDER * SCALE;
                for sx in 0..(SCREEN_W * SCALE) {
                    let px = sx / SCALE;
                    let dst_x = sx + BORDER * SCALE;
                    let src = (py * SCREEN_W + px) * 4;
                    let r = rgba[src] as u32;
                    let g = rgba[src + 1] as u32;
                    let b = rgba[src + 2] as u32;
                    fb[dst_x + dst_y * (WINDOW_W * SCALE)] = (r << 16) | (g << 8) | b;
                }
            }
            win.update_with_buffer(&fb, WINDOW_W * SCALE, WINDOW_H * SCALE)
                .unwrap();
        }
    }
}
