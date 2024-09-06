use bumpalo::Bump;
use criterion::{criterion_group, Criterion};

use uplc::term::Term;

use super::utils;

pub fn run(c: &mut Criterion) {
    c.bench_function("fibonacci", |b| {
        b.iter_with_setup(
            || {
                utils::setup_term(|arena: &Bump| {
                    let double_force = Term::var(arena, 1)
                        .apply(arena, Term::var(arena, 1))
                        .lambda(arena, 0)
                        .delay(arena)
                        .force(arena)
                        .apply(
                            arena,
                            Term::var(arena, 3)
                                .apply(
                                    arena,
                                    Term::var(arena, 1)
                                        .apply(arena, Term::var(arena, 1))
                                        .lambda(arena, 0)
                                        .delay(arena)
                                        .force(arena)
                                        .apply(arena, Term::var(arena, 2)),
                                )
                                .apply(arena, Term::var(arena, 1))
                                .lambda(arena, 0)
                                .lambda(arena, 0),
                        )
                        .lambda(arena, 0)
                        .delay(arena)
                        .delay(arena)
                        .force(arena)
                        .force(arena);

                    let if_condition = Term::if_then_else(arena)
                        .force(arena)
                        .apply(arena, Term::var(arena, 3))
                        .apply(arena, Term::var(arena, 2))
                        .apply(arena, Term::var(arena, 1))
                        .apply(arena, Term::unit(arena))
                        .lambda(arena, 0)
                        .lambda(arena, 0)
                        .lambda(arena, 0)
                        .delay(arena)
                        .force(arena);

                    let add = Term::add_integer(arena)
                        .apply(
                            arena,
                            Term::var(arena, 3).apply(
                                arena,
                                Term::subtract_integer(arena)
                                    .apply(arena, Term::var(arena, 2))
                                    .apply(arena, Term::integer_from(arena, 1)),
                            ),
                        )
                        .apply(
                            arena,
                            Term::var(arena, 3).apply(
                                arena,
                                Term::subtract_integer(arena)
                                    .apply(arena, Term::var(arena, 2))
                                    .apply(arena, Term::integer_from(arena, 2)),
                            ),
                        )
                        .lambda(arena, 0);

                    double_force
                        .apply(
                            arena,
                            if_condition
                                .apply(
                                    arena,
                                    Term::less_than_equals_integer(arena)
                                        .apply(arena, Term::var(arena, 1))
                                        .apply(arena, Term::integer_from(arena, 1)),
                                )
                                .apply(arena, Term::var(arena, 2).lambda(arena, 0))
                                .apply(arena, add)
                                .lambda(arena, 0)
                                .lambda(arena, 0),
                        )
                        .apply(arena, Term::var(arena, 1))
                        .lambda(arena, 0)
                        .apply(arena, Term::integer_from(arena, 20))
                })
            },
            // Benchmark: only the eval call
            |state| state.exec(),
        )
    });
}

criterion_group!(fibonacci, run);
