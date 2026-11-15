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
        let mut usb_transport = rmk::usb::UsbTransport::new(driver, rmk_config.device_config)
            .with_host_service(&host_service);
        // The legacy BLE advertisement has room for 16 name bytes. Keep the
        // full USB/Vial/file name, but advertise an unambiguous shorter name.
        let mut rmk_config = rmk_config;
        rmk_config.device_config.product_name = "roBa_GT_COROPIT";
        // RMK 0.9 owns the BLE stack and split managers inside BleTransport.
        let mut ble_transport = rmk::ble::BleTransport::new(
            ble_controller,
            ble_addr,
            rmk_config,
            [rmk::split::PeripheralMatrixConfig {
                rows: 4,
                cols: 6,
                row_offset: 0,
                col_offset: 0,
            }],
        )
        .with_host_service(&host_service);
        rmk::run_all!(
            adc_device,
            trackball0_id0_device,
            matrix,
            storage,
            keyboard,
            trigger_monitor,
            usb_transport,
            ble_transport,
            wpm_processor,
            watchdog_runner,
            battery_processor,
            trackball0_id0_processor,
        )
        .await;
    }
}
