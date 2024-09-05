use bumpalo::Bump;

use crate::term::Term;

use super::{env::Env, value::Value};

pub fn value_as_term<'a>(arena: &'a Bump, value: &'a Value<'a>) -> &'a Term<'a> {
    match value {
        Value::Con(x) => arena.alloc(Term::Constant(x)),
        Value::Builtin(runtime) => {
            // let mut term = Term::Builtin(fun);

            // for _ in 0..runtime.forces {
            //     term = term.force();
            // }

            // for arg in runtime.args {
            //     term = term.apply(value_as_term(arg));
            // }

            // term
            todo!()
        }
        Value::Delay(body, env) => {
            // with_env(0, env, Term::Delay(body)),
            //
            todo!()
        }
        Value::Lambda {
            parameter,
            body,
            env,
        } => {
            // with_env(
            //     0,
            //     env,
            //     Term::Lambda {
            //         parameter_name: NamedDeBruijn {
            //             text: parameter_name.text.clone(),
            //             index: 0.into(),
            //         }
            //         .into(),
            //         body,
            //     },
            // ),
            todo!()
        } // Value::Constr { tag, fields } => {
        //     //     Term::Constr {
        //     //     tag,
        //     //     fields: fields.into_iter().map(value_as_term).collect(),
        //     // },

        //     todo!()
        // }
        _ => todo!(),
    }
}

fn with_env<'a>(lam_cnt: usize, env: &'a Env<'a>, term: &'a Term<'a>) -> &'a Term<'a> {
    // match term {
    //     Term::Var(name) => {
    //         let index: usize = name.index.into();

    //         if lam_cnt >= index {
    //             Term::Var(name)
    //         } else {
    //             env.get::<usize>(env.len() - (index - lam_cnt))
    //                 .cloned()
    //                 .map_or(Term::Var(name), value_as_term)
    //         }
    //     }
    //     Term::Lambda {
    //         parameter_name,
    //         body,
    //     } => {
    //         let body = with_env(lam_cnt + 1, env, body.as_ref().clone());

    //         Term::Lambda {
    //             parameter_name,
    //             body: body.into(),
    //         }
    //     }
    //     Term::Apply { function, argument } => {
    //         let function = with_env(lam_cnt, env.clone(), function.as_ref().clone());
    //         let argument = with_env(lam_cnt, env, argument.as_ref().clone());

    //         Term::Apply {
    //             function: function.into(),
    //             argument: argument.into(),
    //         }
    //     }

    //     Term::Delay(x) => {
    //         let delay = with_env(lam_cnt, env, x.as_ref().clone());

    //         Term::Delay(delay.into())
    //     }
    //     Term::Force(x) => {
    //         let force = with_env(lam_cnt, env, x.as_ref().clone());

    //         Term::Force(force.into())
    //     }
    //     rest => rest,
    // }
    todo!()
}
