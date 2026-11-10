use super::*;
use rmk::{
    channel::USB_REPORT_CHANNEL,
    config::{BehaviorConfig, PositionalConfig},
    core_traits::Runnable,
    event::{AxisEvent, AxisValType},
    hid::Report,
    host::KeyboardContext,
    keyboard::Keyboard,
    keymap::KeymapData,
    types::{
        action::{Action, KeyAction},
        connection::UsbState,
        keycode::{HidKeyCode, KeyCode},
        modifier::ModifierCombination,
    },
};
use std::{
    future::Future,
    pin::pin,
    task::{Context, Poll, Waker},
};

fn finish<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let mut cx = Context::from_waker(Waker::noop());
    for _ in 0..2000 {
        if let Poll::Ready(value) = future.as_mut().poll(&mut cx) {
            return value;
        }
        embassy_time::MockDriver::get().advance(embassy_time::Duration::from_millis(1));
    }
    panic!("future stalled, possibly channel backpressure or a lost release");
}
fn pump(keyboard: &mut Keyboard<'_>) {
    let mut future = pin!(keyboard.run());
    let mut cx = Context::from_waker(Waker::noop());
    for _ in 0..128 {
        assert!(future.as_mut().poll(&mut cx).is_pending());
        embassy_time::MockDriver::get().advance(embassy_time::Duration::from_millis(1));
    }
}
fn motion(x: i16, y: i16) -> PointingEvent {
    PointingEvent {
        device_id: 0,
        axes: [
            AxisEvent {
                axis: Axis::X,
                typ: AxisValType::Rel,
                value: x,
            },
            AxisEvent {
                axis: Axis::Y,
                typ: AxisValType::Rel,
                value: y,
            },
            AxisEvent {
                axis: Axis::Z,
                typ: AxisValType::Rel,
                value: 0,
            },
        ],
    }
}
fn reports() -> (Vec<(u8, [u8; 6])>, Vec<(i8, i8)>) {
    let (mut keys, mut mouse) = (vec![], vec![]);
    while let Ok(report) = USB_REPORT_CHANNEL.try_receive() {
        match report {
            Report::KeyboardReport(r) => keys.push((r.modifier, r.keycodes)),
            Report::MouseReport(r) => mouse.push((r.x, r.y)),
            _ => {}
        }
    }
    (keys, mouse)
}
fn key(k: HidKeyCode) -> KeyAction {
    KeyAction::Single(Action::Key(KeyCode::Hid(k)))
}

#[test]
fn real_rmk_route_vial_binding_modifiers_macros_and_release() {
    embassy_time::MockDriver::get().reset();
    rmk::state::set_usb_state(UsbState::Configured);
    let mut keys = [[[KeyAction::No; 11]; 5]; 8];
    keys[0][0][0] = KeyAction::Single(Action::LayerOn(3));
    keys[3][2][3] = KeyAction::Single(Action::User(TRIGGER_USER_ID));
    keys[0][2][3] = key(HidKeyCode::V); // outside layer 3 V is still an ordinary key
    for (col, k) in [HidKeyCode::A, HidKeyCode::B, HidKeyCode::C, HidKeyCode::D]
        .into_iter()
        .enumerate()
    {
        keys[3][4][col] = key(k);
    }
    let mut data = KeymapData::new(keys);
    let mut behavior = BehaviorConfig::default();
    let positional = PositionalConfig::default();
    let map = finish(KeyMap::new(&mut data, &mut behavior, &positional));
    let mut keyboard = Keyboard::new(&map);
    let config = PointingProcessorConfig {
        device_id: 0,
        ..Default::default()
    };
    let inner = PointingProcessor::new(
        &map,
        PointingProcessorConfig {
            device_id: 0,
            ..Default::default()
        },
    );
    let trigger = Cell::new(Trigger::default());
    let mut monitor = GestureTriggerMonitor::new(&trigger);
    let mut router = GesturePointingProcessor::new(inner, config, &map, &trigger);
    finish(router.on_pointing_event(motion(5, -6)));
    assert_eq!(reports().1, vec![(5, -6)]);
    finish(keyboard.process_inner(KeyboardEvent::key(0, 0, true)));
    assert_eq!(map.active_layer(), 3);
    reports();
    finish(router.on_pointing_event(motion(7, 8)));
    assert_eq!(
        reports().1,
        vec![(7, 8)],
        "layer 3 alone must be cursor, not scroll"
    );
    finish(monitor.on_keyboard_event(KeyboardEvent::key(2, 3, true)));
    finish(monitor.on_action_event(ActionEvent {
        action: Action::User(TRIGGER_USER_ID),
        keyboard_event: KeyboardEvent::key(2, 3, true),
    }));
    finish(keyboard.process_inner(KeyboardEvent::key(2, 3, true)));
    assert!(reports().0.is_empty(), "trigger V must not type");
    finish(router.on_pointing_event(motion(10, 0)));
    finish(router.on_pointing_event(motion(20, 0)));
    assert_eq!(
        reports(),
        (vec![], vec![]),
        "captured subthreshold motion must not leak"
    );
    // Reset that unfinished stroke, then test every direction through the actual
    // router, pubsub channel, Keyboard, keymap lookup and HID report generation.
    finish(monitor.on_keyboard_event(KeyboardEvent::key(2, 3, false)));
    finish(monitor.on_keyboard_event(KeyboardEvent::key(2, 3, true)));
    finish(monitor.on_action_event(ActionEvent {
        action: Action::User(TRIGGER_USER_ID),
        keyboard_event: KeyboardEvent::key(2, 3, true),
    }));
    for (x, y, k) in [
        (-31, 0, HidKeyCode::A),
        (31, 0, HidKeyCode::B),
        (0, -31, HidKeyCode::C),
        (0, 31, HidKeyCode::D),
    ] {
        embassy_time::MockDriver::get().advance(embassy_time::Duration::from_millis(150));
        finish(router.on_pointing_event(motion(x, y)));
        pump(&mut keyboard);
        let (r, m) = reports();
        assert!(m.is_empty());
        assert_eq!(
            r.iter()
                .filter(|(_, keys)| keys.contains(&(k as u8)))
                .count(),
            1
        );
        assert_eq!(
            r.last(),
            Some(&(0, [0; 6])),
            "paired release must clear the report"
        );
    }
    let ctx = KeyboardContext::new(&map);
    finish(ctx.set_action(3, 4, 1, key(HidKeyCode::E)));
    embassy_time::MockDriver::get().advance(embassy_time::Duration::from_millis(150));
    finish(router.on_pointing_event(motion(31, 0)));
    pump(&mut keyboard);
    let (r, m) = reports();
    assert!(m.is_empty());
    assert!(
        r.iter()
            .any(|(_, keys)| keys.contains(&(HidKeyCode::E as u8)))
    );
    assert!(
        !r.iter()
            .any(|(_, keys)| keys.contains(&(HidKeyCode::B as u8))),
        "live host remap must take effect"
    );
    finish(ctx.set_action(
        3,
        4,
        1,
        KeyAction::Single(Action::KeyWithModifier(
            KeyCode::Hid(HidKeyCode::Tab),
            ModifierCombination::new().with_left_ctrl(true),
        )),
    ));
    embassy_time::MockDriver::get().advance(embassy_time::Duration::from_millis(150));
    finish(router.on_pointing_event(motion(31, 0)));
    pump(&mut keyboard);
    let (r, _) = reports();
    assert!(
        r.iter()
            .any(|(mods, keys)| mods & 1 != 0 && keys.contains(&(HidKeyCode::Tab as u8)))
    );
    assert_eq!(r.last(), Some(&(0, [0; 6])));
    finish(ctx.write_macro_buffer(0, &[1, 1, HidKeyCode::X as u8, 0]));
    finish(ctx.set_action(3, 4, 1, KeyAction::Single(Action::TriggerMacro(0))));
    embassy_time::MockDriver::get().advance(embassy_time::Duration::from_millis(150));
    finish(router.on_pointing_event(motion(31, 0)));
    pump(&mut keyboard);
    let (r, _) = reports();
    assert!(
        r.iter()
            .any(|(_, keys)| keys.contains(&(HidKeyCode::X as u8)))
    );
    assert_eq!(r.last(), Some(&(0, [0; 6])));
    // A layer change between press/release must not leave the old output held.
    finish(keyboard.process_inner(KeyboardEvent::key(4, 0, true)));
    finish(keyboard.process_inner(KeyboardEvent::key(0, 0, false)));
    assert_eq!(map.active_layer(), 0);
    finish(keyboard.process_inner(KeyboardEvent::key(4, 0, false)));
    assert_eq!(reports().0.last(), Some(&(0, [0; 6])));
    finish(monitor.on_keyboard_event(KeyboardEvent::key(2, 3, false)));
    finish(router.on_pointing_event(motion(4, 5)));
    assert_eq!(reports().1, vec![(4, 5)]);
    finish(keyboard.process_inner(KeyboardEvent::key(2, 3, true)));
    finish(keyboard.process_inner(KeyboardEvent::key(2, 3, false)));
    let (r, _) = reports();
    assert!(
        r.iter()
            .any(|(_, keys)| keys.contains(&(HidKeyCode::V as u8)))
    );
    assert_eq!(r.last(), Some(&(0, [0; 6])));
}

#[test]
fn concurrent_real_tasks_drain_keyboard_backpressure_and_do_not_leak_motion() {
    use rmk::event::publish_event_async;
    embassy_time::MockDriver::get().reset();
    rmk::state::set_usb_state(UsbState::Configured);
    reports();
    let mut keys = [[[KeyAction::No; 11]; 5]; 8];
    keys[0][0][0] = KeyAction::Single(Action::LayerOn(3));
    keys[3][2][3] = KeyAction::Single(Action::User(TRIGGER_USER_ID));
    keys[3][4][1] = key(HidKeyCode::A);
    let mut data = KeymapData::new(keys);
    let mut behavior = BehaviorConfig::default();
    let positional = PositionalConfig::default();
    let map = finish(KeyMap::new(&mut data, &mut behavior, &positional));
    let mut keyboard = Keyboard::new(&map);
    let trigger = Cell::new(Trigger::default());
    let mut monitor = GestureTriggerMonitor::new(&trigger);
    let inner = PointingProcessor::new(
        &map,
        PointingProcessorConfig {
            device_id: 0,
            ..Default::default()
        },
    );
    let mut router = GesturePointingProcessor::new(
        inner,
        PointingProcessorConfig {
            device_id: 0,
            ..Default::default()
        },
        &map,
        &trigger,
    );
    let (mut k, mut m, mut r) = (
        pin!(keyboard.run()),
        pin!(monitor.run()),
        pin!(router.run()),
    );
    let mut cx = Context::from_waker(Waker::noop());
    // Establish subscriptions and then poll in the same keyboard/monitor/router
    // order used by the firmware's join tree.
    let mut tick = || {
        assert!(k.as_mut().poll(&mut cx).is_pending());
        assert!(m.as_mut().poll(&mut cx).is_pending());
        assert!(r.as_mut().poll(&mut cx).is_pending());
        embassy_time::MockDriver::get().advance(embassy_time::Duration::from_millis(1));
    };
    tick();
    finish(publish_event_async(KeyboardEvent::key(0, 0, true)));
    finish(publish_event_async(KeyboardEvent::key(2, 3, true)));
    for _ in 0..10 {
        tick();
    }
    assert_eq!(map.active_layer(), 3);
    assert!(trigger.get().pressed);
    reports();
    // At least one subscriber's queue is full before the pointing task tries
    // to publish its synthetic events. The independent monitor must drain it.
    for i in 0..16 {
        finish(publish_event_async(KeyboardEvent::key(1, 1, i % 2 == 0)));
    }
    finish(publish_event_async(motion(31, 0)));
    for _ in 0..100 {
        tick();
    }
    let (keys, mouse) = reports();
    assert!(mouse.is_empty());
    assert_eq!(
        keys.iter()
            .filter(|(_, k)| k.contains(&(HidKeyCode::A as u8)))
            .count(),
        1
    );
    assert_eq!(keys.last(), Some(&(0, [0; 6])));
    finish(publish_event_async(KeyboardEvent::key(2, 3, false)));
    for _ in 0..5 {
        tick();
    }
    finish(publish_event_async(motion(5, -6)));
    for _ in 0..5 {
        tick();
    }
    assert_eq!(reports().1, vec![(5, -6)]);
}

#[test]
fn live_trigger_remap_transparency_multiple_holds_and_release_after_edit() {
    embassy_time::MockDriver::get().reset();
    rmk::state::set_usb_state(UsbState::Configured);
    reports();
    let mut keys = [[[KeyAction::No; 11]; 5]; 8];
    keys[0][0][0] = KeyAction::Single(Action::LayerOn(2));
    keys[0][1][1] = KeyAction::Single(Action::User(TRIGGER_USER_ID));
    keys[2][1][1] = KeyAction::Transparent;
    keys[2][4][1] = key(HidKeyCode::Z);
    keys[0][2][3] = key(HidKeyCode::V);
    let mut data = KeymapData::new(keys);
    let mut behavior = BehaviorConfig::default();
    let positional = PositionalConfig::default();
    let map = finish(KeyMap::new(&mut data, &mut behavior, &positional));
    let ctx = KeyboardContext::new(&map);
    let mut keyboard = Keyboard::new(&map);
    let trigger = Cell::new(Trigger::default());
    let mut monitor = GestureTriggerMonitor::new(&trigger);
    let cfg = || PointingProcessorConfig {
        device_id: 0,
        ..Default::default()
    };
    let mut router =
        GesturePointingProcessor::new(PointingProcessor::new(&map, cfg()), cfg(), &map, &trigger);
    let (mut k, mut m, mut r) = (
        pin!(keyboard.run()),
        pin!(monitor.run()),
        pin!(router.run()),
    );
    let mut cx = Context::from_waker(Waker::noop());
    let mut ticks = |n| {
        for _ in 0..n {
            assert!(k.as_mut().poll(&mut cx).is_pending());
            assert!(m.as_mut().poll(&mut cx).is_pending());
            assert!(r.as_mut().poll(&mut cx).is_pending());
            embassy_time::MockDriver::get().advance(embassy_time::Duration::from_millis(1));
        }
    };
    ticks(1);
    finish(publish_event_async(KeyboardEvent::key(0, 0, true)));
    ticks(20);
    assert_eq!(map.active_layer(), 2);
    finish(publish_event_async(KeyboardEvent::key(1, 1, true)));
    ticks(20);
    assert!(
        trigger.get().pressed,
        "transparent layer must use RMK's resolved action"
    );
    finish(publish_event_async(motion(31, 0)));
    ticks(100);
    let (keys, mouse) = reports();
    assert!(mouse.is_empty());
    assert!(keys.iter().any(|(_, x)| x.contains(&(HidKeyCode::Z as u8))));
    assert_eq!(keys.last(), Some(&(0, [0; 6])));
    // Change another physical position through the exact live host keymap API.
    finish(ctx.set_action(2, 2, 7, KeyAction::Single(Action::User(TRIGGER_USER_ID))));
    finish(publish_event_async(KeyboardEvent::key(2, 7, true)));
    ticks(20);
    finish(publish_event_async(KeyboardEvent::key(1, 1, false)));
    ticks(20);
    assert!(
        trigger.get().pressed,
        "second trigger must keep capture active"
    );
    // Remap a held trigger to KC_NO: raw release still clears its latch.
    finish(ctx.set_action(2, 2, 7, KeyAction::No));
    finish(publish_event_async(KeyboardEvent::key(2, 7, false)));
    ticks(20);
    assert!(!trigger.get().pressed);
    finish(publish_event_async(KeyboardEvent::key(2, 7, true)));
    ticks(20);
    assert!(!trigger.get().pressed, "live removal must take effect");
    finish(publish_event_async(KeyboardEvent::key(2, 7, false)));
    ticks(20);
    // A completed rapid tap must not be armed by a delayed ActionEvent.
    finish(publish_event_async(KeyboardEvent::key(1, 1, true)));
    finish(publish_event_async(KeyboardEvent::key(1, 1, false)));
    ticks(30);
    assert!(!trigger.get().pressed);
    finish(publish_event_async(motion(4, 5)));
    ticks(20);
    assert_eq!(reports().1, vec![(4, 5)]);
    finish(publish_event_async(KeyboardEvent::key(0, 0, false)));
    ticks(20);
    assert_eq!(map.active_layer(), 0);
    finish(publish_event_async(KeyboardEvent::key(2, 3, true)));
    finish(publish_event_async(KeyboardEvent::key(2, 3, false)));
    ticks(30);
    assert!(
        !trigger.get().pressed,
        "physical V is no longer a hardcoded trigger"
    );
    let (keys, _) = reports();
    assert!(keys.iter().any(|(_, x)| x.contains(&(HidKeyCode::V as u8))));
    assert_eq!(keys.last(), Some(&(0, [0; 6])));
}

#[test]
fn three_live_triggers_route_all_twelve_outputs_and_conflicts_capture_safely() {
    use crate::gesture::OUTPUT_POSITIONS;
    embassy_time::MockDriver::get().reset();
    rmk::state::set_usb_state(UsbState::Configured);
    reports();
    let mut keys = [[[KeyAction::No; 11]; 5]; 8];
    let codes = [
        HidKeyCode::A,
        HidKeyCode::B,
        HidKeyCode::C,
        HidKeyCode::D,
        HidKeyCode::E,
        HidKeyCode::F,
        HidKeyCode::G,
        HidKeyCode::H,
        HidKeyCode::I,
        HidKeyCode::J,
        HidKeyCode::K,
        HidKeyCode::L,
    ];
    for bank in 0..3 {
        keys[0][1][bank + 1] = KeyAction::Single(Action::User(TRIGGER_USER_ID + bank as u8));
        for (direction, &(row, col)) in OUTPUT_POSITIONS[bank].iter().enumerate() {
            keys[0][row as usize][col as usize] = key(codes[4 * bank + direction]);
        }
    }
    let mut data = KeymapData::new(keys);
    let mut behavior = BehaviorConfig::default();
    let positional = PositionalConfig::default();
    let map = finish(KeyMap::new(&mut data, &mut behavior, &positional));
    let ctx = KeyboardContext::new(&map);
    let mut keyboard = Keyboard::new(&map);
    let trigger = Cell::new(Trigger::default());
    let mut monitor = GestureTriggerMonitor::new(&trigger);
    let cfg = || PointingProcessorConfig {
        device_id: 0,
        ..Default::default()
    };
    let mut router =
        GesturePointingProcessor::new(PointingProcessor::new(&map, cfg()), cfg(), &map, &trigger);
    let (mut k, mut m, mut r) = (
        pin!(keyboard.run()),
        pin!(monitor.run()),
        pin!(router.run()),
    );
    let mut cx = Context::from_waker(Waker::noop());
    let mut ticks = |n| {
        for _ in 0..n {
            assert!(k.as_mut().poll(&mut cx).is_pending());
            assert!(m.as_mut().poll(&mut cx).is_pending());
            assert!(r.as_mut().poll(&mut cx).is_pending());
            embassy_time::MockDriver::get().advance(embassy_time::Duration::from_millis(1));
        }
    };
    ticks(1);
    for bank in 0..3 {
        finish(publish_event_async(KeyboardEvent::key(1, bank + 1, true)));
        ticks(20);
        assert_eq!(trigger.get().bank, bank);
        for (dir, (x, y)) in [(-31, 0), (31, 0), (0, -31), (0, 31)]
            .into_iter()
            .enumerate()
        {
            embassy_time::MockDriver::get().advance(embassy_time::Duration::from_millis(150));
            finish(publish_event_async(motion(x, y)));
            ticks(40);
            let (keys, mouse) = reports();
            assert!(mouse.is_empty());
            let nonzero: Vec<_> = keys
                .iter()
                .flat_map(|(_, k)| k.iter().copied())
                .filter(|&x| x != 0)
                .collect();
            assert_eq!(
                nonzero,
                vec![codes[bank as usize * 4 + dir] as u8],
                "wrong bank/direction or duplicate output"
            );
            assert_eq!(keys.last(), Some(&(0, [0; 6])));
        }
        finish(publish_event_async(KeyboardEvent::key(1, bank + 1, false)));
        ticks(20);
        assert!(!trigger.get().pressed);
    }
    // Both new banks also resolve output edits made through the live host API.
    for bank in 1..3u8 {
        let (row, col) = OUTPUT_POSITIONS[bank as usize][3];
        finish(ctx.set_action(0, row, col, key(HidKeyCode::F24)));
        finish(publish_event_async(KeyboardEvent::key(1, bank + 1, true)));
        ticks(20);
        embassy_time::MockDriver::get().advance(embassy_time::Duration::from_millis(150));
        finish(publish_event_async(motion(0, 31)));
        ticks(40);
        let (keys, mouse) = reports();
        assert!(mouse.is_empty());
        assert!(
            keys.iter()
                .any(|(_, k)| k.contains(&(HidKeyCode::F24 as u8)))
        );
        assert_eq!(keys.last(), Some(&(0, [0; 6])));
        finish(publish_event_async(KeyboardEvent::key(1, bank + 1, false)));
        ticks(20);
    }
    // Switching banks discards an unfinished stroke. A conflicting hold captures
    // motion but cannot fire either bank or leak cursor motion.
    finish(publish_event_async(KeyboardEvent::key(1, 1, true)));
    ticks(20);
    finish(publish_event_async(motion(30, 0)));
    ticks(20);
    finish(publish_event_async(KeyboardEvent::key(1, 2, true)));
    ticks(20);
    assert_eq!(trigger.get().bank, BANK_COUNT);
    finish(publish_event_async(motion(100, 100)));
    ticks(20);
    assert_eq!(reports(), (vec![], vec![]));
    finish(publish_event_async(KeyboardEvent::key(1, 1, false)));
    ticks(20);
    assert_eq!(trigger.get().bank, 1);
    finish(publish_event_async(motion(1, 0)));
    ticks(20);
    assert_eq!(reports(), (vec![], vec![]));
    embassy_time::MockDriver::get().advance(embassy_time::Duration::from_millis(150));
    finish(publish_event_async(motion(30, 0)));
    ticks(40);
    let (keys, mouse) = reports();
    assert!(mouse.is_empty());
    assert!(keys.iter().any(|(_, k)| k.contains(&(HidKeyCode::F as u8))));
    assert_eq!(keys.last(), Some(&(0, [0; 6])));
    finish(publish_event_async(KeyboardEvent::key(1, 2, false)));
    ticks(20);
    finish(publish_event_async(motion(5, -6)));
    ticks(20);
    assert_eq!(reports().1, vec![(5, -6)]);
}

#[test]
fn virtual_outputs_and_reserved_ble_codes_cannot_arm_any_bank() {
    let trigger = Cell::new(Trigger::default());
    let mut monitor = GestureTriggerMonitor::new(&trigger);
    for bank in 0..3u8 {
        for &(row, col) in &crate::gesture::OUTPUT_POSITIONS[bank as usize] {
            let e = KeyboardEvent::key(row, col, true);
            finish(monitor.on_keyboard_event(e));
            finish(monitor.on_action_event(ActionEvent {
                action: Action::User(TRIGGER_USER_ID + bank),
                keyboard_event: e,
            }));
            assert!(!trigger.get().pressed);
        }
    }
    let e = KeyboardEvent::key(1, 1, true);
    finish(monitor.on_keyboard_event(e));
    for id in 0..10 {
        finish(monitor.on_action_event(ActionEvent {
            action: Action::User(id),
            keyboard_event: e,
        }));
        assert!(!trigger.get().pressed);
    }
}
