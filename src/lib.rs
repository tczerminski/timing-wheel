use std::collections::VecDeque;
use std::time::Duration;

pub struct TimingWheel<T> {
    tick_duration_ns: u128,
    cursor: usize,
    slots: Vec<VecDeque<T>>,
}

pub fn new<T>(
    tick_duration: Duration,
    slot_capacity: usize,
    slots_per_level: u128,
) -> TimingWheel<T> {
    TimingWheel::new(tick_duration, slot_capacity, slots_per_level)
}

impl<T> TimingWheel<T> {
    fn new(tick_duration: Duration, slot_capacity: usize, slots_per_level: u128) -> TimingWheel<T> {
        Self {
            tick_duration_ns: tick_duration.as_nanos(),
            cursor: 0,
            slots: (0..slots_per_level)
                .map(|_| VecDeque::with_capacity(slot_capacity))
                .collect(),
        }
    }

    pub fn schedule(&mut self, delay: Duration, timer: T) {
        let delay_ns = delay.as_nanos();
        let delay_ticks = (delay_ns + self.tick_duration_ns - 1) / self.tick_duration_ns;
        let slot = (self.cursor + delay_ticks as usize) % self.slots.len();
        self.slots[slot].push_back(timer);
    }

    pub fn tick(&mut self) -> Vec<T> {
        let cursor = self.cursor;
        self.cursor = (self.cursor + 1) % self.slots.len();
        self.slots[cursor].drain(..).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn wheel() -> TimingWheel<&'static str> {
        TimingWheel::new(Duration::from_nanos(100), 16, 10)
    }

    #[test]
    fn test_single_timer_exact_tick() {
        let mut timing_wheel = wheel();
        timing_wheel.schedule(Duration::from_nanos(100), "A");
        assert_eq!(timing_wheel.tick(), Vec::<&str>::new());
        assert_eq!(timing_wheel.tick(), vec!["A"]);
    }

    #[test]
    fn test_timer_rounding_up_delay() {
        let mut timing_wheel = wheel();
        timing_wheel.schedule(Duration::from_nanos(101), "B");
        assert_eq!(timing_wheel.tick(), Vec::<&str>::new());
        assert_eq!(timing_wheel.tick(), Vec::<&str>::new());
        assert_eq!(timing_wheel.tick(), Vec::<&str>::from(["B"]));
    }

    #[test]
    fn test_multiple_timers_same_slot() {
        let mut timing_wheel = wheel();
        timing_wheel.schedule(Duration::from_nanos(100), "A");
        timing_wheel.schedule(Duration::from_nanos(100), "B");
        timing_wheel.tick();
        let mut out = timing_wheel.tick();
        out.sort();
        assert_eq!(out, vec!["A", "B"]);
    }

    #[test]
    fn test_wrapping_slots() {
        let mut timing_wheel = wheel();
        timing_wheel.schedule(Duration::from_nanos(1100), "X");
        assert_eq!(timing_wheel.tick(), Vec::<&str>::new());
        assert_eq!(timing_wheel.tick(), Vec::<&str>::from(["X"]));
    }
}
