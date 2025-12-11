# Hierarchical Timing Wheel (Rust)

An efficient and scalable implementation of a **hierarchical timing
wheel** in Rust, designed for managing large numbers of timers with
minimal overhead.\
The library offers a wide timing range thanks to its multi-level
structure and uses `VecDeque` for predictable, fast allocation.

## Features

- **Hierarchical structure** enabling virtually unlimited timer range
  through cascading levels.
- **Amortized constant-time** operations for scheduling and bucket
  rotation.
- Based on **VecDeque** for memory simplicity and predictability.
- No hashed wheel mechanisms, keeping the architecture transparent and
  easy to extend.
- Suitable for real-time systems, servers, simulations, games and
  distributed systems.

## Installation

Add to `Cargo.toml`:

``` toml
[dependencies]
hierarchical-timing-wheel = "0.1"
```

## Concept

A hierarchical timing wheel is composed of multiple levels.\
Each level contains a set of slots representing time intervals.\
If a timer does not fit the current level's range, it is placed in a
higher level.\
On each tick, timers are either executed or moved downward.

## Example

``` rust
use std::thread::sleep;

fn main() {
    let levels = 3;
    let slot_capacity = 16;
    let slots_per_level = 10;

    let mut wheel =
        timing_wheel::hierarchical(levels, slot_capacity, slots_per_level);

    let delay = 5;
    match wheel.schedule(delay, || {
        println!("Timer fired!");
    }) {
        Ok((level, slot)) => println!("Timer placed at level {}, slot {}", level, slot),
        Err(_) => panic!("Timer too large"),
    }

    for _ in 0..6 {
        let timers = wheel.tick(1);
        for timer in timers {
            timer();
        }
        sleep(Duration::from_millis(100));
    }
}
```

## API Overview

- `schedule(delay_ticks, timer)` schedules a timer (it can be anything, really).
- `tick(steps)` advances the wheel by specified number of steps and fires due timers.

## Performance

- Scheduling: amortized **O(1)**
- Tick: **O(K)** where K is the number of timers expiring at that
  moment
- Scales with a number of levels
- No allocations on the hot path except rare `VecDeque` growth

## Why hierarchical timing wheel?

- Extremely large timer ranges without structural complexity
- No hashing or dynamic bucket structures
- Easy to reason about and modify
- Stable performance even under a heavy load

## Testing

``` bash
cargo test
```

## License

MIT
