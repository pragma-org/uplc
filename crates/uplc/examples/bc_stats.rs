use uplc_turbo::{
    arena::Arena,
    binder::DeBruijn,
    bytecode::{compiler, read_u16, read_u32, Op},
    flat,
};

fn main() {
    let script_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "benches/use_cases/plutus_use_cases/auction_1-2.flat".to_string());

    let script = std::fs::read(&script_path).expect("Failed to read script");
    let arena = Arena::new();
    let program = flat::decode::<DeBruijn>(&arena, &script).expect("Failed to decode");

    let compiled = compiler::compile(
        (
            program.version.major(),
            program.version.minor(),
            program.version.patch(),
        ),
        program.term,
    );

    let bc = &compiled.bytecode;
    let mut counts = std::collections::HashMap::new();
    let mut ip = 0;

    while ip < bc.len() {
        let op = bc[ip];
        *counts.entry(op).or_insert(0u32) += 1;
        ip += 1;

        // Skip operands
        match op {
            0x01 => ip += 1, // Var: u8
            0x16 => ip += 4, // VarBig: u32
            0x02 => ip += 6, // Lambda: u32 + u16
            0x03 => ip += 4, // Apply: u32
            0x04 => ip += 6, // Delay: u32 + u16
            0x05 => {}       // Force: no operands
            0x06 => {
                // Constr: u8 tag + u8 nfields + offsets
                ip += 1; // tag
                let nfields = bc[ip] as usize;
                ip += 1;
                ip += nfields * 4;
            }
            0x0B => {
                // ConstrBig: u64 tag + u8 nfields + offsets
                ip += 8;
                let nfields = bc[ip] as usize;
                ip += 1;
                ip += nfields * 4;
            }
            0x07 => {
                // Case: u8 nbranches + offsets
                let nb = bc[ip] as usize;
                ip += 1;
                ip += nb * 4;
            }
            0x08 => ip += 2,  // Const: u16
            0x09 => ip += 1,  // Builtin: u8
            0x0A => {}        // Error
            0x10 => {}        // ForceDelay
            0x11 => ip += 6,  // ApplyLambda: u32 + u16
            0x12 => ip += 1,  // ForceBuiltin: u8
            0x13 => ip += 1,  // Force2Builtin: u8
            0x14 => ip += 1,  // ApplyVar: u8 idx
            0x17 => ip += 8,  // Apply2: u32 + u32
            0x18 => ip += 12, // Apply3: u32 + u32 + u32
            0x15 => ip += 1,  // ForceVar: u8 idx
            0x20 => {}        // ConstUnit
            0x21 => {}        // ConstTrue
            0x22 => {}        // ConstFalse
            0x23 => ip += 1,  // ConstSmallInt: i8
            _ => {
                eprintln!("Unknown opcode {:#04x} at {}", op, ip - 1);
                break;
            }
        }
    }

    let op_names = [
        (0x01, "Var"),
        (0x02, "Lambda"),
        (0x03, "Apply"),
        (0x04, "Delay"),
        (0x05, "Force"),
        (0x06, "Constr"),
        (0x07, "Case"),
        (0x08, "Const"),
        (0x09, "Builtin"),
        (0x0A, "Error"),
        (0x0B, "ConstrBig"),
        (0x16, "VarBig"),
        (0x17, "Apply2"),
        (0x18, "Apply3"),
        (0x10, "ForceDelay"),
        (0x11, "ApplyLambda"),
        (0x12, "ForceBuiltin"),
        (0x13, "Force2Builtin"),
        (0x14, "ApplyVar"),
        (0x15, "ForceVar"),
        (0x20, "ConstUnit"),
        (0x21, "ConstTrue"),
        (0x22, "ConstFalse"),
        (0x23, "ConstSmallInt"),
    ];

    let total: u32 = counts.values().sum();
    let mut sorted: Vec<_> = counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));

    println!("Bytecode: {} bytes, {} opcodes\n", bc.len(), total);
    println!("{:<20} {:>6} {:>8}", "Opcode", "Count", "%");
    println!("{:-<36}", "");
    for (op, count) in &sorted {
        let name = op_names
            .iter()
            .find(|(o, _)| o == *op)
            .map(|(_, n)| *n)
            .unwrap_or("???");
        println!(
            "{:<20} {:>6} {:>7.1}%",
            name,
            count,
            **count as f64 / total as f64 * 100.0
        );
    }

    println!("\nConstant pool: {} entries", compiled.constant_pool.len());
    println!("Lambda info: {} entries", compiled.lambdas.len());
    println!("Delay info: {} entries", compiled.delays.len());
}
