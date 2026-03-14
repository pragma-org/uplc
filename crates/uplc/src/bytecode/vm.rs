use bumpalo::collections::Vec as BumpVec;

use crate::{
    arena::Arena,
    binder::{DeBruijn, Eval},
    builtin::DefaultFunction,
    constant::{Constant, Integer},
    machine::{
        cost_model::{builtin_costs::BuiltinCostModel, StepKind},
        discharge,
        env::Env,
        runtime::{BuiltinSemantics, Runtime},
        value::Value,
        CostModel, ExBudget, Machine, MachineError,
    },
};

use super::{read_u16, read_u32, read_u64, CompiledProgram, Op};

/// Bytecode CEK continuation frame.
enum Frame<'a> {
    AwaitFunTerm {
        arg_ip: u32,
        env: &'a Env<'a, DeBruijn>,
    },
    AwaitArg(&'a Value<'a, DeBruijn>),
    AwaitFunValue(&'a Value<'a, DeBruijn>),
    Force,
    AwaitArgForLambda {
        body_ip: u32,
        env: &'a Env<'a, DeBruijn>,
    },
    Constr {
        env: &'a Env<'a, DeBruijn>,
        tag: usize,
        field_offsets: Vec<u32>,
        next_field: usize,
        values: Vec<&'a Value<'a, DeBruijn>>,
    },
    Cases {
        env: &'a Env<'a, DeBruijn>,
        branch_offsets: Vec<u32>,
    },
}

/// Execute a compiled program using the bytecode VM.
pub fn execute<'a, B: BuiltinCostModel>(
    arena: &'a Arena,
    program: &'a CompiledProgram<'a>,
    initial_budget: ExBudget,
    costs: CostModel<B>,
    semantics: BuiltinSemantics,
) -> crate::machine::EvalResult<'a, DeBruijn> {
    // We embed a Machine for builtin dispatch (reuses the 2000+ line call method)
    let mut machine = Machine::new(arena, initial_budget, costs, semantics);

    let mut vm = Vm {
        arena,
        bytecode: &program.bytecode,
        constant_pool: &program.constant_pool,
        lambdas: &program.lambdas,
        delays: &program.delays,
        ip: 0,
        env: Env::new_in(arena),
        stack: Vec::with_capacity(64),
        machine: &mut machine,
    };

    let term_result = vm.run();

    let consumed = initial_budget - vm.machine.remaining_budget();
    let logs = vm.machine.take_logs();

    crate::machine::EvalResult {
        term: term_result,
        info: crate::machine::info::MachineInfo {
            consumed_budget: consumed,
            logs,
        },
    }
}

struct Vm<'a, 'b, B: BuiltinCostModel> {
    arena: &'a Arena,
    bytecode: &'a [u8],
    constant_pool: &'a [&'a Constant<'a>],
    lambdas: &'a [super::LambdaInfo<'a>],
    delays: &'a [super::DelayInfo<'a>],
    ip: usize,
    env: &'a Env<'a, DeBruijn>,
    stack: Vec<Frame<'a>>,
    machine: &'b mut Machine<'a, B>,
}

/// Phase tag — are we computing (reading bytecode) or returning (processing a value)?
enum Phase<'a> {
    Compute,
    Return(&'a Value<'a, DeBruijn>),
    Done(&'a Value<'a, DeBruijn>),
}

impl<'a, 'b, B: BuiltinCostModel> Vm<'a, 'b, B> {
    fn run(
        &mut self,
    ) -> Result<&'a crate::term::Term<'a, DeBruijn>, MachineError<'a, DeBruijn>> {
        self.machine.spend_budget(self.machine.costs.machine_startup)?;

        let mut phase = Phase::Compute;

        loop {
            phase = match phase {
                Phase::Compute => self.step_compute()?,
                Phase::Return(value) => self.step_return(value)?,
                Phase::Done(value) => {
                    let term = discharge::value_as_term(self.arena, value);
                    return Ok(term);
                }
            };
        }
    }

    #[inline(always)]
    fn step_compute(&mut self) -> Result<Phase<'a>, MachineError<'a, DeBruijn>> {
        let op = self.bytecode[self.ip];
        self.ip += 1;

        match op {
            op if op == Op::Var as u8 => {
                let idx = self.bytecode[self.ip] as usize;
                self.ip += 1;
                self.machine.step_and_maybe_spend(StepKind::Var)?;
                let value = self
                    .env
                    .lookup(idx)
                    .ok_or(MachineError::ExplicitErrorTerm)?;
                Ok(Phase::Return(value))
            }

            op if op == Op::Lambda as u8 => {
                let body_ip = read_u32(self.bytecode, self.ip);
                let lambda_id = read_u16(self.bytecode, self.ip + 4) as usize;
                self.ip += 6;
                self.machine.step_and_maybe_spend(StepKind::Lambda)?;
                let info = &self.lambdas[lambda_id];
                let value = Value::lambda_bc(self.arena, body_ip, self.env, info.parameter, info.body);
                Ok(Phase::Return(value))
            }

            op if op == Op::Apply as u8 => {
                let arg_ip = read_u32(self.bytecode, self.ip);
                self.ip += 4;
                self.machine.step_and_maybe_spend(StepKind::Apply)?;
                self.stack.push(Frame::AwaitFunTerm {
                    arg_ip,
                    env: self.env,
                });
                Ok(Phase::Compute)
            }

            op if op == Op::Delay as u8 => {
                let body_ip = read_u32(self.bytecode, self.ip);
                let delay_id = read_u16(self.bytecode, self.ip + 4) as usize;
                self.ip += 6;
                self.machine.step_and_maybe_spend(StepKind::Delay)?;
                let info = &self.delays[delay_id];
                let value = Value::delay_bc(self.arena, body_ip, self.env, info.body);
                Ok(Phase::Return(value))
            }

            op if op == Op::Force as u8 => {
                self.machine.step_and_maybe_spend(StepKind::Force)?;
                self.stack.push(Frame::Force);
                Ok(Phase::Compute)
            }

            op if op == Op::ForceDelay as u8 => {
                self.machine.step_and_maybe_spend(StepKind::Force)?;
                self.machine.step_and_maybe_spend(StepKind::Delay)?;
                // Body follows inline
                Ok(Phase::Compute)
            }

            op if op == Op::ApplyLambda as u8 => {
                let body_ip = read_u32(self.bytecode, self.ip);
                let _lambda_id = read_u16(self.bytecode, self.ip + 4);
                self.ip += 6;
                self.machine.step_and_maybe_spend(StepKind::Apply)?;
                self.machine.step_and_maybe_spend(StepKind::Lambda)?;
                self.stack.push(Frame::AwaitArgForLambda {
                    body_ip,
                    env: self.env,
                });
                Ok(Phase::Compute)
            }

            op if op == Op::ForceBuiltin as u8 => {
                let fun_id = self.bytecode[self.ip];
                self.ip += 1;
                self.machine.step_and_maybe_spend(StepKind::Force)?;
                self.machine.step_and_maybe_spend(StepKind::Builtin)?;
                let fun = self.arena.alloc(DefaultFunction::from_u8(fun_id));
                let runtime = Runtime::new(self.arena, fun);
                let forced = runtime.force(self.arena);
                let value = if forced.is_ready() {
                    self.machine.call(forced)?
                } else {
                    Value::builtin(self.arena, forced)
                };
                Ok(Phase::Return(value))
            }

            op if op == Op::Force2Builtin as u8 => {
                let fun_id = self.bytecode[self.ip];
                self.ip += 1;
                self.machine.step_and_maybe_spend(StepKind::Force)?;
                self.machine.step_and_maybe_spend(StepKind::Force)?;
                self.machine.step_and_maybe_spend(StepKind::Builtin)?;
                let fun = self.arena.alloc(DefaultFunction::from_u8(fun_id));
                let runtime = Runtime::new(self.arena, fun);
                let forced = runtime.force(self.arena).force(self.arena);
                let value = if forced.is_ready() {
                    self.machine.call(forced)?
                } else {
                    Value::builtin(self.arena, forced)
                };
                Ok(Phase::Return(value))
            }

            op if op == Op::Constr as u8 || op == Op::ConstrBig as u8 => {
                let tag = if op == Op::Constr as u8 {
                    let t = self.bytecode[self.ip] as usize;
                    self.ip += 1;
                    t
                } else {
                    let t = read_u64(self.bytecode, self.ip) as usize;
                    self.ip += 8;
                    t
                };
                let nfields = self.bytecode[self.ip] as usize;
                self.ip += 1;
                self.machine.step_and_maybe_spend(StepKind::Constr)?;

                if nfields == 0 {
                    let value = Value::constr_empty(self.arena, tag);
                    return Ok(Phase::Return(value));
                }

                let mut field_offsets = Vec::with_capacity(nfields);
                for _ in 0..nfields {
                    field_offsets.push(read_u32(self.bytecode, self.ip));
                    self.ip += 4;
                }

                let first_ip = field_offsets[0] as usize;
                self.stack.push(Frame::Constr {
                    env: self.env,
                    tag,
                    field_offsets,
                    next_field: 1,
                    values: Vec::with_capacity(nfields),
                });
                self.ip = first_ip;
                Ok(Phase::Compute)
            }

            op if op == Op::Case as u8 => {
                let nbranches = self.bytecode[self.ip] as usize;
                self.ip += 1;
                self.machine.step_and_maybe_spend(StepKind::Case)?;

                let mut branch_offsets = Vec::with_capacity(nbranches);
                for _ in 0..nbranches {
                    branch_offsets.push(read_u32(self.bytecode, self.ip));
                    self.ip += 4;
                }

                self.stack.push(Frame::Cases {
                    env: self.env,
                    branch_offsets,
                });
                Ok(Phase::Compute)
            }

            op if op == Op::Const as u8 => {
                let idx = read_u16(self.bytecode, self.ip) as usize;
                self.ip += 2;
                self.machine.step_and_maybe_spend(StepKind::Constant)?;
                let constant = self.constant_pool[idx];
                let value = Value::con(self.arena, constant);
                Ok(Phase::Return(value))
            }

            op if op == Op::ConstUnit as u8 => {
                self.machine.step_and_maybe_spend(StepKind::Constant)?;
                let c = self.arena.alloc(Constant::Unit);
                Ok(Phase::Return(Value::con(self.arena, c)))
            }

            op if op == Op::ConstTrue as u8 => {
                self.machine.step_and_maybe_spend(StepKind::Constant)?;
                let c = self.arena.alloc(Constant::Boolean(true));
                Ok(Phase::Return(Value::con(self.arena, c)))
            }

            op if op == Op::ConstFalse as u8 => {
                self.machine.step_and_maybe_spend(StepKind::Constant)?;
                let c = self.arena.alloc(Constant::Boolean(false));
                Ok(Phase::Return(Value::con(self.arena, c)))
            }

            op if op == Op::ConstSmallInt as u8 => {
                let val = self.bytecode[self.ip] as i8;
                self.ip += 1;
                self.machine.step_and_maybe_spend(StepKind::Constant)?;
                let i = self.arena.alloc_integer(Integer::from(val));
                let c = self.arena.alloc(Constant::Integer(i));
                Ok(Phase::Return(Value::con(self.arena, c)))
            }

            op if op == Op::Builtin as u8 => {
                let fun_id = self.bytecode[self.ip];
                self.ip += 1;
                self.machine.step_and_maybe_spend(StepKind::Builtin)?;
                let fun = self.arena.alloc(DefaultFunction::from_u8(fun_id));
                let runtime = Runtime::new(self.arena, fun);
                let value = Value::builtin(self.arena, runtime);
                Ok(Phase::Return(value))
            }

            _op if _op == Op::Error as u8 => Err(MachineError::ExplicitErrorTerm),

            _ => Err(MachineError::ExplicitErrorTerm),
        }
    }

    fn step_return(
        &mut self,
        value: &'a Value<'a, DeBruijn>,
    ) -> Result<Phase<'a>, MachineError<'a, DeBruijn>> {
        match self.stack.pop() {
            Some(Frame::AwaitFunTerm { arg_ip, env }) => {
                self.stack.push(Frame::AwaitArg(value));
                self.env = env;
                self.ip = arg_ip as usize;
                Ok(Phase::Compute)
            }

            Some(Frame::AwaitArg(function)) => self.apply_evaluate(function, value),

            Some(Frame::AwaitArgForLambda { body_ip, env }) => {
                self.env = env.push(self.arena, value);
                self.ip = body_ip as usize;
                Ok(Phase::Compute)
            }

            Some(Frame::AwaitFunValue(argument)) => self.apply_evaluate(value, argument),

            Some(Frame::Force) => self.force_evaluate(value),

            Some(Frame::Constr {
                env,
                tag,
                field_offsets,
                next_field,
                mut values,
            }) => {
                values.push(value);

                if next_field < field_offsets.len() {
                    let next_ip = field_offsets[next_field] as usize;
                    self.stack.push(Frame::Constr {
                        env,
                        tag,
                        field_offsets,
                        next_field: next_field + 1,
                        values,
                    });
                    self.env = env;
                    self.ip = next_ip;
                    Ok(Phase::Compute)
                } else {
                    let mut arena_values =
                        BumpVec::with_capacity_in(values.len(), self.arena.as_bump());
                    for v in &values {
                        arena_values.push(*v);
                    }
                    let arena_values = self.arena.alloc(arena_values);
                    let constr_value = Value::constr(self.arena, tag, arena_values);
                    Ok(Phase::Return(constr_value))
                }
            }

            Some(Frame::Cases {
                env,
                branch_offsets,
            }) => match value {
                Value::Constr(tag, fields) => {
                    if *tag < branch_offsets.len() {
                        for field in fields.iter().rev() {
                            self.stack.push(Frame::AwaitFunValue(*field));
                        }
                        self.env = env;
                        self.ip = branch_offsets[*tag] as usize;
                        Ok(Phase::Compute)
                    } else {
                        Err(MachineError::ExplicitErrorTerm)
                    }
                }
                _ => Err(MachineError::ExplicitErrorTerm),
            },

            None => {
                // Stack empty — done
                self.machine.flush_unbudgeted_steps()?;
                Ok(Phase::Done(value))
            }
        }
    }

    fn force_evaluate(
        &mut self,
        value: &'a Value<'a, DeBruijn>,
    ) -> Result<Phase<'a>, MachineError<'a, DeBruijn>> {
        match value {
            Value::DelayBC { body_ip, env, .. } => {
                self.env = env;
                self.ip = *body_ip as usize;
                Ok(Phase::Compute)
            }
            Value::Delay(_body, _env) => {
                // AST-style delay shouldn't appear in bytecode execution
                Err(MachineError::ExplicitErrorTerm)
            }
            Value::Builtin(runtime) => {
                if runtime.needs_force() {
                    let forced = runtime.force(self.arena);
                    let value = if forced.is_ready() {
                        self.machine.call(forced)?
                    } else {
                        Value::builtin(self.arena, forced)
                    };
                    Ok(Phase::Return(value))
                } else {
                    Err(MachineError::ExplicitErrorTerm)
                }
            }
            _ => Err(MachineError::ExplicitErrorTerm),
        }
    }

    fn apply_evaluate(
        &mut self,
        function: &'a Value<'a, DeBruijn>,
        argument: &'a Value<'a, DeBruijn>,
    ) -> Result<Phase<'a>, MachineError<'a, DeBruijn>> {
        match function {
            Value::LambdaBC { body_ip, env, .. } => {
                self.env = env.push(self.arena, argument);
                self.ip = *body_ip as usize;
                Ok(Phase::Compute)
            }
            Value::Lambda { .. } => {
                // AST-style lambda shouldn't appear in bytecode execution
                Err(MachineError::ExplicitErrorTerm)
            }
            Value::Builtin(runtime) => {
                if !runtime.needs_force() && runtime.is_arrow() {
                    let runtime = runtime.push(self.arena, argument);
                    let value = if runtime.is_ready() {
                        self.machine.call(runtime)?
                    } else {
                        Value::builtin(self.arena, runtime)
                    };
                    Ok(Phase::Return(value))
                } else {
                    Err(MachineError::ExplicitErrorTerm)
                }
            }
            _ => Err(MachineError::ExplicitErrorTerm),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        arena::Arena,
        bytecode::compiler,
        machine::{
            cost_model::builtin_costs::builtin_costs_v3::BuiltinCostsV3,
            CostModel, ExBudget, BuiltinSemantics,
        },
        syn,
    };

    fn eval_source(source: &str) -> String {
        let source: &'static str = Box::leak(source.to_string().into_boxed_str());
        let arena: &'static Arena = Box::leak(Box::new(Arena::new()));

        let program = syn::parse_program(arena, source)
            .into_result()
            .expect("parse failed");

        let compiled = compiler::compile(
            (program.version.major(), program.version.minor(), program.version.patch()),
            program.term,
        );
        let compiled: &'static CompiledProgram = Box::leak(Box::new(compiled));

        let result = execute(
            arena,
            compiled,
            ExBudget::default(),
            CostModel::<BuiltinCostsV3>::default(),
            BuiltinSemantics::V2,
        );

        match result.term {
            Ok(term) => format!("{:?}", term),
            Err(e) => format!("ERROR: {:?}", e),
        }
    }

    #[test]
    fn bc_eval_integer() {
        let result = eval_source("(program 1.0.0 (con integer 42))");
        assert!(result.contains("42"), "got: {result}");
    }

    #[test]
    fn bc_eval_unit() {
        let result = eval_source("(program 1.0.0 (con unit ()))");
        assert!(result.contains("Unit"), "got: {result}");
    }

    #[test]
    fn bc_eval_true() {
        let result = eval_source("(program 1.0.0 (con bool True))");
        assert!(result.contains("true") || result.contains("True"), "got: {result}");
    }

    #[test]
    fn bc_eval_identity() {
        let result = eval_source("(program 1.0.0 [(lam x x) (con integer 7)])");
        assert!(result.contains("7"), "got: {result}");
    }

    #[test]
    fn bc_eval_force_delay() {
        let result = eval_source("(program 1.0.0 (force (delay (con integer 99))))");
        assert!(result.contains("99"), "got: {result}");
    }

    #[test]
    fn bc_eval_add_integer() {
        let result = eval_source(
            "(program 1.0.0 [(builtin addInteger) (con integer 3) (con integer 4)])",
        );
        assert!(result.contains("7"), "got: {result}");
    }

    #[test]
    fn bc_eval_nested_apply() {
        let result = eval_source(
            "(program 1.0.0 [(lam x [(lam y [(builtin addInteger) x y]) (con integer 10)]) (con integer 20)])",
        );
        assert!(result.contains("30"), "got: {result}");
    }

    #[test]
    fn bc_eval_if_then_else() {
        let result = eval_source(
            "(program 1.0.0 (force [(force (builtin ifThenElse)) (con bool True) (delay (con integer 1)) (delay (con integer 2))]))",
        );
        assert!(result.contains("1"), "got: {result}");
    }
}
