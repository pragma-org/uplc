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
}
