#![no_std]

// ── Status Register bits ──
pub const SR_IRQ: u8     = 0x80;
pub const SR_DSR: u8     = 0x40;
pub const SR_DCD: u8     = 0x20;
pub const SR_TX_EMPTY: u8 = 0x10;
pub const SR_RX_FULL: u8  = 0x08;
pub const SR_OVERRUN: u8  = 0x04;
pub const SR_FRAMING: u8  = 0x02;
pub const SR_PARITY: u8   = 0x01;

// ── Command Register masks ──
pub const CMD_PARITY: u8     = 0xC0;
pub const CMD_PARITY_TYPE: u8 = 0x20;
pub const CMD_ECHO: u8       = 0x10;
pub const CMD_IRQ_TX: u8     = 0x08;
pub const CMD_IRQ_RX: u8     = 0x04;
pub const CMD_MODE: u8       = 0x03;
pub const CMD_MODE_NORMAL: u8    = 0;
pub const CMD_MODE_ECHO: u8      = 1;
pub const CMD_MODE_LOOPBACK: u8  = 2;
pub const CMD_MODE_REMOTE: u8    = 3;

pub struct Acia6551 {
    status: u8,
    command: u8,
    control: u8,

    // internal state
    rx_data: u8,
    rx_pending: bool,
    tx_pending: bool,

    // external observer (for testing)
    pub tx_output: Option<u8>,
    pub irq: bool,
}

impl Acia6551 {
    pub fn new() -> Self {
        Acia6551 {
            status: SR_TX_EMPTY | SR_DSR | SR_DCD,
            command: 0,
            control: 0,
            rx_data: 0,
            rx_pending: false,
            tx_pending: false,
            tx_output: None,
            irq: false,
        }
    }

    // ── Register Access ──

    pub fn read(&mut self, addr: u16) -> u8 {
        match (addr & 3) as u8 {
            0 => self.read_data(),
            1 => self.read_status(),
            2 => self.command,
            3 => self.control,
            _ => 0,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match (addr & 3) as u8 {
            0 => self.write_data(val),
            1 => self.reset(),
            2 => { self.command = val; self.update_irq(); }
            3 => self.control = val,
            _ => {}
        }
    }

    fn read_data(&mut self) -> u8 {
        self.rx_pending = false;
        self.status &= !SR_RX_FULL;
        // Also clears overrun flag
        self.status &= !SR_OVERRUN;
        self.update_irq();
        self.rx_data
    }

    fn read_status(&self) -> u8 {
        self.status
    }

    fn write_data(&mut self, val: u8) {
        // In echo mode, data written is immediately received
        match self.command & CMD_MODE {
            CMD_MODE_ECHO => {
                self.receive_byte(val);
            }
            CMD_MODE_LOOPBACK => {
                self.receive_byte(val);
                self.tx_output = Some(val);
            }
            CMD_MODE_REMOTE => {
                self.tx_output = Some(val);
            }
            _ => {
                self.tx_output = Some(val);
            }
        }

        // Clear Tx Empty flag until transmission completes
        self.status &= !SR_TX_EMPTY;
        self.tx_pending = true;
        self.update_irq();
    }

    fn reset(&mut self) {
        self.status = SR_TX_EMPTY | SR_DSR | SR_DCD;
        self.command = 0;
        self.control = 0;
        self.rx_pending = false;
        self.tx_pending = false;
        self.tx_output = None;
        self.irq = false;
    }

    // ── External pin inputs ──

    /// Drive DSR line (call with true when DSR asserted).
    pub fn set_dsr(&mut self, level: bool) {
        if level { self.status |= SR_DSR; } else { self.status &= !SR_DSR; }
    }

    /// Drive DCD line.
    pub fn set_dcd(&mut self, level: bool) {
        if level { self.status |= SR_DCD; } else { self.status &= !SR_DCD; }
    }

    /// Receive a byte from serial line (call when start bit detected).
    pub fn receive(&mut self, data: u8) {
        if self.rx_pending {
            self.status |= SR_OVERRUN; // previous byte was not read
        }
        self.receive_byte(data);
    }

    fn receive_byte(&mut self, data: u8) {
        self.rx_data = data;
        self.rx_pending = true;
        self.status |= SR_RX_FULL;

        // Check parity (simplified)
        let parity_mode = (self.command >> 6) & 3;
        if parity_mode == 1 || parity_mode == 2 {
            // odd or even parity
            let expected_parity = self.rx_data >> 7; // pretend bit 7 is parity
            let computed = if parity_mode == 1 { 1 } else { 0 };
            if expected_parity != computed {
                self.status |= SR_PARITY;
            }
        }

        self.update_irq();
    }

    /// Advance serial transmission (call when one character sent).
    /// Returns true if transmission completed.
    pub fn tx_complete(&mut self) -> bool {
        if self.tx_pending {
            self.tx_pending = false;
            self.status |= SR_TX_EMPTY;
            self.update_irq();
            return true;
        }
        false
    }

    fn update_irq(&mut self) {
        let irq_tx = (self.command & CMD_IRQ_TX != 0) && (self.status & SR_TX_EMPTY != 0);
        let irq_rx = (self.command & CMD_IRQ_RX != 0) && (self.status & SR_RX_FULL != 0);
        let irq = irq_tx || irq_rx;
        self.irq = irq;
        if irq { self.status |= SR_IRQ; } else { self.status &= !SR_IRQ; }
    }

    // ── Utility ──

    pub fn status(&self) -> u8 { self.status }
    pub fn command_reg(&self) -> u8 { self.command }
    pub fn control_reg(&self) -> u8 { self.control }
    pub fn rx_data(&self) -> u8 { self.rx_data }
}

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
