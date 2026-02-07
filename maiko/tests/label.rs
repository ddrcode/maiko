//! Integration test for Label derive macro.

use maiko::Label;

#[derive(Label)]
#[allow(dead_code)]
enum TestLabel {
    Ping,
    Pong(i32, String),
    Data { x: i32, y: i32 },
}

#[test]
fn test_derived_label_unit() {
    assert_eq!(TestLabel::Ping.label(), "Ping");
}

#[test]
fn test_derived_label_tuple() {
    assert_eq!(TestLabel::Pong(1, "test".into()).label(), "Pong");
}

#[test]
fn test_derived_label_struct() {
    assert_eq!(TestLabel::Data { x: 1, y: 2 }.label(), "Data");
}
