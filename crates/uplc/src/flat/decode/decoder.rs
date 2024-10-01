use bumpalo::{collections::Vec as BumpVec, Bump};

use crate::flat::zigzag::ZigZag;

use super::FlatDecodeError;

pub struct Decoder<'b> {
    pub buffer: &'b [u8],
    pub used_bits: usize,
    pub pos: usize,
}

impl<'b> Decoder<'b> {
    pub fn new(bytes: &'b [u8]) -> Decoder<'b> {
        Decoder {
            buffer: bytes,
            pos: 0,
            used_bits: 0,
        }
    }

    /// Decode a word of any size.
    /// This is byte alignment agnostic.
    /// First we decode the next 8 bits of the buffer.
    /// We take the 7 least significant bits as the 7 least significant bits of
    /// the current unsigned integer. If the most significant bit of the 8
    /// bits is 1 then we take the next 8 and repeat the process above,
    /// filling in the next 7 least significant bits of the unsigned integer and
    /// so on. If the most significant bit was instead 0 we stop decoding
    /// any more bits.
    pub fn word(&mut self) -> Result<usize, FlatDecodeError> {
        let mut leading_bit = 1;
        let mut final_word: usize = 0;
        let mut shl: usize = 0;

        // continue looping if lead bit is 1 which is 128 as a u8 otherwise exit
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
    /// This is byte alignment agnostic.
    /// Decode a bit from the buffer.
    /// If 0 then stop.
    /// Otherwise we decode an item in the list with the decoder function passed
    /// in. Then decode the next bit in the buffer and repeat above.
    /// Returns a list of items decoded with the decoder function.
    pub fn list_with<'a, T, F>(
        &mut self,
        arena: &'a Bump,
        decoder_func: F,
    ) -> Result<BumpVec<'a, T>, FlatDecodeError>
    where
        F: Copy + FnOnce(&'a Bump, &mut Decoder) -> Result<T, FlatDecodeError>,
    {
        let mut vec_array = BumpVec::new_in(arena);

        while self.bit()? {
            vec_array.push(decoder_func(arena, self)?)
        }

        Ok(vec_array)
    }

    /// Decode up to 8 bits.
    /// This is byte alignment agnostic.
    /// If num_bits is greater than the 8 we throw an IncorrectNumBits error.
    /// First we decode the next num_bits of bits in the buffer.
    /// If there are less unused bits in the current byte in the buffer than
    /// num_bits, then we decode the remaining bits from the most
    /// significant bits in the next byte in the buffer. Otherwise we decode
    /// the unused bits from the current byte. Returns the decoded value up
    /// to a byte in size.
    pub fn bits8(&mut self, num_bits: usize) -> Result<u8, FlatDecodeError> {
        if num_bits > 8 {
            return Err(FlatDecodeError::IncorrectNumBits);
        }

        self.ensure_bits(num_bits)?;

        let unused_bits = 8 - self.used_bits;
        let leading_zeroes = 8 - num_bits;
        let r = (self.buffer[self.pos] << self.used_bits) >> leading_zeroes;

        let x = if num_bits > unused_bits {
            r | (self.buffer[self.pos + 1] >> (unused_bits + leading_zeroes))
        } else {
            r
        };

        self.drop_bits(num_bits);

        Ok(x)
    }

    /// Ensures the buffer has the required bits passed in by required_bits.
    /// Throws a NotEnoughBits error if there are less bits remaining in the
    /// buffer than required_bits.
    fn ensure_bits(&mut self, required_bits: usize) -> Result<(), FlatDecodeError> {
        if required_bits > (self.buffer.len() - self.pos) * 8 - self.used_bits {
            Err(FlatDecodeError::NotEnoughBits(required_bits))
        } else {
            Ok(())
        }
    }

    /// Increment buffer by num_bits.
    /// If num_bits + used bits is greater than 8,
    /// then increment position by (num_bits + used bits) / 8
    /// Use the left over remainder as the new amount of used bits.
    fn drop_bits(&mut self, num_bits: usize) {
        let all_used_bits = num_bits + self.used_bits;

        self.used_bits = all_used_bits % 8;

        self.pos += all_used_bits / 8;
    }

    /// Decodes a filler of max one byte size.
    /// Decodes bits until we hit a bit that is 1.
    /// Expects that the 1 is at the end of the current byte in the buffer.
    pub fn filler(&mut self) -> Result<(), FlatDecodeError> {
        while self.zero()? {}

        Ok(())
    }

    /// Decode the next bit in the buffer.
    /// If the bit was 0 then return true.
    /// Otherwise return false.
    /// Throws EndOfBuffer error if used at the end of the array.
    fn zero(&mut self) -> Result<bool, FlatDecodeError> {
        let current_bit = self.bit()?;

        Ok(!current_bit)
    }

    /// Decode the next bit in the buffer.
    /// If the bit was 1 then return true.
    /// Otherwise return false.
    /// Throws EndOfBuffer error if used at the end of the array.
    pub fn bit(&mut self) -> Result<bool, FlatDecodeError> {
        if self.pos >= self.buffer.len() {
            return Err(FlatDecodeError::EndOfBuffer);
        }

        let b = self.buffer[self.pos] & (128 >> self.used_bits) > 0;

        self.increment_buffer_by_bit();

        Ok(b)
    }

    /// Decode an isize integer.
    ///
    /// This is byte alignment agnostic.
    /// First we decode the next 8 bits of the buffer.
    /// We take the 7 least significant bits as the 7 least significant bits of
    /// the current unsigned integer. If the most significant bit of the 8
    /// bits is 1 then we take the next 8 and repeat the process above,
    /// filling in the next 7 least significant bits of the unsigned integer and
    /// so on. If the most significant bit was instead 0 we stop decoding
    /// any more bits. Finally we use zigzag to convert the unsigned integer
    /// back to a signed integer.
    pub fn integer(&mut self) -> Result<i128, FlatDecodeError> {
        Ok(self.word()?.zigzag())
    }

    /// Decode a byte array.
    /// Decodes a filler to byte align the buffer,
    /// then decodes the next byte to get the array length up to a max of 255.
    /// We decode bytes equal to the array length to form the byte array.
    /// If the following byte for array length is not 0 we decode it and repeat
    /// above to continue decoding the byte array. We stop once we hit a
    /// byte array length of 0. If array length is 0 for first byte array
    /// length the we return a empty array.
    pub fn bytes<'a>(&mut self, arena: &'a Bump) -> Result<BumpVec<'a, u8>, FlatDecodeError> {
        self.filler()?;
        self.byte_array(arena)
    }

    /// Decode a byte array.
    /// Throws a BufferNotByteAligned error if the buffer is not byte aligned
    /// Decodes the next byte to get the array length up to a max of 255.
    /// We decode bytes equal to the array length to form the byte array.
    /// If the following byte for array length is not 0 we decode it and repeat
    /// above to continue decoding the byte array. We stop once we hit a
    /// byte array length of 0. If array length is 0 for first byte array
    /// length the we return a empty array.
    fn byte_array<'a>(&mut self, arena: &'a Bump) -> Result<BumpVec<'a, u8>, FlatDecodeError> {
        if self.used_bits != 0 {
            return Err(FlatDecodeError::BufferNotByteAligned);
        }

        self.ensure_bytes(1)?;

        let mut blk_len = self.buffer[self.pos] as usize;

        self.pos += 1;

        let mut blk_array = BumpVec::with_capacity_in(blk_len, arena);

        while blk_len != 0 {
            self.ensure_bytes(blk_len + 1)?;

            let decoded_array = &self.buffer[self.pos..self.pos + blk_len];

            blk_array.extend(decoded_array);

            self.pos += blk_len;

            blk_len = self.buffer[self.pos] as usize;

            self.pos += 1
        }

        Ok(blk_array)
    }

    /// Increment used bits by 1.
    /// If all 8 bits are used then increment buffer position by 1.
    fn increment_buffer_by_bit(&mut self) {
        if self.used_bits == 7 {
            self.pos += 1;

            self.used_bits = 0;
        } else {
            self.used_bits += 1;
        }
    }

    /// Ensures the buffer has the required bytes passed in by required_bytes.
    /// Throws a NotEnoughBytes error if there are less bytes remaining in the
    /// buffer than required_bytes.
    fn ensure_bytes(&mut self, required_bytes: usize) -> Result<(), FlatDecodeError> {
        if required_bytes > self.buffer.len() - self.pos {
            Err(FlatDecodeError::NotEnoughBytes(required_bytes))
        } else {
            Ok(())
        }
    }
}
