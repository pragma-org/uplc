use rand::Rng;

use crate::seed::{
    builtin_arity, builtin_force_count, ConstantSeed, ProgramSeed, TermSeed, TypeSeed,
};

use super::{
    constant::{gen_bytestring, gen_constant, gen_constant_of_type, gen_integer},
    Generator,
};

/// Generates well-typed builtin applications.
/// For each builtin, wraps with the correct Force count and applies
/// arguments of plausible types. Much more likely to produce non-error results.
pub struct BuiltinAware {
    pub version: (usize, usize, usize),
}

impl Default for BuiltinAware {
    fn default() -> Self {
        Self { version: (1, 1, 0) }
    }
}

impl Generator for BuiltinAware {
    fn generate_batch(
        &self,
        rng: &mut rand_xoshiro::Xoshiro256PlusPlus,
        batch_size: usize,
    ) -> Vec<ProgramSeed> {
        (0..batch_size)
            .map(|_| ProgramSeed {
                version: self.version,
                term: gen_builtin_application(rng),
            })
            .collect()
    }

    fn name(&self) -> &str {
        "builtin_aware"
    }
}

fn gen_builtin_application(rng: &mut impl Rng) -> TermSeed {
    // Pick a builtin (skip BLS and ledger for now — they need special constant types)
    let builtin_id = pick_builtin(rng);
    let forces = builtin_force_count(builtin_id);
    let arity = builtin_arity(builtin_id);

    // Start with the builtin
    let mut term = TermSeed::Builtin(builtin_id);

    // Apply forces
    for _ in 0..forces {
        term = TermSeed::Force(Box::new(term));
    }

    // Apply arguments of appropriate types
    let arg_types = get_arg_types(builtin_id);
    for i in 0..arity {
        let arg = if i < arg_types.len() {
            gen_arg(rng, &arg_types[i])
        } else {
            TermSeed::Constant(gen_constant(rng))
        };
        term = TermSeed::Apply(Box::new(term), Box::new(arg));
    }

    // Sometimes wrap the whole thing in extra structure for interesting interactions
    if rng.gen_range(0..5) == 0 {
        // Wrap in a lambda and apply to itself
        let body = TermSeed::Lambda(Box::new(TermSeed::Var(1)));
        term = TermSeed::Apply(Box::new(body), Box::new(term));
    }

    term
}

fn pick_builtin(rng: &mut impl Rng) -> u8 {
    // Weighted selection: prefer simpler builtins that are more likely to succeed
    let builtins: &[u8] = &[
        // Integer arithmetic (very high weight)
        0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 4, 5, 6, // Integer comparison
        7, 8, 9, // ByteString ops
        10, 11, 12, 13, 14, 15, 16, 17, // String ops
        22, 23, 24, 25, // Control flow
        26, 26, 26, // Data operations
        37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, // Bool/unit
        27, 28, // Pairs and lists
        29, 30, 31, 32, 33, 34, 35, // Misc constructors
        48, 49, 50, 51, // Hashing
        18, 19, 20, 71, 72, 86, // Bitwise
        73, 74, 75, 76, 77, 78, 79, 81, 82, 83, 84, 85, // Advanced
        87, 88, 89, 90, 91,
    ];
    builtins[rng.gen_range(0..builtins.len())]
}

/// Argument type descriptors for common builtins.
#[derive(Clone, Debug)]
enum ArgType {
    Integer,
    ByteString,
    String,
    Bool,
    Unit,
    Data,
    ListData,
    PairDataData,
    Any,
}

fn gen_arg(rng: &mut impl Rng, arg_type: &ArgType) -> TermSeed {
    TermSeed::Constant(match arg_type {
        ArgType::Integer => ConstantSeed::Integer(gen_integer(rng)),
        ArgType::ByteString => ConstantSeed::ByteString(gen_bytestring(rng)),
        ArgType::String => gen_constant_of_type(rng, &TypeSeed::String, 2),
        ArgType::Bool => ConstantSeed::Boolean(rng.gen()),
        ArgType::Unit => ConstantSeed::Unit,
        ArgType::Data => gen_constant_of_type(rng, &TypeSeed::Data, 3),
        ArgType::ListData => {
            let ty = TypeSeed::List(Box::new(TypeSeed::Data));
            gen_constant_of_type(rng, &ty, 3)
        }
        ArgType::PairDataData => {
            let ty = TypeSeed::Pair(Box::new(TypeSeed::Data), Box::new(TypeSeed::Data));
            gen_constant_of_type(rng, &ty, 3)
        }
        ArgType::Any => gen_constant(rng),
    })
}

fn get_arg_types(builtin_id: u8) -> Vec<ArgType> {
    match builtin_id {
        // Integer binary ops
        0..=6 => vec![ArgType::Integer, ArgType::Integer],
        // Integer comparison
        7..=9 => vec![ArgType::Integer, ArgType::Integer],
        // ByteString ops
        10 => vec![ArgType::ByteString, ArgType::ByteString],
        11 => vec![ArgType::Integer, ArgType::ByteString],
        12 => vec![ArgType::Integer, ArgType::Integer, ArgType::ByteString],
        13 => vec![ArgType::ByteString],
        14 => vec![ArgType::ByteString, ArgType::Integer],
        15..=17 => vec![ArgType::ByteString, ArgType::ByteString],
        // Hashes
        18..=20 | 71 | 72 | 86 => vec![ArgType::ByteString],
        // Signature verification
        21 | 52 | 53 => {
            vec![
                ArgType::ByteString,
                ArgType::ByteString,
                ArgType::ByteString,
            ]
        }
        // String ops
        22 => vec![ArgType::String, ArgType::String],
        23 => vec![ArgType::String, ArgType::String],
        24 => vec![ArgType::String],
        25 => vec![ArgType::ByteString],
        // IfThenElse: bool, a, a
        26 => vec![ArgType::Bool, ArgType::Any, ArgType::Any],
        // ChooseUnit: unit, a
        27 => vec![ArgType::Unit, ArgType::Any],
        // Trace: string, a
        28 => vec![ArgType::String, ArgType::Any],
        // FstPair, SndPair: pair
        29 | 30 => vec![ArgType::PairDataData],
        // ChooseList: list, a, a
        31 => vec![ArgType::ListData, ArgType::Any, ArgType::Any],
        // MkCons: a, list a
        32 => vec![ArgType::Data, ArgType::ListData],
        // HeadList, TailList, NullList: list
        33 | 34 | 35 => vec![ArgType::ListData],
        // ChooseData: data, then 5 branches
        36 => vec![
            ArgType::Data,
            ArgType::Any,
            ArgType::Any,
            ArgType::Any,
            ArgType::Any,
            ArgType::Any,
        ],
        // ConstrData: integer, list data
        37 => vec![ArgType::Integer, ArgType::ListData],
        // MapData, ListData: list
        38 => vec![ArgType::Any],
        39 => vec![ArgType::Any],
        // IData, BData
        40 => vec![ArgType::Integer],
        41 => vec![ArgType::ByteString],
        // UnConstr/UnMap/UnList/UnI/UnB: data
        42..=46 => vec![ArgType::Data],
        // EqualsData
        47 => vec![ArgType::Data, ArgType::Data],
        // MkPairData
        48 => vec![ArgType::Data, ArgType::Data],
        // MkNilData, MkNilPairData
        49 | 50 => vec![ArgType::Unit],
        // SerialiseData
        51 => vec![ArgType::Data],
        // IntegerToByteString
        73 => vec![ArgType::Bool, ArgType::Integer, ArgType::Integer],
        // ByteStringToInteger
        74 => vec![ArgType::Bool, ArgType::ByteString],
        // Bitwise binary ops
        75..=77 => vec![ArgType::Bool, ArgType::ByteString, ArgType::ByteString],
        // ComplementByteString
        78 => vec![ArgType::ByteString],
        // ReadBit
        79 => vec![ArgType::ByteString, ArgType::Integer],
        // WriteBits
        80 => vec![ArgType::ByteString, ArgType::ListData, ArgType::ListData],
        // ReplicateByte
        81 => vec![ArgType::Integer, ArgType::Integer],
        // ShiftByteString, RotateByteString
        82 | 83 => vec![ArgType::ByteString, ArgType::Integer],
        // CountSetBits, FindFirstSetBit
        84 | 85 => vec![ArgType::ByteString],
        // ExpModInteger
        87 => vec![ArgType::Integer, ArgType::Integer, ArgType::Integer],
        // DropList
        88 => vec![ArgType::Integer, ArgType::ListData],
        // LengthOfArray, ListToArray
        89 | 90 => vec![ArgType::Any],
        // IndexArray
        91 => vec![ArgType::Any, ArgType::Integer],
        _ => vec![],
    }
}
