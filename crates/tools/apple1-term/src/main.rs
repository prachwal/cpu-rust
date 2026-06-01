use std::io::{stdout, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{poll, read, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use crossterm::{execute, queue};

const DEFAULT_ROM_DIR: &str = "crates/machines/apple1/roms/apple-1";
const COLS: usize = 40;
const ROWS: usize = 24;
const TICK_BATCH: u32 = 500;
const FRAME_INTERVAL: Duration = Duration::from_millis(33);

struct Screen {
    cells: [[u8; COLS]; ROWS],
    row: usize,
    col: usize,
}

impl Screen {
    fn new() -> Self {
        Screen {
            cells: [[b' '; COLS]; ROWS],
            row: 0,
            col: 0,
        }
    }

    fn push(&mut self, byte: u8) {
        match byte & 0x7F {
            b'\r' | b'\n' => self.newline(),
            0x08 | 0x7F => {
                if self.col > 0 {
                    self.col -= 1;
                    self.cells[self.row][self.col] = b' ';
                }
            }
            ch if (0x20..=0x5F).contains(&ch) => {
                self.cells[self.row][self.col] = ch;
                self.col += 1;
                if self.col >= COLS {
                    self.newline();
                }
            }
            _ => {}
        }
    }

    fn newline(&mut self) {
        self.col = 0;
        if self.row + 1 >= ROWS {
            self.cells.rotate_left(1);
            self.cells[ROWS - 1] = [b' '; COLS];
        } else {
            self.row += 1;
        }
    }

    fn render(&self, pc: u16, instructions: u64, cycles: u64) -> String {
        let mut out = String::new();
        out.push_str(&format!("┌{}┐\r\n", "─".repeat(COLS)));
        for line in &self.cells {
            out.push('│');
            for &ch in line {
                out.push(ch as char);
            }
            out.push_str("│\r\n");
        }
        out.push_str(&format!("└{}┘\r\n", "─".repeat(COLS)));
        out.push_str(&format!(
            "Apple 1  40x24  PC=${:04X}  instr={}  cycles={}  Esc=quit  Ctrl+R=reset\r\n",
            pc, instructions, cycles
        ));
        out
    }
}

fn resolve_rom(path: &Path, name: &str) -> Vec<u8> {
    let file = path.join(name);
    std::fs::read(&file).unwrap_or_else(|e| {
        eprintln!("Cannot read {}: {}", file.display(), e);
        std::process::exit(1);
    })
}

fn key_to_apple1(code: KeyCode) -> Option<u8> {
    match code {
        KeyCode::Enter => Some(b'\r'),
        KeyCode::Backspace => Some(0x7F),
        KeyCode::Char(ch) if ch.is_ascii() => Some(ch.to_ascii_uppercase() as u8),
        _ => None,
    }
}

fn new_emulator(basic: &[u8], wozmon: &[u8]) -> apple1_core::Apple1Emulator {
    let mut emu = apple1_core::Apple1Emulator::new();
    emu.load_roms(basic, wozmon);
    emu
}

fn main() {
    let rom_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_ROM_DIR));

    let basic = resolve_rom(&rom_dir, "basic.bin");
    let wozmon = resolve_rom(&rom_dir, "wozmon.bin");

    let mut emu = new_emulator(&basic, &wozmon);
    let mut screen = Screen::new();

    enable_raw_mode().unwrap();
    execute!(stdout(), Hide, Clear(ClearType::All)).unwrap();

    let mut running = true;
    let mut last_frame = Instant::now();

    while running {
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
                        emu = new_emulator(&basic, &wozmon);
                        screen = Screen::new();
                    }
                    code => {
                        if let Some(ch) = key_to_apple1(code) {
                            emu.press_key(ch);
                        }
                    }
                }
            }
        }

        emu.run(TICK_BATCH);
        for byte in emu.take_display() {
            screen.push(byte);
        }

        let now = Instant::now();
        if now - last_frame >= FRAME_INTERVAL {
            queue!(stdout(), MoveTo(0, 0)).unwrap();
            print!(
                "{}",
                screen.render(emu.get_pc(), emu.get_instructions(), emu.get_cycles())
            );
            stdout().flush().unwrap();
            last_frame = now;
        }

        std::thread::sleep(Duration::from_millis(1));
    }

    let _ = disable_raw_mode();
    let _ = execute!(stdout(), Show, Clear(ClearType::All));
}
