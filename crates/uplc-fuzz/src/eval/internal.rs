use uplc_turbo::{
    arena::Arena,
    binder::DeBruijn,
    machine::{
        cost_model::builtin_costs::{
            builtin_costs_v1::BuiltinCostsV1, builtin_costs_v2::BuiltinCostsV2,
            builtin_costs_v3::BuiltinCostsV3,
        },
        BuiltinSemantics, CostModel, EvalResult, ExBudget, MachineError, PlutusVersion,
    },
    program::Program,
};

use crate::seed::TermSeed;

/// Outcome classification for comparison.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Outcome {
    /// Evaluated to a value (stored as TermSeed for cross-arena comparison).
    Success(TermSeed),
    /// Explicit error term or machine error (not budget).
    EvaluationFailure(String),
    /// Ran out of CPU or memory budget.
    BudgetExceeded,
}

/// Budget consumed by an evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Budget {
    pub cpu: i64,
    pub mem: i64,
}

/// Result of evaluating a program with one engine.
#[derive(Debug, Clone)]
pub struct EngineResult {
    pub outcome: Outcome,
    pub budget: Budget,
    pub logs: Vec<String>,
}

fn classify<'a>(result: &EvalResult<'a, DeBruijn>) -> EngineResult {
    let outcome = match &result.term {
        Ok(term) => Outcome::Success(TermSeed::from_term(term)),
        Err(MachineError::OutOfExError(_)) => Outcome::BudgetExceeded,
        Err(e) => Outcome::EvaluationFailure(format!("{e}")),
    };
    EngineResult {
        outcome,
        budget: Budget {
            cpu: result.info.consumed_budget.cpu,
            mem: result.info.consumed_budget.mem,
        },
        logs: result.info.logs.clone(),
    }
}

/// Evaluate with the tree-walking CEK machine.
pub fn eval_cek<'a>(
    arena: &'a Arena,
    program: &'a Program<'a, DeBruijn>,
    plutus_version: PlutusVersion,
    initial_budget: ExBudget,
) -> EngineResult {
    let result = program.eval_version_budget(arena, plutus_version, initial_budget);
    classify(&result)
}

/// Evaluate with the bytecode VM.
pub fn eval_bytecode<'a>(
    arena: &'a Arena,
    program: &'a Program<'a, DeBruijn>,
    plutus_version: PlutusVersion,
    initial_budget: ExBudget,
) -> EngineResult {
    let compiled = uplc_turbo::bytecode::compiler::compile(
        (
            program.version.major(),
            program.version.minor(),
            program.version.patch(),
        ),
        program.term,
    );

    match plutus_version {
        PlutusVersion::V1 => {
            let result = uplc_turbo::bytecode::vm::execute(
                arena,
                &compiled,
                initial_budget,
                CostModel::<BuiltinCostsV1>::default(),
                BuiltinSemantics::V1,
            );
            classify(&result)
        }
        PlutusVersion::V2 => {
            let result = uplc_turbo::bytecode::vm::execute(
                arena,
                &compiled,
                initial_budget,
                CostModel::<BuiltinCostsV2>::default(),
                BuiltinSemantics::V1,
            );
            classify(&result)
        }
        PlutusVersion::V3 => {
            let result = uplc_turbo::bytecode::vm::execute(
                arena,
                &compiled,
                initial_budget,
                CostModel::<BuiltinCostsV3>::default(),
                BuiltinSemantics::V2,
            );
            classify(&result)
        }
    }
}
