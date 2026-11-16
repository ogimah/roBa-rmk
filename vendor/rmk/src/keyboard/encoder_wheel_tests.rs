//! Exercise real Keyboard wheel dispatch, including a missing encoder release.
use super::*;
use crate::{
    channel::USB_REPORT_CHANNEL,
    config::{BehaviorConfig, PositionalConfig},
    input_device::rotary_encoder::Direction,
    keymap::KeymapData,
};
use core::{
    future::Future,
    pin::pin,
    task::{Context, Poll, Waker},
};
use rmk_types::{
    action::{EncoderAction, KeyAction},
    connection::UsbState,
};

fn finish<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let mut cx = Context::from_waker(Waker::noop());
    for _ in 0..1000 {
        if let Poll::Ready(value) = future.as_mut().poll(&mut cx) {
            return value;
        }
        embassy_time::MockDriver::get().advance(Duration::from_millis(1));
    }
    panic!("wheel dispatch stalled");
}
fn take_mouse() -> std::vec::Vec<usbd_hid::descriptor::MouseReport> {
    let mut out = std::vec::Vec::new();
    while let Ok(report) = USB_REPORT_CHANNEL.try_receive() {
        if let Report::MouseReport(report) = report {
            out.push(report);
        }
    }
    out
}
fn keyboard() -> Keyboard<'static> {
    embassy_time::MockDriver::get().reset();
    crate::state::set_usb_state(UsbState::Configured);
    take_mouse();
    let wheel = |code| KeyAction::Single(Action::Key(KeyCode::Hid(code)));
    let data = std::boxed::Box::leak(std::boxed::Box::new(KeymapData::new_with_encoder(
        [[[
            wheel(HidKeyCode::MouseWheelUp),
            wheel(HidKeyCode::MouseBtn1),
        ]]],
        [[EncoderAction {
            clockwise: wheel(HidKeyCode::MouseWheelUp),
            counter_clockwise: wheel(HidKeyCode::MouseWheelDown),
        }]],
    )));
    let behavior = std::boxed::Box::leak(std::boxed::Box::new(BehaviorConfig::default()));
    let positional =
        std::boxed::Box::leak(std::boxed::Box::new(PositionalConfig::<1, 2>::default()));
    let map = std::boxed::Box::leak(std::boxed::Box::new(finish(KeyMap::new(
        data, behavior, positional,
    ))));
    Keyboard::new(map)
}

#[test]
fn encoder_wheel_does_not_repeat_when_release_is_missing() {
    let mut keyboard = keyboard();
    finish(keyboard.process_inner(KeyboardEvent::rotary_encoder(0, Direction::Clockwise, true)));
    let reports = take_mouse();
    assert_eq!(reports.len(), 1);
    assert_ne!(reports[0].wheel, 0);
    embassy_time::MockDriver::get().advance(Duration::from_secs(5));
    finish(keyboard.fire_mouse_repeat());
    assert!(
        take_mouse().is_empty(),
        "Encoder rotation must not leave an indefinitely held wheel key"
    );
}

#[test]
fn ordinary_wheel_key_still_repeats_until_release() {
    let mut keyboard = keyboard();
    finish(keyboard.process_inner(KeyboardEvent::key(0, 0, true)));
    take_mouse();
    embassy_time::MockDriver::get().advance(Duration::from_secs(5));
    finish(keyboard.fire_mouse_repeat());
    assert!(take_mouse().iter().any(|r| r.wheel != 0));
    finish(keyboard.process_inner(KeyboardEvent::key(0, 0, false)));
    take_mouse();
    embassy_time::MockDriver::get().advance(Duration::from_secs(5));
    finish(keyboard.fire_mouse_repeat());
    assert!(take_mouse().is_empty());
}

#[test]
fn fast_alternating_encoder_steps_and_late_releases_do_not_latch() {
    let mut keyboard = keyboard();
    for i in 0..64 {
        let direction = if i % 2 == 0 {
            Direction::Clockwise
        } else {
            Direction::CounterClockwise
        };
        finish(keyboard.process_inner(KeyboardEvent::rotary_encoder(0, direction, true)));
        let reports = take_mouse();
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].wheel.signum(), if i % 2 == 0 { -1 } else { 1 });
        finish(keyboard.process_inner(KeyboardEvent::rotary_encoder(0, direction, false)));
        assert!(
            take_mouse().is_empty(),
            "Encoder release must not add another wheel report"
        );
    }
    // A duplicate/late release must not create a negative held-key count.
    finish(keyboard.process_inner(KeyboardEvent::rotary_encoder(
        0,
        Direction::Clockwise,
        false,
    )));
    take_mouse();
    embassy_time::MockDriver::get().advance(Duration::from_secs(5));
    finish(keyboard.fire_mouse_repeat());
    assert!(take_mouse().is_empty());
}

#[test]
fn encoder_wheel_preserves_a_held_mouse_button_and_physical_wheel() {
    let mut keyboard = keyboard();
    finish(keyboard.process_inner(KeyboardEvent::key(0, 1, true)));
    take_mouse();
    finish(keyboard.process_inner(KeyboardEvent::key(0, 0, true)));
    take_mouse();
    finish(keyboard.process_inner(KeyboardEvent::rotary_encoder(
        0,
        Direction::CounterClockwise,
        true,
    )));
    let reports = take_mouse();
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].buttons & 1, 1);
    assert_eq!((reports[0].x, reports[0].y), (0, 0));
    assert!(reports[0].wheel > 0);
    // The physical wheel key's independent repeat remains active.
    embassy_time::MockDriver::get().advance(Duration::from_secs(5));
    finish(keyboard.fire_mouse_repeat());
    assert!(
        take_mouse()
            .iter()
            .any(|r| r.wheel < 0 && r.buttons & 1 == 1)
    );
    finish(keyboard.process_inner(KeyboardEvent::key(0, 0, false)));
    finish(keyboard.process_inner(KeyboardEvent::key(0, 1, false)));
    take_mouse();
    embassy_time::MockDriver::get().advance(Duration::from_secs(5));
    finish(keyboard.fire_mouse_repeat());
    assert!(take_mouse().is_empty());
}
