use amaru_uplc::{
    arena::Arena,
    binder::{DeBruijn, Eval},
    flat,
    term::Term,
};

fn max_var_index<'a>(term: &'a Term<'a, DeBruijn>) -> usize {
    match term {
        Term::Var(db) => db.index(),
        Term::Lambda { body, .. } => max_var_index(body),
        Term::Apply { function, argument } => max_var_index(function).max(max_var_index(argument)),
        Term::Delay(body) => max_var_index(body),
        Term::Force(body) => max_var_index(body),
        Term::Constr { fields, .. } => fields.iter().map(|f| max_var_index(f)).max().unwrap_or(0),
        Term::Case { constr, branches } => {
            let c = max_var_index(constr);
            let b = branches.iter().map(|b| max_var_index(b)).max().unwrap_or(0);
            c.max(b)
        }
        _ => 0,
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("provide a .flat file");
    let script = std::fs::read(&path).unwrap();
    let arena = Arena::new();
    let program = flat::decode_ungated::<DeBruijn>(&arena, &script).expect("decode failed");
    let max = max_var_index(program.term);
    eprintln!("Max De Bruijn index: {}", max);
    if max > 255 {
        eprintln!(
            "WARNING: index {} exceeds u8 range — bytecode Var opcode will truncate!",
            max
        );
    }
}
