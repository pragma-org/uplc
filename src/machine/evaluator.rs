use bumpalo::Bump;

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
        let initial_context = Context::no_frame(self.arena);

        let mut state =
            MachineState::compute(self.arena, initial_context, Env::new_in(self.arena), term);

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

                let state = MachineState::return_(self.arena, context, value);

                Ok(state)
            }
            Term::Lambda { parameter, body } => {
                let value = Value::lambda(self.arena, *parameter, body, env);

                let state = MachineState::return_(self.arena, context, value);

                Ok(state)
            }
            Term::Apply { function, argument } => {
                let frame = Context::frame_await_fun_term(self.arena, env, argument, context);

                let state = MachineState::compute(self.arena, frame, env, function);

                Ok(state)
            }
            Term::Delay(_) => todo!(),
            Term::Force(_) => todo!(),
            Term::Case { constr, branches } => todo!(),
            Term::Constr { tag, fields } => todo!(),
            Term::Constant(constant) => {
                let value = Value::con(self.arena, constant);

                let state = MachineState::return_(self.arena, context, value);

                Ok(state)
            }
            Term::Builtin(fun) => {
                let runtime = Runtime::new(self.arena, fun);

                let value = Value::builtin(self.arena, runtime);

                let state = MachineState::return_(self.arena, context, value);

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
                let context = Context::frame_await_arg(self.arena, value, context);

                let state = MachineState::compute(self.arena, context, arg_env, argument);

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

                let state = MachineState::done(self.arena, term);

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
            } => {
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

    pub fn eval_builtin_app(
        &mut self,
        runtime: &'a Runtime<'a>,
    ) -> Result<&'a Value<'a>, MachineError<'a>> {
        runtime.call(self.arena)
    }
}
