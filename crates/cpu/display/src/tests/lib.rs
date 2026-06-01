use super::*;

#[test]
fn test_new_default() {
    let d = Display::new(64, 32);
    assert_eq!(d.width(), 64);
    assert_eq!(d.height(), 32);
    assert_eq!(d.buffer_len(), 2048);
}

#[test]
fn test_clear() {
    let mut d = Display::new(64, 32);
    d.set_pixel(0, 0, 1);
    d.clear();
    assert_eq!(d.get_pixel(0, 0), 0);
}

#[test]
fn test_set_get_pixel() {
    let mut d = Display::new(64, 32);
    d.set_pixel(10, 5, 1);
    assert_eq!(d.get_pixel(10, 5), 1);
    assert_eq!(d.get_pixel(11, 5), 0);
}

#[test]
fn test_out_of_bounds() {
    let mut d = Display::new(64, 32);
    d.set_pixel(100, 100, 1);
    assert_eq!(d.get_pixel(100, 100), 0);
}

#[test]
fn test_get_buffer() {
    let d = Display::new(64, 32);
    assert_eq!(d.get_buffer().len(), 2048);
}
