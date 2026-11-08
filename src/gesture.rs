//! Hardware-independent capture state. Distances are sensor counts, not pixels.
pub const GESTURE_LAYER: u8 = 3;
pub const TRIGGER_ROW: u8 = 2;
pub const TRIGGER_COL: u8 = 3;
pub const OUTPUT_ROW: u8 = 4;
pub const THRESHOLD: i32 = 30;
pub const COOLDOWN_MS: u64 = 150;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}
impl Direction {
    pub const fn col(self) -> u8 {
        match self {
            Self::Left => 0,
            Self::Right => 1,
            Self::Up => 2,
            Self::Down => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Trigger {
    pub pressed: bool,
    pub epoch: u32,
}
impl Trigger {
    pub fn key(&mut self, row: u8, col: u8, pressed: bool) {
        if (row, col) == (TRIGGER_ROW, TRIGGER_COL) && self.pressed != pressed {
            self.pressed = pressed;
            self.epoch = self.epoch.wrapping_add(1);
        }
    }
    pub fn layer_changed(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    Cursor,
    Captured(Option<Direction>),
}

#[derive(Default)]
pub struct Gesture {
    epoch: u32,
    active: bool,
    x: i32,
    y: i32,
    last_fire: Option<u64>,
}
impl Gesture {
    pub fn motion(&mut self, layer: u8, trigger: Trigger, x: i32, y: i32, now: u64) -> Route {
        let active = layer == GESTURE_LAYER && trigger.pressed;
        if active != self.active || trigger.epoch != self.epoch {
            self.x = 0;
            self.y = 0;
        }
        self.active = active;
        self.epoch = trigger.epoch;
        if !active {
            return Route::Cursor;
        }
        self.x = self.x.saturating_add(x).clamp(-1_000_000, 1_000_000);
        self.y = self.y.saturating_add(y).clamp(-1_000_000, 1_000_000);
        if self.x.abs() + self.y.abs() <= THRESHOLD {
            return Route::Captured(None);
        }
        let direction = if self.x.abs() >= self.y.abs() {
            if self.x > 0 {
                Direction::Right
            } else {
                Direction::Left
            }
        } else if self.y > 0 {
            Direction::Down
        } else {
            Direction::Up
        };
        // Discard a threshold crossing during cooldown, as in the reference.
        self.x = 0;
        self.y = 0;
        if self
            .last_fire
            .is_some_and(|last| now.saturating_sub(last) < COOLDOWN_MS)
        {
            return Route::Captured(None);
        }
        self.last_fire = Some(now);
        Route::Captured(Some(direction))
    }
}

/// PMW3610 sensor transforms have already run; apply only processor transforms.
pub fn logical_motion(
    mut x: i32,
    mut y: i32,
    invert_x: bool,
    invert_y: bool,
    swap: bool,
) -> (i32, i32) {
    if invert_x {
        x = -x;
    }
    if invert_y {
        y = -y;
    }
    if swap {
        (x, y) = (y, x);
    }
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn held() -> Trigger {
        Trigger {
            pressed: true,
            epoch: 1,
        }
    }
    #[test]
    fn all_four_directions_fire_without_release() {
        for (x, y, d) in [
            (31, 0, Direction::Right),
            (-31, 0, Direction::Left),
            (0, -31, Direction::Up),
            (0, 31, Direction::Down),
        ] {
            assert_eq!(
                Gesture::default().motion(3, held(), x, y, 0),
                Route::Captured(Some(d))
            );
        }
    }
    #[test]
    fn threshold_is_strict_and_accumulated() {
        let mut g = Gesture::default();
        assert_eq!(g.motion(3, held(), 10, 0, 0), Route::Captured(None));
        assert_eq!(g.motion(3, held(), 20, 0, 1), Route::Captured(None));
        assert_eq!(
            g.motion(3, held(), 1, 0, 2),
            Route::Captured(Some(Direction::Right))
        );
    }
    #[test]
    fn dominant_axis_and_horizontal_tie() {
        for (x, y, d) in [
            (20, 16, Direction::Right),
            (-16, 20, Direction::Down),
            (16, -20, Direction::Up),
            (-16, -16, Direction::Left),
        ] {
            assert_eq!(
                Gesture::default().motion(3, held(), x, y, 0),
                Route::Captured(Some(d))
            );
        }
    }
    #[test]
    fn cooldown_and_no_residual_repeats() {
        let mut g = Gesture::default();
        assert_eq!(
            g.motion(3, held(), 900, 0, 0),
            Route::Captured(Some(Direction::Right))
        );
        assert_eq!(g.motion(3, held(), 0, 99, 149), Route::Captured(None));
        assert_eq!(
            g.motion(3, held(), 0, -31, 150),
            Route::Captured(Some(Direction::Up))
        );
        assert_eq!(g.motion(3, held(), 0, 0, 400), Route::Captured(None));
    }
    #[test]
    fn release_or_layer_change_does_not_fire() {
        let mut g = Gesture::default();
        g.motion(3, held(), 30, 0, 0);
        assert_eq!(
            g.motion(
                3,
                Trigger {
                    pressed: false,
                    epoch: 2
                },
                0,
                0,
                1
            ),
            Route::Cursor
        );
        assert_eq!(
            g.motion(
                3,
                Trigger {
                    pressed: true,
                    epoch: 3
                },
                1,
                0,
                2
            ),
            Route::Captured(None)
        );
        assert_eq!(g.motion(2, held(), 99, 0, 3), Route::Cursor);
        assert_eq!(g.motion(3, held(), 1, 0, 4), Route::Captured(None));
    }
    #[test]
    fn new_episode_without_intermediate_motion_resets_accumulator() {
        let mut g = Gesture::default();
        let mut t = held();
        g.motion(3, t, 30, 0, 0);
        t.key(2, 3, false);
        t.key(2, 3, true);
        assert_eq!(g.motion(3, t, 1, 0, 200), Route::Captured(None));
        t.layer_changed();
        assert_eq!(g.motion(3, t, 29, 0, 201), Route::Captured(None));
    }
    #[test]
    fn normal_motion_passes_in_other_layers_and_without_v() {
        for l in 0..8 {
            if l != 3 {
                assert_eq!(
                    Gesture::default().motion(l, held(), 80, -80, 0),
                    Route::Cursor
                );
            }
        }
        assert_eq!(
            Gesture::default().motion(3, Trigger::default(), 80, -80, 0),
            Route::Cursor
        );
    }
    #[test]
    fn unrelated_keys_cannot_arm_and_epoch_wraps() {
        let mut t = Trigger {
            pressed: false,
            epoch: u32::MAX,
        };
        t.key(1, 3, true);
        assert!(!t.pressed);
        t.key(2, 3, true);
        assert_eq!(t.epoch, 0);
    }
    #[test]
    fn transforms_use_processor_order() {
        assert_eq!(logical_motion(10, -20, true, false, true), (-20, -10));
    }
    #[test]
    fn outputs_are_distinct_virtual_positions() {
        assert_eq!(
            [
                Direction::Left.col(),
                Direction::Right.col(),
                Direction::Up.col(),
                Direction::Down.col()
            ],
            [0, 1, 2, 3]
        );
        assert_eq!(OUTPUT_ROW, 4);
    }
    #[test]
    fn extreme_motion_cannot_overflow() {
        assert_eq!(
            Gesture::default().motion(3, held(), i32::MIN, i32::MIN, 0),
            Route::Captured(Some(Direction::Left))
        );
    }
}
