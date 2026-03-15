use uplc_turbo::{
    arena::Arena,
    binder::{DeBruijn, Eval},
    bytecode::compiler,
    flat,
};

fn main() {
    let path = std::env::args().nth(1).expect("provide a .flat file");
    let script = std::fs::read(&path).unwrap();
    let arena = Arena::new();
    let program = flat::decode::<DeBruijn>(&arena, &script).expect("decode failed");
    let compiled = compiler::compile(
        (program.version.major(), program.version.minor(), program.version.patch()),
        program.term,
    );
    // Write raw bytecode to stdout
    use std::io::Write;
    std::io::stdout().write_all(&compiled.bytecode).unwrap();
}
