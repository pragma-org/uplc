use criterion::criterion_main;

mod simple;

criterion_main! {
    simple::add_integer,
    simple::fibonacci,
}
