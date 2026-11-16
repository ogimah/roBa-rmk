//! Regression tests for RMK PR #1277 using level-sensitive simulated GPIO.
use super::*;
use core::{cell::Cell, convert::Infallible, future::Future, pin::pin, task::{Context, Poll, Waker}};
use std::rc::Rc;

#[derive(Clone)]
struct Pin(Rc<Cell<bool>>); // true = low
impl embedded_hal::digital::ErrorType for Pin { type Error = Infallible; }
impl InputPin for Pin {
    fn is_high(&mut self) -> Result<bool, Infallible> { Ok(!self.0.get()) }
    fn is_low(&mut self) -> Result<bool, Infallible> { Ok(self.0.get()) }
}
impl Pin {
    async fn level(&self, low: bool) -> Result<(), Infallible> {
        core::future::poll_fn(|_| if self.0.get() == low { Poll::Ready(Ok(())) } else { Poll::Pending }).await
    }
}
impl Wait for Pin {
    async fn wait_for_high(&mut self) -> Result<(), Infallible> { self.level(false).await }
    async fn wait_for_low(&mut self) -> Result<(), Infallible> { self.level(true).await }
    async fn wait_for_any_edge(&mut self) -> Result<(), Infallible> { self.level(!self.0.get()).await }
    async fn wait_for_rising_edge(&mut self) -> Result<(), Infallible> { self.level(true).await?; self.level(false).await }
    async fn wait_for_falling_edge(&mut self) -> Result<(), Infallible> { self.level(false).await?; self.level(true).await }
}
fn poll<F: Future>(future: core::pin::Pin<&mut F>) -> Poll<F::Output> {
    future.poll(&mut Context::from_waker(Waker::noop()))
}
fn tick() { embassy_time::MockDriver::get().advance(embassy_time::Duration::from_millis(1)); }

#[test]
fn transition_during_release_wait_is_not_lost() {
    embassy_time::MockDriver::get().reset();
    let a = Rc::new(Cell::new(false));
    let b = Rc::new(Cell::new(false));
    let mut encoder = RotaryEncoder::new(Pin(a.clone()), Pin(b), 0);
    assert_eq!(encoder.update(), Direction::None);
    {
        let mut f = pin!(encoder.read_keyboard_event());
        assert!(poll(f.as_mut()).is_pending());
        a.set(true);
        assert_eq!(poll(f.as_mut()), Poll::Ready(KeyboardEvent::rotary_encoder(0, Direction::CounterClockwise, true)));
    }
    {
        let mut release = pin!(encoder.read_keyboard_event());
        assert!(poll(release.as_mut()).is_pending());
        a.set(false); // Changes while no GPIO wait is armed.
        tick();
        assert_eq!(poll(release.as_mut()), Poll::Ready(KeyboardEvent::rotary_encoder(0, Direction::CounterClockwise, false)));
    }
    let mut next = pin!(encoder.read_keyboard_event());
    assert_eq!(poll(next.as_mut()), Poll::Ready(KeyboardEvent::rotary_encoder(0, Direction::Clockwise, true)));
}

#[test]
fn transitions_during_retry_waits_preserve_a_complete_detent() {
    embassy_time::MockDriver::get().reset();
    let a = Rc::new(Cell::new(false));
    let b = Rc::new(Cell::new(false));
    let mut encoder = RotaryEncoder::with_resolution(Pin(a.clone()), Pin(b.clone()), 4, false, 0);
    assert_eq!(encoder.update(), Direction::None);
    let mut f = pin!(encoder.read_keyboard_event());
    assert!(poll(f.as_mut()).is_pending());
    a.set(true);
    assert!(poll(f.as_mut()).is_pending());
    b.set(true); tick();
    assert!(poll(f.as_mut()).is_pending());
    a.set(false); tick();
    assert!(poll(f.as_mut()).is_pending());
    b.set(false); tick();
    assert_eq!(poll(f.as_mut()), Poll::Ready(KeyboardEvent::rotary_encoder(0, Direction::CounterClockwise, true)));
}
