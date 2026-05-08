//! Top-level UPLC program representation.
//!
//! A [`Program`] wraps a [`Term`] together with a UPLC [`Version`] tag
//! and exposes the primary evaluation methods. All allocations go through the shared
//! [`Arena`].

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

/// A versioned UPLC program ready for evaluation.
///
/// `V` is the variable-binding strategy (typically [`DeBruijn`](crate::binder::DeBruijn)).
/// All fields are arena-allocated references; the program lifetime `'a` ties back to the
/// [`Arena`] it was built in.
#[derive(Debug)]
pub struct Program<'a, V> {
    /// UPLC language version encoded with this program.
    pub version: &'a Version<'a>,
    /// The root term of the program.
    pub term: &'a Term<'a, V>,
}

impl<'a, V> Program<'a, V> {
    /// Allocates a new program with the given version and root term.
    pub fn new(arena: &'a Arena, version: &'a Version<'a>, term: &'a Term<'a, V>) -> &'a Self {
        let program = Program { version, term };

        arena.alloc(program)
    }

    /// Returns a new program whose root term is `self.term` applied to `term`.
    pub fn apply(&'a self, arena: &'a Arena, term: &'a Term<'a, V>) -> &'a Self {
        let term = self.term.apply(arena, term);

        Self::new(arena, self.version, term)
    }
}

impl<'a, V> Program<'a, V>
where
    V: Eval<'a>,
{
    /// Evaluate using Plutus V3 semantics and the default mainnet budget.
    #[must_use]
    pub fn eval(&'a self, arena: &'a Arena) -> EvalResult<'a, V> {
        self.eval_version(arena, PlutusVersion::V3)
    }

    /// Evaluate with the specified Plutus version and the default mainnet budget.
    #[must_use]
    pub fn eval_version(
        &'a self,
        arena: &'a Arena,
        plutus_version: PlutusVersion,
    ) -> EvalResult<'a, V> {
        self.eval_version_budget(arena, plutus_version, ExBudget::default())
    }

    /// Evaluate with an explicit Plutus version and a custom initial [`ExBudget`].
    #[must_use]
    pub fn eval_version_budget(
        &'a self,
        arena: &'a Arena,
        plutus_version: PlutusVersion,
        initial_budget: ExBudget,
    ) -> EvalResult<'a, V> {
        match plutus_version {
            PlutusVersion::V1 => self.evaluate(
                arena,
                CostModel::<BuiltinCostsV1>::default(),
                plutus_version,
                initial_budget,
            ),
            PlutusVersion::V2 => self.evaluate(
                arena,
                CostModel::<BuiltinCostsV2>::default(),
                plutus_version,
                initial_budget,
            ),
            PlutusVersion::V3 => self.evaluate(
                arena,
                CostModel::<BuiltinCostsV3>::default(),
                plutus_version,
                initial_budget,
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
            *self.version,
        );
        let term = machine.run(self.term);
        let info = machine.info();
        EvalResult { term, info }
    }

    /// Evaluate with a fully custom cost-model parameter array and execution budget.
    ///
    /// `cost_model` must be ordered as expected by the corresponding
    /// `BuiltinCostModel` implementation for `plutus_version`.
    #[must_use]
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

/// UPLC program version tag (`major.minor.patch`).
///
/// Encoded as a triple of unsigned integers in the Flat binary format.
/// Use the named constructors to obtain the canonical version for each Plutus era.
#[derive(Debug, Copy, Clone)]
pub struct Version<'a>(&'a (usize, usize, usize));

impl<'a> Version<'a> {
    /// Allocates a version with explicit `major.minor.patch` components.
    pub fn new(arena: &'a Arena, major: usize, minor: usize, patch: usize) -> &'a mut Self {
        let version = arena.alloc((major, minor, patch));

        arena.alloc(Version(version))
    }

    /// Canonical version for Plutus V1 scripts (1.0.0).
    pub fn plutus_v1(arena: &'a Arena) -> &'a mut Self {
        Self::new(arena, 1, 0, 0)
    }

    /// Canonical version for Plutus V2 scripts (1.0.0).
    pub fn plutus_v2(arena: &'a Arena) -> &'a mut Self {
        Self::new(arena, 1, 0, 0)
    }

    /// Canonical version for Plutus V3 scripts (1.1.0).
    pub fn plutus_v3(arena: &'a Arena) -> &'a mut Self {
        Self::new(arena, 1, 1, 0)
    }

    /// Returns `true` if this version is `1.0.0`.
    pub fn is_v1_0_0(&'a self) -> bool {
        self.0 == &(1, 0, 0)
    }

    /// Returns `true` if this version is `1.1.0`.
    pub fn is_v1_1_0(&'a self) -> bool {
        self.0 == &(1, 1, 0)
    }

    /// Returns `true` if this version is a known valid UPLC version.
    pub fn is_valid_version(&'a self) -> bool {
        self.is_v1_0_0() || self.is_v1_1_0()
    }

    /// Returns `true` if this version is below `1.1.0`.
    pub fn is_less_than_1_1_0(&'a self) -> bool {
        self.0 < &(1, 1, 0)
    }

    pub fn is_at_least_1_1_0(&'a self) -> bool {
        self.0 >= &(1, 1, 0)
    }

    /// Returns the major component.
    pub fn major(&'a self) -> usize {
        self.0 .0
    }

    /// Returns the minor component.
    pub fn minor(&'a self) -> usize {
        self.0 .1
    }

    /// Returns the patch component.
    pub fn patch(&'a self) -> usize {
        self.0 .2
    }
}
