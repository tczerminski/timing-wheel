use std::collections::VecDeque;

pub fn hierarchical<T>(
    levels: u32,
    slot_capacity: usize,
    slots_per_level: usize,
) -> HierarchicalTimingWheel<T> {
    HierarchicalTimingWheel::new(levels, slot_capacity, slots_per_level)
}

struct Ring<T> {
    level: u32,
    cursor: usize,
    slots: Vec<VecDeque<(usize, T)>>,
}

impl<T> Ring<T> {
    fn new(level: u32, slot_capacity: usize, slots_per_level: usize) -> Ring<T> {
        Ring {
            level,
            cursor: 0,
            slots: (0..slots_per_level)
                .map(|_| VecDeque::with_capacity(slot_capacity))
                .collect(),
        }
    }

    #[inline]
    fn span(&self) -> usize {
        self.slots.len().pow(self.level)
    }

    #[inline]
    fn capacity(&self) -> usize {
        self.slots.len().pow(self.level + 1)
    }

    fn tick(&mut self) -> Vec<(usize, T)> {
        self.cursor = (self.cursor + 1) % self.slots.len();
        self.slots[self.cursor].drain(..).collect()
    }

    fn place(&mut self, remaining: usize, timer: T) -> usize {
        let slot_offset = remaining / self.span();
        let slot = (self.cursor + slot_offset) % self.slots.len();
        let adjusted_remaining = remaining % self.span();
        self.slots[slot].push_back((adjusted_remaining, timer));
        slot
    }
}

pub struct HierarchicalTimingWheel<T> {
    rings: Vec<Ring<T>>,
}

#[derive(Debug)]
pub enum ScheduleError {
    DelayTooLarge,
}

impl<T> HierarchicalTimingWheel<T> {
    pub(crate) fn new(
        levels: u32,
        slot_capacity: usize,
        slots_per_level: usize,
    ) -> HierarchicalTimingWheel<T> {
        let mut rings: Vec<Ring<T>> = Vec::new();
        for level in 0..levels {
            rings.push(Ring::new(level, slot_capacity, slots_per_level))
        }
        Self { rings }
    }

    pub fn schedule(
        &mut self,
        delay_ticks: usize,
        timer: T,
    ) -> Result<(usize, usize), ScheduleError> {
        let delay_ticks = (delay_ticks == 0) as usize | delay_ticks;
        for (level, ring) in self.rings.iter_mut().enumerate() {
            if delay_ticks < ring.capacity() && delay_ticks >= ring.span() {
                let slot = ring.place(delay_ticks, timer);
                return Ok((level, slot));
            }
        }
        Err(ScheduleError::DelayTooLarge)
    }

    pub fn tick(&mut self, steps: usize) -> Vec<T> {
        let mut due = Vec::new();
        for _ in 0..steps {
            let mut graduated = Vec::new();
            let mut i = 0;
            let mut inner_ticked = false;
            loop {
                let should_tick = i == 0 || (inner_ticked && self.rings[i - 1].cursor == 0);
                if should_tick {
                    let ring = &mut self.rings[i];
                    let timers = ring.tick();
                    if i == 0 {
                        due.extend(timers.into_iter().map(|(_, t)| t));
                    } else {
                        graduated.extend(timers);
                    }
                }
                inner_ticked = should_tick;
                i += 1;
                if i == self.rings.len() {
                    break;
                }
            }
            for (remaining_delay, timer) in graduated {
                if remaining_delay == 0 {
                    due.push(timer);
                } else {
                    self.schedule(remaining_delay, timer).unwrap();
                }
            }
        }
        due
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_timer_exact_tick() {
        let mut timing_wheel = HierarchicalTimingWheel::new(1, 16, 10);
        let (level, slot) = timing_wheel.schedule(1, "A").unwrap();
        assert_eq!(level, 0);
        assert_eq!(slot, 1);
        assert_eq!(timing_wheel.tick(1), vec!["A"]);
    }

    #[test]
    fn test_delay_zero() {
        let mut timing_wheel = HierarchicalTimingWheel::new(1, 16, 10);
        let (level, slot) = timing_wheel.schedule(0, "A").unwrap();
        assert_eq!(level, 0);
        assert_eq!(slot, 1);
        assert_eq!(timing_wheel.tick(1), vec!["A"]);
    }

    #[test]
    fn test_timer_rounding_up_delay() {
        let mut timing_wheel = HierarchicalTimingWheel::new(1, 16, 10);
        let (level, slot) = timing_wheel.schedule(9, "B").unwrap();
        assert_eq!(level, 0);
        assert_eq!(slot, 9);
        for _ in 0..8 {
            assert_eq!(timing_wheel.tick(1), Vec::<&str>::new());
        }
        assert_eq!(timing_wheel.tick(1), Vec::<&str>::from(["B"]));
    }

    #[test]
    fn test_multiple_timers_same_slot() {
        let mut timing_wheel = HierarchicalTimingWheel::new(1, 16, 10);
        let (level, slot) = timing_wheel.schedule(1, "A").unwrap();
        assert_eq!(level, 0);
        assert_eq!(slot, 1);
        let (level, slot) = timing_wheel.schedule(1, "B").unwrap();
        assert_eq!(level, 0);
        assert_eq!(slot, 1);
        let mut out = timing_wheel.tick(1);
        out.sort();
        assert_eq!(out, vec!["A", "B"]);
    }

    #[test]
    fn test_overflow() {
        let mut timing_wheel = HierarchicalTimingWheel::new(1, 16, 10);
        match timing_wheel.schedule(10, "X") {
            Ok(_) => panic!("Expected Err"),
            _ => {}
        }
    }

    #[test]
    fn test_exact_boundary_between_levels() {
        let mut wheel = HierarchicalTimingWheel::new(3, 16, 10);

        let (level, slot) = wheel.schedule(9, "L0").unwrap();
        assert_eq!(level, 0);
        assert_eq!(slot, 9);

        let (level, slot) = wheel.schedule(10, "L1").unwrap();
        assert_eq!(level, 1);
        assert_eq!(slot, 1);

        let (level, slot) = wheel.schedule(99, "L2").unwrap();
        assert_eq!(level, 1);
        assert_eq!(slot, 9);

        let (level, slot) = wheel.schedule(100, "L3").unwrap();
        assert_eq!(level, 2);
        assert_eq!(slot, 1);

        let (level, slot) = wheel.schedule(999, "L4").unwrap();
        assert_eq!(level, 2);
        assert_eq!(slot, 9);

        assert!(wheel.tick(8).is_empty());

        let timers = wheel.tick(1);
        assert_eq!(timers, vec!["L0"]);
        let timers = wheel.tick(1);
        assert_eq!(timers, vec!["L1"]);

        assert!(wheel.tick(88).is_empty());

        let timers = wheel.tick(1);
        assert_eq!(timers, vec!["L2"]);
        let timers = wheel.tick(1);
        assert_eq!(timers, vec!["L3"]);

        assert!(wheel.tick(898).is_empty());

        let timers = wheel.tick(1);
        assert_eq!(timers, vec!["L4"]);
    }
}
