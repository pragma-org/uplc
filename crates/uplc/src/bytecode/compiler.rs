use crate::{
    binder::{DeBruijn, Eval},
    constant::{Constant, Integer},
    term::Term,
};

use super::{write_u32, CompiledProgram, Op};

/// Compile a UPLC term tree into bytecode.
pub fn compile<'a>(
    version: (usize, usize, usize),
    term: &'a Term<'a, DeBruijn>,
) -> CompiledProgram<'a> {
    let mut compiler = Compiler {
        bytecode: Vec::with_capacity(4096),
        constant_pool: Vec::new(),
        lambdas: Vec::new(),
        delays: Vec::new(),
    };

    compiler.compile_term(term);

    CompiledProgram {
        bytecode: compiler.bytecode,
        constant_pool: compiler.constant_pool,
        version,
        lambdas: compiler.lambdas,
        delays: compiler.delays,
    }
}

struct Compiler<'a> {
    bytecode: Vec<u8>,
    constant_pool: Vec<&'a Constant<'a>>,
    lambdas: Vec<super::LambdaInfo<'a>>,
    delays: Vec<super::DelayInfo<'a>>,
}

impl<'a> Compiler<'a> {
    fn compile_term(&mut self, term: &'a Term<'a, DeBruijn>) {
        match term {
            Term::Var(db) => {
                let idx = db.index();
                if idx <= 255 {
                    self.emit(Op::Var as u8);
                    self.emit(idx as u8);
                } else {
                    self.emit(Op::VarBig as u8);
                    self.bytecode.extend_from_slice(&(idx as u32).to_le_bytes());
                }
            }

            Term::Lambda { parameter, body } => {
                let lambda_id = self.lambdas.len() as u16;
                self.lambdas.push(super::LambdaInfo { parameter, body });
                self.emit(Op::Lambda as u8);
                let hole = self.emit_u32_hole();
                self.emit((lambda_id & 0xFF) as u8);
                self.emit(((lambda_id >> 8) & 0xFF) as u8);
                self.patch_u32(hole, self.bytecode.len() as u32);
                self.compile_term(body);
            }

            // Superinstruction: Apply(Lambda(body), arg)
            Term::Apply {
                function: Term::Lambda { parameter, body },
                argument,
            } => {
                let lambda_id = self.lambdas.len() as u16;
                self.lambdas.push(super::LambdaInfo { parameter, body });
                self.emit(Op::ApplyLambda as u8);
                let body_hole = self.emit_u32_hole();
                self.emit((lambda_id & 0xFF) as u8);
                self.emit(((lambda_id >> 8) & 0xFF) as u8);
                self.compile_term(argument);
                self.patch_u32(body_hole, self.bytecode.len() as u32);
                self.compile_term(body);
            }

            // Superinstruction: Apply(Var(idx), arg)
            // Var lookup is immediate, so skip FrameAwaitFunTerm — directly
            // push FrameAwaitArg with the looked-up value, then compute arg.
            Term::Apply {
                function: Term::Var(db),
                argument,
            } if db.index() <= 255 => {
                self.emit(Op::ApplyVar as u8);
                self.emit(db.index() as u8);
                // arg bytecode follows inline
                self.compile_term(argument);
            }

            // Superinstruction: Apply(Apply(Apply(f, arg1), arg2), arg3) — depth 3
            Term::Apply {
                function:
                    Term::Apply {
                        function:
                            Term::Apply {
                                function: inner_fun,
                                argument: arg1,
                            },
                        argument: arg2,
                    },
                argument: arg3,
            } if !matches!(inner_fun, Term::Lambda { .. }) => {
                self.emit(Op::Apply3 as u8);
                let arg1_hole = self.emit_u32_hole();
                let arg2_hole = self.emit_u32_hole();
                let arg3_hole = self.emit_u32_hole();
                self.compile_term(inner_fun);
                self.patch_u32(arg1_hole, self.bytecode.len() as u32);
                self.compile_term(arg1);
                self.patch_u32(arg2_hole, self.bytecode.len() as u32);
                self.compile_term(arg2);
                self.patch_u32(arg3_hole, self.bytecode.len() as u32);
                self.compile_term(arg3);
            }

            // Superinstruction: Apply(Apply(f, arg1), arg2) — depth 2
            Term::Apply {
                function:
                    Term::Apply {
                        function: inner_fun,
                        argument: arg1,
                    },
                argument: arg2,
            } if !matches!(inner_fun, Term::Lambda { .. }) => {
                self.emit(Op::Apply2 as u8);
                let arg1_hole = self.emit_u32_hole();
                let arg2_hole = self.emit_u32_hole();
                self.compile_term(inner_fun);
                self.patch_u32(arg1_hole, self.bytecode.len() as u32);
                self.compile_term(arg1);
                self.patch_u32(arg2_hole, self.bytecode.len() as u32);
                self.compile_term(arg2);
            }

            Term::Apply { function, argument } => {
                self.emit(Op::Apply as u8);
                let arg_hole = self.emit_u32_hole();
                self.compile_term(function);
                self.patch_u32(arg_hole, self.bytecode.len() as u32);
                self.compile_term(argument);
            }

            Term::Delay(body) => {
                let delay_id = self.delays.len() as u16;
                self.delays.push(super::DelayInfo { body });
                self.emit(Op::Delay as u8);
                let hole = self.emit_u32_hole();
                self.emit((delay_id & 0xFF) as u8);
                self.emit(((delay_id >> 8) & 0xFF) as u8);
                self.patch_u32(hole, self.bytecode.len() as u32);
                self.compile_term(body);
            }

            // Superinstruction: Force(Delay(body))
            Term::Force(Term::Delay(body)) => {
                self.emit(Op::ForceDelay as u8);
                self.compile_term(body);
            }

            // Superinstruction: Force(Force(Builtin(f))) — only if builtin needs 2 forces
            Term::Force(Term::Force(Term::Builtin(f))) if f.force_count() >= 2 => {
                self.emit(Op::Force2Builtin as u8);
                self.emit(**f as u8);
            }

            // Superinstruction: Force(Builtin(f)) — only if builtin needs forcing
            Term::Force(Term::Builtin(f)) if f.force_count() >= 1 => {
                self.emit(Op::ForceBuiltin as u8);
                self.emit(**f as u8);
            }

            // Superinstruction: Force(Var(idx))
            Term::Force(Term::Var(db)) if db.index() <= 255 => {
                self.emit(Op::ForceVar as u8);
                self.emit(db.index() as u8);
            }

            Term::Force(body) => {
                self.emit(Op::Force as u8);
                self.compile_term(body);
            }

            Term::Constr { tag, fields } => {
                if *tag <= 255 {
                    self.emit(Op::Constr as u8);
                    self.emit(*tag as u8);
                } else {
                    self.emit(Op::ConstrBig as u8);
                    self.bytecode
                        .extend_from_slice(&(*tag as u64).to_le_bytes());
                }
                self.emit(fields.len() as u8);

                // Emit field offset holes
                let holes: Vec<usize> = (0..fields.len()).map(|_| self.emit_u32_hole()).collect();

                // Compile each field and patch its offset
                for (i, field) in fields.iter().enumerate() {
                    self.patch_u32(holes[i], self.bytecode.len() as u32);
                    self.compile_term(field);
                }
            }

            Term::Case { constr, branches } => {
                self.emit(Op::Case as u8);
                self.emit(branches.len() as u8);

                // Emit branch offset holes
                let holes: Vec<usize> = (0..branches.len()).map(|_| self.emit_u32_hole()).collect();

                // Compile scrutinee (follows inline after offset table)
                self.compile_term(constr);

                // Compile each branch and patch its offset
                for (i, branch) in branches.iter().enumerate() {
                    self.patch_u32(holes[i], self.bytecode.len() as u32);
                    self.compile_term(branch);
                }
            }

            // Specialized constants
            Term::Constant(Constant::Unit) => {
                self.emit(Op::ConstUnit as u8);
            }
            Term::Constant(Constant::Boolean(true)) => {
                self.emit(Op::ConstTrue as u8);
            }
            Term::Constant(Constant::Boolean(false)) => {
                self.emit(Op::ConstFalse as u8);
            }
            Term::Constant(Constant::Integer(i)) if is_small_int(i) => {
                self.emit(Op::ConstSmallInt as u8);
                self.emit(small_int_value(i) as u8);
            }

            Term::Constant(c) => {
                let idx = self.intern_constant(c);
                self.emit(Op::Const as u8);
                self.emit((idx & 0xFF) as u8);
                self.emit(((idx >> 8) & 0xFF) as u8);
            }

            Term::Builtin(f) => {
                self.emit(Op::Builtin as u8);
                self.emit(**f as u8);
            }

            Term::Error => {
                self.emit(Op::Error as u8);
            }
        }
    }

    fn emit(&mut self, byte: u8) {
        self.bytecode.push(byte);
    }

    /// Emit a 4-byte placeholder, return its offset for later patching.
    fn emit_u32_hole(&mut self) -> usize {
        let offset = self.bytecode.len();
        self.bytecode.extend_from_slice(&[0, 0, 0, 0]);
        offset
    }

    /// Patch a previously-emitted u32 hole with the actual value.
    fn patch_u32(&mut self, offset: usize, value: u32) {
        write_u32(&mut self.bytecode, offset, value);
    }

    /// Intern a constant in the pool, returning its index.
    fn intern_constant(&mut self, c: &'a Constant<'a>) -> u16 {
        // Simple linear scan — constant pools are small
        let idx = self.constant_pool.len();
        self.constant_pool.push(c);
        idx as u16
    }
}

fn is_small_int(i: &Integer) -> bool {
    use num::ToPrimitive;
    i.to_i8().is_some()
}

fn small_int_value(i: &Integer) -> i8 {
    use num::ToPrimitive;
    i.to_i8().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{arena::Arena, builtin::DefaultFunction, bytecode::read_u32, syn};

    fn compile_source(source: &str) -> CompiledProgram<'static> {
        let source: &'static str = Box::leak(source.to_string().into_boxed_str());
        let arena = Box::leak(Box::new(Arena::new()));
        let program = syn::parse_program(arena, source)
            .into_result()
            .expect("parse failed");
        compile(
            (
                program.version.major(),
                program.version.minor(),
                program.version.patch(),
            ),
            program.term,
        )
    }

    #[test]
    fn compile_integer_constant() {
        let cp = compile_source("(program 1.0.0 (con integer 42))");
        // Should use ConstSmallInt since 42 fits in i8
        assert_eq!(cp.bytecode[0], Op::ConstSmallInt as u8);
        assert_eq!(cp.bytecode[1] as i8, 42);
    }

    #[test]
    fn compile_large_integer() {
        let cp = compile_source("(program 1.0.0 (con integer 999))");
        // 999 doesn't fit in i8, should use Const
        assert_eq!(cp.bytecode[0], Op::Const as u8);
        assert_eq!(cp.constant_pool.len(), 1);
    }

    #[test]
    fn compile_unit() {
        let cp = compile_source("(program 1.0.0 (con unit ()))");
        assert_eq!(cp.bytecode[0], Op::ConstUnit as u8);
    }

    #[test]
    fn compile_true() {
        let cp = compile_source("(program 1.0.0 (con bool True))");
        assert_eq!(cp.bytecode[0], Op::ConstTrue as u8);
    }

    #[test]
    fn compile_identity() {
        let cp = compile_source("(program 1.0.0 (lam x x))");
        assert_eq!(cp.bytecode[0], Op::Lambda as u8);
        // body offset at bytes 1-4, then body starts with Var
        let body_offset = read_u32(&cp.bytecode, 1) as usize;
        assert_eq!(cp.bytecode[body_offset], Op::Var as u8);
    }

    #[test]
    fn compile_force_delay() {
        let cp = compile_source("(program 1.0.0 (force (delay (con integer 1))))");
        assert_eq!(cp.bytecode[0], Op::ForceDelay as u8);
        assert_eq!(cp.bytecode[1], Op::ConstSmallInt as u8);
    }

    #[test]
    fn compile_apply_lambda() {
        let cp = compile_source("(program 1.0.0 [(lam x x) (con integer 5)])");
        assert_eq!(cp.bytecode[0], Op::ApplyLambda as u8);
    }

    #[test]
    fn compile_force_builtin() {
        let cp = compile_source("(program 1.0.0 (force (builtin headList)))");
        assert_eq!(cp.bytecode[0], Op::ForceBuiltin as u8);
    }

    #[test]
    fn compile_force_force_builtin() {
        let cp = compile_source("(program 1.0.0 (force (force (builtin fstPair))))");
        assert_eq!(cp.bytecode[0], Op::Force2Builtin as u8);
    }

    #[test]
    fn compile_error() {
        let cp = compile_source("(program 1.0.0 (error))");
        assert_eq!(cp.bytecode[0], Op::Error as u8);
    }

    #[test]
    fn compile_builtin() {
        let cp = compile_source("(program 1.0.0 (builtin addInteger))");
        assert_eq!(cp.bytecode[0], Op::Builtin as u8);
        assert_eq!(cp.bytecode[1], DefaultFunction::AddInteger as u8);
    }
}
