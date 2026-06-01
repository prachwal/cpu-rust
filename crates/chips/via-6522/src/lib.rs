//! MOS 6522 VIA — Versatile Interface Adapter
//!
//! ## Register map (16 bytes, base at system-dependent address)
//!   +$0  PORTB   — I/O port B (read/write)
//!   +$1  PORTA   — I/O port A (read/write)
//!   +$2  DDRB    — Data direction B (1=output)
//!   +$3  DDRA    — Data direction A
//!   +$4  T1C-L   — Timer 1 counter low (read) / T1L-L latch low (write)
//!   +$5  T1C-H   — Timer 1 counter high (read clears IFR bit6) / T1L-H (write starts)
//!   +$6  T1L-L   — Timer 1 latch low (read/write)
//!   +$7  T1L-H   — Timer 1 latch high (read/write)
//!   +$8  T2C-L   — Timer 2 counter low / latch low
//!   +$9  T2C-H   — Timer 2 counter high (read clears IFR bit5) / latch high (write starts)
//!   +$A  SR      — Shift register
//!   +$B  ACR     — Auxiliary control register
//!   +$C  PCR     — Peripheral control register
//!   +$D  IFR     — Interrupt flag register (read & clear bits)
//!   +$E  IER     — Interrupt enable register
//!   +$F  PORTA   — Same as +$1, different handshake
//!
//! ## IRQ flags (IFR/IER bit positions)
//!   0  CA2    1  CA1    2  SR    3  CB2    4  CB1    5  T2    6  T1
//!
//! ## Timer 1 (T1)
//!   16-bit down-counter with 16-bit latch.
//!   ACR bit 7 = 0: one-shot mode (stops on underflow)
//!   ACR bit 7 = 1: free-running mode (reloads on underflow)
//!   Writing T1C-H (register +5) reloads counter from latch and starts.
//!
//! ## Timer 2 (T2)
//!   16-bit down-counter (one-shot only).
//!   Writing T2C-H (register +9) reloads and starts.
//!
//! ## CB1/CA1 control lines
//!   PCR bits select edge sensitivity. In the PET:
//!   CB1 is connected to vertical blank → use as edge-triggered interrupt.

#![no_std]

/// Bits for IFR/IER
pub const IRQ_CA2: u8 = 1 << 0;
pub const IRQ_CA1: u8 = 1 << 1;
pub const IRQ_SR:  u8 = 1 << 2;
pub const IRQ_CB2: u8 = 1 << 3;
pub const IRQ_CB1: u8 = 1 << 4;
pub const IRQ_T2:  u8 = 1 << 5;
pub const IRQ_T1:  u8 = 1 << 6;

/// VIA 6522 state
pub struct Via6522 {
    // Programmer-visible registers
    pub regs: [u8; 16],

    // Internal timer state
    t1_counter: u16,
    t1_latch: u16,
    t2_counter: u16,
    t2_latch: u16,

    /// IRQ output line (driven low when an enabled interrupt is pending)
    pub irq: bool,
}

impl Via6522 {
    pub fn new() -> Self {
        Via6522 {
            regs: [0; 16],
            t1_counter: 0, t1_latch: 0xFFFF,
            t2_counter: 0, t2_latch: 0xFFFF,
            irq: false,
        }
    }

    // ---- Public helpers for external input (CA1/CB1 trigger) ----

    /// Trigger a CA1 edge interrupt (sets IFR_CA1)
    pub fn trigger_ca1(&mut self) {
        self.ifr_set(IRQ_CA1);
    }

    /// Trigger a CB1 edge interrupt (sets IFR_CB1)
    pub fn trigger_cb1(&mut self) {
        self.ifr_set(IRQ_CB1);
    }

    /// Get the current port A value (DDR-masked)
    pub fn port_a_output(&self) -> u8 {
        self.regs[1] & self.regs[3]
    }

    /// Get the current port B value (DDR-masked)
    pub fn port_b_output(&self) -> u8 {
        self.regs[0] & self.regs[2]
    }

    // ---- Read ----
    pub fn read(&mut self, addr: u16) -> u8 {
        let idx = (addr & 0x0F) as usize;
        match idx {
            0 | 1 => {
                // PORTB/PORTA: return output reg for DDR=1, input for DDR=0
                // Input values come from external sources (set_input_*)
                self.regs[idx]
            }
            2 => self.regs[2],       // DDRB
            3 => self.regs[3],       // DDRA
            4 => (self.t1_counter & 0xFF) as u8, // T1C-L
            5 => {
                let h = (self.t1_counter >> 8) as u8;
                self.ifr_clear(IRQ_T1);
                h
            }                              // T1C-H
            6 => (self.t1_latch & 0xFF) as u8,  // T1L-L
            7 => (self.t1_latch >> 8) as u8,    // T1L-H
            8 => (self.t2_counter & 0xFF) as u8, // T2C-L
            9 => {
                let h = (self.t2_counter >> 8) as u8;
                self.ifr_clear(IRQ_T2);
                h
            }                              // T2C-H
            10 => self.regs[10],           // SR
            11 => self.regs[11],           // ACR
            12 => self.regs[12],           // PCR
            13 => {
                let v = self.regs[13];      // IFR (read clears all)
                self.regs[13] = 0;
                self.update_irq();
                v
            }
            14 => self.regs[14] | 0x80,    // IER (bit 7 always 1 on read)
            15 => self.regs[1],            // PORTA alt (same as +1)
            _ => 0,
        }
    }

    // ---- Write ----
    pub fn write(&mut self, addr: u16, val: u8) {
        let idx = (addr & 0x0F) as usize;
        match idx {
            0 => self.regs[0] = val,            // PORTB
            1 | 15 => self.regs[1] = val,        // PORTA
            2 => self.regs[2] = val,             // DDRB
            3 => self.regs[3] = val,             // DDRA
            4 => {
                self.t1_latch = (self.t1_latch & 0xFF00) | val as u16; // T1L-L
            }
            5 => {
                self.t1_latch = (self.t1_latch & 0x00FF) | ((val as u16) << 8);
                self.t1_counter = self.t1_latch;  // reload from latch
                if self.acr() & 0x40 == 0 {
                    self.ifr_clear(IRQ_T1); // one-shot: clear T1 flag on start
                }
            }
            6 => {
                self.t1_latch = (self.t1_latch & 0xFF00) | val as u16; // T1L-L latch only
            }
            7 => {
                self.t1_latch = (self.t1_latch & 0x00FF) | ((val as u16) << 8); // T1L-H
            }
            8 => {
                self.t2_latch = (self.t2_latch & 0xFF00) | val as u16; // T2L-L
            }
            9 => {
                self.t2_latch = (self.t2_latch & 0x00FF) | ((val as u16) << 8);
                self.t2_counter = self.t2_latch;
                self.ifr_clear(IRQ_T2);
            }
            10 => self.regs[10] = val,          // SR
            11 => self.regs[11] = val,          // ACR
            12 => self.regs[12] = val,          // PCR
            13 => {},                           // IFR (write ignored)
            14 => {                             // IER
                if val & 0x80 != 0 {
                    self.regs[14] |= val & 0x7F;
                } else {
                    self.regs[14] &= !(val & 0x7F);
                }
                self.update_irq();
            }
            _ => {}
        }
    }

    // ---- Cycle tick — call once per system clock cycle ----
    /// Advance all timers by `cycles` system clock ticks.
    /// Returns true if IRQ was triggered.
    pub fn tick(&mut self, cycles: u64) -> bool {
        for _ in 0..cycles {
            // Timer 1
            let t1_free = (self.acr() & 0x80) != 0;
            if self.t1_counter > 0 {
                self.t1_counter -= 1;
                if self.t1_counter == 0 {
                    if t1_free { self.t1_counter = self.t1_latch; }
                    self.ifr_set(IRQ_T1);
                }
            }
            // Timer 2
            if self.t2_counter > 0 {
                self.t2_counter -= 1;
                if self.t2_counter == 0 {
                    self.ifr_set(IRQ_T2);
                }
            }
        }
        self.irq
    }

    // ---- Internal helpers ----

    fn acr(&self) -> u8 { self.regs[11] }

    fn ifr_set(&mut self, bits: u8) {
        self.regs[13] |= bits & 0x7F;
        self.update_irq();
    }
    fn ifr_clear(&mut self, bits: u8) {
        self.regs[13] &= !(bits & 0x7F);
        self.update_irq();
    }
    fn update_irq(&mut self) {
        let pending = self.regs[13] & 0x7F;
        let enabled = self.regs[14] & 0x7F;
        if (pending & enabled) != 0 {
            self.regs[13] |= 0x80;
            self.irq = true;
        } else {
            self.regs[13] &= 0x7F;
            self.irq = false;
        }
    }
}

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
