use bumpalo::{collections::Vec as BumpVec, Bump};

use crate::{
    builtin::DefaultFunction,
    constant::{integer_from, Constant, Integer},
};

#[derive(Debug, PartialEq, Clone)]
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
        branches: BumpVec<'a, &'a Term<'a>>,
    },

    Constr {
        // TODO: revisit what the best type is for this
        tag: usize,
        fields: BumpVec<'a, &'a Term<'a>>,
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

    pub fn lambda(&'a self, arena: &'a Bump, parameter: usize) -> &'a Term<'a> {
        arena.alloc(Term::Lambda {
            parameter,
            body: self,
        })
    }

    pub fn force(&'a self, arena: &'a Bump) -> &'a Term<'a> {
        arena.alloc(Term::Force(self))
    }

    pub fn delay(&'a self, arena: &'a Bump) -> &'a Term<'a> {
        arena.alloc(Term::Delay(self))
    }

    pub fn constant(arena: &'a Bump, constant: &'a Constant<'a>) -> &'a Term<'a> {
        arena.alloc(Term::Constant(constant))
    }

    pub fn constr(arena: &'a Bump, tag: usize, fields: BumpVec<'a, &'a Term<'a>>) -> &'a Term<'a> {
        arena.alloc(Term::Constr { tag, fields })
    }

    pub fn integer(arena: &'a Bump, i: &'a Integer) -> &'a Term<'a> {
        let constant = arena.alloc(Constant::Integer(i));

        Term::constant(arena, constant)
    }

    pub fn integer_from(arena: &'a Bump, i: i128) -> &'a Term<'a> {
        Self::integer(arena, integer_from(arena, i))
    }

    pub fn bytestring(arena: &'a Bump, bytes: BumpVec<'a, u8>) -> &'a Self {
        let constant = arena.alloc(Constant::ByteString(bytes));

        Term::constant(arena, constant)
    }

    pub fn bool(arena: &'a Bump, v: bool) -> &'a Term<'a> {
        let constant = arena.alloc(Constant::Boolean(v));

        Term::constant(arena, constant)
    }

    pub fn unit(arena: &'a Bump) -> &'a Term<'a> {
        let constant = Constant::unit(arena);

        Term::constant(arena, constant)
    }

    pub fn builtin(arena: &'a Bump, fun: &'a DefaultFunction) -> &'a Term<'a> {
        arena.alloc(Term::Builtin(fun))
    }

    pub fn error(arena: &'a Bump) -> &'a Term<'a> {
        arena.alloc(Term::Error)
    }

    pub fn add_integer(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::AddInteger);

        Term::builtin(arena, fun)
    }

    pub fn subtract_integer(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::SubtractInteger);

        Term::builtin(arena, fun)
    }
    pub fn less_than_equals_integer(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::LessThanEqualsInteger);

        Term::builtin(arena, fun)
    }

    pub fn if_then_else(arena: &'a Bump) -> &'a Term<'a> {
        let fun = arena.alloc(DefaultFunction::IfThenElse);

        Term::builtin(arena, fun)
    }
}
