use std::io::{stdout, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{poll, read, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use crossterm::{execute, queue};

const DEFAULT_ROM_DIR: &str = "crates/machines/pet/roms/pet-2001";
const COLS: usize = 40;
const ROWS: usize = 25;
const TICK_BATCH: u32 = 1_000;
const FRAME_INTERVAL: Duration = Duration::from_millis(33);

struct RomSpec {
    name: &'static str,
    size: usize,
    sha256: &'static str,
}

const ROMS: &[RomSpec] = &[
    RomSpec {
        name: "basic-c000.bin",
        size: 4096,
        sha256: "2c72ad52b53f522b2c112768d5912ca74df426aa2aa89569f3a0a230908fbfe3",
    },
    RomSpec {
        name: "basic-d000.bin",
        size: 4096,
        sha256: "41a447ce5a6972acd3fcc950996f5b82383f835ad7a1f56728c6945853b7ab80",
    },
    RomSpec {
        name: "editor.bin",
        size: 2048,
        sha256: "5338b1dffebd695f6110c9995e16f7536bb63a909c2a0062b4789de46904ade9",
    },
    RomSpec {
        name: "kernal.bin",
        size: 4096,
        sha256: "056d5e84a6e4f2b5b40a4109f49515c2496a4bf99139a9f59abe486ded5cd03d",
    },
    RomSpec {
        name: "chargen.bin",
        size: 2048,
        sha256: "da3374c21d6ea440cef5f338ce3f524e0a1e40dcb1ef64446c695f92c636f1fa",
    },
];

struct Roms {
    basic_c000: Vec<u8>,
    basic_d000: Vec<u8>,
    editor: Vec<u8>,
    kernal: Vec<u8>,
}

fn read_rom(dir: &Path, spec: &RomSpec) -> Vec<u8> {
    let file = dir.join(spec.name);
    let bytes = std::fs::read(&file).unwrap_or_else(|e| {
        eprintln!("Cannot read {}: {}", file.display(), e);
        std::process::exit(1);
    });
    if bytes.len() != spec.size {
        eprintln!(
            "ROM integrity error: {} has {} bytes, expected {}",
            file.display(),
            bytes.len(),
            spec.size
        );
        std::process::exit(1);
    }
    let sha256 = sha256sum(&file).unwrap_or_else(|| {
        eprintln!(
            "ROM integrity error: cannot run sha256sum for {}",
            file.display()
        );
        std::process::exit(1);
    });
    if sha256 != spec.sha256 {
        eprintln!(
            "ROM integrity error: {} sha256={} expected {}",
            file.display(),
            sha256,
            spec.sha256
        );
        std::process::exit(1);
    }
    bytes
}

fn sha256sum(file: &Path) -> Option<String> {
    let output = Command::new("sha256sum").arg(file).output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()?
        .split_whitespace()
        .next()
        .map(str::to_owned)
}

fn load_roms(dir: &Path, verbose: bool) -> Roms {
    if verbose {
        eprintln!("PET ROM set: {}", dir.display());
        for spec in ROMS {
            eprintln!(
                "  {}  size={}  sha256={}",
                spec.name, spec.size, spec.sha256
            );
        }
    }

    Roms {
        basic_c000: read_rom(dir, &ROMS[0]),
        basic_d000: read_rom(dir, &ROMS[1]),
        editor: read_rom(dir, &ROMS[2]),
        kernal: read_rom(dir, &ROMS[3]),
    }
}

fn new_pet(roms: &Roms) -> pet_core::Pet2001 {
    let mut pet = pet_core::Pet2001::new();
    pet.load_roms(
        &roms.basic_c000,
        &roms.basic_d000,
        &roms.editor,
        &roms.kernal,
    );
    pet
}

fn screen_bytes(pet: &pet_core::Pet2001) -> &[u8] {
    unsafe { std::slice::from_raw_parts(pet.screen_ptr(), pet.screen_len()) }
}

fn pet_screen_code_to_char(code: u8) -> char {
    match code & 0x7F {
        0x00 => '@',
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

fn render_screen(pet: &pet_core::Pet2001) -> String {
    let screen = screen_bytes(pet);
    let mut out = String::new();
    out.push_str(&format!("┌{}┐\r\n", "─".repeat(COLS)));
    for row in 0..ROWS {
        out.push('│');
        for col in 0..COLS {
            out.push(pet_screen_code_to_char(screen[row * COLS + col]));
        }
        out.push_str("│\r\n");
    }
    out.push_str(&format!("└{}┘\r\n", "─".repeat(COLS)));
    out.push_str(&format!(
        "PET 2001  40x25  PC=${:04X}  instr={}  cycles={}  Esc=quit  Ctrl+R=reset\r\n",
        pet.get_pc(),
        pet.get_instructions(),
        pet.get_cycles()
    ));
    out
}

fn screen_contains(pet: &pet_core::Pet2001, ch: char) -> bool {
    screen_bytes(pet)
        .iter()
        .any(|&code| pet_screen_code_to_char(code) == ch)
}

fn smoke_test(roms: &Roms) {
    let mut pet = new_pet(roms);
    pet.run(500_000);

    if !screen_contains(&pet, 'R') {
        eprintln!("PET smoke failed: boot screen does not contain READY text");
        eprintln!("{}", render_screen(&pet));
        std::process::exit(1);
    }

    pet.type_text("PRINT 2+2\r");
    pet.run(300_000);

    if !screen_contains(&pet, '4') {
        eprintln!("PET smoke failed: PRINT 2+2 did not produce 4");
        eprintln!("{}", render_screen(&pet));
        std::process::exit(1);
    }

    println!("PET smoke passed: boot prompt and BASIC input/output work");
}

fn key_to_pet_ascii(code: KeyCode) -> Option<u8> {
    match code {
        KeyCode::Enter => Some(b'\r'),
        KeyCode::Backspace => Some(0x7F),
        KeyCode::Char(ch) if ch.is_ascii() => Some(ch as u8),
        _ => None,
    }
}

fn main() {
    let mut check_roms = false;
    let mut smoke = false;
    let mut rom_dir = PathBuf::from(DEFAULT_ROM_DIR);
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--check-roms" => check_roms = true,
            "--smoke" => smoke = true,
            _ => rom_dir = PathBuf::from(arg),
        }
    }

    let roms = load_roms(&rom_dir, check_roms);
    if check_roms {
        println!("PET ROM integrity check passed: {}", rom_dir.display());
        return;
    }
    if smoke {
        smoke_test(&roms);
        return;
    }

    let mut pet = new_pet(&roms);

    enable_raw_mode().unwrap();
    execute!(stdout(), Hide, Clear(ClearType::All)).unwrap();

    let mut running = true;
    let mut last_frame = Instant::now();

    while running {
        let now = Instant::now();

        while poll(Duration::ZERO).unwrap() {
            if let Event::Key(key) = read().unwrap() {
                if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
                    continue;
                }

                match key.code {
                    KeyCode::Esc => running = false,
                    KeyCode::Char('r') | KeyCode::Char('R')
                        if key.modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        pet = new_pet(&roms);
                    }
                    code => {
                        if let Some(byte) = key_to_pet_ascii(code) {
                            pet.type_ascii(byte);
                        }
                    }
                }
            }
        }

        pet.run(TICK_BATCH);

        if now - last_frame >= FRAME_INTERVAL {
            queue!(stdout(), MoveTo(0, 0)).unwrap();
            print!("{}", render_screen(&pet));
            stdout().flush().unwrap();
            last_frame = now;
        }

        std::thread::sleep(Duration::from_millis(1));
    }

    let _ = disable_raw_mode();
    let _ = execute!(stdout(), Show, Clear(ClearType::All));
}
