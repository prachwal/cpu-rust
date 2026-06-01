use super::*;
    use mos6502_config::MachineConfig;
use mos6502_memory::Memory;

fn setup() -> (Cpu, Memory) {
    let config = MachineConfig::default();
    let cpu = Cpu::new(config.clone());
    let memory = Memory::new(&config);
    (cpu, memory)
}

#[test]
fn test_decode_all_known() {
    let known: [u8; 56] = [
        0x00,0x01,0x05,0x06,0x08,0x09,0x0A,0x0D,0x0E,0x10,0x11,0x15,0x16,0x18,0x19,0x1D,
        0x1E,0x20,0x21,0x24,0x25,0x26,0x28,0x29,0x2A,0x2C,0x2D,0x2E,0x30,0x31,0x35,0x36,
        0x38,0x39,0x3D,0x3E,0x40,0x41,0x45,0x46,0x48,0x49,0x4A,0x4C,0x4D,0x4E,0x50,0x51,
        0x55,0x56,0x58,0x59,0x5D,0x5E,0x60,0x61,
    ];
    for &op in &known {
        assert!(decode(op).is_some(), "opcode {:02X} should be known", op);
    }
}

#[test]
fn test_lda_immediate() {
    let (mut cpu, mut mem) = setup();
    mem.write(cpu.pc, 0xA9); mem.write(cpu.pc+1, 0x42);
    let cyc = execute(&mut cpu, &mut mem);
    assert_eq!(cpu.a, 0x42);
    assert!(!cpu.sr.n()); assert!(!cpu.sr.z());
    assert_eq!(cyc, 2);
}

#[test]
fn test_tax() {
    let (mut cpu, mut mem) = setup();
    cpu.a = 0xFF;
    mem.write(cpu.pc, 0xAA);
    execute(&mut cpu, &mut mem);
    assert_eq!(cpu.x, 0xFF);
    assert!(cpu.sr.n());
}

    #[test]
    fn test_kil_halts_cpu() {
        let (mut cpu, mut mem) = setup();
        let pc = cpu.pc;
        mem.write(pc, 0x02); // KIL
        let cyc = execute(&mut cpu, &mut mem);
        assert!(cpu.halted);
        assert_eq!(cyc, 2);
    }

    #[test]
    fn test_kil_all_12_opcodes_halt() {
        let kils = [0x02, 0x12, 0x22, 0x32, 0x42, 0x52, 0x62, 0x72, 0x92, 0xB2, 0xD2, 0xF2];
        for &op in &kils {
            let (mut cpu, mut mem) = setup();
            assert!(decode(op).is_some(), "KIL {:02X} should decode", op);
            mem.write(cpu.pc, op);
            execute(&mut cpu, &mut mem);
            assert!(cpu.halted, "KIL {:02X} should halt", op);
        }
    }

    #[test]
    fn test_adc_bcd_simple() {
        let (mut cpu, mut mem) = setup();
        // 15 + 15 = 30 in BCD = 0x30
        cpu.a = 0x15;
        cpu.sr.set_d(true);
        mem.write(cpu.pc, 0x69); mem.write(cpu.pc + 1, 0x15);
        execute(&mut cpu, &mut mem);
        assert_eq!(cpu.a, 0x30);
        assert!(!cpu.sr.c());
    }

    #[test]
    fn test_adc_bcd_carry() {
        let (mut cpu, mut mem) = setup();
        // 99 + 1 = 100 in BCD = 0x00 with carry
        cpu.a = 0x99;
        cpu.sr.set_d(true);
        mem.write(cpu.pc, 0x69); mem.write(cpu.pc + 1, 0x01);
        execute(&mut cpu, &mut mem);
        assert_eq!(cpu.a, 0x00);
        assert!(cpu.sr.c());
    }

    #[test]
    fn test_adc_bcd_with_carry_in() {
        let (mut cpu, mut mem) = setup();
        // 10 + 10 + 1 (carry) = 21 in BCD = 0x21
        cpu.a = 0x10;
        cpu.sr.set_d(true);
        cpu.sr.set_c(true);
        mem.write(cpu.pc, 0x69); mem.write(cpu.pc + 1, 0x10);
        execute(&mut cpu, &mut mem);
        assert_eq!(cpu.a, 0x21);
        assert!(!cpu.sr.c());
    }

    #[test]
    fn test_adc_bcd_no_carry_flag_unchanged() {
        let (mut cpu, mut mem) = setup();
        cpu.a = 0x05;
        cpu.sr.set_d(true);
        mem.write(cpu.pc, 0x69); mem.write(cpu.pc + 1, 0x03);
        execute(&mut cpu, &mut mem);
        assert_eq!(cpu.a, 0x08);
        assert!(!cpu.sr.c());
    }

    #[test]
    fn test_sbc_bcd_simple() {
        let (mut cpu, mut mem) = setup();
        // 30 - 15 = 15 in BCD = 0x15
        cpu.a = 0x30;
        cpu.sr.set_d(true);
        cpu.sr.set_c(true); // no borrow
        mem.write(cpu.pc, 0xE9); mem.write(cpu.pc + 1, 0x15);
        execute(&mut cpu, &mut mem);
        assert_eq!(cpu.a, 0x15);
        assert!(cpu.sr.c());
    }

    #[test]
    fn test_nop_all_modes_run_with_correct_cycles() {
        // Test each NOP mode individually for decode + cycle correctness
        let cases: [(&[u8], u8); 6] = [
            (&[0x80, 0xFF], 2),         // Immediate
            (&[0x04, 0x80], 3),         // ZeroPage
            (&[0x44, 0x80], 3),         // ZeroPage
            (&[0x14, 0x80], 4),         // ZeroPageX
            (&[0x0C, 0x00, 0x20], 4),  // Absolute
            (&[0x1C, 0x00, 0x10], 4),  // AbsoluteX no cross
        ];
        let mut opcode: u8 = 0x80;
        for (data, expected) in &cases {
            let (mut cpu, mut mem) = setup();
            for (i, &b) in data.iter().enumerate() {
                mem.write(cpu.pc + i as u16, b);
            }
            let cyc = execute(&mut cpu, &mut mem);
            assert_eq!(cyc, *expected, "opcode {:02X}", data[0]);
        }
    }

    #[test]
    fn test_nop_page_cross_extra_cycle() {
        let (mut cpu, mut mem) = setup();
        cpu.x = 0xFF;
        mem.write(cpu.pc, 0x1C); mem.write(cpu.pc + 1, 0x01); mem.write(cpu.pc + 2, 0x10);
        let cyc = execute(&mut cpu, &mut mem);
        assert_eq!(cyc, 5, "page cross should add 1 cycle");
    }

    #[test]
    fn test_undocumented_disabled_blocks_slo() {
        let (mut cpu, mut mem) = setup();
        cpu.config.quirks.undocumented_ops = false;
        mem.write(cpu.pc, 0x07); mem.write(cpu.pc + 1, 0x80); // *SLO ZeroPage
        let cyc = execute(&mut cpu, &mut mem);
        assert_eq!(cyc, 2, "blocked illegal -> 2-cycle NOP");
    }

    #[test]
    fn test_undocumented_disabled_blocks_lax() {
        let (mut cpu, mut mem) = setup();
        cpu.config.quirks.undocumented_ops = false;
        let pc = cpu.pc;
        mem.write(pc, 0xA7); mem.write(pc + 1, 0x80); mem.write(0x80, 0x42); // *LAX ZeroPage from $80
        let cyc = execute(&mut cpu, &mut mem);
        assert_ne!(cpu.a, 0x42, "LAX should not load when disabled");
        assert_eq!(cyc, 2, "blocked illegal -> 2-cycle NOP");
    }

    #[test]
    fn test_dcp_zeropage() {
        let (mut cpu, mut mem) = setup();
        cpu.a = 0x05;
        mem.write(0x80, 0x03); mem.write(cpu.pc, 0xC7); mem.write(cpu.pc + 1, 0x80);
        execute(&mut cpu, &mut mem);
        assert_eq!(mem.read(0x80), 0x02); // DEC $03 → $02
        assert!(cpu.sr.c());               // A >= $02 ($05 >= $02)
        assert!(!cpu.sr.z());              // A != $02
    }

    #[test]
    fn test_dcp_sets_zero_flag() {
        let (mut cpu, mut mem) = setup();
        cpu.a = 0x04;
        mem.write(0x80, 0x05); mem.write(cpu.pc, 0xC7); mem.write(cpu.pc + 1, 0x80);
        execute(&mut cpu, &mut mem);
        assert_eq!(mem.read(0x80), 0x04); // DEC $05 → $04
        assert!(cpu.sr.z());               // A == $04
    }

    #[test]
    fn test_isc_zeropage() {
        let (mut cpu, mut mem) = setup();
        cpu.a = 0x10;
        cpu.sr.set_c(true);
        mem.write(0x80, 0x05); mem.write(cpu.pc, 0xE7); mem.write(cpu.pc + 1, 0x80);
        execute(&mut cpu, &mut mem);
        assert_eq!(mem.read(0x80), 0x06); // INC $05 → $06
        // SBC $06 from A=$10 with C=1: $10 - $06 - 0 = $0A
        assert_eq!(cpu.a, 0x0A);
    }

    #[test]
    fn test_anc_sets_carry_from_bit7() {
        let (mut cpu, mut mem) = setup();
        cpu.a = 0x80;
        mem.write(cpu.pc, 0x0B); mem.write(cpu.pc + 1, 0xFF);
        execute(&mut cpu, &mut mem);
        assert_eq!(cpu.a, 0x80);
        assert!(cpu.sr.c());
        assert!(cpu.sr.n());
    }

    #[test]
    fn test_alr_and_lsr() {
        let (mut cpu, mut mem) = setup();
        cpu.a = 0xFF;
        mem.write(cpu.pc, 0x4B); mem.write(cpu.pc + 1, 0x0F);
        execute(&mut cpu, &mut mem);
        assert_eq!(cpu.a, 0x07); // AND $0F → $0F, LSR → $07
        assert!(cpu.sr.c());      // LSB was 1 → carry set
    }

    #[test]
    fn test_arr_and_ror() {
        let (mut cpu, mut mem) = setup();
        cpu.a = 0xFF;
        cpu.sr.set_c(true);
        mem.write(cpu.pc, 0x6B); mem.write(cpu.pc + 1, 0x80);
        execute(&mut cpu, &mut mem);
        // AND $80 → $80, ROR with C=1 → $C0
        assert_eq!(cpu.a, 0xC0);
        assert!(!cpu.sr.c());      // LSB was 0
    }

    #[test]
    fn test_sax_immediate() {
        let (mut cpu, mut mem) = setup();
        cpu.a = 0x0F;
        cpu.x = 0xFF;
        mem.write(cpu.pc, 0xCB); mem.write(cpu.pc + 1, 0x05);
        execute(&mut cpu, &mut mem);
        // A&X = $0F, $0F - $05 = $0A
        assert_eq!(cpu.x, 0x0A);
    }

    #[test]
    fn test_rmw_cmos_dummy_read() {
        let (mut cpu, mut mem) = setup();
        cpu.config.quirks.rmw = RmwBehavior::Cmos;
        // ASL memory with dummy read instead of dummy write
        mem.write(0x80, 0x01);
        mem.write(cpu.pc, 0x06); mem.write(cpu.pc + 1, 0x80);
        execute(&mut cpu, &mut mem);
        assert_eq!(mem.read(0x80), 0x02); // ASL $01 → $02
    }

    #[test]
    fn test_rmw_nmos_dummy_write() {
        let (mut cpu, mut mem) = setup();
        cpu.config.quirks.rmw = RmwBehavior::Nmos;
        mem.write(0x80, 0x01);
        mem.write(cpu.pc, 0x06); mem.write(cpu.pc + 1, 0x80);
        execute(&mut cpu, &mut mem);
        assert_eq!(mem.read(0x80), 0x02);
    }

    #[test]
    fn test_jmp_indirect_nmos_bug() {
        // NMOS: JMP ($10FF) — indirect addr $10FF has low byte $FF
        // Bug: reads hi target byte from $1000 (same page) instead of $1100
        let (mut cpu, mut mem) = setup();
        cpu.config.quirks.jmp_indirect_bug = true;
        mem.write(0x10FF, 0x78);                          // lo byte of target
        mem.write(0x1000, 0x56);                           // NMOS: hi from $1000 → target $5678
        mem.write(0x1100, 0x34);                           // CMOS would read hi from $1100 → $3478
        mem.write(cpu.pc, 0x6C); mem.write(cpu.pc + 1, 0xFF); mem.write(cpu.pc + 2, 0x10);
        execute(&mut cpu, &mut mem);
        assert_eq!(cpu.pc, 0x5678); // NMOS: hi from $1000
    }

    #[test]
    fn test_jmp_indirect_cmos_no_bug() {
        // CMOS: same JMP ($10FF) — reads hi from $1100 normally
        let (mut cpu, mut mem) = setup();
        cpu.config.quirks.jmp_indirect_bug = false;
        mem.write(0x10FF, 0x78);
        mem.write(0x1000, 0x56);                           // NMOS would read this
        mem.write(0x1100, 0x34);                           // CMOS: hi from $1100 → $3478
        mem.write(cpu.pc, 0x6C); mem.write(cpu.pc + 1, 0xFF); mem.write(cpu.pc + 2, 0x10);
        execute(&mut cpu, &mut mem);
        assert_eq!(cpu.pc, 0x3478); // CMOS: hi from $1100
    }

    #[test]
    fn test_stp_halts_cpu() {
        let mut cpu = Cpu::new(MachineConfig::w65c02());
        let mut mem = Memory::new(&MachineConfig::w65c02());
        let pc = cpu.pc;
        mem.write(pc, 0xDB); // STP
        let cyc = execute(&mut cpu, &mut mem);
        assert!(cpu.stopped);
        assert_eq!(cyc, 2);
    }

    #[test]
    fn test_wai_waits() {
        let mut cpu = Cpu::new(MachineConfig::w65c02());
        let mut mem = Memory::new(&MachineConfig::w65c02());
        mem.write(cpu.pc, 0xCB);
        let cyc = execute(&mut cpu, &mut mem);
        assert!(cpu.waiting);
        assert_eq!(cyc, 2);
    }

    #[test]
    fn test_bra_branches_always() {
        let mut cpu = Cpu::new(MachineConfig::w65c02());
        let mut mem = Memory::new(&MachineConfig::w65c02());
        let pc = cpu.pc;
        mem.write(pc, 0x80); mem.write(pc + 1, 0x10);
        let cyc = execute(&mut cpu, &mut mem);
        assert_eq!(cpu.pc, pc + 2 + 16);
        assert!(cyc == 3 || cyc == 4);
    }

    #[test]
    fn test_phx_plx() {
        let mut cpu = Cpu::new(MachineConfig::w65c02());
        let mut mem = Memory::new(&MachineConfig::w65c02());
        let pc = cpu.pc;
        cpu.x = 0x42;
        mem.write(pc, 0xDA); // PHX
        execute(&mut cpu, &mut mem);
        assert_eq!(cpu.sp, 0xFE);
        cpu.x = 0x00;
        mem.write(pc + 1, 0xFA); // PLX
        execute(&mut cpu, &mut mem);
        assert_eq!(cpu.x, 0x42);
    }

    #[test]
    fn test_bit_immediate_cmos() {
        let mut cpu = Cpu::new(MachineConfig::w65c02());
        let mut mem = Memory::new(&MachineConfig::w65c02());
        cpu.a = 0x80;
        mem.write(cpu.pc, 0x89); mem.write(cpu.pc + 1, 0x80);
        execute(&mut cpu, &mut mem);
        assert!(cpu.sr.n());
        assert!(!cpu.sr.z());
    }

    #[test]
    fn test_adc_carry() {
    let (mut cpu, mut mem) = setup();
    cpu.a = 0xFF;
    mem.write(cpu.pc, 0x69); mem.write(cpu.pc+1, 0x01);
    execute(&mut cpu, &mut mem);
    assert_eq!(cpu.a, 0x00);
    assert!(cpu.sr.c());
    assert!(cpu.sr.z());
}

#[test]
fn test_sbc_no_borrow() {
    let (mut cpu, mut mem) = setup();
    cpu.a = 0x0A;
    cpu.sr.set_c(true);
    mem.write(cpu.pc, 0xE9); mem.write(cpu.pc + 1, 0x05);
    execute(&mut cpu, &mut mem);
    assert_eq!(cpu.a, 0x05);
    assert!(cpu.sr.c());
}

#[test]
fn test_and_immediate() {
    let (mut cpu, mut mem) = setup();
    cpu.a = 0xFF;
    mem.write(cpu.pc, 0x29); mem.write(cpu.pc + 1, 0x0F);
    execute(&mut cpu, &mut mem);
    assert_eq!(cpu.a, 0x0F);
}

#[test]
fn test_ora_immediate() {
    let (mut cpu, mut mem) = setup();
    cpu.a = 0xF0;
    mem.write(cpu.pc, 0x09); mem.write(cpu.pc + 1, 0x0F);
    execute(&mut cpu, &mut mem);
    assert_eq!(cpu.a, 0xFF);
}

#[test]
fn test_eor_immediate() {
    let (mut cpu, mut mem) = setup();
    cpu.a = 0xFF;
    mem.write(cpu.pc, 0x49); mem.write(cpu.pc + 1, 0x0F);
    execute(&mut cpu, &mut mem);
    assert_eq!(cpu.a, 0xF0);
}

#[test]
fn test_jmp_absolute() {
    let (mut cpu, mut mem) = setup();
    mem.write(cpu.pc, 0x4C); mem.write(cpu.pc + 1, 0x00); mem.write(cpu.pc + 2, 0x30);
    execute(&mut cpu, &mut mem);
    assert_eq!(cpu.pc, 0x3000);
}

#[test]
fn test_jsr_rts() {
    let (mut cpu, mut mem) = setup();
    cpu.sp = 0xFF;
    // JSR to $3000
    mem.write(cpu.pc, 0x20); mem.write(cpu.pc + 1, 0x00); mem.write(cpu.pc + 2, 0x30);
    let jsr_pc = cpu.pc;
    execute(&mut cpu, &mut mem);
    assert_eq!(cpu.pc, 0x3000);
    assert_eq!(cpu.sp, 0xFD); // 2 bytes pushed
    // RTS back
    mem.write(cpu.pc, 0x60);
    execute(&mut cpu, &mut mem);
    assert_eq!(cpu.pc, jsr_pc + 3); // return to instruction after JSR
}

#[test]
fn test_bcc_taken() {
    let (mut cpu, mut mem) = setup();
    cpu.sr.set_c(false);
    mem.write(cpu.pc, 0x90); mem.write(cpu.pc + 1, 0x05);
    let pc = cpu.pc;
    execute(&mut cpu, &mut mem);
    assert_eq!(cpu.pc, pc + 2 + 5);
}

#[test]
fn test_beq_not_taken() {
    let (mut cpu, mut mem) = setup();
    cpu.sr.set_z(false);
    mem.write(cpu.pc, 0xF0); mem.write(cpu.pc + 1, 0x10);
    let pc = cpu.pc;
    execute(&mut cpu, &mut mem);
    assert_eq!(cpu.pc, pc + 2); // not taken, PC += 2
}
