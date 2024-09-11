use criterion::criterion_main;

mod benchmarks;

criterion_main! {
    benchmarks::add_integer,
    benchmarks::fibonacci,
}
