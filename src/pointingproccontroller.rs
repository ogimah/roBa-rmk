use rmk::{
    event::{publish_event, LayerChangeEvent, PointingProcessorEvent},
    input_device::pointing::{CursorConfig, PointingMode, ScrollConfig},
};
use rmk::macros::processor;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GestureDirection {
    Left,
    Right,
    Up,
    Down,
}

#[processor(subscribe = [LayerChangeEvent])]
#[derive(Default)]
pub struct PointingProcessorController {
    gesture_layer: u8,
    gesture_cooldown_ms: u32,
    last_gesture_ms: u32,
}

impl PointingProcessorController {
    pub fn new() -> Self {
        Self {
            gesture_layer: 4,
            gesture_cooldown_ms: 120,
            last_gesture_ms: 0,
        }
    }

    fn is_gesture_layer(layer: u8) -> bool {
        layer == 4
    }

    fn classify_motion(dx: i32, dy: i32) -> Option<GestureDirection> {
        const THRESHOLD: i32 = 32;

        // Gesture layer: freeze cursor motion while evaluating raw trackball movement.
        // Only a simple 4-direction classification is supported initially.
        if dx.abs() <= THRESHOLD && dy < -THRESHOLD {
            return Some(GestureDirection::Up);
        }
        if dx.abs() <= THRESHOLD && dy > THRESHOLD {
            return Some(GestureDirection::Down);
        }
        if dx > THRESHOLD && dy.abs() <= THRESHOLD {
            return Some(GestureDirection::Right);
        }
        if dx < -THRESHOLD && dy.abs() <= THRESHOLD {
            return Some(GestureDirection::Left);
        }

        None
    }

    fn cooldown_elapsed(&self, now_ms: u32) -> bool {
        now_ms.wrapping_sub(self.last_gesture_ms) >= self.gesture_cooldown_ms
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

            // Layer 4: gesture layer.
            // Raw trackball motion is evaluated here; the cursor should be frozen while
            // the gesture direction is interpreted, then the corresponding custom action
            // is emitted.
            4 => {
                publish_event(PointingProcessorEvent {
                    device_id: 0,
                    mode: PointingMode::Cursor(CursorConfig::default()),
                });
                self.gesture_layer = 4;
            }

            // All other layers: normal cursor mode
            _ => {
                publish_event(PointingProcessorEvent {
                    device_id: 0,
                    mode: PointingMode::Cursor(CursorConfig::default()),
                });
                self.gesture_layer = 0;
            }
        }
    }
}
