pub mod rom;

use acia_6551::{Acia6551, SR_RX_FULL};
use cpu_bus::Bus;
use cpu_machine::{Machine, SerialMachine};

const RAM_SIZE: usize = 0x4000; // 16KB RAM
const ACIA_BASE: u16 = 0x6000;
const ROM_SIZE: usize = 0x8000; // 32KB ROM ($8000-$FFFF)

/// Ben Eater-style 6502 computer
pub struct Eater6502Bus {
    pub ram: Vec<u8>,
    pub rom: Vec<u8>,       // WozMon or custom monitor
    pub acia: Acia6551,
    tx_fifo: Vec<u8>,
}

impl Eater6502Bus {
    pub fn new(rom: Vec<u8>) -> Self {
        let mut r = rom;
        r.resize(ROM_SIZE, 0xFF);
        Eater6502Bus {
            ram: vec![0; RAM_SIZE],
            rom: r,
            acia: Acia6551::new(),
            tx_fifo: Vec::new(),
        }
    }

    /// Receive a byte from terminal → ACIA
    pub fn receive_byte(&mut self, data: u8) {
        self.acia.receive(data);
    }

    /// Read all transmitted bytes so far
    pub fn drain_tx(&mut self) -> Vec<u8> {
        self.tx_fifo.drain(..).collect()
    }

    /// Read the next transmitted byte (FIFO order)
    pub fn read_transmitted(&mut self) -> Option<u8> {
        if self.tx_fifo.is_empty() { return None; }
        Some(self.tx_fifo.remove(0))
    }
}

impl Bus for Eater6502Bus {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.ram[addr as usize],
            0x6000..=0x6003 => self.acia.read(addr - ACIA_BASE),
            0x8000..=0xFFFF => self.rom[(addr - 0x8000) as usize],
            _ => 0xFF,
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x3FFF => self.ram[addr as usize] = val,
            0x6000..=0x6003 => {
                if (addr - ACIA_BASE) == 0 {
                    // Writing to data register: capture for testing & complete transmission
                    self.tx_fifo.push(val);
                    self.acia.write(addr - ACIA_BASE, val);
                    self.acia.tx_complete(); // transmission completes immediately
                } else {
                    self.acia.write(addr - ACIA_BASE, val);
                }
            }
            _ => {} // ROM is read-only
        }
    }
}

pub struct Eater6502 {
    pub cpu: mos6502_core::Emulator,
    pub bus: Eater6502Bus,
}

impl Eater6502 {
    pub fn new(rom: Vec<u8>) -> Self {
        let cfg = mos6502_config::MachineConfig::nmos6502();
        let emu = mos6502_core::Emulator::new_with_config(&cfg.to_json()).expect("config");

        let mut machine = Eater6502 {
            cpu: emu,
            bus: Eater6502Bus::new(rom),
        };

        // Set reset vector
        let reset_lo = machine.bus.read(0xFFFC);
        let reset_hi = machine.bus.read(0xFFFD);
        machine.cpu.set_register_pc((reset_hi as u16) << 8 | reset_lo as u16);
        machine.cpu.set_register_sp(0xFF);
        machine
    }

    pub fn tick(&mut self) -> u8 {
        self.cpu.tick_bus(&mut self.bus)
    }

    pub fn get_pc(&self) -> u16 { self.cpu.get_register_pc() }
}

impl Machine for Eater6502 {
    fn tick(&mut self) { self.cpu.tick_bus(&mut self.bus); }
    fn pc(&self) -> u16 { self.cpu.get_register_pc() }
    fn sp(&self) -> u8 { self.cpu.get_register_sp() }
    fn a(&self) -> u8 { self.cpu.get_register_a() }
    fn x(&self) -> u8 { self.cpu.get_register_x() }
    fn y(&self) -> u8 { self.cpu.get_register_y() }
    fn p(&self) -> u8 { self.cpu.get_status_register() }
    fn cycles(&self) -> u64 { self.cpu.get_cycle_count() }
    fn instructions(&self) -> u64 { self.cpu.get_instruction_count() }
}

impl SerialMachine for Eater6502 {
    fn serial_send(&mut self, byte: u8) { self.bus.receive_byte(byte); }
    fn serial_recv(&mut self) -> Option<u8> { self.bus.read_transmitted() }
    fn serial_send_ready(&mut self) -> bool {
        self.bus.acia.status() & SR_RX_FULL == 0
    }
}

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
