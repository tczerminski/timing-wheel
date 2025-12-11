use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn bench_schedule_slot_overload(c: &mut Criterion) {
    let delay = 100;

    c.bench_function("schedule_overload", |b| {
        b.iter(|| {
            let mut wheel = timing_wheel::hierarchical(16, 1024, 3);

            for i in 0..3072usize {
                black_box(wheel.schedule(delay / 2, black_box(i)).unwrap());
            }
        });
    });

    c.bench_function("tick_heavy", |b| {
        b.iter(|| {
            let mut wheel = timing_wheel::hierarchical(16, 1024, 3);

            for i in 0..3072usize {
                wheel.schedule(delay / 2, i).unwrap();
            }

            black_box(wheel.tick(100));
        });
    });
}

criterion_group!(benches, bench_schedule_slot_overload);
criterion_main!(benches);
