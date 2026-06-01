use std::collections::VecDeque;
use std::io::{stdin, stdout, Read, Write};
use std::time::{Duration, Instant};

use cpu_machine::{Machine, SerialMachine};
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{poll, read, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use crossterm::{execute, queue};

const STATUS_INTERVAL: Duration = Duration::from_millis(100);
const PIPE_BURST: u64 = 100_000;
const COLS: usize = 80;
const ROWS: usize = 24;
const APPLE1_BASIC: &[u8] = include_bytes!("../../../machines/apple1/roms/apple-1/basic.bin");
const APPLE1_WOZMON: &[u8] = include_bytes!("../../../machines/apple1/roms/apple-1/wozmon.bin");

struct Apple1Serial {
    emu: apple1_core::Apple1Emulator,
    output: VecDeque<u8>,
}

impl Apple1Serial {
    fn new() -> Self {
        let mut emu = apple1_core::Apple1Emulator::new();
        emu.load_roms(APPLE1_BASIC, APPLE1_WOZMON);
        Self {
            emu,
            output: VecDeque::new(),
        }
    }
}

impl Machine for Apple1Serial {
    fn tick(&mut self) {
        self.emu.run(1);
    }

    fn pc(&self) -> u16 {
        self.emu.get_pc()
    }

    fn sp(&self) -> u8 {
        self.emu.get_sp()
    }

    fn a(&self) -> u8 {
        self.emu.get_a()
    }

    fn x(&self) -> u8 {
        self.emu.get_x()
    }

    fn y(&self) -> u8 {
        self.emu.get_y()
    }

    fn p(&self) -> u8 {
        self.emu.get_p()
    }

    fn cycles(&self) -> u64 {
        self.emu.get_cycles()
    }

    fn instructions(&self) -> u64 {
        self.emu.get_instructions()
    }
}

impl SerialMachine for Apple1Serial {
    fn serial_send(&mut self, byte: u8) {
        let byte = match byte {
            b'\r' | b'\n' => b'\r',
            b'a'..=b'z' => byte.to_ascii_uppercase(),
            _ => byte,
        };
        self.emu.press_key(byte);
    }

    fn serial_recv(&mut self) -> Option<u8> {
        while self.output.is_empty() {
            self.output
                .extend(self.emu.take_display().into_iter().filter(|&b| b != 0x7F));
            if self.output.is_empty() {
                return None;
            }
        }
        self.output.pop_front()
    }

    fn serial_send_ready(&mut self) -> bool {
        true
    }
}

struct Screen {
    cells: [[u8; COLS]; ROWS],
    row: usize,
    col: usize,
}

impl Screen {
    fn new() -> Self {
        Self {
            cells: [[b' '; COLS]; ROWS],
            row: 0,
            col: 0,
        }
    }

    fn push(&mut self, byte: u8) {
        match byte {
            b'\r' | b'\n' => self.newline(),
            0x08 | 0x7F => {
                if self.col > 0 {
                    self.col -= 1;
                    self.cells[self.row][self.col] = b' ';
                }
            }
            ch if ch.is_ascii_graphic() || ch == b' ' => {
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

    fn render(&self, status: &str) -> String {
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
        out.push_str(status);
        out.push_str("  Esc=quit  Ctrl-C=quit\r\n");
        out
    }
}

fn create_machine(kind: &str, rom_file: Option<&str>) -> Result<Box<dyn SerialMachine>, String> {
    match kind {
        "apple1" | "wozmon" => Ok(Box::new(Apple1Serial::new())),
        "eater" => {
            let rom: Vec<u8> = if let Some(path) = rom_file {
                let data = std::fs::read(path).expect("Cannot read ROM file");
                let mut full = vec![0xFF; 0x8000];
                let len = data.len().min(0x8000);
                full[..len].copy_from_slice(&data[..len]);
                full
            } else {
                eater_6502::rom::generate_monitor()
            };
            Ok(Box::new(eater_6502::Eater6502::new(rom)))
        }
        _ => Err(format!(
            "Unknown machine: {kind}. Supported: eater, apple1, wozmon"
        )),
    }
}

fn status_line(m: &dyn SerialMachine, instr: u64, key: Option<u8>) -> String {
    let last = key.map(|b| {
        if b.is_ascii_graphic() || b == b' ' {
            b as char
        } else {
            '?'
        }
    });
    format!(
        "PC=${:04X}  A=${:02X}  X=${:02X}  Y=${:02X}  SP=${:02X}  P={:08b}  instr={}  last={:?}",
        m.pc(),
        m.a(),
        m.x(),
        m.y(),
        m.sp(),
        m.p(),
        instr,
        last,
    )
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut machine_kind = "eater";
    let mut rom_file: Option<String> = None;
    let mut load_file: Option<(String, u16)> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--machine" => {
                i += 1;
                machine_kind = &args[i];
            }
            "--rom" => {
                i += 1;
                rom_file = Some(args[i].clone());
            }
            "--load" => {
                i += 1;
                let path = args[i].clone();
                let addr = if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    i += 1;
                    u16::from_str_radix(args[i].trim_start_matches("0x"), 16)
                        .expect("Invalid address")
                } else {
                    0x0200
                };
                load_file = Some((path, addr));
            }
            h if h == "-h" || h == "--help" => {
                println!("serial-term -- Universal serial terminal for CPU emulators");
                println!();
                println!("USAGE:");
                println!("  serial-term [OPTIONS]");
                println!();
                println!("OPTIONS:");
                println!(
                    "  --machine <kind>    Machine type: eater, apple1, wozmon (default: eater)"
                );
                println!("  --rom <file>        Custom ROM file");
                println!("  --load <file> [addr] Load binary at address (default 0x0200)");
                println!();
                println!("KEYS:");
                println!("  Any printable char   Send to machine");
                println!("  Enter                Send CR (0x0D)");
                println!("  Backspace            Send BS (0x08)");
                println!("  Ctrl-C / ESC         Quit");
                return;
            }
            _ => {}
        }
        i += 1;
    }

    let is_tty = atty::is(atty::Stream::Stdin);
    if is_tty {
        enable_raw_mode().unwrap();
        execute!(stdout(), Hide, Clear(ClearType::All)).unwrap();
    }

    // Non-blocking stdin for pipe mode
    if !is_tty {
        use std::os::unix::io::AsRawFd;
        let fd = stdin().as_raw_fd();
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        unsafe {
            libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
        }
    }

    let mut machine = match create_machine(machine_kind, rom_file.as_deref()) {
        Ok(machine) => machine,
        Err(err) => {
            if is_tty {
                let _ = disable_raw_mode();
                let _ = execute!(stdout(), Show, Clear(ClearType::All));
            }
            eprintln!("serial-term: {err}");
            std::process::exit(2);
        }
    };

    let mut instr_count: u64 = 0;

    if let Some((_path, _addr)) = load_file {
        eprintln!("Warning: --load not yet supported (use --rom)");
    }

    // Warm up: run init before accepting input
    for _ in 0..200 {
        machine.tick();
        instr_count += 1;
    }

    let mut last_status = Instant::now();
    let mut running = true;
    let mut done = false;
    let mut last_key: Option<u8> = None;
    let mut stdin_buf: VecDeque<u8> = VecDeque::new();
    let mut guard = [0u8; 1024];
    let mut screen = Screen::new();

    // Main loop
    while running {
        // ── Input ──
        if is_tty {
            while poll(Duration::ZERO).unwrap() {
                match read().unwrap() {
                    Event::Key(ke) if ke.kind == KeyEventKind::Press => match ke.code {
                        KeyCode::Esc => running = false,
                        KeyCode::Char('c')
                            if ke
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            running = false
                        }
                        KeyCode::Enter => {
                            stdin_buf.push_back(0x0D);
                        }
                        KeyCode::Backspace => {
                            stdin_buf.push_back(0x08);
                        }
                        KeyCode::Char(ch) => {
                            stdin_buf.push_back(ch as u8);
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        } else if !done {
            loop {
                match stdin().read(&mut guard) {
                    Ok(0) => {
                        done = true;
                        break;
                    }
                    Ok(n) => {
                        stdin_buf.extend(&guard[..n]);
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(e) => {
                        eprintln!("stdin error: {e}");
                        running = false;
                        break;
                    }
                }
            }
        }

        // ── Execute ──
        let burst = if is_tty {
            500 // batch to process key input faster
        } else if done {
            PIPE_BURST.min(stdin_buf.len().max(1) as u64 * 500)
        } else {
            1
        };
        for _ in 0..burst {
            // Feed a byte only when ACIA is ready
            if let (true, Some(b)) = (machine.serial_send_ready(), stdin_buf.pop_front()) {
                last_key = Some(b);
                machine.serial_send(b);
            }

            machine.tick();
            instr_count += 1;

            // Drain serial output
            while let Some(byte) = machine.serial_recv() {
                if is_tty {
                    screen.push(byte);
                } else {
                    let _ = stdout().write(&[byte]);
                    let _ = stdout().flush();
                }
            }
        }

        // ── Render/status ──
        let now = Instant::now();
        if is_tty && now - last_status >= STATUS_INTERVAL {
            let line = status_line(&*machine, instr_count, last_key);
            queue!(stdout(), MoveTo(0, 0)).unwrap();
            print!("{}", screen.render(&line));
            stdout().flush().unwrap();
            last_status = now;
        }

        // ── Termination (pipe mode) ──
        if done && stdin_buf.is_empty() {
            // Final drain
            for _ in 0..1000 {
                machine.tick();
                while let Some(byte) = machine.serial_recv() {
                    let _ = stdout().write(&[byte]);
                }
            }
            running = false;
        }

        // ── Yield (give OS time to process I/O) ──
        std::thread::sleep(Duration::from_micros(if is_tty { 1 } else { 100 }));
    }

    // Restore terminal
    if is_tty {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), Show, Clear(ClearType::All));
    }
    eprintln!();
    eprintln!(
        "serial-term: {} instructions, {} cycles.",
        instr_count,
        machine.cycles()
    );
}
