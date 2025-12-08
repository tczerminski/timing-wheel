use crate::ScheduleError::DelayTooLarge;
use std::collections::VecDeque;
use std::time::Duration;

struct Ring<T> {
    cursor: usize,
    slots: Vec<VecDeque<T>>,
}

impl<T> Ring<T> {
    fn new(slot_capacity: usize, slots_per_level: usize) -> Ring<T> {
        Ring {
            cursor: 0,
            slots: (0..slots_per_level)
                .map(|_| VecDeque::with_capacity(slot_capacity))
                .collect(),
        }
    }

    fn tick(&mut self) -> Vec<T> {
        let cursor = self.cursor;
        self.cursor = (self.cursor + 1) % self.slots.len();
        self.slots[cursor].drain(..).collect()
    }

    fn place(&mut self, delay_ticks: usize, timer: T) -> usize {
        let slot = (self.cursor + delay_ticks) % self.slots.len();
        self.slots[slot].push_back(timer);
        slot
    }
}

pub struct HierarchicalTimingWheel<T> {
    tick_duration_ns: u128,
    rings: Vec<(usize, Ring<T>)>,
}

pub fn new<T>(
    tick_duration: Duration,
    slot_capacity: usize,
    slots_per_level: usize,
    levels: usize,
) -> HierarchicalTimingWheel<T> {
    HierarchicalTimingWheel::new(tick_duration, levels, slot_capacity, slots_per_level)
}

#[derive(Debug)]
pub enum ScheduleError {
    DelayTooLarge,
}

impl<T> HierarchicalTimingWheel<T> {
    fn new(
        tick_duration: Duration,
        levels: usize,
        slot_capacity: usize,
        slots_per_level: usize,
    ) -> HierarchicalTimingWheel<T> {
        let tick_duration_ns = tick_duration.as_nanos();
        Self {
            tick_duration_ns,
            rings: (0..levels)
                .map(|level| {
                    (
                        slots_per_level * (level + 1),
                        Ring::new(slot_capacity, slots_per_level),
                    )
                })
                .collect(),
        }
    }

    pub fn schedule(&mut self, delay: Duration, timer: T) -> Result<(usize, usize), ScheduleError> {
        let delay_ns = delay.as_nanos();
        let delay_ticks =
            (delay_ns + self.tick_duration_ns - 1) as usize / self.tick_duration_ns as usize;
        for (level, (threshold, ring)) in self.rings.iter_mut().enumerate() {
            if delay_ticks < *threshold {
                let slot = ring.place(delay_ticks, timer);
                return Ok((level, slot));
            }
        }
        Err(DelayTooLarge)
    }

    pub fn tick(&mut self) -> Vec<T> {
        let mut due = Vec::new();
        for (_, ring) in self.rings.iter_mut() {
            due.extend(ring.tick());
            if ring.cursor > 0 {
                break;
            }
        }
        due
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn wheel() -> HierarchicalTimingWheel<&'static str> {
        HierarchicalTimingWheel::new(Duration::from_nanos(100), 1, 16, 10)
    }

    #[test]
    fn test_single_timer_exact_tick() {
        let mut timing_wheel = wheel();
        timing_wheel
            .schedule(Duration::from_nanos(100), "A")
            .unwrap();
        assert_eq!(timing_wheel.tick(), Vec::<&str>::new());
        assert_eq!(timing_wheel.tick(), vec!["A"]);
    }

    #[test]
    fn test_timer_rounding_up_delay() {
        let mut timing_wheel = wheel();
        timing_wheel
            .schedule(Duration::from_nanos(101), "B")
            .unwrap();
        assert_eq!(timing_wheel.tick(), Vec::<&str>::new());
        assert_eq!(timing_wheel.tick(), Vec::<&str>::new());
        assert_eq!(timing_wheel.tick(), Vec::<&str>::from(["B"]));
    }

    #[test]
    fn test_multiple_timers_same_slot() {
        let mut timing_wheel = wheel();
        timing_wheel
            .schedule(Duration::from_nanos(100), "A")
            .unwrap();
        timing_wheel
            .schedule(Duration::from_nanos(100), "B")
            .unwrap();
        timing_wheel.tick();
        let mut out = timing_wheel.tick();
        out.sort();
        assert_eq!(out, vec!["A", "B"]);
    }

    #[test]
    fn test_wrapping_slots() {
        let mut timing_wheel = wheel();
        match timing_wheel.schedule(Duration::from_nanos(1100), "X") {
            Ok(_) => panic!("Expected Err"),
            _ => {}
        }
    }
}
