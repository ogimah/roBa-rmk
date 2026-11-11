use crate::gesture::{
    BANK_COUNT, Gesture, Route, SCROLL_USER_ID, TRIGGER_USER_ID, Trigger, logical_motion,
    output_position,
};
use core::cell::Cell;
use rmk::{
    core_traits::Runnable,
    event::{
        ActionEvent, Axis, EventSubscriber, KeyboardEvent, KeyboardEventPos, LayerChangeEvent,
        PointingEvent, PointingProcessorEvent, SubscribableEvent, publish_event_async,
    },
    input_device::pointing::{
        PointingMode, PointingProcessor, PointingProcessorConfig,
        PointingProcessorProcessorEventEnum, ScrollConfig,
    },
    keymap::KeyMap,
    macros::processor,
    processor::Processor,
    types::action::Action,
};

// Separate task: drain both queues while the motion task publishes synthetic keys.
// ActionEvent comes from RMK after live keymap/transparent-layer resolution.
// Physical releases clear the latch even if Vial remaps a held key to KC_NO.
pub struct GestureTriggerMonitor<'a> {
    trigger: &'a Cell<Trigger>,
    down: [bool; 44],
    armed: [u8; 44],
}
impl<'a> GestureTriggerMonitor<'a> {
    pub fn new(trigger: &'a Cell<Trigger>) -> Self {
        Self {
            trigger,
            down: [false; 44],
            armed: [0; 44],
        }
    }
    fn index(event: KeyboardEvent) -> Option<usize> {
        if let KeyboardEventPos::Key(pos) = event.pos {
            if pos.row < 4 && pos.col < 11 && (pos.row, pos.col) != (0, 5) {
                return Some(usize::from(pos.row) * 11 + usize::from(pos.col));
            }
        }
        None // Synthetic gesture outputs and encoders cannot become triggers.
    }
    fn update(&self) {
        let mut t = self.trigger.get();
        let mask = self
            .armed
            .iter()
            .filter(|&&bank| (1..=BANK_COUNT).contains(&bank))
            .fold(0, |mask, &bank| mask | (1 << (bank - 1)));
        t.set_mask(mask);
        t.set_scroll(self.armed.contains(&(BANK_COUNT + 1)));
        self.trigger.set(t);
    }
    async fn on_keyboard_event(&mut self, event: KeyboardEvent) {
        if let Some(i) = Self::index(event) {
            self.down[i] = event.pressed;
            if !event.pressed {
                self.armed[i] = 0;
            }
            self.update();
        }
    }
    async fn on_action_event(&mut self, event: ActionEvent) {
        if let Some(i) = Self::index(event.keyboard_event) {
            if !event.keyboard_event.pressed {
                self.armed[i] = 0;
            } else if let Action::User(id) = event.action {
                if (TRIGGER_USER_ID..TRIGGER_USER_ID + BANK_COUNT).contains(&id) && self.down[i] {
                    self.armed[i] = id - TRIGGER_USER_ID + 1;
                } else if id == SCROLL_USER_ID && self.down[i] {
                    self.armed[i] = BANK_COUNT + 1;
                }
            }
            self.update();
        }
    }
    async fn on_layer_change_event(&mut self, _: LayerChangeEvent) {
        let mut t = self.trigger.get();
        t.layer_changed();
        self.trigger.set(t);
    }
}
impl Runnable for GestureTriggerMonitor<'_> {
    async fn run(&mut self) -> ! {
        use rmk::embassy_futures::select::{Either3, select3};
        let mut raw = KeyboardEvent::subscriber();
        let mut actions = ActionEvent::subscriber();
        let mut layers = LayerChangeEvent::subscriber();
        loop {
            match select3(raw.next_event(), actions.next_event(), layers.next_event()).await {
                Either3::First(e) => self.on_keyboard_event(e).await,
                Either3::Second(e) => {
                    // Catch up physical state before arming a delayed action. A
                    // completed quick tap must never latch the trigger as held.
                    while let Some(raw_event) = raw.try_next_message_pure() {
                        self.on_keyboard_event(raw_event).await;
                    }
                    self.on_action_event(e).await;
                }
                Either3::Third(e) => self.on_layer_change_event(e).await,
            }
        }
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
    scroll: PointingProcessor<'a>,
    scroll_epoch: (u32, u32),
}
impl<'a> GesturePointingProcessor<'a> {
    pub fn new(
        inner: PointingProcessor<'a>,
        config: PointingProcessorConfig,
        keymap: &'a KeyMap<'a>,
        trigger: &'a Cell<Trigger>,
    ) -> Self {
        let scroll = Self::new_scroll(keymap, &config);
        Self {
            inner,
            config,
            keymap,
            trigger,
            gesture: Gesture::default(),
            scroll,
            scroll_epoch: (0, 0),
        }
    }
    fn new_scroll(
        keymap: &'a KeyMap<'a>,
        config: &PointingProcessorConfig,
    ) -> PointingProcessor<'a> {
        let mut processor = PointingProcessor::new(keymap, config.clone());
        processor.set_pointing_mode(PointingMode::Scroll(ScrollConfig {
            multiplier_x: 0,
            divisor_x: 0,
            ..Default::default()
        }));
        processor
    }
    async fn on_pointing_event(&mut self, event: PointingEvent) {
        if event.device_id != self.config.device_id {
            self.inner
                .process(PointingProcessorProcessorEventEnum::Pointing(event))
                .await;
            return;
        }
        let trigger = self.trigger.get();
        let epoch = (trigger.scroll_epoch, trigger.epoch);
        if epoch != self.scroll_epoch {
            // RMK's mode setter does NOT reset its private accumulator. Recreate
            // only this dedicated scroll processor, using its public constructor.
            // Epochs also catch release/repress or gesture preemption with no
            // intermediate pointing event, preventing stale fractional motion.
            self.scroll = Self::new_scroll(self.keymap, &self.config);
            self.scroll_epoch = epoch;
        }
        if trigger.scroll_pressed && !trigger.pressed {
            self.scroll
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
            trigger,
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
                if let Some((row, col)) = output_position(trigger.bank, dir) {
                    publish_event_async(KeyboardEvent::key(row, col, true)).await;
                    publish_event_async(KeyboardEvent::key(row, col, false)).await;
                }
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
