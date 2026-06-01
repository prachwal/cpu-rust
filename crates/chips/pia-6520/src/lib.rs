//! MOS 6520 / 6821 PIA — Peripheral Interface Adapter
//!
//! ## Register map (per PIA, 4 addresses)
//!   base+0  — Port A data (read: input OR output reg; write: output reg)
//!   base+1  — Port A control (CRA)
//!   base+2  — Port B data
//!   base+3  — Port B control (CRB)
//!
//! ## Port I/O
//!   Each port bit can be input (DDR=0) or output (DDR=1).
//!   - `read(addr)` returns: output reg for DDR=1 bits, input latch for DDR=0 bits
//!   - `write(addr, val)` writes to output reg (drives DDR=1 pins)
//!
//! ## Usage
//!   ```
//!   use pia_6520::Pia6821;
//!   let mut pia = Pia6821::new();
//!   // Connect port A bit 0 as input, set value
//!   pia.set_input_a(0x01);
//!   // Read port A data
//!   let val = pia.read(0, 0x00, 0x00);  // addr & 3 == 0, port A/B inputs = 0
//!   ```

#![no_std]

/// MOS 6821 PIA state
pub struct Pia6821 {
    /// Output registers
    pub ora: u8,
    pub orb: u8,
    /// Data direction registers (1 = output)
    pub ddra: u8,
    pub ddrb: u8,
    /// Control registers
    pub cra: u8,
    pub crb: u8,
    /// Input latches (read-only from pins)
    input_a: u8,
    input_b: u8,
}

impl Pia6821 {
    pub fn new() -> Self {
        Pia6821 {
            ora: 0, orb: 0,
            ddra: 0, ddrb: 0,
            cra: 0, crb: 0,
            input_a: 0, input_b: 0,
        }
    }

    /// Read from a PIA register (addr & 3 gives the register index).
    /// `port_a_input` / `port_b_input` are the current pin values for DDR=0 bits.
    pub fn read(&self, addr: u16, port_a_input: u8, port_b_input: u8) -> u8 {
        match addr & 3 {
            0 => (self.ora & self.ddra) | (port_a_input & !self.ddra),
            1 => self.cra,
            2 => (self.orb & self.ddrb) | (port_b_input & !self.ddrb),
            3 => self.crb,
            _ => 0,
        }
    }

    /// Write to a PIA register.
    /// Returns the value written to port pins (for DDR=1 bits).
    pub fn write(&mut self, addr: u16, val: u8) {
        match addr & 3 {
            0 => self.ora = val,
            1 => self.cra = val,
            2 => self.orb = val,
            3 => self.crb = val,
            _ => {}
        }
    }

    /// Set external input values on port A pins (for DDR=0 bits)
    pub fn set_input_a(&mut self, val: u8) {
        self.input_a = val;
    }

    /// Set external input values on port B pins (for DDR=0 bits)
    pub fn set_input_b(&mut self, val: u8) {
        self.input_b = val;
    }

    /// Get the value currently driven on port A output pins (DDR=1 bits)
    pub fn output_a(&self) -> u8 {
        self.ora & self.ddra
    }

    /// Get the value currently driven on port B output pins (DDR=1 bits)
    pub fn output_b(&self) -> u8 {
        self.orb & self.ddrb
    }
}

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
