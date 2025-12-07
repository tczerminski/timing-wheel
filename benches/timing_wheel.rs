use criterion::{Criterion, criterion_group, criterion_main};
use std::time::Duration;

fn bench_schedule_slot_overload(c: &mut Criterion) {
    let delay = Duration::from_micros(100);
    let mut timing_wheel = timing_wheel::new(delay, 16, 1024);
    c.bench_function("tick", |b| {
        b.iter(|| {
            for i in 0..1024 {
                timing_wheel.schedule(delay / 2, i);
            }
            timing_wheel.tick();
        })
    });
}

criterion_group!(benches, bench_schedule_slot_overload);
criterion_main!(benches);
