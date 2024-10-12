use bumpalo::Bump;

use crate::{
    machine::{BuiltinSemantics, CostModel, EvalResult, ExBudget, Machine},
    term::Term,
};

#[derive(Debug)]
pub struct Program<'a> {
    pub version: &'a Version<'a>,
    pub term: &'a Term<'a>,
}

impl<'a> Program<'a> {
    pub fn new(arena: &'a Bump, version: &'a Version<'a>, term: &'a Term<'a>) -> &'a Self {
        let program = Program { version, term };

        arena.alloc(program)
    }

    pub fn eval(&'a self, arena: &'a Bump) -> EvalResult<'a> {
        let mut machine = Machine::new(
            arena,
            ExBudget::default(),
            CostModel::default(),
            // TODO: I think we may actually need
            // to derive this from the plutus version?
            // maybe not though
            if self.version.is_v1_1_0() {
                BuiltinSemantics::V2
            } else {
                BuiltinSemantics::V1
            },
        );

        let term = machine.run(self.term);
        let mut info = machine.info();

        info.consumed_budget = ExBudget::default() - info.consumed_budget;

        EvalResult { term, info }
    }

    pub fn apply(&'a self, arena: &'a Bump, term: &'a Term<'a>) -> &'a Self {
        let term = self.term.apply(arena, term);

        Self::new(arena, self.version, term)
    }
}

#[derive(Debug)]
pub struct Version<'a>(&'a (usize, usize, usize));

impl<'a> Version<'a> {
    pub fn new(arena: &'a Bump, major: usize, minor: usize, patch: usize) -> &'a mut Self {
        let version = arena.alloc((major, minor, patch));

        arena.alloc(Version(version))
    }

    pub fn plutus_v1(arena: &'a Bump) -> &'a mut Self {
        Self::new(arena, 1, 0, 0)
    }

    pub fn plutus_v2(arena: &'a Bump) -> &'a mut Self {
        Self::new(arena, 1, 0, 0)
    }

    pub fn plutus_v3(arena: &'a Bump) -> &'a mut Self {
        Self::new(arena, 1, 1, 0)
    }

    pub fn is_v1_0_0(&'a self) -> bool {
        self.0 .0 == 1 && self.0 .1 == 0 && self.0 .2 == 0
    }

    pub fn is_v1_1_0(&'a self) -> bool {
        self.0 .0 == 1 && self.0 .1 == 1 && self.0 .2 == 0
    }

    pub fn is_valid_version(&'a self) -> bool {
        self.is_v1_0_0() || self.is_v1_1_0()
    }
}
