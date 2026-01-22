use crate::{
    arena::Arena,
    binder::Eval,
    machine::{
        cost_model::builtin_costs::{
            builtin_costs_v1::BuiltinCostsV1, builtin_costs_v2::BuiltinCostsV2,
            builtin_costs_v3::BuiltinCostsV3, BuiltinCostModel,
        },
        BuiltinSemantics, CostModel, EvalResult, ExBudget, Machine, PlutusVersion,
    },
    term::Term,
};

#[derive(Debug)]
pub struct Program<'a, V> {
    pub version: &'a Version<'a>,
    pub term: &'a Term<'a, V>,
}

impl<'a, V> Program<'a, V> {
    pub fn new(arena: &'a Arena, version: &'a Version<'a>, term: &'a Term<'a, V>) -> &'a Self {
        let program = Program { version, term };

        arena.alloc(program)
    }

    pub fn apply(&'a self, arena: &'a Arena, term: &'a Term<'a, V>) -> &'a Self {
        let term = self.term.apply(arena, term);

        Self::new(arena, self.version, term)
    }
}

impl<'a, V> Program<'a, V>
where
    V: Eval<'a>,
{
    pub fn eval(&'a self, arena: &'a Arena) -> EvalResult<'a, V> {
        self.eval_version(arena, PlutusVersion::V3)
    }

    /// Evaluate with explicit Plutus version
    pub fn eval_version(
        &'a self,
        arena: &'a Arena,
        plutus_version: PlutusVersion,
    ) -> EvalResult<'a, V> {
        match plutus_version {
            PlutusVersion::V1 => self.evaluate(
                arena,
                CostModel::<BuiltinCostsV1>::default(),
                plutus_version,
                ExBudget::default(),
            ),
            PlutusVersion::V2 => self.evaluate(
                arena,
                CostModel::<BuiltinCostsV2>::default(),
                plutus_version,
                ExBudget::default(),
            ),
            PlutusVersion::V3 => self.evaluate(
                arena,
                CostModel::<BuiltinCostsV3>::default(),
                plutus_version,
                ExBudget::default(),
            ),
        }
    }

    fn evaluate<B: BuiltinCostModel>(
        &'a self,
        arena: &'a Arena,
        cost_model: CostModel<B>,
        plutus_version: PlutusVersion,
        initial_budget: ExBudget,
    ) -> EvalResult<'a, V> {
        let mut machine = Machine::new(
            arena,
            initial_budget,
            cost_model,
            BuiltinSemantics::from(&plutus_version),
        );
        let term = machine.run(self.term);
        let mut info = machine.info();
        info.consumed_budget = initial_budget - info.consumed_budget;
        EvalResult { term, info }
    }

    pub fn eval_with_params(
        &'a self,
        arena: &'a Arena,
        plutus_version: PlutusVersion,
        cost_model: &[i64],
        initial_budget: ExBudget,
    ) -> EvalResult<'a, V> {
        match plutus_version {
            PlutusVersion::V1 => self.evaluate(
                arena,
                CostModel::<BuiltinCostsV1>::initialize_cost_model(&plutus_version, cost_model),
                plutus_version,
                initial_budget,
            ),
            PlutusVersion::V2 => self.evaluate(
                arena,
                CostModel::<BuiltinCostsV2>::initialize_cost_model(&plutus_version, cost_model),
                plutus_version,
                initial_budget,
            ),
            PlutusVersion::V3 => self.evaluate(
                arena,
                CostModel::<BuiltinCostsV3>::initialize_cost_model(&plutus_version, cost_model),
                plutus_version,
                initial_budget,
            ),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Version<'a>(&'a (usize, usize, usize));

impl<'a> Version<'a> {
    pub fn new(arena: &'a Arena, major: usize, minor: usize, patch: usize) -> &'a mut Self {
        let version = arena.alloc((major, minor, patch));

        arena.alloc(Version(version))
    }

    pub fn plutus_v1(arena: &'a Arena) -> &'a mut Self {
        Self::new(arena, 1, 0, 0)
    }

    pub fn plutus_v2(arena: &'a Arena) -> &'a mut Self {
        Self::new(arena, 1, 0, 0)
    }

    pub fn plutus_v3(arena: &'a Arena) -> &'a mut Self {
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

    pub fn is_less_than_1_1_0(&'a self) -> bool {
        self.0 .0 == 0 || self.0 .1 == 0
    }

    pub fn major(&'a self) -> usize {
        self.0 .0
    }

    pub fn minor(&'a self) -> usize {
        self.0 .1
    }

    pub fn patch(&'a self) -> usize {
        self.0 .2
    }
}
