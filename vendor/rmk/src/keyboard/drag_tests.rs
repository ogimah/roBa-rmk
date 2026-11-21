//! MB1/MB2 drag locks: use actual Vial keycode conversion, keyboard, pointing and transport queues.
use super::*;
use crate::channel::{USB_REPORT_CHANNEL, send_hid_report, try_send_hid_report};
use crate::config::{BehaviorConfig, PositionalConfig};
use crate::k;
use crate::keymap::KeymapData;
use crate::state::{self, set_usb_state};
use crate::test_support::test_block_on as block_on;
use rmk_types::{
    action::EncoderAction,
    connection::{ConnectionType, UsbState},
};
use usbd_hid::descriptor::MouseReport;

fn zero_mouse() -> MouseReport {
    MouseReport {
        buttons: 0,
        x: 0,
        y: 0,
        wheel: 0,
        pan: 0,
    }
}

fn mouse() -> std::vec::Vec<MouseReport> {
    let mut out = std::vec::Vec::new();
    while let Ok(r) = USB_REPORT_CHANNEL.try_receive() {
        if let Report::MouseReport(r) = r {
            out.push(r)
        }
    }
    out
}
fn keyboard() -> Keyboard<'static> {
    state::set_drag_button(0, false);
    state::set_drag_button(1, false);
    set_usb_state(UsbState::Configured);
    state::set_preferred_connection(ConnectionType::Usb);
    USB_REPORT_CHANNEL.clear();
    let from = crate::host::via::keycode_convert::from_via_keycode;
    let keys = [
        from(0x7e0e),
        from(0x7e0f),
        k!(MouseBtn1),
        k!(MouseBtn2),
        k!(MouseWheelUp),
        k!(MouseRight),
        from(0x7e10),
        from(0x7e11),
    ];
    assert_eq!(keys[0], KeyAction::Single(Action::User(14)));
    assert_eq!(keys[1], KeyAction::Single(Action::User(15)));
    assert_eq!(keys[6], KeyAction::Single(Action::User(16)));
    assert_eq!(keys[7], KeyAction::Single(Action::User(17)));
    let data = Box::leak(Box::new(KeymapData::new_with_encoder(
        [[keys]],
        [[EncoderAction::new(k!(MouseWheelUp), k!(MouseWheelDown))]],
    )));
    let behavior = Box::leak(Box::new(BehaviorConfig::default()));
    let positions = Box::leak(Box::new(PositionalConfig::<1, 8>::default()));
    let map = Box::leak(Box::new(block_on(KeyMap::new(data, behavior, positions))));
    Keyboard::new(map)
}
async fn tap(k: &mut Keyboard<'static>, col: u8) {
    k.process_inner(KeyboardEvent::key(0, col, true)).await;
    k.process_inner(KeyboardEvent::key(0, col, false)).await;
}

#[test]
fn drag_taps_are_idempotent_and_key_release_does_not_drop_mb1() {
    let mut k = keyboard();
    block_on(async {
        tap(&mut k, 1).await;
        assert_eq!(mouse().last().unwrap().buttons, 0);
        tap(&mut k, 0).await;
        assert_eq!(mouse().last().unwrap().buttons, 1);
        tap(&mut k, 0).await;
        assert_eq!(mouse().last().unwrap().buttons, 1);
        tap(&mut k, 1).await;
        assert_eq!(mouse().last().unwrap().buttons, 0);
        tap(&mut k, 1).await;
        assert_eq!(mouse().last().unwrap().buttons, 0);
    });
}

#[test]
fn drag_and_normal_buttons_have_independent_ownership() {
    let mut k = keyboard();
    block_on(async {
        tap(&mut k, 0).await;
        mouse();
        tap(&mut k, 2).await;
        assert!(mouse().iter().all(|r| r.buttons == 1));
        k.process_inner(KeyboardEvent::key(0, 3, true)).await;
        assert_eq!(mouse().last().unwrap().buttons, 3);
        tap(&mut k, 1).await;
        assert_eq!(mouse().last().unwrap().buttons, 2);
        k.process_inner(KeyboardEvent::key(0, 3, false)).await;
        assert_eq!(mouse().last().unwrap().buttons, 0);
        k.process_inner(KeyboardEvent::key(0, 2, true)).await;
        mouse();
        tap(&mut k, 0).await;
        mouse();
        tap(&mut k, 1).await;
        assert_eq!(
            mouse().last().unwrap().buttons,
            1,
            "DD_OFF must preserve physically held MB1"
        );
        k.process_inner(KeyboardEvent::key(0, 2, false)).await;
        assert_eq!(mouse().last().unwrap().buttons, 0);
    });
}

#[test]
fn drag_survives_actual_pointing_scroll_and_encoder_wheel_reports() {
    use crate::event::{Axis, AxisEvent, AxisValType, PointingEvent};
    use crate::input_device::pointing::{
        PointingMode, PointingProcessor, PointingProcessorConfig, PointingProcessorProcessorEventEnum, ScrollConfig,
    };
    use crate::input_device::rotary_encoder::Direction;
    use crate::processor::Processor;
    let mut k = keyboard();
    block_on(async {
        tap(&mut k, 0).await;
        mouse();
        let mut pointer = PointingProcessor::new(k.keymap, PointingProcessorConfig::default());
        let event = || {
            PointingProcessorProcessorEventEnum::Pointing(PointingEvent {
                device_id: 0,
                axes: [
                    AxisEvent {
                        typ: AxisValType::Rel,
                        axis: Axis::X,
                        value: 24,
                    },
                    AxisEvent {
                        typ: AxisValType::Rel,
                        axis: Axis::Y,
                        value: 24,
                    },
                    AxisEvent {
                        typ: AxisValType::Rel,
                        axis: Axis::Z,
                        value: 0,
                    },
                ],
            })
        };
        pointer.process(event()).await;
        let r = mouse().pop().unwrap();
        assert_eq!(r.buttons, 1);
        assert_ne!(r.x, 0);
        pointer.set_pointing_mode(PointingMode::Scroll(ScrollConfig::default()));
        pointer.process(event()).await;
        let r = mouse().pop().unwrap();
        assert_eq!(r.buttons, 1);
        assert_ne!(r.wheel, 0);
        k.process_inner(KeyboardEvent::rotary_encoder(0, Direction::Clockwise, true))
            .await;
        let r = mouse().pop().unwrap();
        assert_eq!(r.buttons, 1);
        assert_ne!(r.wheel, 0);
        k.process_inner(KeyboardEvent::rotary_encoder(0, Direction::Clockwise, false))
            .await;
        assert!(mouse().is_empty(), "encoder release must not emit or clear the drag");
        tap(&mut k, 1).await;
        assert_eq!(mouse().last().unwrap().buttons, 0);
        pointer.process(event()).await;
        assert_eq!(mouse().last().unwrap().buttons, 0);
    });
}

#[test]
fn drag_disconnect_and_reconnect_do_not_rearm_latch() {
    let mut k = keyboard();
    block_on(async {
        tap(&mut k, 0).await;
        mouse();
        set_usb_state(UsbState::Disabled);
        assert_eq!(state::drag_buttons(0), 0);
        assert!(matches!(
            USB_REPORT_CHANNEL.try_receive(),
            Ok(Report::KeyboardReport(_))
        ));
        assert_eq!(mouse().last().unwrap().buttons, 0);
        tap(&mut k, 0).await;
        assert_eq!(state::drag_buttons(0), 0, "offline ON is ignored");
        set_usb_state(UsbState::Configured);
        mouse();
        send_hid_report(Report::MouseReport(MouseReport { x: 12, ..zero_mouse() })).await;
        assert_eq!(mouse().last().unwrap().buttons, 0);
    });
}

#[test]
fn blocked_mouse_send_resamples_drag_after_off() {
    use core::future::Future;
    use core::{
        pin::pin,
        task::{Context, Poll, Waker},
    };
    let mut k = keyboard();
    block_on(tap(&mut k, 0));
    block_on(tap(&mut k, 6));
    mouse();
    for _ in 0..crate::REPORT_CHANNEL_SIZE {
        USB_REPORT_CHANNEL.try_send(Report::MouseReport(zero_mouse())).unwrap();
    }
    let mut f = pin!(send_hid_report(Report::MouseReport(MouseReport {
        x: 7,
        ..zero_mouse()
    })));
    let mut cx = Context::from_waker(Waker::noop());
    assert!(matches!(f.as_mut().poll(&mut cx), Poll::Pending));
    state::set_drag_button(0, false);
    USB_REPORT_CHANNEL.clear();
    assert!(matches!(f.as_mut().poll(&mut cx), Poll::Ready(())));
    let r = mouse().pop().unwrap();
    assert_eq!(r.buttons, 2, "waiting report must keep only the still-active right latch");
    assert_eq!(r.x, 7);
}

#[test]
fn nonblocking_mouse_reports_also_keep_drag_and_do_not_touch_keyboard_reports() {
    let mut k = keyboard();
    block_on(tap(&mut k, 0));
    mouse();
    try_send_hid_report(Report::MouseReport(MouseReport {
        buttons: 2,
        x: 3,
        ..zero_mouse()
    }));
    let r = mouse().pop().unwrap();
    assert_eq!(r.buttons, 3);
    assert_eq!(r.x, 3);
    try_send_hid_report(Report::KeyboardReport(KeyboardReport::default()));
    assert!(matches!(USB_REPORT_CHANNEL.try_receive(),Ok(Report::KeyboardReport(r)) if r.modifier==0));
}

#[cfg(feature = "_ble")]
#[test]
fn drag_usb_ble_and_profile_switches_clear_both_latch_and_mouse_output() {
    use crate::channel::BLE_REPORT_CHANNEL;
    use rmk_types::ble::BleState;
    let mut k = keyboard();
    block_on(async {
        state::set_ble_state(BleState::Connected);
        BLE_REPORT_CHANNEL.clear();
        tap(&mut k, 0).await;
        mouse();
        state::set_preferred_connection(ConnectionType::Ble);
        assert_eq!(state::drag_buttons(0), 0);
        assert_eq!(mouse().last().unwrap().buttons, 0);
        tap(&mut k, 0).await;
        assert!(matches!(BLE_REPORT_CHANNEL.try_receive(),Ok(Report::MouseReport(r)) if r.buttons==1));
        state::set_ble_profile(1);
        assert_eq!(state::drag_buttons(0), 0);
        assert!(matches!(
            BLE_REPORT_CHANNEL.try_receive(),
            Ok(Report::KeyboardReport(_))
        ));
        assert!(matches!(BLE_REPORT_CHANNEL.try_receive(),Ok(Report::MouseReport(r)) if r.buttons==0));
        state::set_ble_state(BleState::Connected);
        tap(&mut k, 0).await;
        assert!(matches!(BLE_REPORT_CHANNEL.try_receive(),Ok(Report::MouseReport(r)) if r.buttons==1));
        state::set_preferred_connection(ConnectionType::Usb);
        assert_eq!(state::drag_buttons(0), 0);
        assert!(matches!(
            BLE_REPORT_CHANNEL.try_receive(),
            Ok(Report::KeyboardReport(_))
        ));
        assert!(matches!(BLE_REPORT_CHANNEL.try_receive(),Ok(Report::MouseReport(r)) if r.buttons==0));
        mouse();
        send_hid_report(Report::MouseReport(MouseReport { x: 1, ..zero_mouse() })).await;
        assert_eq!(mouse().last().unwrap().buttons, 0);
    });
}

#[test]
fn left_and_right_latches_are_independent_in_both_orders() {
    let mut k = keyboard();
    block_on(async {
        for (col, expected) in [
            (0, 1),
            (6, 3),
            (1, 2),
            (7, 0),
            (6, 2),
            (6, 2),
            (0, 3),
            (7, 1),
            (1, 0),
            (7, 0),
        ] {
            tap(&mut k, col).await;
            assert_eq!(mouse().last().unwrap().buttons, expected, "command at column {col}");
        }
    });
}

#[test]
fn right_drag_and_normal_left_right_buttons_coexist() {
    let mut k = keyboard();
    block_on(async {
        tap(&mut k, 6).await;
        assert_eq!(mouse().last().unwrap().buttons, 2);
        tap(&mut k, 3).await;
        assert!(mouse().iter().all(|r| r.buttons == 2));
        k.process_inner(KeyboardEvent::key(0, 2, true)).await;
        assert_eq!(mouse().last().unwrap().buttons, 3);
        k.process_inner(KeyboardEvent::key(0, 2, false)).await;
        assert_eq!(mouse().last().unwrap().buttons, 2);
        k.process_inner(KeyboardEvent::key(0, 3, true)).await;
        mouse();
        tap(&mut k, 7).await;
        assert_eq!(
            mouse().last().unwrap().buttons,
            2,
            "DD2- preserves physically held right button"
        );
        k.process_inner(KeyboardEvent::key(0, 3, false)).await;
        assert_eq!(mouse().last().unwrap().buttons, 0);
    });
}

#[test]
fn both_latches_survive_motion_scroll_and_encoder_until_individual_release() {
    use crate::event::{Axis, AxisEvent, AxisValType, PointingEvent};
    use crate::input_device::pointing::{
        PointingMode, PointingProcessor, PointingProcessorConfig, PointingProcessorProcessorEventEnum, ScrollConfig,
    };
    use crate::input_device::rotary_encoder::Direction;
    use crate::processor::Processor;
    let mut k = keyboard();
    block_on(async {
        tap(&mut k, 0).await;
        tap(&mut k, 6).await;
        mouse();
        let mut pointer = PointingProcessor::new(k.keymap, PointingProcessorConfig::default());
        let event = || {
            PointingProcessorProcessorEventEnum::Pointing(PointingEvent {
                device_id: 0,
                axes: [
                    AxisEvent {
                        typ: AxisValType::Rel,
                        axis: Axis::X,
                        value: 24,
                    },
                    AxisEvent {
                        typ: AxisValType::Rel,
                        axis: Axis::Y,
                        value: 24,
                    },
                    AxisEvent {
                        typ: AxisValType::Rel,
                        axis: Axis::Z,
                        value: 0,
                    },
                ],
            })
        };
        pointer.process(event()).await;
        assert_eq!(mouse().last().unwrap().buttons, 3);
        pointer.set_pointing_mode(PointingMode::Scroll(ScrollConfig::default()));
        pointer.process(event()).await;
        assert_eq!(mouse().last().unwrap().buttons, 3);
        k.process_inner(KeyboardEvent::rotary_encoder(0, Direction::Clockwise, true))
            .await;
        assert_eq!(mouse().last().unwrap().buttons, 3);
        tap(&mut k, 1).await;
        assert_eq!(mouse().last().unwrap().buttons, 2);
        pointer.process(event()).await;
        assert_eq!(mouse().last().unwrap().buttons, 2);
        tap(&mut k, 7).await;
        assert_eq!(mouse().last().unwrap().buttons, 0);
    });
}

#[cfg(feature = "_ble")]
#[test]
fn connection_switch_clears_both_latches() {
    use crate::channel::BLE_REPORT_CHANNEL;
    use rmk_types::ble::BleState;
    let mut k = keyboard();
    block_on(async {
        state::set_ble_state(BleState::Connected);
        BLE_REPORT_CHANNEL.clear();
        tap(&mut k, 0).await;
        tap(&mut k, 6).await;
        mouse();
        assert_eq!(state::drag_buttons(0), 3);
        state::set_preferred_connection(ConnectionType::Ble);
        assert_eq!(state::drag_buttons(0), 0);
        assert_eq!(mouse().last().unwrap().buttons, 0);
        send_hid_report(Report::MouseReport(zero_mouse())).await;
        assert!(matches!(BLE_REPORT_CHANNEL.try_receive(),Ok(Report::MouseReport(r)) if r.buttons==0));
        tap(&mut k, 0).await;
        tap(&mut k, 6).await;
        while BLE_REPORT_CHANNEL.try_receive().is_ok() {}
        assert_eq!(state::drag_buttons(0), 3);
        state::set_ble_profile(1);
        assert_eq!(state::drag_buttons(0), 0);
        assert!(matches!(
            BLE_REPORT_CHANNEL.try_receive(),
            Ok(Report::KeyboardReport(_))
        ));
        assert!(matches!(BLE_REPORT_CHANNEL.try_receive(),Ok(Report::MouseReport(r)) if r.buttons==0));
    });
}
