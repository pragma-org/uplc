use crate::machine::PlutusVersion;

/// All built-in functions available in the UPLC language.
///
/// The discriminant values match the Flat encoding used in on-chain scripts.
/// Not every function is available in every Plutus version; the runtime
/// checks availability before dispatch.
#[non_exhaustive]
#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DefaultFunction {
    // --- Integer ---
    /// Adds two integers.
    AddInteger = 0,
    /// Subtracts the second integer from the first.
    SubtractInteger = 1,
    /// Multiplies two integers.
    MultiplyInteger = 2,
    /// Truncated division (rounds towards negative infinity).
    DivideInteger = 3,
    /// Truncated quotient (rounds towards zero).
    QuotientInteger = 4,
    /// Remainder after [`QuotientInteger`](Self::QuotientInteger).
    RemainderInteger = 5,
    /// Modulo after [`DivideInteger`](Self::DivideInteger).
    ModInteger = 6,
    /// Tests two integers for equality.
    EqualsInteger = 7,
    /// Returns `true` if the first integer is strictly less than the second.
    LessThanInteger = 8,
    /// Returns `true` if the first integer is less than or equal to the second.
    LessThanEqualsInteger = 9,

    // --- ByteString ---
    /// Concatenates two byte strings.
    AppendByteString = 10,
    /// Prepends a byte (given as an integer 0–255) to a byte string.
    ConsByteString = 11,
    /// Extracts a sub-byte-string by offset and length.
    SliceByteString = 12,
    /// Returns the length of a byte string.
    LengthOfByteString = 13,
    /// Returns the byte at a given index.
    IndexByteString = 14,
    /// Tests two byte strings for equality.
    EqualsByteString = 15,
    /// Lexicographic less-than on byte strings.
    LessThanByteString = 16,
    /// Lexicographic less-than-or-equal on byte strings.
    LessThanEqualsByteString = 17,

    // --- Cryptography ---
    /// SHA-256 hash of a byte string.
    Sha2_256 = 18,
    /// SHA3-256 hash of a byte string.
    Sha3_256 = 19,
    /// Blake2b-256 hash of a byte string.
    Blake2b_256 = 20,
    /// Keccak-256 hash of a byte string (Plutus V3, protocol version ≥ 9).
    Keccak_256 = 71,
    /// Blake2b-224 hash of a byte string (Plutus V3, protocol version ≥ 9).
    Blake2b_224 = 72,
    /// Verifies an Ed25519 signature given `(public_key, message, signature)`.
    VerifyEd25519Signature = 21,
    /// Verifies an ECDSA secp256k1 signature given `(public_key, message, signature)`.
    VerifyEcdsaSecp256k1Signature = 52,
    /// Verifies a Schnorr secp256k1 signature given `(public_key, message, signature)`.
    VerifySchnorrSecp256k1Signature = 53,

    // --- String ---
    /// Concatenates two UTF-8 strings.
    AppendString = 22,
    /// Tests two strings for equality.
    EqualsString = 23,
    /// Encodes a string to a byte string (UTF-8).
    EncodeUtf8 = 24,
    /// Decodes a byte string to a string (UTF-8); errors on invalid bytes.
    DecodeUtf8 = 25,

    // --- Control ---
    /// Polymorphic conditional; forces the chosen branch.
    IfThenElse = 26,
    /// Evaluates a unit value and returns the provided second argument.
    ChooseUnit = 27,
    /// Logs a trace message and returns the second argument unchanged.
    Trace = 28,

    // --- Pairs ---
    /// Returns the first element of a pair.
    FstPair = 29,
    /// Returns the second element of a pair.
    SndPair = 30,

    // --- Lists ---
    /// Pattern-matches a list, selecting the nil or cons branch.
    ChooseList = 31,
    /// Prepends an element to a typed list.
    MkCons = 32,
    /// Returns the first element of a non-empty list.
    HeadList = 33,
    /// Returns the list without its first element.
    TailList = 34,
    /// Returns `true` if the list is empty.
    NullList = 35,

    // --- Data ---
    /// Pattern-matches a `Data` value across all five constructors.
    ChooseData = 36,
    /// Constructs a `Data` constr value from a tag and a list of fields.
    ConstrData = 37,
    /// Lifts a map of `Data` values into a `Data` map.
    MapData = 38,
    /// Lifts a list of `Data` values into `Data`.
    ListData = 39,
    /// Lifts an integer into `Data`.
    IData = 40,
    /// Lifts a byte string into `Data`.
    BData = 41,
    /// Deconstructs a `Data` constr into `(tag, fields)`.
    UnConstrData = 42,
    /// Extracts the map from a `Data` map value.
    UnMapData = 43,
    /// Extracts the list from a `Data` list value.
    UnListData = 44,
    /// Extracts the integer from a `Data` integer value.
    UnIData = 45,
    /// Extracts the byte string from a `Data` byte-string value.
    UnBData = 46,
    /// Tests two `Data` values for structural equality.
    EqualsData = 47,
    /// CBOR-serialises a `Data` value to a byte string.
    SerialiseData = 51,

    // --- Data constructors ---
    /// Constructs a `Data` pair.
    MkPairData = 48,
    /// Constructs an empty `Data` list.
    MkNilData = 49,
    /// Constructs an empty `Data` map (list of pairs).
    MkNilPairData = 50,

    // --- BLS12-381 (Plutus V3) ---
    /// Point addition in BLS12-381 G1.
    Bls12_381_G1_Add = 54,
    /// Point negation in BLS12-381 G1.
    Bls12_381_G1_Neg = 55,
    /// Scalar multiplication in BLS12-381 G1.
    Bls12_381_G1_ScalarMul = 56,
    /// Equality test for BLS12-381 G1 points.
    Bls12_381_G1_Equal = 57,
    /// Compresses a BLS12-381 G1 point to 48 bytes.
    Bls12_381_G1_Compress = 58,
    /// Decompresses 48 bytes into a BLS12-381 G1 point.
    Bls12_381_G1_Uncompress = 59,
    /// Hashes a byte string to a BLS12-381 G1 point using a domain-separation tag.
    Bls12_381_G1_HashToGroup = 60,
    /// Point addition in BLS12-381 G2.
    Bls12_381_G2_Add = 61,
    /// Point negation in BLS12-381 G2.
    Bls12_381_G2_Neg = 62,
    /// Scalar multiplication in BLS12-381 G2.
    Bls12_381_G2_ScalarMul = 63,
    /// Equality test for BLS12-381 G2 points.
    Bls12_381_G2_Equal = 64,
    /// Compresses a BLS12-381 G2 point to 96 bytes.
    Bls12_381_G2_Compress = 65,
    /// Decompresses 96 bytes into a BLS12-381 G2 point.
    Bls12_381_G2_Uncompress = 66,
    /// Hashes a byte string to a BLS12-381 G2 point using a domain-separation tag.
    Bls12_381_G2_HashToGroup = 67,
    /// Computes the BLS12-381 Miller loop (G1 × G2 → GT).
    Bls12_381_MillerLoop = 68,
    /// Multiplies two BLS12-381 GT (Miller-loop result) elements.
    Bls12_381_MulMlResult = 69,
    /// Checks equality of two BLS12-381 pairings (final-exponentiation verify).
    Bls12_381_FinalVerify = 70,

    // --- Bitwise (Plutus V3) ---
    /// Converts an integer to a byte string with given endianness and size.
    IntegerToByteString = 73,
    /// Converts a byte string to an integer with given endianness.
    ByteStringToInteger = 74,
    /// Bitwise AND of two byte strings, with a padding-semantics flag.
    AndByteString = 75,
    /// Bitwise OR of two byte strings, with a padding-semantics flag.
    OrByteString = 76,
    /// Bitwise XOR of two byte strings, with a padding-semantics flag.
    XorByteString = 77,
    /// Bitwise complement of a byte string.
    ComplementByteString = 78,
    /// Reads a single bit at the given index.
    ReadBit = 79,
    /// Writes a list of `(index, bit)` pairs into a byte string.
    WriteBits = 80,
    /// Creates a byte string of given length filled with a single byte.
    ReplicateByte = 81,
    /// Logical shift of a byte string by a signed number of bits.
    ShiftByteString = 82,
    /// Rotation of a byte string by a signed number of bits.
    RotateByteString = 83,
    /// Counts the number of set bits (popcount) in a byte string.
    CountSetBits = 84,
    /// Returns the index of the lowest set bit, or -1 if none.
    FindFirstSetBit = 85,
    /// RIPEMD-160 hash of a byte string.
    Ripemd_160 = 86,

    // --- van Rossem (protocol version ≥ 11) ---
    /// Modular exponentiation: `base ^ exp mod modulus`.
    ExpModInteger = 87,
    /// Drops the first `n` elements of a list.
    DropList = 88,
    /// Returns the length of an array.
    LengthOfArray = 89,
    /// Converts a list to an array.
    ListToArray = 90,
    /// Returns the element at a given index in an array.
    IndexArray = 91,
}

impl DefaultFunction {
    /// Number of [`Force`](crate::term::Term::Force) applications required before
    /// this built-in accepts value arguments (0 for monomorphic, 1–2 for polymorphic).
    pub fn force_count(&self) -> usize {
        match self {
            DefaultFunction::AddInteger => 0,
            DefaultFunction::SubtractInteger => 0,
            DefaultFunction::MultiplyInteger => 0,
            DefaultFunction::DivideInteger => 0,
            DefaultFunction::QuotientInteger => 0,
            DefaultFunction::RemainderInteger => 0,
            DefaultFunction::ModInteger => 0,
            DefaultFunction::EqualsInteger => 0,
            DefaultFunction::LessThanInteger => 0,
            DefaultFunction::LessThanEqualsInteger => 0,
            DefaultFunction::AppendByteString => 0,
            DefaultFunction::ConsByteString => 0,
            DefaultFunction::SliceByteString => 0,
            DefaultFunction::LengthOfByteString => 0,
            DefaultFunction::IndexByteString => 0,
            DefaultFunction::EqualsByteString => 0,
            DefaultFunction::LessThanByteString => 0,
            DefaultFunction::LessThanEqualsByteString => 0,
            DefaultFunction::Sha2_256 => 0,
            DefaultFunction::Sha3_256 => 0,
            DefaultFunction::Blake2b_224 => 0,
            DefaultFunction::Blake2b_256 => 0,
            DefaultFunction::Keccak_256 => 0,
            DefaultFunction::VerifyEd25519Signature => 0,
            DefaultFunction::VerifyEcdsaSecp256k1Signature => 0,
            DefaultFunction::VerifySchnorrSecp256k1Signature => 0,
            DefaultFunction::AppendString => 0,
            DefaultFunction::EqualsString => 0,
            DefaultFunction::EncodeUtf8 => 0,
            DefaultFunction::DecodeUtf8 => 0,
            DefaultFunction::IfThenElse => 1,
            DefaultFunction::ChooseUnit => 1,
            DefaultFunction::Trace => 1,
            DefaultFunction::FstPair => 2,
            DefaultFunction::SndPair => 2,
            DefaultFunction::ChooseList => 2,
            DefaultFunction::MkCons => 1,
            DefaultFunction::HeadList => 1,
            DefaultFunction::TailList => 1,
            DefaultFunction::NullList => 1,
            DefaultFunction::ChooseData => 1,
            DefaultFunction::ConstrData => 0,
            DefaultFunction::MapData => 0,
            DefaultFunction::ListData => 0,
            DefaultFunction::IData => 0,
            DefaultFunction::BData => 0,
            DefaultFunction::UnConstrData => 0,
            DefaultFunction::UnMapData => 0,
            DefaultFunction::UnListData => 0,
            DefaultFunction::UnIData => 0,
            DefaultFunction::UnBData => 0,
            DefaultFunction::EqualsData => 0,
            DefaultFunction::SerialiseData => 0,
            DefaultFunction::MkPairData => 0,
            DefaultFunction::MkNilData => 0,
            DefaultFunction::MkNilPairData => 0,
            DefaultFunction::Bls12_381_G1_Add => 0,
            DefaultFunction::Bls12_381_G1_Neg => 0,
            DefaultFunction::Bls12_381_G1_ScalarMul => 0,
            DefaultFunction::Bls12_381_G1_Equal => 0,
            DefaultFunction::Bls12_381_G1_Compress => 0,
            DefaultFunction::Bls12_381_G1_Uncompress => 0,
            DefaultFunction::Bls12_381_G1_HashToGroup => 0,
            DefaultFunction::Bls12_381_G2_Add => 0,
            DefaultFunction::Bls12_381_G2_Neg => 0,
            DefaultFunction::Bls12_381_G2_ScalarMul => 0,
            DefaultFunction::Bls12_381_G2_Equal => 0,
            DefaultFunction::Bls12_381_G2_Compress => 0,
            DefaultFunction::Bls12_381_G2_Uncompress => 0,
            DefaultFunction::Bls12_381_G2_HashToGroup => 0,
            DefaultFunction::Bls12_381_MillerLoop => 0,
            DefaultFunction::Bls12_381_MulMlResult => 0,
            DefaultFunction::Bls12_381_FinalVerify => 0,
            DefaultFunction::IntegerToByteString => 0,
            DefaultFunction::ByteStringToInteger => 0,
            DefaultFunction::AndByteString => 0,
            DefaultFunction::OrByteString => 0,
            DefaultFunction::XorByteString => 0,
            DefaultFunction::ComplementByteString => 0,
            DefaultFunction::ReadBit => 0,
            DefaultFunction::WriteBits => 0,
            DefaultFunction::ReplicateByte => 0,
            DefaultFunction::ShiftByteString => 0,
            DefaultFunction::RotateByteString => 0,
            DefaultFunction::CountSetBits => 0,
            DefaultFunction::FindFirstSetBit => 0,
            DefaultFunction::Ripemd_160 => 0,
            DefaultFunction::ExpModInteger => 0,
            DefaultFunction::DropList => 1,
            DefaultFunction::LengthOfArray => 1,
            DefaultFunction::ListToArray => 1,
            DefaultFunction::IndexArray => 1,
        }
    }

    /// Number of value arguments this built-in expects after any required `Force`s.
    pub fn arity(&self) -> usize {
        match self {
            DefaultFunction::AddInteger => 2,
            DefaultFunction::SubtractInteger => 2,
            DefaultFunction::MultiplyInteger => 2,
            DefaultFunction::DivideInteger => 2,
            DefaultFunction::QuotientInteger => 2,
            DefaultFunction::RemainderInteger => 2,
            DefaultFunction::ModInteger => 2,
            DefaultFunction::EqualsInteger => 2,
            DefaultFunction::LessThanInteger => 2,
            DefaultFunction::LessThanEqualsInteger => 2,
            DefaultFunction::AppendByteString => 2,
            DefaultFunction::ConsByteString => 2,
            DefaultFunction::SliceByteString => 3,
            DefaultFunction::LengthOfByteString => 1,
            DefaultFunction::IndexByteString => 2,
            DefaultFunction::EqualsByteString => 2,
            DefaultFunction::LessThanByteString => 2,
            DefaultFunction::LessThanEqualsByteString => 2,
            DefaultFunction::Sha2_256 => 1,
            DefaultFunction::Sha3_256 => 1,
            DefaultFunction::Blake2b_224 => 1,
            DefaultFunction::Blake2b_256 => 1,
            DefaultFunction::Keccak_256 => 1,
            DefaultFunction::VerifyEd25519Signature => 3,
            DefaultFunction::VerifyEcdsaSecp256k1Signature => 3,
            DefaultFunction::VerifySchnorrSecp256k1Signature => 3,
            DefaultFunction::AppendString => 2,
            DefaultFunction::EqualsString => 2,
            DefaultFunction::EncodeUtf8 => 1,
            DefaultFunction::DecodeUtf8 => 1,
            DefaultFunction::IfThenElse => 3,
            DefaultFunction::ChooseUnit => 2,
            DefaultFunction::Trace => 2,
            DefaultFunction::FstPair => 1,
            DefaultFunction::SndPair => 1,
            DefaultFunction::ChooseList => 3,
            DefaultFunction::MkCons => 2,
            DefaultFunction::HeadList => 1,
            DefaultFunction::TailList => 1,
            DefaultFunction::NullList => 1,
            DefaultFunction::ChooseData => 6,
            DefaultFunction::ConstrData => 2,
            DefaultFunction::MapData => 1,
            DefaultFunction::ListData => 1,
            DefaultFunction::IData => 1,
            DefaultFunction::BData => 1,
            DefaultFunction::UnConstrData => 1,
            DefaultFunction::UnMapData => 1,
            DefaultFunction::UnListData => 1,
            DefaultFunction::UnIData => 1,
            DefaultFunction::UnBData => 1,
            DefaultFunction::EqualsData => 2,
            DefaultFunction::SerialiseData => 1,
            DefaultFunction::MkPairData => 2,
            DefaultFunction::MkNilData => 1,
            DefaultFunction::MkNilPairData => 1,
            DefaultFunction::Bls12_381_G1_Add => 2,
            DefaultFunction::Bls12_381_G1_Neg => 1,
            DefaultFunction::Bls12_381_G1_ScalarMul => 2,
            DefaultFunction::Bls12_381_G1_Equal => 2,
            DefaultFunction::Bls12_381_G1_Compress => 1,
            DefaultFunction::Bls12_381_G1_Uncompress => 1,
            DefaultFunction::Bls12_381_G1_HashToGroup => 2,
            DefaultFunction::Bls12_381_G2_Add => 2,
            DefaultFunction::Bls12_381_G2_Neg => 1,
            DefaultFunction::Bls12_381_G2_ScalarMul => 2,
            DefaultFunction::Bls12_381_G2_Equal => 2,
            DefaultFunction::Bls12_381_G2_Compress => 1,
            DefaultFunction::Bls12_381_G2_Uncompress => 1,
            DefaultFunction::Bls12_381_G2_HashToGroup => 2,
            DefaultFunction::Bls12_381_MillerLoop => 2,
            DefaultFunction::Bls12_381_MulMlResult => 2,
            DefaultFunction::Bls12_381_FinalVerify => 2,
            DefaultFunction::IntegerToByteString => 3,
            DefaultFunction::ByteStringToInteger => 2,
            DefaultFunction::AndByteString => 3,
            DefaultFunction::OrByteString => 3,
            DefaultFunction::XorByteString => 3,
            DefaultFunction::ComplementByteString => 1,
            DefaultFunction::ReadBit => 2,
            DefaultFunction::WriteBits => 3,
            DefaultFunction::ReplicateByte => 2,
            DefaultFunction::ShiftByteString => 2,
            DefaultFunction::RotateByteString => 2,
            DefaultFunction::CountSetBits => 1,
            DefaultFunction::FindFirstSetBit => 1,
            DefaultFunction::Ripemd_160 => 1,
            DefaultFunction::ExpModInteger => 3,
            DefaultFunction::DropList => 2,
            DefaultFunction::LengthOfArray => 1,
            DefaultFunction::ListToArray => 1,
            DefaultFunction::IndexArray => 2,
        }
    }

    /// Check whether this builtin is available for a given Plutus ledger language
    /// and major protocol version.
    ///
    /// Follows the Haskell's `builtinsIntroducedIn` mapping from plutus-ledger-api:
    /// <https://github.com/IntersectMBO/plutus/blob/master/plutus-ledger-api/src/PlutusLedgerApi/Common/Versions.hs>
    pub fn is_available_in(&self, plutus_version: PlutusVersion, pv: u32) -> bool {
        use DefaultFunction::*;

        // batch1: the original 51 builtins from Alonzo (PV 5)
        let batch1 = matches!(
            self,
            AddInteger
                | SubtractInteger
                | MultiplyInteger
                | DivideInteger
                | QuotientInteger
                | RemainderInteger
                | ModInteger
                | EqualsInteger
                | LessThanInteger
                | LessThanEqualsInteger
                | AppendByteString
                | ConsByteString
                | SliceByteString
                | LengthOfByteString
                | IndexByteString
                | EqualsByteString
                | LessThanByteString
                | LessThanEqualsByteString
                | Sha2_256
                | Sha3_256
                | Blake2b_256
                | VerifyEd25519Signature
                | AppendString
                | EqualsString
                | EncodeUtf8
                | DecodeUtf8
                | IfThenElse
                | ChooseUnit
                | Trace
                | FstPair
                | SndPair
                | ChooseList
                | MkCons
                | HeadList
                | TailList
                | NullList
                | ChooseData
                | ConstrData
                | MapData
                | ListData
                | IData
                | BData
                | UnConstrData
                | UnMapData
                | UnListData
                | UnIData
                | UnBData
                | EqualsData
                | MkPairData
                | MkNilData
                | MkNilPairData
        );

        // batch2: SerialiseData (Vasil, PV 7)
        let batch2 = matches!(self, SerialiseData);

        // batch3: secp256k1 (Valentine, PV 8)
        let batch3 = matches!(
            self,
            VerifyEcdsaSecp256k1Signature | VerifySchnorrSecp256k1Signature
        );

        // batch4a: BLS + Keccak + Blake2b_224 (Chang, PV 9 for V3; van Rossem, PV 11 for V1/V2)
        let batch4a = matches!(
            self,
            Bls12_381_G1_Add
                | Bls12_381_G1_Neg
                | Bls12_381_G1_ScalarMul
                | Bls12_381_G1_Equal
                | Bls12_381_G1_Compress
                | Bls12_381_G1_Uncompress
                | Bls12_381_G1_HashToGroup
                | Bls12_381_G2_Add
                | Bls12_381_G2_Neg
                | Bls12_381_G2_ScalarMul
                | Bls12_381_G2_Equal
                | Bls12_381_G2_Compress
                | Bls12_381_G2_Uncompress
                | Bls12_381_G2_HashToGroup
                | Bls12_381_MillerLoop
                | Bls12_381_MulMlResult
                | Bls12_381_FinalVerify
                | Keccak_256
                | Blake2b_224
        );

        // batch4b: integer-bytestring conversions (Chang, PV 9 for V3; Plomin, PV 10 for V2)
        let batch4b = matches!(self, IntegerToByteString | ByteStringToInteger);

        // batch5: bitwise operations + Ripemd_160 (Plomin, PV 10 for V3)
        let batch5 = matches!(
            self,
            AndByteString
                | OrByteString
                | XorByteString
                | ComplementByteString
                | ReadBit
                | WriteBits
                | ReplicateByte
                | ShiftByteString
                | RotateByteString
                | CountSetBits
                | FindFirstSetBit
                | Ripemd_160
        );

        // batch6: van Rossem (PV 11) — ExpModInteger, DropList, LengthOfArray, ListToArray, IndexArray
        // Not explicitly matched because PV >= 11 returns true for all builtins.

        match plutus_version {
            PlutusVersion::V1 => {
                if pv >= 11 {
                    true
                } else {
                    batch1
                }
            }
            PlutusVersion::V2 => {
                if pv >= 11 {
                    true
                } else if pv >= 10 {
                    batch1 || batch2 || batch3 || batch4b
                } else if pv >= 8 {
                    batch1 || batch2 || batch3
                } else {
                    // PV 7
                    batch1 || batch2
                }
            }
            PlutusVersion::V3 => {
                if pv >= 11 {
                    true
                } else if pv >= 10 {
                    batch1 || batch2 || batch3 || batch4a || batch4b || batch5
                } else {
                    // PV 9
                    batch1 || batch2 || batch3 || batch4a || batch4b
                }
            }
        }
    }
}
