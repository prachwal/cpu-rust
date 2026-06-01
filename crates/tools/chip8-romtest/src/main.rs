use std::path::PathBuf;
use std::time::Instant;

const ROM_DIR: &str = "crates/machines/chip8/tests/roms";
const MAX_INSTRUCTIONS: u32 = 200_000;

struct RomInfo {
    name: String,
    path: PathBuf,
}

fn collect_roms(path: &str) -> Vec<RomInfo> {
    let dir = PathBuf::from(path);
    if !dir.is_dir() { return vec![]; }

    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "ch8").unwrap_or(false))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    entries
        .into_iter()
        .map(|e| RomInfo {
            name: e.file_name().to_string_lossy().to_string(),
            path: e.path(),
        })
        .collect()
}

fn render_display(buffer: &[u8], width: u16, height: u16) -> String {
    let w = width as usize;
    let h = height as usize;
    let mut out = String::new();

    out.push('\n');
    // top border
    out.push_str(&format!("  ┌{}┐\n", "─".repeat(w)));

    for row in (0..h).step_by(2) {
        out.push_str("  │");
        for col in 0..w {
            let top = buffer[row * w + col] != 0;
            let bot = if row + 1 < h {
                buffer[(row + 1) * w + col] != 0
            } else {
                false
            };
            let ch = match (top, bot) {
                (true, true) => '█',
                (true, false) => '▀',
                (false, true) => '▄',
                (false, false) => ' ',
            };
            out.push(ch);
        }
        out.push_str("│\n");
    }

    // bottom border
    out.push_str(&format!("  └{}┘", "─".repeat(w)));
    out
}

fn run_rom(rom: &RomInfo, show_display: bool) {
    let data = match std::fs::read(&rom.path) {
        Ok(d) => d,
        Err(e) => { println!("  FAIL: {}\n", e); return; }
    };

    let mut emu = chip8_machine::Emulator::new();
    emu.load_rom(&data);

    let start = Instant::now();
    let mut instr_count: u32 = 0;

    for _ in 0..MAX_INSTRUCTIONS {
        emu.tick();
        instr_count += 1;
        if emu.paused() { break; }
    }

    let elapsed = start.elapsed();
    let speed = instr_count as f64 / elapsed.as_secs_f64().max(0.0001);

    print!("  {} instr, {:.0} ips, {:.1}ms",
        instr_count, speed, elapsed.as_secs_f64() * 1000.0);

    if emu.paused() {
        println!(" (halted at PC=${:04X})", emu.get_register_pc());
    } else {
        println!(" (timeout)");
    }

    print!("  V: ");
    for i in 0..16 {
        let v = emu.get_register_v(i);
        if v != 0 {
            print!("V{:X}=${:02X} ", i, v);
        }
    }
    println!();
    println!("  PC=${:04X}", emu.get_register_pc());

    if show_display {
        let buf = emu.get_display();
        let w = emu.get_display_width();
        let h = emu.get_display_height();
        println!("{}", render_display(&buf, w, h));
    }
}

fn usage() -> ! {
    eprintln!("Usage: chip8-romtest [OPTIONS] [path]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --show    Render final display state to terminal");
    eprintln!();
    eprintln!("If path is a directory, all .ch8 ROMs are tested.");
    eprintln!("If path is a file, that ROM is tested.");
    eprintln!("If omitted, defaults to {}", ROM_DIR);
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut show_display = false;
    let mut path: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--show" => show_display = true,
            "--help" | "-h" => usage(),
            s if s.starts_with('-') => {
                eprintln!("Unknown option: {}", s);
                usage();
            }
            p => path = Some(p.to_owned()),
        }
        i += 1;
    }

    let roms = match path {
        Some(ref p) => {
            let pb = PathBuf::from(p);
            if pb.is_dir() {
                collect_roms(p)
            } else if pb.is_file() {
                vec![RomInfo {
                    name: pb.file_name().unwrap().to_string_lossy().to_string(),
                    path: pb,
                }]
            } else {
                eprintln!("Path not found: {}", p);
                std::process::exit(1);
            }
        }
        None => collect_roms(ROM_DIR),
    };

    if roms.is_empty() {
        let dir = path.as_deref().unwrap_or(ROM_DIR);
        eprintln!("No .ch8 ROMs found in {}", dir);
        std::process::exit(1);
    }

    println!("CHIP-8 ROM Test ({} ROMs)\n", roms.len());

    for rom in &roms {
        println!("{}", rom.name);
        run_rom(rom, show_display);
        println!();
    }
}
