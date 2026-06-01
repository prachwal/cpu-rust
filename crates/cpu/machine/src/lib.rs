/// Universal Machine trait for emulators.
///
/// Any CPU-based machine can implement this trait to be used
/// with generic tooling (serial-term, debugger, etc.).

/// Core machine interface: CPU registers + execution
pub trait Machine {
    /// Execute one CPU instruction
    fn tick(&mut self);
    /// Program counter
    fn pc(&self) -> u16;
    /// Stack pointer
    fn sp(&self) -> u8;
    /// Accumulator
    fn a(&self) -> u8;
    /// X index register
    fn x(&self) -> u8;
    /// Y index register
    fn y(&self) -> u8;
    /// Status register (P)
    fn p(&self) -> u8;
    /// Total cycles executed
    fn cycles(&self) -> u64;
    /// Total instructions executed
    fn instructions(&self) -> u64;
}

/// Machine with serial (ACIA/UART) I/O
pub trait SerialMachine: Machine {
    /// Send a byte from host to the machine (keyboard input)
    fn serial_send(&mut self, byte: u8);
    /// Receive a byte from the machine to host (display output), if available
    fn serial_recv(&mut self) -> Option<u8>;
    /// Returns true if the machine is ready to accept a byte
    /// (i.e., the serial receive buffer is empty)
    fn serial_send_ready(&mut self) -> bool;
}

// Re-export for convenience
pub use crate as prelude;
