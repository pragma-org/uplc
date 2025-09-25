use bumpalo::{collections::Vec as BumpVec, Bump};

use crate::{
    binder::Eval,
    constant::Constant,
    machine::{context::Context, env::Env, state::MachineState},
    term::Term,
};

use super::{
    cost_model::StepKind,
    discharge,
    info::MachineInfo,
    runtime::{BuiltinSemantics, Runtime},
    value::Value,
    CostModel, ExBudget, MachineError,
};

pub const CONS_BRANCH: usize = 0;
pub const NILS_BRANCH: usize = 1;

pub struct Machine<'a> {
    pub(super) arena: &'a Bump,
    ex_budget: ExBudget,
    unbudgeted_steps: [u8; 10],
    pub(super) costs: CostModel,
    slippage: u8,
    pub(super) logs: Vec<String>,
    pub(super) semantics: BuiltinSemantics,
}

impl<'a> Machine<'a> {
    pub fn new(
        arena: &'a Bump,
        initial_budget: ExBudget,
        costs: CostModel,
        semantics: BuiltinSemantics,
    ) -> Self {
        Machine {
            arena,
            ex_budget: initial_budget,
            unbudgeted_steps: [0; 10],
            costs,
            slippage: 200,
            logs: Vec::new(),
            semantics,
        }
    }

    pub fn info(self) -> MachineInfo {
        MachineInfo {
            consumed_budget: self.ex_budget,
            logs: self.logs,
        }
    }

    pub fn run<V>(&mut self, term: &'a Term<'a, V>) -> Result<&'a Term<'a, V>, MachineError<'a, V>>
    where
        V: Eval<'a>,
    {
        self.spend_budget(ExBudget::start_up())?;

        let initial_context = Context::no_frame(self.arena);

        let mut state =
            MachineState::compute(self.arena, initial_context, Env::new_in(self.arena), term);

        loop {
            let step = match state {
                MachineState::Compute(context, env, term) => self.compute(context, env, term),
                MachineState::Return(context, value) => self.return_compute(context, value),
                MachineState::Done(term) => {
                    return Ok(term);
                }
            };

            state = step?;
        }
    }

    pub fn compute<V>(
        &mut self,
        context: &'a Context<'a, V>,
        env: &'a Env<'a, V>,
        term: &'a Term<'a, V>,
    ) -> Result<&'a mut MachineState<'a, V>, MachineError<'a, V>>
    where
        V: Eval<'a>,
    {
        match term {
            Term::Var(name) => {
                self.step_and_maybe_spend(StepKind::Var)?;

                let value = env
                    .lookup(name.index())
                    .ok_or(MachineError::OpenTermEvaluated(term))?;

                let state = MachineState::return_(self.arena, context, value);

                Ok(state)
            }
            Term::Lambda { parameter, body } => {
                self.step_and_maybe_spend(StepKind::Lambda)?;

                let value = Value::lambda(self.arena, *parameter, body, env);

                let state = MachineState::return_(self.arena, context, value);

                Ok(state)
            }
            Term::Apply { function, argument } => {
                self.step_and_maybe_spend(StepKind::Apply)?;

                let frame = Context::frame_await_fun_term(self.arena, env, argument, context);

                let state = MachineState::compute(self.arena, frame, env, function);

                Ok(state)
            }
            Term::Delay(body) => {
                self.step_and_maybe_spend(StepKind::Delay)?;

                let value = Value::delay(self.arena, body, env);

                let state = MachineState::return_(self.arena, context, value);

                Ok(state)
            }
            Term::Force(body) => {
                self.step_and_maybe_spend(StepKind::Force)?;

                let frame = Context::frame_force(self.arena, context);

                let state = MachineState::compute(self.arena, frame, env, body);

                Ok(state)
            }
            Term::Constr { tag, fields } => {
                self.step_and_maybe_spend(StepKind::Constr)?;

                if let Some((first, terms)) = fields.split_first() {
                    let frame = Context::frame_constr_empty(self.arena, env, *tag, terms, context);

                    let state = MachineState::compute(self.arena, frame, env, first);

                    Ok(state)
                } else {
                    let value = Value::constr_empty(self.arena, *tag);

                    let state = MachineState::return_(self.arena, context, value);

                    Ok(state)
                }
            }
            Term::Case { constr, branches } => {
                self.step_and_maybe_spend(StepKind::Case)?;

                let frame = Context::frame_cases(self.arena, env, branches, context);

                let state = MachineState::compute(self.arena, frame, env, constr);

                Ok(state)
            }
            Term::Constant(constant) => {
                self.step_and_maybe_spend(StepKind::Constant)?;

                let value = Value::con(self.arena, constant);

                let state = MachineState::return_(self.arena, context, value);

                Ok(state)
            }
            Term::Builtin(fun) => {
                self.step_and_maybe_spend(StepKind::Builtin)?;

                let runtime = Runtime::new(self.arena, fun);

                let value = Value::builtin(self.arena, runtime);

                let state = MachineState::return_(self.arena, context, value);

                Ok(state)
            }
            Term::Error => Err(MachineError::ExplicitErrorTerm),
        }
    }

    pub fn return_compute<V>(
        &mut self,
        context: &'a Context<'a, V>,
        value: &'a Value<'a, V>,
    ) -> Result<&'a mut MachineState<'a, V>, MachineError<'a, V>>
    where
        V: Eval<'a>,
    {
        match context {
            Context::FrameAwaitFunTerm(arg_env, argument, context) => {
                let context = Context::frame_await_arg(self.arena, value, context);

                let state = MachineState::compute(self.arena, context, arg_env, argument);

                Ok(state)
            }
            Context::FrameAwaitArg(function, context) => {
                self.apply_evaluate(context, function, value)
            }
            Context::FrameAwaitFunValue(argument, context) => {
                self.apply_evaluate(context, value, argument)
            }
            Context::FrameForce(context) => self.force_evaluate(context, value),
            Context::FrameConstr(env, tag, terms, values, context) => {
                let mut new_values = BumpVec::with_capacity_in(values.len() + 1, self.arena);

                for value in values.iter() {
                    new_values.push(*value);
                }

                new_values.push(value);

                let values = self.arena.alloc(new_values);

                if let Some((first, terms)) = terms.split_first() {
                    let frame =
                        Context::frame_constr(self.arena, env, *tag, terms, values, context);

                    let state = MachineState::compute(self.arena, frame, env, first);

                    Ok(state)
                } else {
                    let value = Value::constr(self.arena, *tag, values);

                    let state = MachineState::return_(self.arena, context, value);

                    Ok(state)
                }
            }
            Context::FrameCases(env, branches, context) => match value {
                Value::Constr(tag, fields) => {
                    // kind of awkward tbh cause tag is already usize so how can it be bigger
                    // than the max
                    // if *tag > usize::MAX {
                    //     return Err(MachineError::MaxConstrTagExceeded(value));
                    // }

                    if let Some(branch) = branches.get(*tag) {
                        let frame = self.transfer_arg_stack(fields, context);

                        let state = MachineState::compute(self.arena, frame, env, branch);

                        Ok(state)
                    } else {
                        Err(MachineError::MissingCaseBranch(branches, value))
                    }
                }
                Value::Con(constant) => match constant {
                    Constant::Integer(scrutinee) => match usize::try_from(*scrutinee) {
                        Ok(scrutinee_usize) => branches
                            .get(scrutinee_usize)
                            .map(|branch| MachineState::compute(self.arena, context, env, branch))
                            .ok_or(MachineError::CekCaseBuiltinError(branches, value, format!("case {:?} is out of bounds for the given number of branches: {:?}", value, branches.len()))),
                        Err(_) => Err(MachineError::CekCaseBuiltinError(branches, value, format!("case {:?} is out of bounds for the given number of branches: {:?}", value, branches.len())))
                    },
                    Constant::Boolean(scrutinee) => {
                        if branches.len() > 2 {
                            return Err(MachineError::CekCaseBuiltinError(branches, value, "Casing on bool requires exactly one branch or two branches".to_string()));
                        }
                        branches
                            .get(*scrutinee as usize)
                            .map(|branch| MachineState::compute(self.arena, context, env, branch))
                            .ok_or(MachineError::CekCaseBuiltinError(branches, value, format!("case {:?} is out of bounds for the given number of branches: {:?}", value, branches.len())))
                    }
                    Constant::Unit => {
                        if branches.len() > 1 {
                            return Err(MachineError::CekCaseBuiltinError(branches, value, "Casing on unit only allows exactly one branch".to_string()));
                        }
                        branches
                            .get(CONS_BRANCH)
                            .map(|branch| MachineState::compute(self.arena, context, env, branch))
                            .ok_or(MachineError::CekCaseBuiltinError(branches, value, format!("case {:?} is out of bounds for the given number of branches: {:?}", value, branches.len())))
                    }
                    // Caseing on pairs expects a single branch that takes two arguments for each values of the pair.
                    Constant::ProtoPair(_, _, left_constant, right_constant) => {
                        if branches.len() > 1 {
                            return Err(MachineError::CekCaseBuiltinError(branches, value, "Casing on pair requires exactly one branch".to_string()));
                        }
                        branches
                            .get(CONS_BRANCH)
                            .map(|branch| {
                                let right_value: &Value<'_, V> =
                                    Value::con(self.arena, right_constant);
                                let right_frame: &Context<'_, V> = Context::frame_await_fun_value(
                                    self.arena,
                                    right_value,
                                    context,
                                );
                                let left_value = Value::con(self.arena, left_constant);
                                let left_frame = Context::frame_await_fun_value(
                                    self.arena,
                                    left_value,
                                    right_frame,
                                );
                                MachineState::compute(self.arena, left_frame, env, branch)
                            })
                            .ok_or(MachineError::CekCaseBuiltinError(branches, value, format!("case {:?} is out of bounds for the given number of branches: {:?}", value, branches.len())))
                    }
                    // When matching (case-ing) on a builtin list, exactly one or two branches are allowed:
                    // - With a single branch, it is assumed the list is non-empty; the branch receives the head and tail as arguments.
                    //   If the list is actually empty, script evaluation will fail.
                    // - With two branches, the nils branch is selected for the empty list (receiving no arguments),
                    //   and the cons branch is selected for a non-empty list (receiving the head and tail as arguments).
                    //
                    // Note: In the Haskell implementation, when a list contains only a single element,
                    // the tail argument passed to the branch is an empty list.
                    Constant::ProtoList(list_type, list) => {
                        if branches.len() > 2 {
                            return Err(MachineError::CekCaseBuiltinError(branches, value, "Casing on list requires exactly one branch or two branches".to_string()));
                        }

                        match list.split_first() {
                            None => {
                                branches
                                .get(NILS_BRANCH)
                                .map(|branch| {
                                    let frame = self.transfer_arg_stack(&[], context);
                                    MachineState::compute(self.arena, frame, env, branch)
                                })
                                .ok_or(MachineError::CekCaseBuiltinError(branches, value, format!("case {:?} is out of bounds for the given number of branches: {:?}", value, branches.len())))
                            }
                            Some((head, tail)) => {
                                branches
                                .get(CONS_BRANCH)
                                .map(|branch| {
                                    let tail_value = if tail.is_empty() {
                                        let empty_list_const =
                                            Constant::proto_list(self.arena, list_type, &[]);
                                        Value::con(self.arena, empty_list_const)
                                    } else {
                                        Value::con(self.arena, tail.last().unwrap())
                                    };

                                    let tail_frame = Context::frame_await_fun_value(
                                        self.arena, tail_value, context,
                                    );
                                    let head_value = Value::con(self.arena, head);
                                    let head_frame = Context::frame_await_fun_value(
                                        self.arena, head_value, tail_frame,
                                    );

                                    MachineState::compute(self.arena, head_frame, env, branch)
                                })
                                .ok_or(MachineError::CekCaseBuiltinError(branches, value, format!("case {:?} is out of bounds for the given number of branches: {:?}", value, branches.len())))
                            }
                        }
                    }
                    _ => Err(MachineError::CekCaseBuiltinError(branches, value, format!("Cannot case on constant of type {constant:?}"))),
                },
                v => Err(MachineError::NonConstrScrutinized(v)),
            },
            Context::NoFrame => {
                if self.unbudgeted_steps[9] > 0 {
                    self.spend_unbudgeted_steps()?;
                }

                let term = discharge::value_as_term(self.arena, value);

                let state = MachineState::done(self.arena, term);

                Ok(state)
            }
        }
    }

    fn force_evaluate<V>(
        &mut self,
        context: &'a Context<'a, V>,
        value: &'a Value<'a, V>,
    ) -> Result<&'a mut MachineState<'a, V>, MachineError<'a, V>>
    where
        V: Eval<'a>,
    {
        match value {
            Value::Delay(term, env) => Ok(MachineState::compute(self.arena, context, env, term)),
            Value::Builtin(runtime) => {
                if runtime.needs_force() {
                    let value = if runtime.is_ready() {
                        self.call(runtime)?
                    } else {
                        Value::builtin(self.arena, runtime.force(self.arena))
                    };

                    let state = MachineState::return_(self.arena, context, value);

                    Ok(state)
                } else {
                    let term = discharge::value_as_term(self.arena, value);

                    Err(MachineError::BuiltinTermArgumentExpected(term))
                }
            }
            rest => Err(MachineError::NonPolymorphicInstantiation(rest)),
        }
    }

    fn apply_evaluate<V>(
        &mut self,
        context: &'a Context<'a, V>,
        function: &'a Value<'a, V>,
        argument: &'a Value<'a, V>,
    ) -> Result<&'a mut MachineState<'a, V>, MachineError<'a, V>>
    where
        V: Eval<'a>,
    {
        match function {
            Value::Lambda { body, env, .. } => {
                let new_env = env.push(self.arena, argument);

                let state = MachineState::compute(self.arena, context, new_env, body);

                Ok(state)
            }
            Value::Builtin(runtime) => {
                if !runtime.needs_force() && runtime.is_arrow() {
                    let runtime = runtime.push(self.arena, argument);

                    let value = if runtime.is_ready() {
                        self.eval_builtin_app(runtime)?
                    } else {
                        Value::builtin(self.arena, runtime)
                    };

                    let state = MachineState::return_(self.arena, context, value);

                    Ok(state)
                } else {
                    let term = discharge::value_as_term(self.arena, function);

                    Err(MachineError::UnexpectedBuiltinTermArgument(term))
                }
            }
            rest => Err(MachineError::NonFunctionApplication(argument, rest)),
        }
    }

    fn eval_builtin_app<V>(
        &mut self,
        runtime: &'a Runtime<'a, V>,
    ) -> Result<&'a Value<'a, V>, MachineError<'a, V>>
    where
        V: Eval<'a>,
    {
        self.call(runtime)
    }

    fn transfer_arg_stack<V>(
        &mut self,
        fields: &'a [&'a Value<'a, V>],
        context: &'a Context<'a, V>,
    ) -> &'a Context<'a, V>
    where
        V: Eval<'a>,
    {
        let mut c = context;

        for field in fields.iter().rev() {
            c = Context::frame_await_fun_value(self.arena, *field, c);
        }

        c
    }

    fn step_and_maybe_spend<V>(&mut self, step: StepKind) -> Result<(), MachineError<'a, V>>
    where
        V: Eval<'a>,
    {
        let index = step as usize;

        self.unbudgeted_steps[index] += 1;
        self.unbudgeted_steps[9] += 1;

        if self.unbudgeted_steps[9] >= self.slippage {
            self.spend_unbudgeted_steps()?;
        }

        Ok(())
    }

    fn spend_unbudgeted_steps<V>(&mut self) -> Result<(), MachineError<'a, V>>
    where
        V: Eval<'a>,
    {
        for step_kind in 0..self.unbudgeted_steps.len() - 1 {
            let mut unspent_step_budget = self.costs.machine_costs.get(step_kind);

            unspent_step_budget.occurrences(self.unbudgeted_steps[step_kind] as i64);

            self.spend_budget(unspent_step_budget)?;

            self.unbudgeted_steps[step_kind] = 0;
        }

        self.unbudgeted_steps[9] = 0;

        Ok(())
    }

    pub(super) fn spend_budget<V>(
        &mut self,
        spend_budget: ExBudget,
    ) -> Result<(), MachineError<'a, V>>
    where
        V: Eval<'a>,
    {
        self.ex_budget.mem -= spend_budget.mem;
        self.ex_budget.cpu -= spend_budget.cpu;

        if self.ex_budget.mem < 0 || self.ex_budget.cpu < 0 {
            Err(MachineError::OutOfExError(self.ex_budget))
        } else {
            Ok(())
        }
    }
}
