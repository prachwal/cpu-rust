use cpu_bus::Bus;
use cpu_display::Display;
use cpu_keyboard::Keyboard;

const RAM_SIZE: usize = 1024;
const RIOT_BASE_003: u16 = 0x1800;
const RIOT_BASE_002: u16 = 0x1C00;
const RIOT_IO_SIZE: u16 = 0x08;
const RIOT_ROM_OFFSET: u16 = 0x08;

pub struct Kim1Bus {
    ram: Vec<u8>,
    rom_002: Vec<u8>,
    rom_003: Vec<u8>,
    riot2_pa: u8, riot2_pb: u8,
    riot3_pa: u8, riot3_pb: u8,
    led_segments: [u8; 6],
    display: Display,
    keypad: Keyboard,
}

impl Kim1Bus {
    fn riot_read(&self, base: u16, addr: u16) -> u8 {
        let offset = addr - base;
        if offset < RIOT_IO_SIZE {
            let (pa, pb) = if base == RIOT_BASE_002 {
                (self.riot2_pa, self.riot2_pb)
            } else {
                (self.riot3_pa, self.riot3_pb)
            };
            match offset { 0 => pa, 1 => pb, _ => 0 }
        } else if offset < 0x400 {
            let idx = (offset - RIOT_ROM_OFFSET) as usize;
            if base == RIOT_BASE_002 {
                self.rom_002.get(idx).copied().unwrap_or(0xFF)
            } else {
                self.rom_003.get(idx).copied().unwrap_or(0xFF)
            }
        } else { 0 }
    }

    fn riot_write(&mut self, base: u16, addr: u16, val: u8) {
        let offset = addr - base;
        if offset >= RIOT_IO_SIZE { return; }
        let riot2 = base == RIOT_BASE_002;
        match offset {
            0 => {
                if riot2 { self.riot2_pa = val; } else { self.riot3_pa = val; }
            }
            1 => {
                if riot2 {
                    self.riot2_pb = val;
                    for d in 0..6 {
                        if val & (1 << d) == 0 { self.led_segments[d] = self.riot2_pa; }
                    }
                } else { self.riot3_pb = val; }
            }
            _ => {}
        }
    }

    pub fn render_display(&mut self) {
        self.display.clear();
        let w = self.display.width() as usize;
        for (i, &seg) in self.led_segments.iter().enumerate() {
            if seg == 0 { continue; }
            let ox = i * 13;
            let mut dbuf = [0u8; 13 * 13];
            cpu_segment::render(seg, &mut dbuf, 13, 13, 13);
            for (j, &px) in dbuf.iter().enumerate() {
                if px != 0 {
                    let x = ox + j % 13;
                    let y = j / 13;
                    if x < w { self.display.set_pixel(x as u8, y as u8, 1); }
                }
            }
        }
    }

    pub fn get_display_buffer(&self) -> Vec<u8> { self.display.get_buffer() }
    pub fn get_display_width(&self) -> u16 { self.display.width() }
    pub fn get_display_height(&self) -> u16 { self.display.height() }
    pub fn led_segments(&self) -> &[u8; 6] { &self.led_segments }
    pub fn key_down(&mut self, key: u8) { self.keypad.press(key); }
    pub fn key_up(&mut self, key: u8) { self.keypad.release(key); }

    fn read_vector(&self, addr: u16) -> u16 {
        let lo = self.riot_read(RIOT_BASE_002, addr - 0x1C00 + RIOT_ROM_OFFSET);
        let hi = self.riot_read(RIOT_BASE_002, addr - 0x1C00 + RIOT_ROM_OFFSET + 1);
        (hi as u16) << 8 | lo as u16
    }
}

impl Bus for Kim1Bus {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x03FF => self.ram[addr as usize],
            0x1800..=0x1BFF => self.riot_read(RIOT_BASE_003, addr),
            0x1C00..=0x1FFF => self.riot_read(RIOT_BASE_002, addr),
            // Vectors: NMI=FFFA, RESET=FFFC, IRQ=FFFE
            // These map to 6530-002 ROM $1FFA-$1FFF
            0xFFFA..=0xFFFF => {
                let rom_addr = 0x1FFA + (addr - 0xFFFA);
                self.riot_read(RIOT_BASE_002, rom_addr)
            }
            _ => 0xFF,
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x03FF => self.ram[addr as usize] = val,
            0x1800..=0x1BFF => self.riot_write(RIOT_BASE_003, addr, val),
            0x1C00..=0x1FFF => self.riot_write(RIOT_BASE_002, addr, val),
            _ => {}
        }
    }
}

pub struct Kim1 {
    pub cpu: mos6502_core::Emulator,
    pub bus: Kim1Bus,
}

impl Kim1 {
    pub fn new(rom_002: Vec<u8>, rom_003: Vec<u8>) -> Self {
        let cfg = mos6502_config::MachineConfig::nmos6502();
        let emu = mos6502_core::Emulator::new_with_config(&cfg.to_json()).expect("config");

        let mut kim = Kim1 {
            cpu: emu,
            bus: Kim1Bus {
                ram: vec![0; RAM_SIZE],
                rom_002: { let mut r = rom_002; r.resize(1024, 0xFF); r },
                rom_003: { let mut r = rom_003; r.resize(1024, 0xFF); r },
                riot2_pa: 0, riot2_pb: 0xFF,
                riot3_pa: 0, riot3_pb: 0xFF,
                led_segments: [0; 6],
                display: Display::new(78, 15),
                keypad: Keyboard::new(),
            },
        };

        // Set PC from reset vector in ROM
        let reset_vec = kim.bus.read_vector(0xFFFC);
        kim.cpu.set_register_pc(reset_vec);
        kim
    }

    pub fn tick(&mut self) {
        self.cpu.tick_bus(&mut self.bus);
    }

    pub fn step(&mut self) -> u8 {
        self.cpu.tick_bus(&mut self.bus)
    }

    pub fn get_register_pc(&self) -> u16 { self.cpu.get_register_pc() }
    pub fn get_register_a(&self) -> u8 { self.cpu.get_register_a() }
    pub fn get_register_x(&self) -> u8 { self.cpu.get_register_x() }
    pub fn get_register_y(&self) -> u8 { self.cpu.get_register_y() }
    pub fn get_register_sp(&self) -> u8 { self.cpu.get_register_sp() }

    pub fn render_display(&mut self) {
        self.bus.render_display();
    }

    pub fn key_down(&mut self, key: u8) { self.bus.key_down(key); }
    pub fn key_up(&mut self, key: u8) { self.bus.key_up(key); }
    pub fn get_display_buffer(&self) -> Vec<u8> { self.bus.get_display_buffer() }
    pub fn get_display_width(&self) -> u16 { self.bus.get_display_width() }
    pub fn get_display_height(&self) -> u16 { self.bus.get_display_height() }
}

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
