use criterion::criterion_main;

mod benchmarks;

criterion_main! {
    benchmarks::turbo,
    benchmarks::plutus_use_cases,
    benchmarks::add_integer,
    benchmarks::fibonacci,
}
