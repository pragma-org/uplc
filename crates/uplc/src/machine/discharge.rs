use bumpalo::collections::CollectIn;
use bumpalo::collections::Vec as BumpVec;

use crate::arena::Arena;
use crate::{binder::Eval, term::Term};

use super::{env::Env, value::Value};

pub fn value_as_term<'a, V>(arena: &'a Arena, value: &'a Value<'a, V>) -> &'a Term<'a, V>
where
    V: Eval<'a>,
{
    match value {
        Value::Con(x) => arena.alloc(Term::Constant(x)),
        Value::Builtin(runtime) => {
            let mut term = Term::builtin(arena, runtime.fun);

            for _ in 0..runtime.forces {
                term = term.force(arena);
            }

            for i in 0..runtime.arg_count() {
                term = term.apply(arena, value_as_term(arena, runtime.arg(i)));
            }

            term
        }
        Value::Delay(body, env) => with_env(arena, 0, env, body.delay(arena)),
        Value::Lambda {
            parameter,
            body,
            env,
        } => with_env(arena, 0, env, body.lambda(arena, parameter)),
        Value::Constr(tag, fields) => {
            let fields: BumpVec<'_, _> = fields
                .iter()
                .map(|value| value_as_term(arena, value))
                .collect_in(arena.as_bump());

            let fields = arena.alloc(fields);

            Term::constr(arena, *tag, fields)
        }
        Value::LambdaBC { lambda_id, env, .. } => {
            // Discharge requires AST lookup tables — fall back to error
            // unless called via value_as_term_bc with context
            Term::error(arena)
        }
        Value::DelayBC { delay_id, env, .. } => {
            Term::error(arena)
        }
    }
}

/// Discharge a value to a term with bytecode closure support.
/// Requires lookup tables from the compiled program.
pub fn value_as_term_bc<'a>(
    arena: &'a Arena,
    value: &'a Value<'a, crate::binder::DeBruijn>,
    lambdas: &[crate::bytecode::LambdaInfo<'a>],
    delays: &[crate::bytecode::DelayInfo<'a>],
) -> &'a Term<'a, crate::binder::DeBruijn> {
    use crate::binder::DeBruijn;
    use bumpalo::collections::CollectIn;

    match value {
        Value::Con(x) => arena.alloc(Term::Constant(x)),
        Value::Builtin(runtime) => {
            let mut term = Term::builtin(arena, runtime.fun);
            for _ in 0..runtime.forces {
                term = term.force(arena);
            }
            for i in 0..runtime.arg_count() {
                term = term.apply(arena, value_as_term_bc(arena, runtime.arg(i), lambdas, delays));
            }
            term
        }
        Value::Lambda { parameter, body, env } => {
            with_env_bc(arena, 0, env, body.lambda(arena, parameter), lambdas, delays)
        }
        Value::Delay(body, env) => with_env_bc(arena, 0, env, body.delay(arena), lambdas, delays),
        Value::Constr(tag, fields) => {
            let fields: BumpVec<'_, _> = fields
                .iter()
                .map(|v| value_as_term_bc(arena, v, lambdas, delays))
                .collect_in(arena.as_bump());
            let fields = arena.alloc(fields);
            Term::constr(arena, *tag, fields)
        }
        Value::LambdaBC { lambda_id, env, .. } => {
            let info = &lambdas[*lambda_id as usize];
            with_env_bc(arena, 0, env, info.body.lambda(arena, info.parameter), lambdas, delays)
        }
        Value::DelayBC { delay_id, env, .. } => {
            let info = &delays[*delay_id as usize];
            with_env_bc(arena, 0, env, info.body.delay(arena), lambdas, delays)
        }
    }
}

fn with_env<'a, V>(
    arena: &'a Arena,
    lam_cnt: usize,
    env: &'a Env<'a, V>,
    term: &'a Term<'a, V>,
) -> &'a Term<'a, V>
where
    V: Eval<'a>,
{
    match term {
        Term::Var(name) => {
            let index = name.index();

            if lam_cnt >= index {
                Term::var(arena, name)
            } else {
                env.lookup(index - lam_cnt).map_or_else(
                    || Term::var(arena, *name),
                    |value| value_as_term(arena, value),
                )
            }
        }
        Term::Lambda { parameter, body } => {
            let body = with_env(arena, lam_cnt + 1, env, body);

            body.lambda(arena, *parameter)
        }
        Term::Apply { function, argument } => {
            let function = with_env(arena, lam_cnt, env, function);
            let argument = with_env(arena, lam_cnt, env, argument);

            function.apply(arena, argument)
        }

        Term::Delay(x) => {
            let body = with_env(arena, lam_cnt, env, x);

            body.delay(arena)
        }
        Term::Force(x) => {
            let body = with_env(arena, lam_cnt, env, x);

            body.force(arena)
        }
        Term::Case { constr, branches } => {
            let constr = with_env(arena, lam_cnt, env, constr);

            let branches: BumpVec<'_, _> = branches
                .iter()
                .map(|b| with_env(arena, lam_cnt, env, b))
                .collect_in(arena.as_bump());

            let branches = arena.alloc(branches);

            Term::case(arena, constr, branches)
        }
        Term::Constr { tag, fields } => {
            let fields: BumpVec<'_, _> = fields
                .iter()
                .map(|f| with_env(arena, lam_cnt, env, f))
                .collect_in(arena.as_bump());

            let fields = arena.alloc(fields);

            Term::constr(arena, *tag, fields)
        }
        rest => rest,
    }
}

fn with_env_bc<'a>(
    arena: &'a Arena,
    lam_cnt: usize,
    env: &'a Env<'a, crate::binder::DeBruijn>,
    term: &'a Term<'a, crate::binder::DeBruijn>,
    lambdas: &[crate::bytecode::LambdaInfo<'a>],
    delays: &[crate::bytecode::DelayInfo<'a>],
) -> &'a Term<'a, crate::binder::DeBruijn> {
    match term {
        Term::Var(name) => {
            let index = name.index();

            if lam_cnt >= index {
                Term::var(arena, name)
            } else {
                env.lookup(index - lam_cnt).map_or_else(
                    || Term::var(arena, *name),
                    |value| value_as_term_bc(arena, value, lambdas, delays),
                )
            }
        }
        Term::Lambda { parameter, body } => {
            let body = with_env_bc(arena, lam_cnt + 1, env, body, lambdas, delays);

            body.lambda(arena, *parameter)
        }
        Term::Apply { function, argument } => {
            let function = with_env_bc(arena, lam_cnt, env, function, lambdas, delays);
            let argument = with_env_bc(arena, lam_cnt, env, argument, lambdas, delays);

            function.apply(arena, argument)
        }
        Term::Delay(x) => {
            let body = with_env_bc(arena, lam_cnt, env, x, lambdas, delays);

            body.delay(arena)
        }
        Term::Force(x) => {
            let body = with_env_bc(arena, lam_cnt, env, x, lambdas, delays);

            body.force(arena)
        }
        Term::Case { constr, branches } => {
            let constr = with_env_bc(arena, lam_cnt, env, constr, lambdas, delays);

            let branches: BumpVec<'_, _> = branches
                .iter()
                .map(|b| with_env_bc(arena, lam_cnt, env, b, lambdas, delays))
                .collect_in(arena.as_bump());

            let branches = arena.alloc(branches);

            Term::case(arena, constr, branches)
        }
        Term::Constr { tag, fields } => {
            let fields: BumpVec<'_, _> = fields
                .iter()
                .map(|f| with_env_bc(arena, lam_cnt, env, f, lambdas, delays))
                .collect_in(arena.as_bump());

            let fields = arena.alloc(fields);

            Term::constr(arena, *tag, fields)
        }
        rest => rest,
    }
}
