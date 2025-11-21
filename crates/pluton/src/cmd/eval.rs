use std::io::{self, Read};

use miette::IntoDiagnostic;
use uplc_turbo::machine::PlutusVersion;

#[derive(clap::Args)]
pub struct Args {
    #[clap(short, long)]
    file: Option<String>,
    #[clap(long)]
    flat: bool,
    #[clap(short = 'A', long)]
    args: Vec<String>,
    #[clap(short = 'v', long)]
    plutus_version: Option<String>,
}

fn parse_plutus_version(s: &str) -> Result<PlutusVersion, String> {
    match s.to_lowercase().as_str() {
        "v1" => Ok(PlutusVersion::V1),
        "v2" => Ok(PlutusVersion::V2),
        "v3" => Ok(PlutusVersion::V3),
        _ => Err(format!(
            "Unknown Plutus version: '{s}'. Valid options: v1, v2, v3"
        )),
    }
}

impl Args {
    pub fn exec(self) -> miette::Result<()> {
        let handle = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(move || self.exec_inner())
            .into_diagnostic()?;

        handle
            .join()
            .map_err(|_| miette::miette!("Thread panicked"))?
    }

    fn exec_inner(self) -> miette::Result<()> {
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

        if let Some(version_str) = self.plutus_version {
            let version =
                parse_plutus_version(&version_str).map_err(|e| miette::miette!("{}", e))?;

            let eval_result = program.eval_version(&arena, version);
            println!("{eval_result:#?}");
        } else {
            let eval_result = program.eval(&arena);

            println!("{eval_result:#?}");
        }

        Ok(())
    }
}
