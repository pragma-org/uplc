pub mod builtin;
pub mod constant;
pub mod machine;
pub mod program;
pub mod term;

#[cfg(test)]
mod tests {
    use crate::program::Version;

    use super::program::Program;
    use super::term::Term;

    #[test]
    fn it_works() {
        let arena = bumpalo::Bump::new();

        let term = arena.alloc(Term::Var(2));

        let version = Version::plutus_v3(&arena);

        let program = Program::new(&arena, version, term);

        assert!(program.version.is_plutus_v3())
    }
}
