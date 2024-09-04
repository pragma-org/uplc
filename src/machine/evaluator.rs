use bumpalo::Bump;

use crate::{
    machine::{context::Context, env::Env, state::MachineState},
    term::Term,
};

use super::{value::Value, EvalResult, MachineError};

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
            Term::Var(_) => {
                let value = env.lookup(term)?;

                Ok(self.arena.alloc(MachineState::Return(context, value)))
            }
            Term::Lambda { parameter, body } => todo!(),
            Term::Apply { function, argument } => todo!(),
            Term::Delay(_) => todo!(),
            Term::Force(_) => todo!(),
            Term::Case { constr, branches } => todo!(),
            Term::Constr { tag, fields } => todo!(),
            Term::Constant(_) => todo!(),
            Term::Builtin(_) => todo!(),
            Term::Error => todo!(),
        }
    }

    pub fn return_compute(
        &mut self,
        context: &'a Context<'a>,
        value: &'a Value<'a>,
    ) -> Result<&'a mut MachineState<'a>, MachineError<'a>> {
        match context {
            Context::FrameAwaitArg(_, _) => todo!(),
            Context::FrameAwaitFunTerm(_, _, _) => todo!(),
            Context::FrameAwaitFunValue(_, _) => todo!(),
            Context::FrameForce(_) => todo!(),
            Context::FrameConstr(_, _, _, _, _) => todo!(),
            Context::FrameCases(_, _, _) => todo!(),
            Context::NoFrame => todo!(),
        }
    }
}
