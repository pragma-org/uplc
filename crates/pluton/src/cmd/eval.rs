use std::io::{self, Read};

use miette::IntoDiagnostic;

#[derive(clap::Args)]
pub struct Args {
    #[clap(short, long)]
    file: Option<String>,
    #[clap(short = 'A', long)]
    args: Vec<String>,
}

impl Args {
    pub fn exec(self) -> miette::Result<()> {
        let program = if let Some(file_path) = self.file {
            std::fs::read_to_string(file_path).into_diagnostic()?
        } else {
            let mut buffer = String::new();

            io::stdin().read_to_string(&mut buffer).into_diagnostic()?;

            buffer
        };

        let arena = uplc::bumpalo::Bump::new();

        let parse_result = uplc::syn::parse_program(&arena, &program).into_result();

        let program = match parse_result {
            Ok(program) => program,
            Err(errs) => {
                let errs = errs
                    .into_iter()
                    .map(|e| format!("{}", e))
                    .collect::<Vec<_>>()
                    .join("\n");

                miette::bail!("failed to parse program\n{}", errs);
            }
        };

        let mut parsed_args = vec![];

        for (index, arg) in self.args.iter().enumerate() {
            let parse_result = uplc::syn::parse_term(&arena, arg).into_result();

            let term = match parse_result {
                Ok(term) => term,
                Err(errs) => {
                    let errs = errs
                        .into_iter()
                        .map(|e| format!("{}", e))
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

        println!("{:#?}", eval_result);

        Ok(())
    }
}
