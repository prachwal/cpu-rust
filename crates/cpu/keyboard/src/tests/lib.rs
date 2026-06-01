use super::*;

#[test]
fn test_press_release() {
    let mut kb = Keyboard::new();
    assert!(!kb.is_pressed(0xA));
    kb.press(0xA);
    assert!(kb.is_pressed(0xA));
    kb.release(0xA);
    assert!(!kb.is_pressed(0xA));
}

#[test]
fn test_any_pressed() {
    let mut kb = Keyboard::new();
    assert_eq!(kb.any_pressed(), None);
    kb.press(0x5);
    assert_eq!(kb.any_pressed(), Some(0x5));
}

#[test]
fn test_out_of_range_ignored() {
    let mut kb = Keyboard::new();
    kb.press(0xFF);
    assert!(!kb.is_pressed(0xFF));
}
