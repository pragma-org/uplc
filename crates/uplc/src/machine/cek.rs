use bumpalo::collections::Vec as BumpVec;

use crate::{
    arena::Arena,
    binder::Eval,
    constant::Constant,
    machine::{
        context::{ContextStack, Frame},
        cost_model::builtin_costs::BuiltinCostModel,
        env::Env,
        state::MachineState,
    },
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

pub struct Machine<'a, B: BuiltinCostModel, V: Eval<'a>> {
    pub(super) arena: &'a Arena,
    ex_budget: ExBudget,
    unbudgeted_steps: [u8; 10],
    pub(super) costs: CostModel<B>,
    slippage: u8,
    pub(super) logs: Vec<String>,
    pub(super) semantics: BuiltinSemantics,
    _marker: std::marker::PhantomData<V>,
}

impl<'a, B: BuiltinCostModel, V: Eval<'a>> Machine<'a, B, V> {
    pub fn new(
        arena: &'a Arena,
        initial_budget: ExBudget,
        costs: CostModel<B>,
        semantics: BuiltinSemantics,
    ) -> Self {
        Self {
            arena,
            ex_budget: initial_budget,
            unbudgeted_steps: [0; 10],
            costs,
            slippage: 200,
            logs: Vec::new(),
            semantics,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn info(self) -> MachineInfo {
        MachineInfo {
            consumed_budget: self.ex_budget,
            logs: self.logs,
        }
    }

    pub fn run(&mut self, term: &'a Term<'a, V>) -> Result<&'a Term<'a, V>, MachineError<'a, V>> {
        self.spend_budget(self.costs.machine_startup)?;

        let mut stack = ContextStack::new();

        let mut state = MachineState::Compute(Env::new_in(self.arena), term);

        loop {
            state = match state {
                MachineState::Compute(env, term) => self.compute(&mut stack, env, term)?,
                MachineState::Return(value) => self.return_compute(&mut stack, value)?,
                MachineState::Done(term) => {
                    return Ok(term);
                }
            };
        }
    }

    pub fn compute(
        &mut self,
        stack: &mut ContextStack<'a, V>,
        env: &'a Env<'a, V>,
        term: &'a Term<'a, V>,
    ) -> Result<MachineState<'a, V>, MachineError<'a, V>> {
        match term {
            Term::Var(name) => {
                self.step_and_maybe_spend(StepKind::Var)?;

                let value = env
                    .lookup(name.index())
                    .ok_or(MachineError::OpenTermEvaluated(term))?;

                Ok(MachineState::Return(value))
            }
            Term::Lambda { parameter, body } => {
                self.step_and_maybe_spend(StepKind::Lambda)?;

                let value = Value::lambda(self.arena, *parameter, body, env);

                Ok(MachineState::Return(value))
            }
            Term::Apply { function, argument } => {
                self.step_and_maybe_spend(StepKind::Apply)?;

                // Fast path: Apply(Lambda(body), arg)
                if let Term::Lambda { body, .. } = function {
                    self.step_and_maybe_spend(StepKind::Lambda)?;
                    stack.push(Frame::FrameAwaitArgForLambda(body, env));
                    return Ok(MachineState::Compute(env, argument));
                }

                stack.push(Frame::FrameAwaitFunTerm(env, argument));

                Ok(MachineState::Compute(env, function))
            }
            Term::Delay(body) => {
                self.step_and_maybe_spend(StepKind::Delay)?;

                let value = Value::delay(self.arena, body, env);

                Ok(MachineState::Return(value))
            }
            Term::Force(body) => {
                self.step_and_maybe_spend(StepKind::Force)?;

                // Fast path: Force(Delay(body))
                if let Term::Delay(inner) = body {
                    self.step_and_maybe_spend(StepKind::Delay)?;
                    return Ok(MachineState::Compute(env, inner));
                }

                // Fast path: Force(Builtin(b))
                if let Term::Builtin(fun) = body {
                    self.step_and_maybe_spend(StepKind::Builtin)?;
                    let runtime = Runtime::new(self.arena, fun);
                    if runtime.needs_force() {
                        let forced = runtime.force(self.arena);
                        let value = if forced.is_ready() {
                            self.call(forced)?
                        } else {
                            Value::builtin(self.arena, forced)
                        };
                        return Ok(MachineState::Return(value));
                    }
                }

                // Fast path: Force(Force(Builtin(b)))
                if let Term::Force(inner) = body {
                    if let Term::Builtin(fun) = inner {
                        self.step_and_maybe_spend(StepKind::Force)?;
                        self.step_and_maybe_spend(StepKind::Builtin)?;
                        let runtime = Runtime::new(self.arena, fun);
                        if runtime.needs_force() {
                            let forced = runtime.force(self.arena);
                            if forced.needs_force() {
                                let forced2 = forced.force(self.arena);
                                let value = if forced2.is_ready() {
                                    self.call(forced2)?
                                } else {
                                    Value::builtin(self.arena, forced2)
                                };
                                return Ok(MachineState::Return(value));
                            }
                        }
                    }
                }

                stack.push(Frame::FrameForce);

                Ok(MachineState::Compute(env, body))
            }
            Term::Constr { tag, fields } => {
                self.step_and_maybe_spend(StepKind::Constr)?;

                if let Some((first, terms)) = fields.split_first() {
                    let empty = BumpVec::new_in(self.arena.as_bump());
                    let empty = self.arena.alloc(empty);
                    stack.push(Frame::FrameConstr(env, *tag, terms, empty));

                    Ok(MachineState::Compute(env, first))
                } else {
                    let value = Value::constr_empty(self.arena, *tag);

                    Ok(MachineState::Return(value))
                }
            }
            Term::Case { constr, branches } => {
                self.step_and_maybe_spend(StepKind::Case)?;

                stack.push(Frame::FrameCases(env, branches));

                Ok(MachineState::Compute(env, constr))
            }
            Term::Constant(constant) => {
                self.step_and_maybe_spend(StepKind::Constant)?;

                let value = Value::con(self.arena, constant);

                Ok(MachineState::Return(value))
            }
            Term::Builtin(fun) => {
                self.step_and_maybe_spend(StepKind::Builtin)?;

                let runtime = Runtime::new(self.arena, fun);

                let value = Value::builtin(self.arena, runtime);

                Ok(MachineState::Return(value))
            }
            Term::Error => Err(MachineError::ExplicitErrorTerm),
        }
    }

    pub fn return_compute(
        &mut self,
        stack: &mut ContextStack<'a, V>,
        value: &'a Value<'a, V>,
    ) -> Result<MachineState<'a, V>, MachineError<'a, V>> {
        match stack.pop() {
            Some(Frame::FrameAwaitFunTerm(arg_env, argument)) => {
                stack.push(Frame::FrameAwaitArg(value));

                Ok(MachineState::Compute(arg_env, argument))
            }
            Some(Frame::FrameAwaitArg(function)) => {
                self.apply_evaluate(stack, function, value)
            }
            Some(Frame::FrameAwaitArgForLambda(body, env)) => {
                let new_env = env.push(self.arena, value);
                Ok(MachineState::Compute(new_env, body))
            }
            Some(Frame::FrameAwaitFunValue(argument)) => {
                self.apply_evaluate(stack, value, argument)
            }
            Some(Frame::FrameForce) => self.force_evaluate(stack, value),
            Some(Frame::FrameConstr(env, tag, terms, values)) => {
                let mut new_values =
                    BumpVec::with_capacity_in(values.len() + 1, self.arena.as_bump());

                for v in values.iter() {
                    new_values.push(*v);
                }

                new_values.push(value);

                let values = self.arena.alloc(new_values);

                if let Some((first, terms)) = terms.split_first() {
                    stack.push(Frame::FrameConstr(env, tag, terms, values));

                    Ok(MachineState::Compute(env, first))
                } else {
                    let value = Value::constr(self.arena, tag, values);

                    Ok(MachineState::Return(value))
                }
            }
            Some(Frame::FrameCases(env, branches)) => match value {
                Value::Constr(tag, fields) => {
                    if let Some(branch) = branches.get(*tag) {
                        self.transfer_arg_stack(stack, fields);

                        Ok(MachineState::Compute(env, branch))
                    } else {
                        Err(MachineError::MissingCaseBranch(branches, value))
                    }
                }
                Value::Con(constant) => self.case_on_constant(stack, constant, env, branches),
                v => Err(MachineError::NonConstrScrutinized(v)),
            },
            None => {
                if self.unbudgeted_steps[9] > 0 {
                    self.spend_unbudgeted_steps()?;
                }

                let term = discharge::value_as_term(self.arena, value);

                Ok(MachineState::Done(term))
            }
        }
    }

    fn force_evaluate(
        &mut self,
        stack: &mut ContextStack<'a, V>,
        value: &'a Value<'a, V>,
    ) -> Result<MachineState<'a, V>, MachineError<'a, V>> {
        match value {
            Value::Delay(term, env) => Ok(MachineState::Compute(env, term)),
            Value::Builtin(runtime) => {
                if runtime.needs_force() {
                    let value = if runtime.is_ready() {
                        self.call(runtime)?
                    } else {
                        Value::builtin(self.arena, runtime.force(self.arena))
                    };

                    Ok(MachineState::Return(value))
                } else {
                    let term = discharge::value_as_term(self.arena, value);

                    Err(MachineError::BuiltinTermArgumentExpected(term))
                }
            }
            rest => Err(MachineError::NonPolymorphicInstantiation(rest)),
        }
    }

    fn apply_evaluate(
        &mut self,
        stack: &mut ContextStack<'a, V>,
        function: &'a Value<'a, V>,
        argument: &'a Value<'a, V>,
    ) -> Result<MachineState<'a, V>, MachineError<'a, V>> {
        match function {
            Value::Lambda { body, env, .. } => {
                let new_env = env.push(self.arena, argument);

                Ok(MachineState::Compute(new_env, body))
            }
            Value::Builtin(runtime) => {
                if !runtime.needs_force() && runtime.is_arrow() {
                    let runtime = runtime.push(self.arena, argument);

                    let value = if runtime.is_ready() {
                        self.call(runtime)?
                    } else {
                        Value::builtin(self.arena, runtime)
                    };

                    Ok(MachineState::Return(value))
                } else {
                    let term = discharge::value_as_term(self.arena, function);

                    Err(MachineError::UnexpectedBuiltinTermArgument(term))
                }
            }
            rest => Err(MachineError::NonFunctionApplication(argument, rest)),
        }
    }

    fn case_on_constant(
        &mut self,
        stack: &mut ContextStack<'a, V>,
        constant: &'a Constant<'a>,
        env: &'a Env<'a, V>,
        branches: &'a [&'a Term<'a, V>],
    ) -> Result<MachineState<'a, V>, MachineError<'a, V>> {
        match constant {
            Constant::Boolean(b) => {
                if branches.is_empty() || branches.len() > 2 {
                    return Err(MachineError::NonConstrScrutinized(Value::con(
                        self.arena, constant,
                    )));
                }
                let tag: usize = if *b { 1 } else { 0 };
                if tag >= branches.len() {
                    return Err(MachineError::NonConstrScrutinized(Value::con(
                        self.arena, constant,
                    )));
                }
                Ok(MachineState::Compute(env, branches[tag]))
            }
            Constant::Unit => {
                if branches.len() != 1 {
                    return Err(MachineError::NonConstrScrutinized(Value::con(
                        self.arena, constant,
                    )));
                }
                Ok(MachineState::Compute(env, branches[0]))
            }
            Constant::Integer(int_val) => {
                use num::ToPrimitive;
                let tag = int_val.to_usize().ok_or_else(|| {
                    MachineError::NonConstrScrutinized(Value::con(self.arena, constant))
                })?;
                if tag >= branches.len() {
                    return Err(MachineError::NonConstrScrutinized(Value::con(
                        self.arena, constant,
                    )));
                }
                Ok(MachineState::Compute(env, branches[tag]))
            }
            Constant::ProtoList(typ, items) => {
                if !items.is_empty() {
                    // Non-empty list: branch 0, with head and tail as arguments
                    if branches.is_empty() {
                        return Err(MachineError::NonConstrScrutinized(Value::con(
                            self.arena, constant,
                        )));
                    }

                    let head_val: &'a Value<'a, V> = Value::con(self.arena, items[0]);
                    let tail_const = Constant::proto_list(self.arena, typ, &items[1..]);
                    let tail_val: &'a Value<'a, V> = Value::con(self.arena, tail_const);

                    let fields: &'a [&'a Value<'a, V>] = self.arena.alloc([head_val, tail_val]);
                    self.transfer_arg_stack(stack, fields);

                    Ok(MachineState::Compute(env, branches[0]))
                } else {
                    // Empty list: branch 1
                    if branches.len() >= 2 {
                        Ok(MachineState::Compute(env, branches[1]))
                    } else {
                        Err(MachineError::NonConstrScrutinized(Value::con(
                            self.arena, constant,
                        )))
                    }
                }
            }
            Constant::ProtoPair(_t1, _t2, fst, snd) => {
                if branches.len() != 1 {
                    return Err(MachineError::NonConstrScrutinized(Value::con(
                        self.arena, constant,
                    )));
                }

                let fst_val: &'a Value<'a, V> = Value::con(self.arena, fst);
                let snd_val: &'a Value<'a, V> = Value::con(self.arena, snd);

                let fields: &'a [&'a Value<'a, V>] = self.arena.alloc([fst_val, snd_val]);
                self.transfer_arg_stack(stack, fields);

                Ok(MachineState::Compute(env, branches[0]))
            }
            _ => Err(MachineError::NonConstrScrutinized(Value::con(
                self.arena, constant,
            ))),
        }
    }

    fn transfer_arg_stack(
        &mut self,
        stack: &mut ContextStack<'a, V>,
        fields: &'a [&'a Value<'a, V>],
    ) {
        for field in fields.iter().rev() {
            stack.push(Frame::FrameAwaitFunValue(*field));
        }
    }

    fn step_and_maybe_spend(&mut self, step: StepKind) -> Result<(), MachineError<'a, V>> {
        let index = step as usize;

        self.unbudgeted_steps[index] += 1;
        self.unbudgeted_steps[9] += 1;

        if self.unbudgeted_steps[9] >= self.slippage {
            self.spend_unbudgeted_steps()?;
        }

        Ok(())
    }

    fn spend_unbudgeted_steps(&mut self) -> Result<(), MachineError<'a, V>> {
        for step_kind in 0..self.unbudgeted_steps.len() - 1 {
            let mut unspent_step_budget = self.costs.machine_costs.get(step_kind);

            unspent_step_budget.occurrences(self.unbudgeted_steps[step_kind] as i64);

            self.spend_budget(unspent_step_budget)?;

            self.unbudgeted_steps[step_kind] = 0;
        }

        self.unbudgeted_steps[9] = 0;

        Ok(())
    }

    pub(super) fn spend_budget(
        &mut self,
        spend_budget: ExBudget,
    ) -> Result<(), MachineError<'a, V>> {
        self.ex_budget.mem -= spend_budget.mem;
        self.ex_budget.cpu -= spend_budget.cpu;

        if self.ex_budget.mem < 0 || self.ex_budget.cpu < 0 {
            Err(MachineError::OutOfExError(self.ex_budget))
        } else {
            Ok(())
        }
    }
}
