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
/// All offsets reference positions in the bytecode array directly,
/// avoiding heap allocation for offset vectors.
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
        /// Byte offset in bytecode where the field offset table starts.
        offsets_start: usize,
        nfields: usize,
        next_field: usize,
        values: BumpVec<'a, &'a Value<'a, DeBruijn>>,
    },
    Cases {
        env: &'a Env<'a, DeBruijn>,
        /// Byte offset in bytecode where the branch offset table starts.
        offsets_start: usize,
        nbranches: usize,
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

    // Pre-wrap constants as Values to avoid per-opcode arena allocation
    let pre_wrapped: Vec<&'a Value<'a, DeBruijn>> = program
        .constant_pool
        .iter()
        .map(|c| Value::con(arena, c))
        .collect();

    // Pre-wrap specialized constants
    let val_unit: &'a Value<'a, DeBruijn> = Value::con(arena, arena.alloc(Constant::Unit));
    let val_true: &'a Value<'a, DeBruijn> = Value::con(arena, arena.alloc(Constant::Boolean(true)));
    let val_false: &'a Value<'a, DeBruijn> = Value::con(arena, arena.alloc(Constant::Boolean(false)));

    // Pre-create bare builtin values (no forces/args yet)
    let mut builtin_values: Vec<&'a Value<'a, DeBruijn>> = Vec::with_capacity(92);
    for i in 0..92u8 {
        let fun = arena.alloc(DefaultFunction::from_u8(i));
        let runtime = Runtime::new(arena, fun);
        builtin_values.push(Value::builtin(arena, runtime));
    }
    let builtin_values = builtin_values;

    let mut vm = Vm {
        arena,
        bytecode: &program.bytecode,
        constant_values: &pre_wrapped,
        val_unit,
        val_true,
        val_false,
        builtin_values: &builtin_values,
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
    constant_values: &'b [&'a Value<'a, DeBruijn>],
    val_unit: &'a Value<'a, DeBruijn>,
    val_true: &'a Value<'a, DeBruijn>,
    val_false: &'a Value<'a, DeBruijn>,
    builtin_values: &'b [&'a Value<'a, DeBruijn>],
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
                    let term = discharge::value_as_term_bc(
                        self.arena, value, self.lambdas, self.delays,
                    );
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
            0x01 => {
                let idx = self.bytecode[self.ip] as usize;
                self.ip += 1;
                self.machine.step_and_maybe_spend(StepKind::Var)?;
                let value = self
                    .env
                    .lookup(idx)
                    .ok_or(MachineError::ExplicitErrorTerm)?;
                Ok(Phase::Return(value))
            }

            0x02 => {
                let body_ip = read_u32(self.bytecode, self.ip);
                let lambda_id = read_u16(self.bytecode, self.ip + 4);
                self.ip += 6;
                self.machine.step_and_maybe_spend(StepKind::Lambda)?;
                let value = Value::lambda_bc(self.arena, body_ip, lambda_id, self.env);
                Ok(Phase::Return(value))
            }

            0x03 => {
                let arg_ip = read_u32(self.bytecode, self.ip);
                self.ip += 4;
                self.machine.step_and_maybe_spend(StepKind::Apply)?;
                self.stack.push(Frame::AwaitFunTerm {
                    arg_ip,
                    env: self.env,
                });
                Ok(Phase::Compute)
            }

            0x04 => {
                let body_ip = read_u32(self.bytecode, self.ip);
                let delay_id = read_u16(self.bytecode, self.ip + 4);
                self.ip += 6;
                self.machine.step_and_maybe_spend(StepKind::Delay)?;
                let value = Value::delay_bc(self.arena, body_ip, delay_id, self.env);
                Ok(Phase::Return(value))
            }

            0x05 => {
                self.machine.step_and_maybe_spend(StepKind::Force)?;
                self.stack.push(Frame::Force);
                Ok(Phase::Compute)
            }

            0x10 => {
                self.machine.step_and_maybe_spend(StepKind::Force)?;
                self.machine.step_and_maybe_spend(StepKind::Delay)?;
                // Body follows inline
                Ok(Phase::Compute)
            }

            0x11 => {
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

            0x12 => {
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

            0x13 => {
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

            // ForceVar: Force(Var(idx)) — skip FrameForce, directly force
            0x15 => {
                let idx = self.bytecode[self.ip] as usize;
                self.ip += 1;
                self.machine.step_and_maybe_spend(StepKind::Force)?;
                self.machine.step_and_maybe_spend(StepKind::Var)?;
                let value = self.env.lookup(idx)
                    .ok_or(MachineError::ExplicitErrorTerm)?;
                return self.force_evaluate(value);
            }

            // ApplyVar: Apply(Var(idx), arg) — skip FrameAwaitFunTerm
            0x14 => {
                let idx = self.bytecode[self.ip] as usize;
                self.ip += 1;
                self.machine.step_and_maybe_spend(StepKind::Apply)?;
                self.machine.step_and_maybe_spend(StepKind::Var)?;
                let fun_value = self.env.lookup(idx)
                    .ok_or(MachineError::ExplicitErrorTerm)?;
                self.stack.push(Frame::AwaitArg(fun_value));
                // arg follows inline
                Ok(Phase::Compute)
            }

            0x06 | 0x0B => {
                let tag = if op == 0x06 {
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

                // Field offsets are inline in bytecode — just record where they start
                let offsets_start = self.ip;
                self.ip += nfields * 4; // skip past the offset table

                let first_ip = read_u32(self.bytecode, offsets_start) as usize;
                self.stack.push(Frame::Constr {
                    env: self.env,
                    tag,
                    offsets_start,
                    nfields,
                    next_field: 1,
                    values: BumpVec::with_capacity_in(nfields, self.arena.as_bump()),
                });
                self.ip = first_ip;
                Ok(Phase::Compute)
            }

            0x07 => {
                let nbranches = self.bytecode[self.ip] as usize;
                self.ip += 1;
                self.machine.step_and_maybe_spend(StepKind::Case)?;

                let offsets_start = self.ip;
                self.ip += nbranches * 4; // skip past the offset table

                self.stack.push(Frame::Cases {
                    env: self.env,
                    offsets_start,
                    nbranches,
                });
                // Scrutinee follows inline
                Ok(Phase::Compute)
            }

            0x08 => {
                let idx = read_u16(self.bytecode, self.ip) as usize;
                self.ip += 2;
                self.machine.step_and_maybe_spend(StepKind::Constant)?;
                let value = self.constant_values[idx];
                Ok(Phase::Return(value))
            }

            0x20 => {
                self.machine.step_and_maybe_spend(StepKind::Constant)?;
                Ok(Phase::Return(self.val_unit))
            }

            0x21 => {
                self.machine.step_and_maybe_spend(StepKind::Constant)?;
                Ok(Phase::Return(self.val_true))
            }

            0x22 => {
                self.machine.step_and_maybe_spend(StepKind::Constant)?;
                Ok(Phase::Return(self.val_false))
            }

            0x23 => {
                let val = self.bytecode[self.ip] as i8;
                self.ip += 1;
                self.machine.step_and_maybe_spend(StepKind::Constant)?;
                let i = self.arena.alloc_integer(Integer::from(val));
                let c = self.arena.alloc(Constant::Integer(i));
                Ok(Phase::Return(Value::con(self.arena, c)))
            }

            0x09 => {
                let fun_id = self.bytecode[self.ip] as usize;
                self.ip += 1;
                self.machine.step_and_maybe_spend(StepKind::Builtin)?;
                let value = self.builtin_values[fun_id];
                Ok(Phase::Return(value))
            }

            0x0A => Err(MachineError::ExplicitErrorTerm),

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
                offsets_start,
                nfields,
                next_field,
                mut values,
            }) => {
                values.push(value);

                if next_field < nfields {
                    let next_ip = read_u32(self.bytecode, offsets_start + next_field * 4) as usize;
                    self.stack.push(Frame::Constr {
                        env,
                        tag,
                        offsets_start,
                        nfields,
                        next_field: next_field + 1,
                        values,
                    });
                    self.env = env;
                    self.ip = next_ip;
                    Ok(Phase::Compute)
                } else {
                    let values = self.arena.alloc(values);
                    let constr_value = Value::constr(self.arena, tag, values);
                    Ok(Phase::Return(constr_value))
                }
            }

            Some(Frame::Cases {
                env,
                offsets_start,
                nbranches,
            }) => match value {
                Value::Constr(tag, fields) => {
                    if *tag < nbranches {
                        for field in fields.iter().rev() {
                            self.stack.push(Frame::AwaitFunValue(*field));
                        }
                        self.env = env;
                        self.ip = read_u32(self.bytecode, offsets_start + *tag * 4) as usize;
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
