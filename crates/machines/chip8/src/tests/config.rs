use super::*;

#[test]
fn test_default_json_roundtrip() {
    let c = MachineConfig::default();
    let json = c.to_json();
    let c2 = MachineConfig::from_json(&json).unwrap();
    assert_eq!(c.machine, c2.machine);
    assert_eq!(c.cpu.quirks.shift_vy, c2.cpu.quirks.shift_vy);
}

#[test]
fn test_schip_config() {
    let c = MachineConfig::schip();
    assert_eq!(c.display.width, 128);
    assert_eq!(c.cpu.r#type, "schip");
}

#[test]
fn test_xochip_config() {
    let c = MachineConfig::xochip();
    assert_eq!(c.cpu.quirks.shift_vy, true);
    assert_eq!(c.cpu.quirks.memory_inc_i, true);
    assert_eq!(c.cpu.quirks.vf_reset, false);
}

#[test]
fn test_display_area() {
    let c = MachineConfig::default();
    assert_eq!(c.display_area(), 2048);
    let c2 = MachineConfig::schip();
    assert_eq!(c2.display_area(), 8192);
}

#[test]
fn test_quirks_tuple() {
    let c = MachineConfig::default();
    let (a, b, c_val) = c.quirks();
    assert_eq!(a, false);
    assert_eq!(b, false);
    assert_eq!(c_val, true);
}

#[test]
fn test_invalid_json() {
    let r = MachineConfig::from_json("not json");
    assert!(r.is_err());
}
