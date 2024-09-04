pub mod builtin;
pub mod constant;
pub mod machine;
pub mod program;
pub mod term;
pub mod typ;

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::machine::Machine;
    use crate::program::Version;

    use super::program::Program;
    use super::term::Term;

    #[test]
    fn it_works() {
        let arena = bumpalo::Bump::new();

        let term = Term::add_integer(&arena)
            .apply(&arena, Term::integer(&arena, 1))
            .apply(&arena, Term::integer(&arena, 3));

        let version = Version::plutus_v3(&arena);

        let program = Program::new(&arena, version, term);

        let mut machine = Machine::new(&arena);

        let result = machine.run(term);

        assert_eq!(result.result.unwrap(), Term::integer(&arena, 4));
    }
}
