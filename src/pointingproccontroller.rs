use rmk::{
    event::{publish_event, LayerChangeEvent, PointingProcessorEvent},
    input_device::pointing::{CursorConfig, PointingMode, ScrollConfig},
};
use rmk::macros::processor;

#[processor(subscribe = [LayerChangeEvent])]
#[derive(Default)]
pub struct PointingProcessorController;

impl PointingProcessorController {
    pub fn new() -> Self {
        Self
    }

    async fn on_layer_change_event(&mut self, event: LayerChangeEvent) {
        match event.0 {
            // Layer 3: trackball scroll mode
            3 => {
                publish_event(PointingProcessorEvent {
                    device_id: 0,
                    mode: PointingMode::Scroll(ScrollConfig {
                        multiplier_x: 1,
                        divisor_x: 16,
                        multiplier_y: 1,
                        divisor_y: 16,
                        invert_x: false,
                        invert_y: false,
                    }),
                });
            }

            // All other layers: normal cursor mode
            _ => {
                publish_event(PointingProcessorEvent {
                    device_id: 0,
                    mode: PointingMode::Cursor(CursorConfig::default()),
                });
            }
        }
    }
}
