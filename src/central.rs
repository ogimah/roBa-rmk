#![no_main]
#![no_std]
#![deny(unused_must_use)]

mod gesture;
mod pointingproccontroller;

use rmk::macros::rmk_central;

#[rmk_central]
mod keyboard_central {
    #[Overwritten(entry)]
    fn run_with_gesture() {
        let trigger = core::cell::Cell::new(crate::gesture::Trigger::default());
        let mut trigger_monitor =
            crate::pointingproccontroller::GestureTriggerMonitor::new(&trigger);
        let mut trackball0_id0_processor =
            crate::pointingproccontroller::GesturePointingProcessor::new(
                trackball0_id0_processor,
                rmk::input_device::pointing::PointingProcessorConfig {
                    device_id: trackball0_id0_device.id,
                    ..Default::default()
                },
                &keymap,
                &trigger,
            );
        // Preserve all generated devices and tasks. The original pointing
        // processor is owned by the wrapper and must not also run separately.
        let mut wpm_processor = rmk::processor::builtin::wpm::WpmProcessor::new();
        let mut usb_transport = rmk::usb::UsbTransport::new(driver, rmk_config.device_config);
        let mut ble_transport = rmk::ble::BleTransport::new(&stack, rmk_config).await;
        rmk::embassy_futures::join::join3(
            async {
                rmk::run_all!(
                    adc_device,
                    trackball0_id0_device,
                    matrix,
                    storage,
                    keyboard,
                    trigger_monitor,
                    host_service,
                    usb_transport,
                    ble_transport,
                    wpm_processor,
                    watchdog_runner,
                    battery_processor,
                    trackball0_id0_processor,
                )
                .await;
            },
            rmk::split::central::run_peripheral_manager::<4, 6, 0, 0, _>(
                0,
                &peripheral_addrs,
                &stack,
            ),
            rmk::split::ble::central::scan_peripherals(&stack, &peripheral_addrs),
        )
        .await;
    }
}
