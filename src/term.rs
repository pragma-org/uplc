use bumpalo::{collections::Vec as BumpVec, Bump};

use crate::{builtin::DefaultFunction, constant::Constant};

#[derive(Debug, PartialEq)]
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

    Constant(&'a Constant<'a>),

    Builtin(&'a DefaultFunction),

    Error,
}

impl<'a> Term<'a> {
    pub fn var(arena: &'a Bump, i: usize) -> &'a Term<'a> {
        arena.alloc(Term::Var(i))
    }

    pub fn apply(&'a self, arena: &'a Bump, argument: &'a Term<'a>) -> &'a Term<'a> {
        arena.alloc(Term::Apply {
            function: self,
            argument,
        })
    }

    pub fn constant(arena: &'a Bump, constant: &'a Constant<'a>) -> &'a Term<'a> {
        arena.alloc(Term::Constant(constant))
    }

    pub fn integer(arena: &'a Bump, i: i128) -> &'a Term<'a> {
        let constant = arena.alloc(Constant::Integer(i));

        Term::constant(arena, constant)
    }

    pub fn builtin(arena: &'a Bump, fun: &'a DefaultFunction) -> &'a Term<'a> {
        arena.alloc(Term::Builtin(fun))
    }

    pub fn add_integer(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::AddInteger);
        Term::builtin(arena, fun)
    }
}
