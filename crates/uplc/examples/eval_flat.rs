use uplc_turbo::{arena::Arena, binder::DeBruijn, flat, machine::PlutusVersion};

fn main() {
    let path = std::env::args().nth(1).expect("provide a .flat file");
    let script = std::fs::read(&path).unwrap();
    let arena = Arena::new();
    let program = flat::decode::<DeBruijn>(&arena, &script).expect("decode failed");
    let result = program.eval_version(&arena, PlutusVersion::V3);

    match result.term {
        Ok(term) => eprintln!("OK: {term:?}"),
        Err(e) => eprintln!("FAIL: {e:?}"),
    }
}
