//! Stock port acceptance tests: real Vial dispatcher -> live keymap -> keyboard -> USB HID reports.
use super::*;
use crate::config::{BehaviorConfig, PositionalConfig, VialConfig};
use crate::hid::ViaReport;
use crate::host::context::KeyboardContext;
use crate::host::via::keycode_convert::from_via_keycode;
use crate::host::via::test_process_vial as process_vial;
use crate::test_support::test_block_on as block_on;
use rmk_types::protocol::vial::{VialCommand, VialDynamic};

fn keyboard() -> Keyboard<'static> {
    crate::channel::USB_REPORT_CHANNEL.clear();
    crate::state::set_usb_state(rmk_types::connection::UsbState::Configured);
    let mut actions = [[[KeyAction::Transparent; 14]; 1]; 4];
    for n in 0..10 {
        actions[0][0][n] = from_via_keycode(if n == 0 { 0x62 } else { 0x58 + n as u16 });
    }
    actions[0][0][10] = from_via_keycode(0xe0); // LCtrl
    actions[0][0][11] = from_via_keycode(0xe4); // RCtrl
    actions[0][0][12] = from_via_keycode(0xe1); // LShift
    actions[0][0][13] = from_via_keycode(0x432c); // LT(3, Space)
    let data = Box::leak(Box::new(crate::keymap::KeymapData::new(actions)));
    let behavior = Box::leak(Box::new(BehaviorConfig::default()));
    let positions = Box::leak(Box::new(PositionalConfig::<1, 14>::default()));
    let keymap = Box::leak(Box::new(block_on(KeyMap::new(data, behavior, positions))));
    Keyboard::new(keymap)
}

async fn request(k: &Keyboard<'_>, op: VialDynamic, idx: u8, wire: Option<[u8; 10]>) -> ViaReport {
    let ctx = KeyboardContext::new(k.keymap);
    let mut report = ViaReport::default();
    report.output_data[0..4].copy_from_slice(&[0xfe, VialCommand::DynamicEntryOp as u8, op as u8, idx]);
    if let Some(w) = wire {
        report.output_data[4..14].copy_from_slice(&w);
    }
    #[cfg(feature = "host_lock")]
    let locker = crate::host::lock::HostLock::new(&[], k.keymap, true, Duration::from_millis(100));
    process_vial(
        &mut report,
        &VialConfig::default(),
        #[cfg(feature = "host_lock")]
        &locker,
        &ctx,
    )
    .await;
    report
}

fn wire(n: u8) -> [u8; 10] {
    let trigger = if n == 0 { 0x62 } else { 0x58 + n };
    let output = if n == 0 { 0x27 } else { 0x1d + n };
    // Both Ctrl sides, keep Ctrl, all layers, enabled + all activation events,
    // keep the override active when a second number is pressed.
    [trigger, 0, output, 0, 0xff, 0xff, 0x11, 0, 0, 0xa7]
}
async fn install(k: &Keyboard<'_>, n: u8, w: [u8; 10]) {
    assert_eq!(
        request(k, VialDynamic::DynamicVialKeyOverrideSet, n, Some(w))
            .await
            .input_data[0],
        0
    );
    let response = request(k, VialDynamic::DynamicVialKeyOverrideGet, n, None).await;
    assert_eq!(response.input_data[0], 0);
    assert_eq!(&response.input_data[1..11], &w);
    #[cfg(feature = "storage")]
    assert!(
        matches!(crate::channel::FLASH_CHANNEL.receive().await, crate::storage::FlashOperationMessage::Fork {idx,..} if idx==n)
    );
}
async fn step(k: &mut Keyboard<'static>, col: u8, pressed: bool) {
    k.process_inner(KeyboardEvent::key(0, col, pressed)).await;
    let mut last = None;
    while let Ok(report) = crate::channel::USB_REPORT_CHANNEL.try_receive() {
        if let crate::hid::Report::KeyboardReport(report) = report {
            last = Some(report);
        }
    }
    if col < 13 {
        let actual = last.expect("a resolved number/modifier must reach the USB HID queue");
        let expected = k.build_keyboard_report(pressed);
        assert_eq!(actual.keycodes, expected.keycodes);
        assert_eq!(actual.modifier, expected.modifier);
    }
}
fn assert_hid(k: &mut Keyboard<'_>, codes: &[u8], mods: u8) {
    let report = k.build_keyboard_report(true);
    let actual: std::vec::Vec<_> = report.keycodes.into_iter().filter(|c| *c != 0).collect();
    assert_eq!(actual, codes);
    assert_eq!(report.modifier, mods);
}

#[test]
fn vial_override_digits_ctrl_sides_shift_and_release_orders() {
    let mut k = keyboard();
    block_on(async {
        let entries = request(&k, VialDynamic::DynamicVialGetNumberOfEntries, 0, None).await;
        assert_eq!(entries.input_data[2], 32);
        assert_eq!(entries.input_data[0], crate::MORSE_MAX_NUM as u8);
        for n in 0..10u8 {
            let mut standard = wire(n);
                standard[9] = 0x87; // Vial: enabled + the first three activation options only
                install(&k, n, standard).await;
            let keypad = if n == 0 { 0x62 } else { 0x58 + n };
            let normal = if n == 0 { 0x27 } else { 0x1d + n };
            step(&mut k, n, true).await;
            assert_hid(&mut k, &[keypad], 0);
            step(&mut k, n, false).await;
            assert_hid(&mut k, &[], 0);
            for (ctrl, mods) in [(10, 1), (11, 0x10)] {
                // Ctrl first, number released first.
                step(&mut k, ctrl, true).await;
                step(&mut k, n, true).await;
                assert_hid(&mut k, &[normal], mods);
                step(&mut k, n, false).await;
                assert_hid(&mut k, &[], mods);
                step(&mut k, ctrl, false).await;
                assert_hid(&mut k, &[], 0);
                // Number first; Ctrl activates mid-hold, Ctrl released first.
                step(&mut k, n, true).await;
                assert_hid(&mut k, &[keypad], 0);
                step(&mut k, ctrl, true).await;
                assert_hid(&mut k, &[normal], mods);
                step(&mut k, ctrl, false).await;
                assert_hid(&mut k, &[keypad], 0);
                step(&mut k, n, false).await;
                assert_hid(&mut k, &[], 0);
            }
            step(&mut k, 10, true).await;
            step(&mut k, 12, true).await;
            step(&mut k, n, true).await;
            assert_hid(&mut k, &[normal], 3);
            step(&mut k, n, false).await;
            step(&mut k, 12, false).await;
            step(&mut k, 10, false).await;
            assert_hid(&mut k, &[], 0);
        }
    });
}

#[test]
fn vial_override_live_disable_layer_negative_suppression_and_multiple_digits() {
    let mut k = keyboard();
    block_on(async {
        install(&k, 1, wire(1)).await;
        install(&k, 2, wire(2)).await;
        step(&mut k, 10, true).await;
        step(&mut k, 1, true).await;
        step(&mut k, 2, true).await;
        assert_hid(&mut k, &[0x1e, 0x1f], 1);
        step(&mut k, 1, false).await;
        assert_hid(&mut k, &[0x1f], 1);
        step(&mut k, 2, false).await;
        step(&mut k, 10, false).await;
        let mut w = wire(1);
        w[9] &= 0x7f;
        install(&k, 1, w).await;
        step(&mut k, 10, true).await;
        step(&mut k, 1, true).await;
        assert_hid(&mut k, &[0x59], 1);
        step(&mut k, 1, false).await;
        step(&mut k, 10, false).await;
        w = wire(1);
        w[4] = 8;
        w[5] = 0; // source layer 3 only
        install(&k, 1, w).await;
        step(&mut k, 10, true).await;
        step(&mut k, 1, true).await;
        assert_hid(&mut k, &[0x59], 1);
        step(&mut k, 1, false).await;
        step(&mut k, 10, false).await;
        // LT + Ctrl + keypad: three physical keys are not a Combo.
        // Set an explicit layer-3 keypad mapping, since transparent keys have source layer 0.
        k.keymap
            .set_action_at(KeyboardEventPos::key_pos(1, 0), 3, from_via_keycode(0x59));
        step(&mut k, 13, true).await;
        embassy_time::Timer::after_millis(1000).await;
        if let Some(held) = k.next_buffered_key() {
            k.process_buffered_key(held).await;
        }
        step(&mut k, 10, true).await;
        step(&mut k, 1, true).await;
        assert_hid(&mut k, &[0x1e], 1);
        step(&mut k, 1, false).await;
        step(&mut k, 10, false).await;
        step(&mut k, 13, false).await;
        assert_hid(&mut k, &[], 0);
        w = wire(1);
        w[7] = 2;
        w[8] = 0x11; // negative LShift, suppress Ctrl
        install(&k, 1, w).await;
        step(&mut k, 10, true).await;
        step(&mut k, 12, true).await;
        step(&mut k, 1, true).await;
        assert_hid(&mut k, &[0x59], 3);
        step(&mut k, 12, false).await;
        assert_hid(&mut k, &[0x1e], 0);
        step(&mut k, 1, false).await;
        step(&mut k, 10, false).await;
        assert_hid(&mut k, &[], 0);
        assert_eq!(
            request(&k, VialDynamic::DynamicVialKeyOverrideSet, 32, Some(wire(1)))
                .await
                .input_data[0],
            1
        );
        assert_eq!(
            request(&k, VialDynamic::DynamicVialKeyOverrideGet, 32, None)
                .await
                .input_data[0],
            1
        );
        install(&k, 31, wire(9)).await; // highest advertised slot is writable
        let empty = [0; 10];
        install(&k, 31, empty).await;
    });
}
