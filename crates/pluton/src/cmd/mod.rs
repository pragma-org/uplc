use clap::Parser;

mod eval;

/// Pluton a swiss army knife for Untyped Plutus Core
#[derive(Parser)]
pub enum Cmd {
    /// Evaluate an Untyped Plutus Core program
    Eval(eval::Args),
}

impl Default for Cmd {
    fn default() -> Cmd {
        Cmd::parse()
    }
}

impl Cmd {
    pub fn exec(self) -> miette::Result<()> {
        match self {
            Cmd::Eval(args) => args.exec(),
        }
    }
}
