use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};

use minifb::{Key, Window, WindowOptions};

const DEFAULT_ROM_DIR: &str = "crates/machines/pet/roms/pet-2001";
const SCREEN_W: usize = 320; // 40 cols * 8 px
const SCREEN_H: usize = 200; // 25 rows * 8 px
const BORDER: usize = 8;
const WINDOW_W: usize = SCREEN_W + BORDER * 2;
const WINDOW_H: usize = SCREEN_H + BORDER * 2;
const SCALE: usize = 2;
const TICK_BATCH: u32 = 16_666;
const INPUT_TICK_BATCH: u32 = 16_666;
const CONTROL_DELAY: Duration = Duration::from_millis(40);

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
    rom_dir: PathBuf,
    smoke: bool,
}

impl Args {
    fn parse() -> Self {
        let mut rom_dir = PathBuf::from(DEFAULT_ROM_DIR);
        let mut smoke = false;

        for arg in std::env::args().skip(1) {
            if arg == "--smoke" {
                smoke = true;
            } else {
                rom_dir = PathBuf::from(arg);
            }
        }

        Self { rom_dir, smoke }
    }
}

struct Roms {
    basic_c000: Vec<u8>,
    basic_d000: Vec<u8>,
    editor: Vec<u8>,
    kernal: Vec<u8>,
    chargen: Vec<u8>,
}

fn read_rom(dir: &Path, name: &str) -> Vec<u8> {
    let path = dir.join(name);
    std::fs::read(&path).unwrap_or_else(|err| {
        eprintln!("pet-window: cannot read {}: {err}", path.display());
        std::process::exit(1);
    })
}

fn load_roms(dir: &Path) -> Roms {
    let mut chargen = read_rom(dir, "chargen.bin");
    patch_readable_zero(&mut chargen);

    Roms {
        basic_c000: read_rom(dir, "basic-c000.bin"),
        basic_d000: read_rom(dir, "basic-d000.bin"),
        editor: read_rom(dir, "editor.bin"),
        kernal: read_rom(dir, "kernal.bin"),
        chargen,
    }
}

fn patch_readable_zero(chargen: &mut [u8]) {
    // PET 2001 editor ROM writes screen code 0x00 for digit '0'.
    // Patch chargen index 0x00 to render the digit '0' glyph instead of '@'.
    const ZERO: [u8; 8] = [0x3C, 0x42, 0x42, 0x42, 0x42, 0x42, 0x3C, 0x00];
    if chargen.len() >= ZERO.len() {
        chargen[0..ZERO.len()].copy_from_slice(&ZERO);
    }
}

fn new_pet(roms: &Roms) -> pet_core::Pet2001 {
    let mut pet = pet_core::Pet2001::new();
    pet.load_roms_with_chargen(
        &roms.basic_c000,
        &roms.basic_d000,
        &roms.editor,
        &roms.kernal,
        &roms.chargen,
    );
    pet
}

fn screen_bytes(pet: &pet_core::Pet2001) -> &[u8] {
    unsafe { std::slice::from_raw_parts(pet.screen_ptr(), pet.screen_len()) }
}

fn pet_screen_code_to_char(code: u8) -> char {
    match code & 0x7F {
        0x00 => ' ',
        0x01..=0x1A => (b'A' + ((code & 0x7F) - 1)) as char,
        0x20..=0x3F => (code & 0x7F) as char,
        0x40..=0x5A => (b'A' + ((code & 0x7F) - 0x40)) as char,
        0x5B => '[',
        0x5C => '\\',
        0x5D => ']',
        0x5E => '^',
        0x5F => '_',
        _ => ' ',
    }
}

fn screen_text(pet: &pet_core::Pet2001) -> String {
    let mut text = String::new();
    for row in 0..25 {
        for col in 0..40 {
            text.push(pet_screen_code_to_char(screen_bytes(pet)[row * 40 + col]));
        }
        text.push('\n');
    }
    text
}

fn pet_text_byte(ch: char) -> Option<u8> {
    if !ch.is_ascii() {
        return None;
    }

    match ch {
        // Enter/Backspace are handled through CONTROL_KEYS so they can be
        // delayed behind any text delivered by the same GUI frame.
        '\r' | '\n' | '\u{8}' | '\u{7F}' => None,
        ch if ch.is_ascii_graphic() || ch == ' ' => Some(ch as u8),
        _ => None,
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

fn draw_framebuffer(rgba: &[u8], fb: &mut [u32]) {
    fb.fill(0);

    // Copy the emulated PET display into a frontend border. The border is not
    // part of the PET video signal; it keeps edge glyphs away from window
    // decorations and makes screenshots easier to inspect.
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
}

fn smoke_test(roms: &Roms) {
    let mut pet = new_pet(roms);
    pet.run(500_000);
    for byte in b"10 PRINT 10\rLIST\r" {
        pet.type_ascii(*byte);
        pet.run(5_000);
    }
    pet.run(300_000);

    let text = screen_text(&pet);
    assert!(
        text.contains("10 PRINT 10") && !text.contains("10 PRINT 1@") && !text.contains("SYNTAX ERROR"),
        "LIST smoke failed:\n{text}"
    );

    let rgba = pet.take_display();
    let non_black = rgba
        .chunks(4)
        .filter(|p| p[0] != 0 || p[1] != 0 || p[2] != 0)
        .count();
    assert!(non_black > 0, "smoke render produced a blank frame");
    eprintln!("[pet-window] Smoke: render ok, non-black pixels={non_black}");
}

fn main() {
    let args = Args::parse();
    let roms = load_roms(&args.rom_dir);

    eprintln!("[pet-window] ROMs: loading from {}", args.rom_dir.display());
    let mut pet = new_pet(&roms);
    eprintln!("[pet-window] Boot: running 500000 instructions...");
    pet.run(500_000);
    eprintln!("[pet-window] Boot: PC=${:04X}", pet.get_pc());
    {
        let ram = unsafe { std::slice::from_raw_parts(pet.ram_ptr(), 256) };
        let col = ram[0x00C6];
        let screen_ptr = ram[0x00C4] as u16 | ((ram[0x00C5] as u16) << 8);
        eprintln!("[pet-window] Boot: cursor col={col} screen_ptr=${screen_ptr:04X} (valid={})", 
            (0x8000u16..0x8000+1000).contains(&screen_ptr));
    }

    if args.smoke {
        smoke_test(&roms);
        return;
    }

    let mut win = Window::new(
        "Commodore PET 2001",
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

    let mut fb = vec![0u32; WINDOW_W * SCALE * WINDOW_H * SCALE];
    let mut log_screen_countdown: i32 = 0;
    let mut input_queue = VecDeque::new();
    let mut control_queue = VecDeque::new();
    let mut control_down = vec![false; CONTROL_KEYS.len()];
    eprintln!("[pet-window] Ready. ESC=quit");

    while win.is_open() && !win.is_key_down(Key::Escape) {
        let now = Instant::now();

        for ch in typed_chars.borrow_mut().drain(..) {
            match pet_text_byte(ch) {
                Some(byte) => {
                    eprintln!(
                        "[pet-window] Text: {:?} -> {} (${:02X})",
                        ch,
                        ascii_label(byte),
                        byte
                    );
                    input_queue.push_back(byte);
                }
                None => {
                    eprintln!("[pet-window] Text ignored: {:?} (U+{:04X})", ch, ch as u32);
                }
            }
        }

        for (idx, (key, byte, name)) in CONTROL_KEYS.iter().enumerate() {
            let down = win.is_key_down(*key);
            if down && !control_down[idx] {
                eprintln!(
                    "[pet-window] Key: {} -> {} (${:02X})",
                    name,
                    ascii_label(*byte),
                    byte
                );
                control_queue.push_back((now + CONTROL_DELAY, *byte));
            }
            control_down[idx] = down;
        }

        while control_queue
            .front()
            .is_some_and(|(ready_at, _)| *ready_at <= now)
        {
            if let Some((_, byte)) = control_queue.pop_front() {
                input_queue.push_back(byte);
            }
        }

        let mut remaining = TICK_BATCH;
        while remaining > 0 {
            if pet.keyboard_buffer_count() < 8 {
                if let Some(byte) = input_queue.pop_front() {
                    eprintln!("[pet-window] KBD: -> keyboard buffer byte=${:02X} ('{}')",
                        byte, if byte.is_ascii_graphic() || byte == b' ' { byte as char } else { '?' });
                    pet.type_ascii(byte);
                    if byte == b'\r' {
                        log_screen_countdown = 10; // log for 10 frames after CR
                    }
                }
            }

            let batch = remaining.min(INPUT_TICK_BATCH);
            pet.run(batch);
            remaining -= batch;
        }

        if log_screen_countdown > 0 {
            log_screen_countdown -= 1;
            if log_screen_countdown == 0 {
                let screen = unsafe { std::slice::from_raw_parts(pet.screen_ptr(), 1000) };
                for r in 0..6 {
                    let codes: Vec<String> = (0..40).map(|c| format!("${:02X}", screen[r*40+c])).collect();
                    eprintln!("[pet-window] VRAM row{}: {}", r, codes.join(" "));
                }
            }
        }

        let rgba = pet.take_display();
        if rgba.len() >= SCREEN_W * SCREEN_H * 4 {
            draw_framebuffer(&rgba, &mut fb);
            win.update_with_buffer(&fb, WINDOW_W * SCALE, WINDOW_H * SCALE)
                .unwrap();
        }
    }
}
