/// Build a 6502 ROM for the Eater 6502 computer.
/// ACIA at $6000, ROM at $8000-$FFFF, RAM at $0000-$3FFF.

/// Simple 6502 assembler builder.
pub struct Asm {
    buf: Vec<u8>,
    labels: std::collections::HashMap<String, u16>,
    pending: Vec<(String, u16)>, // label references to patch later
}

impl Asm {
    pub fn new() -> Self {
        Self { buf: Vec::new(), labels: std::collections::HashMap::new(), pending: Vec::new() }
    }
    pub fn org(&mut self, _addr: u16) {}
    pub fn pos(&self) -> u16 { self.buf.len() as u16 + 0x8000 }
    pub fn label(&mut self, name: &str) {
        self.labels.insert(name.to_string(), self.pos());
    }
    pub fn patch_label(&mut self, name: &str, at: u16) {
        self.pending.push((name.to_string(), at));
    }
    pub fn byte(&mut self, b: u8) {
        self.buf.push(b);
    }
    pub fn bytes(&mut self, bs: &[u8]) {
        self.buf.extend_from_slice(bs);
    }
    pub fn imm(&mut self, b: u8) {
        self.buf.push(b);
    }
    pub fn addr(&mut self, a: u16) {
        self.buf.push((a & 0xFF) as u8);
        self.buf.push((a >> 8) as u8);
    }
    pub fn reloc(&mut self, target: u16) {
        let offset = target.wrapping_sub(self.pos() + 1) as i16 as i8;
        self.byte(offset as u8);
    }
    pub fn resolve(&mut self) {
        for (name, at) in &self.pending {
            let target = self.labels.get(name).expect(&format!("label {name} not found"));
            // `at` is address of branch opcode; `at + 2` is next instruction.
            let offset = target.wrapping_sub(*at + 2) as i16 as i8;
            // Write the offset at the byte immediately after the opcode
            self.buf[*at as usize - 0x8000 + 1] = offset as u8;
        }
    }
    pub fn finish(self, rom_offset: u16) -> Vec<u8> {
        assert!(rom_offset >= 0x8000);
        let start = (rom_offset - 0x8000) as usize;
        let mut rom = vec![0xFF; 0x8000];
        rom[start..start + self.buf.len()].copy_from_slice(&self.buf);
        // Reset vector
        rom[0x7FFC] = (rom_offset & 0xFF) as u8;
        rom[0x7FFD] = (rom_offset >> 8) as u8;
        rom[0x7FFA] = (rom_offset & 0xFF) as u8;
        rom[0x7FFB] = (rom_offset >> 8) as u8;
        rom[0x7FFE] = (rom_offset & 0xFF) as u8;
        rom[0x7FFF] = (rom_offset >> 8) as u8;
        rom
    }
}

/// Emit a wait-for-status-bit loop.
/// Polls A_STS, AND with mask, BEQ back.
fn wait_bit(a: &mut Asm, sts_addr: u16, mask: u8) {
    let pos = a.pos();
    a.byte(0xAD); a.addr(sts_addr);  // LDA sts_addr
    a.byte(0x29); a.imm(mask);       // AND #mask
    a.byte(0xF0); a.reloc(pos);       // BEQ pos
}

fn send_byte(a: &mut Asm, byte: u8) {
    wait_bit(a, 0x6001, 0x10); // wait Tx empty
    a.byte(0xA9); a.imm(byte);       // LDA #byte
    a.byte(0x8D); a.addr(0x6000);    // STA data
}

pub fn generate_monitor() -> Vec<u8> {
    let mut a = Asm::new();

    // Init
    a.byte(0x78);       // SEI
    a.byte(0xD8);       // CLD
    a.byte(0xA2); a.imm(0xFF); // LDX #$FF
    a.byte(0x9A);       // TXS

    // Reset ACIA
    a.byte(0xA9); a.imm(0x00);  // LDA #0
    a.byte(0x8D); a.addr(0x6001); // STA $6001
    // Set command
    a.byte(0xA9); a.imm(0x0B);  // LDA #$0B
    a.byte(0x8D); a.addr(0x6002); // STA $6002
    // Set control
    a.byte(0xA9); a.imm(0x1F);  // LDA #$1F
    a.byte(0x8D); a.addr(0x6003); // STA $6003

    send_byte(&mut a, b'>');

    // Main loop
    a.label("main_loop");

    // Wait for Rx full (status bit 3)
    wait_bit(&mut a, 0x6001, 0x08);

    // Read data
    a.byte(0xAD); a.addr(0x6000); // LDA $6000
    a.byte(0x85); a.imm(0x00);    // STA $00 (save char)

    // Wait for Tx empty (status bit 4)
    wait_bit(&mut a, 0x6001, 0x10);

    // Echo char
    a.byte(0xA5); a.imm(0x00);    // LDA $00
    a.byte(0x8D); a.addr(0x6000); // STA $6000

    // Check if CR
    a.byte(0xC9); a.imm(0x0D); // CMP #$0D
    a.patch_label("main_loop", a.pos());
    a.byte(0xD0); a.reloc(0); // BNE main_loop (patched)

    send_byte(&mut a, 0x0A);
    send_byte(&mut a, b'>');

    a.byte(0x4C); a.addr(0x8000); // JMP main_loop (absolute)
    // Actually should jump to main_loop label... let's fix the label ref
    // The JMP uses absolute, so we patch later
    // But the label was already defined, so we can compute directly

    a.resolve();

    // Now fix up the JMP absolute address by editing the buffer directly
    // The JMP is at the end of the buffer; its operand is the last 2 bytes
    let main_loop_addr = a.labels.get("main_loop").unwrap();
    let len = a.buf.len();
    a.buf[len - 2] = (main_loop_addr & 0xFF) as u8;
    a.buf[len - 1] = (main_loop_addr >> 8) as u8;

    a.finish(0x8000)
}
