use bumpalo::Bump;

use bumpalo::collections::Vec as BumpVec;

use crate::{
    machine::{context::Context, env::Env, state::MachineState},
    term::Term,
};

use super::{discharge, runtime::Runtime, value::Value, EvalResult, MachineError};

pub struct Machine<'a> {
    arena: &'a Bump,
}

impl<'a> Machine<'a> {
    pub fn new(arena: &'a Bump) -> Self {
        Machine { arena }
    }

    pub fn run(mut self, term: &'a Term<'a>) -> EvalResult<'a> {
        let initial_context = self.arena.alloc(Context::NoFrame);

        let mut state = self.arena.alloc(MachineState::Compute(
            initial_context,
            Env::new_in(self.arena),
            term,
        ));

        loop {
            let step = match state {
                MachineState::Compute(context, env, term) => self.compute(context, env, term),
                MachineState::Return(context, value) => self.return_compute(context, value),
                MachineState::Done(term) => {
                    return EvalResult { result: Ok(term) };
                }
            };

            state = match step {
                Ok(new_state) => new_state,
                Err(error) => return EvalResult { result: Err(error) },
            };
        }
    }

    pub fn compute(
        &mut self,
        context: &'a Context<'a>,
        env: &'a Env<'a>,
        term: &'a Term<'a>,
    ) -> Result<&'a mut MachineState<'a>, MachineError<'a>> {
        match term {
            Term::Var(name) => {
                let value = env
                    .lookup(*name)
                    .ok_or(MachineError::OpenTermEvaluated(term))?;

                Ok(self.arena.alloc(MachineState::Return(context, value)))
            }
            Term::Lambda { parameter, body } => {
                let value = self.arena.alloc(Value::Lambda {
                    parameter: *parameter,
                    body,
                    env,
                });

                let state = self.arena.alloc(MachineState::Return(context, value));

                Ok(state)
            }
            Term::Apply { function, argument } => {
                let frame = self
                    .arena
                    .alloc(Context::FrameAwaitFunTerm(env, argument, context));

                let state = self
                    .arena
                    .alloc(MachineState::Compute(frame, env, function));

                Ok(state)
            }
            Term::Delay(_) => todo!(),
            Term::Force(_) => todo!(),
            Term::Case { constr, branches } => todo!(),
            Term::Constr { tag, fields } => todo!(),
            Term::Constant(constant) => {
                let value = self.arena.alloc(Value::Con(constant));

                let state = self.arena.alloc(MachineState::Return(context, value));

                Ok(state)
            }
            Term::Builtin(def_fun) => {
                let runtime = self.arena.alloc(Runtime {
                    args: BumpVec::new_in(self.arena),
                    fun: def_fun,
                    forces: 0,
                });

                let value = self.arena.alloc(Value::Builtin(runtime));

                let state = self.arena.alloc(MachineState::Return(context, value));

                Ok(state)
            }
            Term::Error => Err(MachineError::ExplicitErrorTerm),
        }
    }

    pub fn return_compute(
        &mut self,
        context: &'a Context<'a>,
        value: &'a Value<'a>,
    ) -> Result<&'a mut MachineState<'a>, MachineError<'a>> {
        match context {
            Context::FrameAwaitFunTerm(arg_env, argument, context) => {
                let context = self.arena.alloc(Context::FrameAwaitArg(value, context));

                let state = self
                    .arena
                    .alloc(MachineState::Compute(context, arg_env, argument));

                Ok(state)
            }
            Context::FrameAwaitArg(function, context) => {
                self.apply_evaluate(context, function, value)
            }
            Context::FrameAwaitFunValue(_, _) => todo!(),
            Context::FrameForce(_) => todo!(),
            Context::FrameConstr(_, _, _, _, _) => todo!(),
            Context::FrameCases(_, _, _) => todo!(),
            Context::NoFrame => {
                let term = discharge::value_as_term(self.arena, value);

                let state = self.arena.alloc(MachineState::Done(term));

                Ok(state)
            }
        }
    }

    pub fn apply_evaluate(
        &mut self,
        context: &'a Context<'a>,
        function: &'a Value<'a>,
        argument: &'a Value<'a>,
    ) -> Result<&'a mut MachineState<'a>, MachineError<'a>> {
        match function {
            Value::Lambda {
                parameter,
                body,
                env,
            } => todo!(),
            Value::Builtin(runtime) => {
                if !runtime.needs_force() && runtime.is_arrow() {
                    let runtime = self.arena.alloc(Runtime {
                        args: runtime.args.clone(),
                        fun: runtime.fun,
                        forces: runtime.forces,
                    });

                    runtime.args.push(argument);

                    let value = if runtime.is_ready() {
                        self.eval_builtin_app(runtime)?
                    } else {
                        self.arena.alloc(Value::Builtin(runtime))
                    };

                    let state = self.arena.alloc(MachineState::Return(context, value));

                    Ok(state)
                } else {
                    todo!("Add proper error")
                }
            }
            rest => todo!("Add Proper Error"),
        }
    }

    pub fn eval_builtin_app(
        &mut self,
        runtime: &'a Runtime<'a>,
    ) -> Result<&'a Value<'a>, MachineError<'a>> {
        runtime.call(&self.arena)
    }
}
