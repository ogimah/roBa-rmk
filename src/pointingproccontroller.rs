use crate::gesture::{Gesture, OUTPUT_ROW, Route, Trigger, logical_motion};
use core::cell::Cell;
use rmk::{
    event::{
        Axis, KeyboardEvent, KeyboardEventPos, LayerChangeEvent, PointingEvent,
        PointingProcessorEvent, publish_event_async,
    },
    input_device::pointing::{
        PointingProcessor, PointingProcessorConfig, PointingProcessorProcessorEventEnum,
    },
    keymap::KeyMap,
    macros::processor,
    processor::Processor,
};

// Independent task: keep draining KeyboardEvent while the motion task awaits
// publication of synthetic press/release events to that same channel.
#[processor(subscribe = [KeyboardEvent, LayerChangeEvent])]
pub struct GestureTriggerMonitor<'a> {
    trigger: &'a Cell<Trigger>,
}
impl<'a> GestureTriggerMonitor<'a> {
    pub fn new(trigger: &'a Cell<Trigger>) -> Self {
        Self { trigger }
    }
    async fn on_keyboard_event(&mut self, event: KeyboardEvent) {
        if let KeyboardEventPos::Key(pos) = event.pos {
            let mut t = self.trigger.get();
            t.key(pos.row, pos.col, event.pressed);
            self.trigger.set(t);
        }
    }
    async fn on_layer_change_event(&mut self, _: LayerChangeEvent) {
        let mut t = self.trigger.get();
        t.layer_changed();
        self.trigger.set(t);
    }
}

#[cfg(test)]
#[path = "../tests/gesture-host/router_tests.rs"]
mod tests;

// The sole pointing subscriber replaces the original processor's runnable.
#[processor(subscribe = [PointingEvent, PointingProcessorEvent])]
pub struct GesturePointingProcessor<'a> {
    inner: PointingProcessor<'a>,
    config: PointingProcessorConfig,
    keymap: &'a KeyMap<'a>,
    trigger: &'a Cell<Trigger>,
    gesture: Gesture,
}
impl<'a> GesturePointingProcessor<'a> {
    pub fn new(
        inner: PointingProcessor<'a>,
        config: PointingProcessorConfig,
        keymap: &'a KeyMap<'a>,
        trigger: &'a Cell<Trigger>,
    ) -> Self {
        Self {
            inner,
            config,
            keymap,
            trigger,
            gesture: Gesture::default(),
        }
    }
    async fn on_pointing_event(&mut self, event: PointingEvent) {
        if event.device_id != self.config.device_id {
            self.inner
                .process(PointingProcessorProcessorEventEnum::Pointing(event))
                .await;
            return;
        }
        let (mut x, mut y) = (0, 0);
        for axis in event.axes {
            match axis.axis {
                Axis::X => x = i32::from(axis.value),
                Axis::Y => y = i32::from(axis.value),
                _ => {}
            }
        }
        let (x, y) = logical_motion(
            x,
            y,
            self.config.invert_x,
            self.config.invert_y,
            self.config.swap_xy,
        );
        match self.gesture.motion(
            self.keymap.active_layer(),
            self.trigger.get(),
            x,
            y,
            embassy_time::Instant::now().as_millis(),
        ) {
            Route::Cursor => {
                self.inner
                    .process(PointingProcessorProcessorEventEnum::Pointing(event))
                    .await
            }
            Route::Captured(None) => {}
            Route::Captured(Some(dir)) => {
                // RMK resolves the live Vial binding and caches its layer for release.
                publish_event_async(KeyboardEvent::key(OUTPUT_ROW, dir.col(), true)).await;
                publish_event_async(KeyboardEvent::key(OUTPUT_ROW, dir.col(), false)).await;
            }
        }
    }
    async fn on_pointing_processor_event(&mut self, event: PointingProcessorEvent) {
        self.inner
            .process(PointingProcessorProcessorEventEnum::PointingProcessor(
                event,
            ))
            .await;
    }
}
