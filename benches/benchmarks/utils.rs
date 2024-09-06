use bumpalo::Bump;
use ouroboros::self_referencing;

use uplc::{
    program::{Program, Version},
    term::Term,
};

#[self_referencing]
pub struct BenchState {
    pub arena: Bump,
    #[borrows(arena)]
    #[covariant]
    pub program: &'this Program<'this>,
}

impl BenchState {
    #[inline]
    pub fn exec(&self) {
        self.with_program(|program| {
            self.with_arena(|arena| {
                let _ = program.eval(arena);
            });
        });
    }
}

pub fn setup_program<F>(program_builder: F) -> BenchState
where
    F: for<'this> FnOnce(&'this Bump) -> &'this Program<'this>,
{
    let arena = Bump::new();

    let builder = BenchStateBuilder {
        arena,
        program_builder,
    };

    builder.build()
}

#[inline]
pub fn setup_term<F>(term_builder: F) -> BenchState
where
    F: for<'this> FnOnce(&'this Bump) -> &'this Term<'this>,
{
    let arena = Bump::new();

    let builder = BenchStateBuilder {
        arena,
        program_builder: |arena| {
            let term = term_builder(arena);

            let version = Version::plutus_v3(arena);

            Program::new(arena, version, term)
        },
    };

    builder.build()
}
