use super::*;

#[test]
fn test_default_nmos() {
    let c = MachineConfig::default();
    assert_eq!(c.family, CpuFamily::Nmos6502);
    assert!(c.quirks.jmp_indirect_bug);
}

#[test]
fn test_cmos_jmp_fixed() {
    let c = MachineConfig::w65c02();
    assert!(!c.quirks.jmp_indirect_bug);
}

#[test]
fn test_nes_no_bcd() {
    let c = MachineConfig::ricoh2a03();
    assert!(!c.quirks.bcd_available);
}

#[test]
fn test_atari2600_8kb() {
    let c = MachineConfig::atari2600();
    assert_eq!(c.memory.size, 8192);
    assert_eq!(c.start_address, 0xF000);
}

#[test]
fn test_cmos_has_stp_wai() {
    let c = MachineConfig::w65c02();
    assert!(c.quirks.stp_available);
    assert!(c.quirks.wai_available);
}

#[test]
fn test_ricoh_no_stp() {
    let c = MachineConfig::ricoh2a03();
    assert!(!c.quirks.stp_available);
}

#[test]
fn test_json_roundtrip() {
    let c = MachineConfig::ricoh2a03();
    let json = c.to_json();
    let c2 = MachineConfig::from_json(&json).unwrap();
    assert_eq!(c.family, c2.family);
    assert_eq!(c.quirks.bcd_available, c2.quirks.bcd_available);
}

#[test]
fn test_all_variants_distinct() {
    let v: Vec<MachineConfig> = vec![
        MachineConfig::nmos6502(),
        MachineConfig::w65c02(),
        MachineConfig::r65c02(),
        MachineConfig::ricoh2a03(),
        MachineConfig::mos6510(),
        MachineConfig::mos6507(),
    ];
    let mut families: Vec<_> = v.iter().map(|c| c.family).collect();
    families.sort();
    families.dedup();
    assert_eq!(families.len(), 6, "niektóre varianty się powtarzają");
}

#[test]
fn test_r65c02_quirks() {
    let c = MachineConfig::r65c02();
    assert!(!c.quirks.jmp_indirect_bug);
    assert!(c.quirks.bcd_available);
    assert!(c.quirks.stp_available);
    assert!(c.quirks.wai_available);
    assert_eq!(c.quirks.rmw, RmwBehavior::Cmos);
}

#[test]
fn test_apple1_preset() {
    let c = MachineConfig::apple1();
    assert!(c.is_apple1);
    assert_eq!(c.family, CpuFamily::Nmos6502);
}

#[test]
fn test_apple2_preset_not_apple1() {
    let c = MachineConfig::apple2();
    assert!(!c.is_apple1);
}

#[test]
fn test_nmos_presets_exist() {
    let _ = MachineConfig::mos6510();
    let _ = MachineConfig::mos6507();
}

#[test]
fn test_cpu_quirks_presets() {
    let _ = CpuQuirks::nmos6510();
    let _ = CpuQuirks::nmos8502();
    let _ = CpuQuirks::nmos6507();
}

#[test]
fn test_non_exhaustive_allows_new_variants() {
    // Compile check: match without wildcard would fail
    match CpuFamily::Nmos6502 {
        CpuFamily::Nmos6502 => {}
        _ => {}
    }
}
