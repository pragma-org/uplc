use std::fmt;

use serde::{Deserialize, Serialize};
use uplc_turbo::{
    arena::Arena,
    binder::{DeBruijn, Eval},
    builtin::DefaultFunction,
    constant::Constant,
    data::PlutusData,
    program::{Program, Version},
    term::Term,
    typ::Type,
};

/// Arena-free, Send + Sync, Clone program representation.
/// The unit of fuzzing: generators produce seeds, workers materialize them.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ProgramSeed {
    pub version: (usize, usize, usize),
    pub term: TermSeed,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TermSeed {
    Var(usize),
    Lambda(Box<TermSeed>),
    Apply(Box<TermSeed>, Box<TermSeed>),
    Delay(Box<TermSeed>),
    Force(Box<TermSeed>),
    Case {
        constr: Box<TermSeed>,
        branches: Vec<TermSeed>,
    },
    Constr {
        tag: usize,
        fields: Vec<TermSeed>,
    },
    Constant(ConstantSeed),
    Builtin(u8),
    Error,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ConstantSeed {
    Integer(i128),
    ByteString(Vec<u8>),
    String(String),
    Boolean(bool),
    Unit,
    Data(DataSeed),
    List(TypeSeed, Vec<ConstantSeed>),
    Pair(TypeSeed, TypeSeed, Box<ConstantSeed>, Box<ConstantSeed>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TypeSeed {
    Bool,
    Integer,
    String,
    ByteString,
    Unit,
    List(Box<TypeSeed>),
    Pair(Box<TypeSeed>, Box<TypeSeed>),
    Data,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DataSeed {
    Constr(u64, Vec<DataSeed>),
    Map(Vec<(DataSeed, DataSeed)>),
    List(Vec<DataSeed>),
    Integer(i128),
    ByteString(Vec<u8>),
}

// Helper: allocate a slice from a Vec into the arena.
fn alloc_slice<T: Copy>(arena: &Arena, items: Vec<T>) -> &[T] {
    arena.alloc_slice(items)
}

impl ProgramSeed {
    pub fn materialize<'a>(&self, arena: &'a Arena) -> &'a Program<'a, DeBruijn> {
        let version = Version::new(arena, self.version.0, self.version.1, self.version.2);
        let term = self.term.materialize(arena);
        Program::new(arena, version, term)
    }

    /// Count total AST nodes.
    pub fn node_count(&self) -> usize {
        self.term.node_count()
    }
}

impl TermSeed {
    pub fn materialize<'a>(&self, arena: &'a Arena) -> &'a Term<'a, DeBruijn> {
        match self {
            TermSeed::Var(idx) => Term::var(arena, DeBruijn::new(arena, *idx)),
            TermSeed::Lambda(body) => {
                let body = body.materialize(arena);
                body.lambda(arena, DeBruijn::zero(arena))
            }
            TermSeed::Apply(fun, arg) => {
                let fun = fun.materialize(arena);
                let arg = arg.materialize(arena);
                fun.apply(arena, arg)
            }
            TermSeed::Delay(body) => {
                let body = body.materialize(arena);
                body.delay(arena)
            }
            TermSeed::Force(body) => {
                let body = body.materialize(arena);
                body.force(arena)
            }
            TermSeed::Case { constr, branches } => {
                let constr = constr.materialize(arena);
                let branch_refs: Vec<&'a Term<'a, DeBruijn>> =
                    branches.iter().map(|b| b.materialize(arena)).collect();
                let branches_slice = alloc_slice(arena, branch_refs);
                Term::case(arena, constr, branches_slice)
            }
            TermSeed::Constr { tag, fields } => {
                let field_refs: Vec<&'a Term<'a, DeBruijn>> =
                    fields.iter().map(|f| f.materialize(arena)).collect();
                let fields_slice = alloc_slice(arena, field_refs);
                Term::constr(arena, *tag, fields_slice)
            }
            TermSeed::Constant(c) => {
                let constant = c.materialize(arena);
                Term::constant(arena, constant)
            }
            TermSeed::Builtin(id) => {
                let fun = arena.alloc(DefaultFunction::from_u8(*id));
                arena.alloc(Term::Builtin(fun))
            }
            TermSeed::Error => arena.alloc(Term::Error),
        }
    }

    pub fn from_term(term: &Term<'_, DeBruijn>) -> Self {
        match term {
            Term::Var(v) => TermSeed::Var(v.index()),
            Term::Lambda { body, .. } => TermSeed::Lambda(Box::new(TermSeed::from_term(body))),
            Term::Apply { function, argument } => TermSeed::Apply(
                Box::new(TermSeed::from_term(function)),
                Box::new(TermSeed::from_term(argument)),
            ),
            Term::Delay(body) => TermSeed::Delay(Box::new(TermSeed::from_term(body))),
            Term::Force(body) => TermSeed::Force(Box::new(TermSeed::from_term(body))),
            Term::Case { constr, branches } => TermSeed::Case {
                constr: Box::new(TermSeed::from_term(constr)),
                branches: branches.iter().map(|b| TermSeed::from_term(b)).collect(),
            },
            Term::Constr { tag, fields } => TermSeed::Constr {
                tag: *tag,
                fields: fields.iter().map(|f| TermSeed::from_term(f)).collect(),
            },
            Term::Constant(c) => TermSeed::Constant(ConstantSeed::from_constant(c)),
            Term::Builtin(f) => TermSeed::Builtin(**f as u8),
            Term::Error => TermSeed::Error,
        }
    }

    pub fn node_count(&self) -> usize {
        match self {
            TermSeed::Var(_) | TermSeed::Constant(_) | TermSeed::Builtin(_) | TermSeed::Error => 1,
            TermSeed::Lambda(body) | TermSeed::Delay(body) | TermSeed::Force(body) => {
                1 + body.node_count()
            }
            TermSeed::Apply(f, a) => 1 + f.node_count() + a.node_count(),
            TermSeed::Case { constr, branches } => {
                1 + constr.node_count() + branches.iter().map(|b| b.node_count()).sum::<usize>()
            }
            TermSeed::Constr { fields, .. } => {
                1 + fields.iter().map(|f| f.node_count()).sum::<usize>()
            }
        }
    }
}

impl ConstantSeed {
    fn materialize<'a>(&self, arena: &'a Arena) -> &'a Constant<'a> {
        match self {
            ConstantSeed::Integer(i) => Constant::integer_from(arena, *i),
            ConstantSeed::ByteString(bs) => {
                let bytes: &'a [u8] = arena.alloc(bs.clone().into_boxed_slice());
                Constant::byte_string(arena, bytes)
            }
            ConstantSeed::String(s) => {
                let s: &'a str = arena.alloc(s.clone().into_boxed_str());
                Constant::string(arena, s)
            }
            ConstantSeed::Boolean(b) => Constant::bool(arena, *b),
            ConstantSeed::Unit => Constant::unit(arena),
            ConstantSeed::Data(d) => {
                let data = d.materialize(arena);
                Constant::data(arena, data)
            }
            ConstantSeed::List(ty, items) => {
                let ty = ty.materialize(arena);
                let item_refs: Vec<&'a Constant<'a>> =
                    items.iter().map(|c| c.materialize(arena)).collect();
                let items_slice = alloc_slice(arena, item_refs);
                Constant::proto_list(arena, ty, items_slice)
            }
            ConstantSeed::Pair(ty1, ty2, fst, snd) => {
                let ty1 = ty1.materialize(arena);
                let ty2 = ty2.materialize(arena);
                let fst = fst.materialize(arena);
                let snd = snd.materialize(arena);
                Constant::proto_pair(arena, ty1, ty2, fst, snd)
            }
        }
    }

    fn from_constant(c: &Constant<'_>) -> Self {
        match c {
            Constant::Integer(i) => {
                let val: i128 = i128::try_from(*i).unwrap_or(0);
                ConstantSeed::Integer(val)
            }
            Constant::ByteString(bs) => ConstantSeed::ByteString(bs.to_vec()),
            Constant::String(s) => ConstantSeed::String(s.to_string()),
            Constant::Boolean(b) => ConstantSeed::Boolean(*b),
            Constant::Unit => ConstantSeed::Unit,
            Constant::Data(d) => ConstantSeed::Data(DataSeed::from_plutus_data(d)),
            Constant::ProtoList(ty, items) => ConstantSeed::List(
                TypeSeed::from_type(ty),
                items
                    .iter()
                    .map(|c| ConstantSeed::from_constant(c))
                    .collect(),
            ),
            Constant::ProtoPair(ty1, ty2, fst, snd) => ConstantSeed::Pair(
                TypeSeed::from_type(ty1),
                TypeSeed::from_type(ty2),
                Box::new(ConstantSeed::from_constant(fst)),
                Box::new(ConstantSeed::from_constant(snd)),
            ),
            // BLS/Value/Array types: represent as unit for seed purposes
            _ => ConstantSeed::Unit,
        }
    }
}

impl TypeSeed {
    fn materialize<'a>(&self, arena: &'a Arena) -> &'a Type<'a> {
        match self {
            TypeSeed::Bool => Type::bool(arena),
            TypeSeed::Integer => Type::integer(arena),
            TypeSeed::String => Type::string(arena),
            TypeSeed::ByteString => Type::byte_string(arena),
            TypeSeed::Unit => Type::unit(arena),
            TypeSeed::List(inner) => Type::list(arena, inner.materialize(arena)),
            TypeSeed::Pair(fst, snd) => {
                Type::pair(arena, fst.materialize(arena), snd.materialize(arena))
            }
            TypeSeed::Data => Type::data(arena),
        }
    }

    fn from_type(t: &Type<'_>) -> Self {
        match t {
            Type::Bool => TypeSeed::Bool,
            Type::Integer => TypeSeed::Integer,
            Type::String => TypeSeed::String,
            Type::ByteString => TypeSeed::ByteString,
            Type::Unit => TypeSeed::Unit,
            Type::List(inner) => TypeSeed::List(Box::new(TypeSeed::from_type(inner))),
            Type::Pair(fst, snd) => TypeSeed::Pair(
                Box::new(TypeSeed::from_type(fst)),
                Box::new(TypeSeed::from_type(snd)),
            ),
            Type::Data => TypeSeed::Data,
            _ => TypeSeed::Unit,
        }
    }
}

impl DataSeed {
    fn materialize<'a>(&self, arena: &'a Arena) -> &'a PlutusData<'a> {
        match self {
            DataSeed::Constr(tag, fields) => {
                let field_refs: Vec<&'a PlutusData<'a>> =
                    fields.iter().map(|d| d.materialize(arena)).collect();
                let fields_slice = alloc_slice(arena, field_refs);
                PlutusData::constr(arena, *tag, fields_slice)
            }
            DataSeed::Map(entries) => {
                let entry_refs: Vec<(&'a PlutusData<'a>, &'a PlutusData<'a>)> = entries
                    .iter()
                    .map(|(k, v)| (k.materialize(arena), v.materialize(arena)))
                    .collect();
                let entries_slice = alloc_slice(arena, entry_refs);
                PlutusData::map(arena, entries_slice)
            }
            DataSeed::List(items) => {
                let item_refs: Vec<&'a PlutusData<'a>> =
                    items.iter().map(|d| d.materialize(arena)).collect();
                let items_slice = alloc_slice(arena, item_refs);
                PlutusData::list(arena, items_slice)
            }
            DataSeed::Integer(i) => PlutusData::integer_from(arena, *i),
            DataSeed::ByteString(bs) => {
                let bytes: &'a [u8] = arena.alloc(bs.clone().into_boxed_slice());
                PlutusData::byte_string(arena, bytes)
            }
        }
    }

    fn from_plutus_data(d: &PlutusData<'_>) -> Self {
        match d {
            PlutusData::Constr { tag, fields } => DataSeed::Constr(
                *tag,
                fields
                    .iter()
                    .map(|f| DataSeed::from_plutus_data(f))
                    .collect(),
            ),
            PlutusData::Map(entries) => DataSeed::Map(
                entries
                    .iter()
                    .map(|(k, v)| (DataSeed::from_plutus_data(k), DataSeed::from_plutus_data(v)))
                    .collect(),
            ),
            PlutusData::List(items) => DataSeed::List(
                items
                    .iter()
                    .map(|i| DataSeed::from_plutus_data(i))
                    .collect(),
            ),
            PlutusData::Integer(i) => {
                let val: i128 = i128::try_from(*i).unwrap_or(0);
                DataSeed::Integer(val)
            }
            PlutusData::ByteString(bs) => DataSeed::ByteString(bs.to_vec()),
        }
    }
}

// --- UPLC text format serialization ---

impl fmt::Display for ProgramSeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(program {}.{}.{} {})",
            self.version.0, self.version.1, self.version.2, self.term
        )
    }
}

impl fmt::Display for TermSeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TermSeed::Var(idx) => write!(f, "i_{idx}"),
            TermSeed::Lambda(body) => write!(f, "(lam i_0 {body})"),
            TermSeed::Apply(fun, arg) => write!(f, "[{fun} {arg}]"),
            TermSeed::Delay(body) => write!(f, "(delay {body})"),
            TermSeed::Force(body) => write!(f, "(force {body})"),
            TermSeed::Case { constr, branches } => {
                write!(f, "(case {constr}")?;
                for b in branches {
                    write!(f, " {b}")?;
                }
                write!(f, ")")
            }
            TermSeed::Constr { tag, fields } => {
                write!(f, "(constr {tag}")?;
                for field in fields {
                    write!(f, " {field}")?;
                }
                write!(f, ")")
            }
            TermSeed::Constant(c) => write!(f, "{c}"),
            TermSeed::Builtin(id) => write!(f, "(builtin {})", builtin_name(*id)),
            TermSeed::Error => write!(f, "(error)"),
        }
    }
}

impl fmt::Display for ConstantSeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConstantSeed::Integer(i) => write!(f, "(con integer {i})"),
            ConstantSeed::ByteString(bs) => {
                write!(f, "(con bytestring #")?;
                for b in bs {
                    write!(f, "{b:02x}")?;
                }
                write!(f, ")")
            }
            ConstantSeed::String(s) => write!(f, "(con string \"{s}\")"),
            ConstantSeed::Boolean(true) => write!(f, "(con bool True)"),
            ConstantSeed::Boolean(false) => write!(f, "(con bool False)"),
            ConstantSeed::Unit => write!(f, "(con unit ())"),
            ConstantSeed::Data(d) => write!(f, "(con data {d})"),
            ConstantSeed::List(ty, items) => {
                write!(f, "(con (list {ty}) [")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write_constant_value(f, item)?;
                }
                write!(f, "])")
            }
            ConstantSeed::Pair(ty1, ty2, fst, snd) => {
                write!(f, "(con (pair {ty1} {ty2}) (")?;
                write_constant_value(f, fst)?;
                write!(f, ", ")?;
                write_constant_value(f, snd)?;
                write!(f, "))")
            }
        }
    }
}

/// Write a constant value (without the `con type` wrapper) for use inside lists/pairs.
fn write_constant_value(f: &mut fmt::Formatter<'_>, c: &ConstantSeed) -> fmt::Result {
    match c {
        ConstantSeed::Integer(i) => write!(f, "{i}"),
        ConstantSeed::ByteString(bs) => {
            write!(f, "#")?;
            for b in bs {
                write!(f, "{b:02x}")?;
            }
            Ok(())
        }
        ConstantSeed::String(s) => write!(f, "\"{s}\""),
        ConstantSeed::Boolean(true) => write!(f, "True"),
        ConstantSeed::Boolean(false) => write!(f, "False"),
        ConstantSeed::Unit => write!(f, "()"),
        ConstantSeed::Data(d) => write!(f, "{d}"),
        ConstantSeed::List(_, items) => {
            write!(f, "[")?;
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write_constant_value(f, item)?;
            }
            write!(f, "]")
        }
        ConstantSeed::Pair(_, _, fst, snd) => {
            write!(f, "(")?;
            write_constant_value(f, fst)?;
            write!(f, ", ")?;
            write_constant_value(f, snd)?;
            write!(f, ")")
        }
    }
}

impl fmt::Display for TypeSeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeSeed::Bool => write!(f, "bool"),
            TypeSeed::Integer => write!(f, "integer"),
            TypeSeed::String => write!(f, "string"),
            TypeSeed::ByteString => write!(f, "bytestring"),
            TypeSeed::Unit => write!(f, "unit"),
            TypeSeed::List(inner) => write!(f, "(list {inner})"),
            TypeSeed::Pair(fst, snd) => write!(f, "(pair {fst} {snd})"),
            TypeSeed::Data => write!(f, "data"),
        }
    }
}

impl fmt::Display for DataSeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataSeed::Constr(tag, fields) => {
                write!(f, "(Constr {tag} [")?;
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{field}")?;
                }
                write!(f, "])")
            }
            DataSeed::Map(entries) => {
                write!(f, "(Map [")?;
                for (i, (k, v)) in entries.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "({k}, {v})")?;
                }
                write!(f, "])")
            }
            DataSeed::List(items) => {
                write!(f, "(List [")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "])")
            }
            DataSeed::Integer(i) => write!(f, "(I {i})"),
            DataSeed::ByteString(bs) => {
                write!(f, "(B #")?;
                for b in bs {
                    write!(f, "{b:02x}")?;
                }
                write!(f, ")")
            }
        }
    }
}

fn builtin_name(id: u8) -> &'static str {
    match id {
        0 => "addInteger",
        1 => "subtractInteger",
        2 => "multiplyInteger",
        3 => "divideInteger",
        4 => "quotientInteger",
        5 => "remainderInteger",
        6 => "modInteger",
        7 => "equalsInteger",
        8 => "lessThanInteger",
        9 => "lessThanEqualsInteger",
        10 => "appendByteString",
        11 => "consByteString",
        12 => "sliceByteString",
        13 => "lengthOfByteString",
        14 => "indexByteString",
        15 => "equalsByteString",
        16 => "lessThanByteString",
        17 => "lessThanEqualsByteString",
        18 => "sha2_256",
        19 => "sha3_256",
        20 => "blake2b_256",
        21 => "verifyEd25519Signature",
        22 => "appendString",
        23 => "equalsString",
        24 => "encodeUtf8",
        25 => "decodeUtf8",
        26 => "ifThenElse",
        27 => "chooseUnit",
        28 => "trace",
        29 => "fstPair",
        30 => "sndPair",
        31 => "chooseList",
        32 => "mkCons",
        33 => "headList",
        34 => "tailList",
        35 => "nullList",
        36 => "chooseData",
        37 => "constrData",
        38 => "mapData",
        39 => "listData",
        40 => "iData",
        41 => "bData",
        42 => "unConstrData",
        43 => "unMapData",
        44 => "unListData",
        45 => "unIData",
        46 => "unBData",
        47 => "equalsData",
        48 => "mkPairData",
        49 => "mkNilData",
        50 => "mkNilPairData",
        51 => "serialiseData",
        52 => "verifyEcdsaSecp256k1Signature",
        53 => "verifySchnorrSecp256k1Signature",
        54 => "bls12_381_G1_add",
        55 => "bls12_381_G1_neg",
        56 => "bls12_381_G1_scalarMul",
        57 => "bls12_381_G1_equal",
        58 => "bls12_381_G1_compress",
        59 => "bls12_381_G1_uncompress",
        60 => "bls12_381_G1_hashToGroup",
        61 => "bls12_381_G2_add",
        62 => "bls12_381_G2_neg",
        63 => "bls12_381_G2_scalarMul",
        64 => "bls12_381_G2_equal",
        65 => "bls12_381_G2_compress",
        66 => "bls12_381_G2_uncompress",
        67 => "bls12_381_G2_hashToGroup",
        68 => "bls12_381_millerLoop",
        69 => "bls12_381_mulMlResult",
        70 => "bls12_381_finalVerify",
        71 => "keccak_256",
        72 => "blake2b_224",
        73 => "integerToByteString",
        74 => "byteStringToInteger",
        75 => "andByteString",
        76 => "orByteString",
        77 => "xorByteString",
        78 => "complementByteString",
        79 => "readBit",
        80 => "writeBits",
        81 => "replicateByte",
        82 => "shiftByteString",
        83 => "rotateByteString",
        84 => "countSetBits",
        85 => "findFirstSetBit",
        86 => "ripemd_160",
        87 => "expModInteger",
        88 => "dropList",
        89 => "lengthOfArray",
        90 => "listToArray",
        91 => "indexArray",
        92 => "bls12_381_G1_multiScalarMul",
        93 => "bls12_381_G2_multiScalarMul",
        94 => "insertCoin",
        95 => "lookupCoin",
        96 => "unionValue",
        97 => "valueContains",
        98 => "valueData",
        99 => "unValueData",
        100 => "scaleValue",
        _ => "addInteger",
    }
}

/// Get force_count for a builtin by ID (mirrors DefaultFunction::force_count).
pub fn builtin_force_count(id: u8) -> usize {
    match id {
        26 | 27 | 28 | 32 | 33 | 34 | 35 | 36 | 88 | 89 | 90 | 91 => 1,
        29..=31 => 2,
        _ => 0,
    }
}

/// Get arity for a builtin by ID (mirrors DefaultFunction::arity).
pub fn builtin_arity(id: u8) -> usize {
    match id {
        13 | 18 | 19 | 20 | 24 | 25 | 29 | 30 | 33 | 34 | 35 | 38 | 39 | 40 | 41 | 42 | 43 | 44
        | 45 | 46 | 49 | 50 | 51 | 55 | 58 | 59 | 62 | 65 | 66 | 71 | 72 | 78 | 84 | 85 | 86
        | 89 | 90 | 98 | 99 => 1,
        0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 14 | 15 | 16 | 17 | 22 | 23 | 27 | 28
        | 32 | 37 | 47 | 48 | 54 | 56 | 57 | 60 | 61 | 63 | 64 | 67 | 68 | 69 | 70 | 74 | 75
        | 76 | 77 | 79 | 82 | 83 | 88 | 91 | 92 | 93 | 94 | 95 | 96 | 97 | 100 => 2,
        12 | 21 | 26 | 31 | 52 | 53 | 73 | 80 => 3,
        36 => 6,
        _ => 1,
    }
}
