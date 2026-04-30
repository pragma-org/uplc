use amaru_uplc::{arena::Arena, machine::PlutusVersion, syn};

fn main() {
    let path = std::env::args().nth(1).expect("provide a .uplc file");
    let source = std::fs::read_to_string(&path).unwrap();
    let arena = Arena::new();
    let program = syn::parse_program(&arena, &source)
        .into_result()
        .expect("parse failed");
    let result = program.eval_version(&arena, PlutusVersion::V3);
    let info = result.info;

    match result.term {
        Ok(term) => {
            // Print expected output in program format
            println!("(program 1.1.0 {term:?})");
        }
        Err(_) => {
            println!("evaluation failure");
        }
    }

    // Print budget to stderr
    eprintln!(
        "({{cpu: {}\n| mem: {}}})",
        info.consumed_budget.cpu, info.consumed_budget.mem
    );
}
