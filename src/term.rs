use bumpalo::collections::Vec as BumpVec;

use crate::{builtin::DefaultFunction, constant::Constant};

pub enum Term<'a> {
    Var(usize),

    Lambda {
        parameter: usize,
        body: &'a Term<'a>,
    },

    Apply {
        function: &'a Term<'a>,
        argument: &'a Term<'a>,
    },

    Delay(&'a Term<'a>),

    Force(&'a Term<'a>),

    Case {
        constr: &'a Term<'a>,
        branches: BumpVec<'a, Term<'a>>,
    },

    Constr {
        // TODO: revisit what the best type is for this
        tag: usize,
        fields: BumpVec<'a, Term<'a>>,
    },

    Constant(Constant<'a>),

    Builtin(DefaultFunction),

    Error,
}
