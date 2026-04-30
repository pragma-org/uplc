use bumpalo::collections::{String as BumpString, Vec as BumpVec};

use crate::{
    arena::Arena, builtin::DefaultFunction, constant::Integer, flat::zigzag::ZigZag,
    machine::PlutusVersion, program::Version,
};

use super::FlatDecodeError;

/// FLAT format decoder with a 64-bit accumulator for fast bit-level reads.
///
/// Position is tracked as `bit_pos` (total bits consumed from start of buffer).
/// The accumulator holds pre-fetched bits from the buffer, avoiding per-bit
/// bounds checks and byte-crossing logic in the hot path.
pub struct Decoder<'b> {
    pub buffer: &'b [u8],
    /// Total number of bits consumed from the buffer so far.
    /// This is the single source of truth for position.
    bit_pos: usize,
    /// 64-bit accumulator: valid bits are left-aligned (MSB side).
    accum: u64,
    /// Number of valid bits remaining in accum.
    accum_bits: u32,
}

pub struct Ctx<'a> {
    pub arena: &'a Arena,
    pub version: Option<&'a Version<'a>>,
    pub plutus_version: Option<PlutusVersion>,
    pub protocol_version: Option<u32>,
}

impl<'a> Ctx<'a> {
    /// Returns true if gating is active and the program version is pre-1.1.0.
    pub fn program_is_pre_1_1_0(&self) -> bool {
        match self.version {
            Some(v) => v.is_less_than_1_1_0(),
            None => false,
        }
    }

    /// Returns true if the given builtin is NOT available under the current
    /// plutus_version / protocol_version combination.
    pub fn is_builtin_gated(&self, func: &DefaultFunction) -> bool {
        match (self.plutus_version, self.protocol_version) {
            (Some(pv), Some(proto)) => !func.is_available_in(pv, proto),
            _ => false,
        }
    }
}

impl<'b> Decoder<'b> {
    pub fn new(bytes: &'b [u8]) -> Decoder<'b> {
        let mut d = Decoder {
            buffer: bytes,
            bit_pos: 0,
            accum: 0,
            accum_bits: 0,
        };
        d.refill();
        d
    }

    /// Refill the accumulator from the buffer. Loads bytes until we have
    /// at least 56 bits (or exhaust the buffer).
    #[inline(always)]
    fn refill(&mut self) {
        let mut next_byte = (self.bit_pos + self.accum_bits as usize).div_ceil(8);
        let buf_len = self.buffer.len();

        while self.accum_bits <= 56 && next_byte < buf_len {
            self.accum |= (self.buffer[next_byte] as u64) << (56 - self.accum_bits);
            self.accum_bits += 8;
            next_byte += 1;
        }
    }

    /// Decode a word of any size (variable-length unsigned integer).
    pub fn word(&mut self) -> Result<usize, FlatDecodeError> {
        let mut leading_bit = 1;
        let mut final_word: usize = 0;
        let mut shl: usize = 0;

        while leading_bit > 0 {
            let word8 = self.bits8(8)?;
            let word7 = word8 & 127;
            final_word |= (word7 as usize) << shl;
            shl += 7;
            leading_bit = word8 & 128;
        }

        Ok(final_word)
    }

    /// Decode a list of items with a decoder function.
    pub fn list_with<'a, T, F>(
        &mut self,
        ctx: &mut Ctx<'a>,
        decoder_func: F,
    ) -> Result<BumpVec<'a, T>, FlatDecodeError>
    where
        F: Copy + FnOnce(&mut Ctx<'a>, &mut Decoder) -> Result<T, FlatDecodeError>,
    {
        let mut vec_array = BumpVec::new_in(ctx.arena.as_bump());

        while self.bit()? {
            vec_array.push(decoder_func(ctx, self)?)
        }

        Ok(vec_array)
    }

    /// Decode up to 8 bits from the accumulator.
    #[inline(always)]
    pub fn bits8(&mut self, num_bits: usize) -> Result<u8, FlatDecodeError> {
        debug_assert!(num_bits <= 8);

        if (self.accum_bits as usize) < num_bits {
            self.refill();
            if (self.accum_bits as usize) < num_bits {
                return Err(FlatDecodeError::NotEnoughBits(num_bits));
            }
        }

        let x = (self.accum >> (64 - num_bits)) as u8;
        self.accum <<= num_bits;
        self.accum_bits -= num_bits as u32;
        self.bit_pos += num_bits;

        Ok(x)
    }

    /// Decode a single bit.
    #[inline(always)]
    pub fn bit(&mut self) -> Result<bool, FlatDecodeError> {
        if self.accum_bits == 0 {
            self.refill();
            if self.accum_bits == 0 {
                return Err(FlatDecodeError::EndOfBuffer);
            }
        }

        let b = (self.accum >> 63) != 0;
        self.accum <<= 1;
        self.accum_bits -= 1;
        self.bit_pos += 1;

        Ok(b)
    }

    /// Decodes a filler (skip zero bits until we hit a 1, aligning to byte boundary).
    pub fn filler(&mut self) -> Result<(), FlatDecodeError> {
        while !self.bit()? {}
        Ok(())
    }

    /// Decode an integer (zigzag-encoded big_word).
    pub fn integer(&mut self) -> Result<Integer, FlatDecodeError> {
        // Fast path: try to decode as u64 first (covers the vast majority of integers).
        // Only fall back to BigInt for integers > 63 bits (9+ encoded bytes).
        let mut val: u64 = 0;
        let mut shift: u32 = 0;

        loop {
            let word8 = self.bits8(8)?;
            val |= ((word8 & 0x7F) as u64) << shift;
            shift += 7;

            if word8 & 0x80 == 0 {
                // Finished — convert via zigzag using u64 arithmetic
                // ZigZag: if LSB=0, val>>1; if LSB=1, -(val>>1)-1
                let unsigned = val;
                let signed = if unsigned & 1 == 0 {
                    (unsigned >> 1) as i64
                } else {
                    -((unsigned >> 1) as i64) - 1
                };
                return Ok(Integer::from(signed));
            }

            if shift >= 63 {
                // Overflow u64 — fall back to BigInt for remaining bytes
                let mut big = Integer::from(val);
                loop {
                    let word8 = self.bits8(8)?;
                    let part = Integer::from(word8 & 0x7F);
                    big |= part << shift;
                    shift += 7;
                    if word8 & 0x80 == 0 {
                        return Ok(ZigZag::unzigzag(&big));
                    }
                }
            }
        }
    }

    /// Decode a variable-length big integer (unsigned).
    pub fn big_word(&mut self) -> Result<Integer, FlatDecodeError> {
        // Fast path: try u64 first
        let mut val: u64 = 0;
        let mut shift: u32 = 0;

        loop {
            let word8 = self.bits8(8)?;
            val |= ((word8 & 0x7F) as u64) << shift;
            shift += 7;

            if word8 & 0x80 == 0 {
                return Ok(Integer::from(val));
            }

            if shift >= 63 {
                // Overflow — fall back to BigInt
                let mut big = Integer::from(val);
                loop {
                    let word8 = self.bits8(8)?;
                    let part = Integer::from(word8 & 0x7F);
                    big |= part << shift;
                    shift += 7;
                    if word8 & 0x80 == 0 {
                        return Ok(big);
                    }
                }
            }
        }
    }

    /// Decode a byte-aligned byte array. Calls filler() first to align,
    /// then reads chunked byte data directly from the buffer.
    pub fn bytes<'a>(&mut self, arena: &'a Arena) -> Result<BumpVec<'a, u8>, FlatDecodeError> {
        self.filler()?;
        self.byte_array(arena)
    }

    /// Read a chunked byte array from the buffer. Requires byte alignment.
    fn byte_array<'a>(&mut self, arena: &'a Arena) -> Result<BumpVec<'a, u8>, FlatDecodeError> {
        // After filler, we should be byte-aligned
        if !self.bit_pos.is_multiple_of(8) {
            return Err(FlatDecodeError::BufferNotByteAligned);
        }

        // Drain the accumulator — switch to direct buffer reading
        let pos = self.bit_pos / 8;
        self.accum = 0;
        self.accum_bits = 0;

        // Read directly from buffer
        let mut cur = pos;
        self.ensure_bytes_at(cur, 1)?;
        let mut blk_len = self.buffer[cur] as usize;
        cur += 1;

        let mut blk_array = BumpVec::with_capacity_in(blk_len, arena.as_bump());

        while blk_len != 0 {
            self.ensure_bytes_at(cur, blk_len + 1)?;

            blk_array.extend(&self.buffer[cur..cur + blk_len]);
            cur += blk_len;

            blk_len = self.buffer[cur] as usize;
            cur += 1;
        }

        // Update position and refill accumulator
        self.bit_pos = cur * 8;
        self.refill();

        Ok(blk_array)
    }

    /// Decode a UTF-8 string (byte array interpreted as UTF-8).
    pub fn utf8<'a>(&mut self, arena: &'a Arena) -> Result<&'a str, FlatDecodeError> {
        let b = self.bytes(arena)?;

        let s =
            BumpString::from_utf8(b).map_err(|e| FlatDecodeError::DecodeUtf8(e.utf8_error()))?;
        let s = arena.alloc(s);

        Ok(s)
    }

    /// Check that at least `required_bytes` are available starting at `pos`.
    fn ensure_bytes_at(&self, pos: usize, required_bytes: usize) -> Result<(), FlatDecodeError> {
        if pos + required_bytes > self.buffer.len() {
            Err(FlatDecodeError::NotEnoughBytes(required_bytes))
        } else {
            Ok(())
        }
    }

    // Legacy compatibility accessors (used by some external code)

    pub fn used_bits(&self) -> usize {
        self.bit_pos % 8
    }

    pub fn pos(&self) -> usize {
        self.bit_pos / 8
    }
}
