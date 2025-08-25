use std::io::{self, Read};

use miette::IntoDiagnostic;

#[derive(clap::Args)]
pub struct Args {
    #[clap(short, long)]
    file: Option<String>,
    #[clap(long)]
    flat: bool,
    #[clap(short = 'A', long)]
    args: Vec<String>,
}

impl Args {
    pub fn exec(self) -> miette::Result<()> {
        let arena = uplc_turbo::bumpalo::Bump::with_capacity(1_024_000);

        let program = if let Some(file_path) = self.file {
            std::fs::read(file_path).into_diagnostic()?
        } else {
            let mut buffer = Vec::new();

            io::stdin().read(&mut buffer).into_diagnostic()?;

            buffer
        };

        let mut program_string = String::new();

        let program = if self.flat {
            uplc_turbo::flat::decode(&arena, &program).into_diagnostic()?
        } else {
            {
                let temp = String::from_utf8(program).into_diagnostic()?;

                program_string.push_str(&temp);
            }

            let parse_result =
                uplc_turbo::syn::parse_program(&arena, &program_string).into_result();

            match parse_result {
                Ok(program) => program,
                Err(errs) => {
                    let errs = errs
                        .into_iter()
                        .map(|e| format!("{e}"))
                        .collect::<Vec<_>>()
                        .join("\n");

                    miette::bail!("failed to parse program\n{}", errs);
                }
            }
        };

        let mut parsed_args = vec![];

        for (index, arg) in self.args.iter().enumerate() {
            let parse_result = uplc_turbo::syn::parse_term(&arena, arg).into_result();

            let term = match parse_result {
                Ok(term) => term,
                Err(errs) => {
                    let errs = errs
                        .into_iter()
                        .map(|e| format!("{e}"))
                        .collect::<Vec<_>>()
                        .join("\n");

                    miette::bail!("failed to parse argument {}: {}\n{}", index + 1, arg, errs);
                }
            };

            parsed_args.push(term);
        }

        let program = parsed_args
            .into_iter()
            .fold(program, |program, arg| program.apply(&arena, arg));

        let eval_result = program.eval(&arena);

        println!("{eval_result:#?}");

        Ok(())
    }
}
