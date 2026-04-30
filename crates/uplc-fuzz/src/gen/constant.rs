use rand::Rng;

use crate::seed::{ConstantSeed, DataSeed, TypeSeed};

/// Generate a random constant of a given type.
pub fn gen_constant_of_type(rng: &mut impl Rng, ty: &TypeSeed, depth: usize) -> ConstantSeed {
    match ty {
        TypeSeed::Integer => ConstantSeed::Integer(gen_integer(rng)),
        TypeSeed::Bool => ConstantSeed::Boolean(rng.gen()),
        TypeSeed::ByteString => ConstantSeed::ByteString(gen_bytestring(rng)),
        TypeSeed::String => ConstantSeed::String(gen_string(rng)),
        TypeSeed::Unit => ConstantSeed::Unit,
        TypeSeed::Data => ConstantSeed::Data(gen_data(rng, depth.min(3))),
        TypeSeed::List(inner) => {
            let len = rng.gen_range(0..=3);
            let items = (0..len)
                .map(|_| gen_constant_of_type(rng, inner, depth.saturating_sub(1)))
                .collect();
            ConstantSeed::List(inner.as_ref().clone(), items)
        }
        TypeSeed::Pair(fst, snd) => ConstantSeed::Pair(
            fst.as_ref().clone(),
            snd.as_ref().clone(),
            Box::new(gen_constant_of_type(rng, fst, depth.saturating_sub(1))),
            Box::new(gen_constant_of_type(rng, snd, depth.saturating_sub(1))),
        ),
    }
}

/// Generate a random constant (any type).
pub fn gen_constant(rng: &mut impl Rng) -> ConstantSeed {
    let ty = gen_type(rng, 2);
    gen_constant_of_type(rng, &ty, 3)
}

/// Generate a random type (for constant generation).
pub fn gen_type(rng: &mut impl Rng, depth: usize) -> TypeSeed {
    if depth == 0 {
        return gen_leaf_type(rng);
    }
    match rng.gen_range(0..10) {
        0 => TypeSeed::List(Box::new(gen_type(rng, depth - 1))),
        1 => TypeSeed::Pair(
            Box::new(gen_type(rng, depth - 1)),
            Box::new(gen_type(rng, depth - 1)),
        ),
        _ => gen_leaf_type(rng),
    }
}

fn gen_leaf_type(rng: &mut impl Rng) -> TypeSeed {
    match rng.gen_range(0..6) {
        0 => TypeSeed::Integer,
        1 => TypeSeed::Bool,
        2 => TypeSeed::ByteString,
        3 => TypeSeed::String,
        4 => TypeSeed::Unit,
        _ => TypeSeed::Data,
    }
}

pub fn gen_integer(rng: &mut impl Rng) -> i128 {
    // Bias toward small values but occasionally produce large ones
    match rng.gen_range(0..20) {
        0 => 0,
        1 => 1,
        2 => -1,
        3 => 2,
        4..=6 => rng.gen_range(-100..=100),
        7..=10 => rng.gen_range(-10_000..=10_000),
        11..=14 => rng.gen_range(-1_000_000..=1_000_000),
        15..=17 => rng.gen_range(i64::MIN as i128..=i64::MAX as i128),
        // Boundary values
        18 => i128::from(i64::MAX),
        _ => i128::from(i64::MIN),
    }
}

pub fn gen_bytestring(rng: &mut impl Rng) -> Vec<u8> {
    let len = match rng.gen_range(0..10) {
        0 => 0,
        1..=4 => rng.gen_range(1..=8),
        5..=7 => rng.gen_range(1..=32),
        8 => rng.gen_range(1..=64),
        _ => rng.gen_range(28..=32), // common hash sizes
    };
    (0..len).map(|_| rng.gen()).collect()
}

fn gen_string(rng: &mut impl Rng) -> String {
    let len = rng.gen_range(0..=16);
    (0..len)
        .map(|_| {
            // ASCII printable, avoiding backslash and quote for simplicity
            let c = rng.gen_range(0x20..=0x7E) as u8;
            if c == b'\\' || c == b'"' {
                'a'
            } else {
                c as char
            }
        })
        .collect()
}

fn gen_data(rng: &mut impl Rng, depth: usize) -> DataSeed {
    if depth == 0 {
        return match rng.gen_range(0..2) {
            0 => DataSeed::Integer(gen_integer(rng)),
            _ => DataSeed::ByteString(gen_bytestring(rng)),
        };
    }
    match rng.gen_range(0..5) {
        0 => {
            let tag = rng.gen_range(0u64..=5);
            let nfields = rng.gen_range(0..=3);
            let fields = (0..nfields).map(|_| gen_data(rng, depth - 1)).collect();
            DataSeed::Constr(tag, fields)
        }
        1 => {
            let len = rng.gen_range(0..=2);
            let entries = (0..len)
                .map(|_| (gen_data(rng, depth - 1), gen_data(rng, depth - 1)))
                .collect();
            DataSeed::Map(entries)
        }
        2 => {
            let len = rng.gen_range(0..=3);
            let items = (0..len).map(|_| gen_data(rng, depth - 1)).collect();
            DataSeed::List(items)
        }
        3 => DataSeed::Integer(gen_integer(rng)),
        _ => DataSeed::ByteString(gen_bytestring(rng)),
    }
}
