use crate::{constant::Integer, flat::zigzag::ZigZag};

use super::FlatEncodeError;

#[derive(Default)]
pub struct Encoder {
    pub buffer: Vec<u8>,
    // Int
    used_bits: i64,
    // Int
    current_byte: u8,
}

impl Encoder {
    /// Encode a unsigned integer of any size.
    /// This is byte alignment agnostic.
    /// We encode the 7 least significant bits of the unsigned byte. If the char
    /// value is greater than 127 we encode a leading 1 followed by
    /// repeating the above for the next 7 bits and so on.
    pub fn word(&mut self, c: usize) -> &mut Self {
        let mut d = c;
        loop {
            let mut w = (d & 127) as u8;
            d >>= 7;

            if d != 0 {
                w |= 128;
            }
            self.bits(8, w);

            if d == 0 {
                break;
            }
        }

        self
    }

    /// Encode a `bool` value. This is byte alignment agnostic.
    /// Uses the next unused bit in the current byte to encode this information.
    /// One for true and Zero for false
    pub fn bool(&mut self, x: bool) -> &mut Self {
        if x {
            self.one();
        } else {
            self.zero();
        }

        self
    }

    /// Encode an arbitrarily sized integer.
    ///
    /// This is byte alignment agnostic.
    /// First we use zigzag once to double the number and encode the negative
    /// sign as the least significant bit. Next we encode the 7 least
    /// significant bits of the unsigned integer. If the number is greater than
    /// 127 we encode a leading 1 followed by repeating the encoding above for
    /// the next 7 bits and so on.
    pub fn integer(&mut self, i: &Integer) -> &mut Self {
        self.big_word(i.zigzag());

        self
    }

    /// Encodes up to 8 bits of information and is byte alignment agnostic.
    /// Uses unused bits in the current byte to write out the passed in byte
    /// value. Overflows to the most significant digits of the next byte if
    /// number of bits to use is greater than unused bits. Expects that
    /// number of bits to use is greater than or equal to required bits by the
    /// value. The param num_bits is i64 to match unused_bits type.
    pub fn bits(&mut self, num_bits: i64, val: u8) -> &mut Self {
        match (num_bits, val) {
            (1, 0) => self.zero(),
            (1, 1) => self.one(),
            (2, 0) => {
                self.zero();
                self.zero();
            }
            (2, 1) => {
                self.zero();
                self.one();
            }
            (2, 2) => {
                self.one();
                self.zero();
            }
            (2, 3) => {
                self.one();
                self.one();
            }
            (_, _) => {
                self.used_bits += num_bits;
                let unused_bits = 8 - self.used_bits;
                match unused_bits {
                    0 => {
                        self.current_byte |= val;
                        self.next_word();
                    }
                    x if x > 0 => {
                        self.current_byte |= val << x;
                    }
                    x => {
                        let used = -x;
                        self.current_byte |= val >> used;
                        self.next_word();
                        self.current_byte = val << (8 - used);
                        self.used_bits = used;
                    }
                }
            }
        }

        self
    }

    /// Encode a byte array.
    /// Uses filler to byte align the buffer, then writes byte array length up
    /// to 255. Following that it writes the next 255 bytes from the array.
    /// We repeat writing length up to 255 and the next 255 bytes until we reach
    /// the end of the byte array. After reaching the end of the byte array
    /// we write a 0 byte. Only write 0 byte if the byte array is empty.
    pub fn bytes(&mut self, x: &[u8]) -> Result<&mut Self, FlatEncodeError> {
        // use filler to write current buffer so bits used gets reset
        self.filler();

        self.byte_array(x)
    }

    /// Encode a byte array in a byte aligned buffer. Throws exception if any
    /// bits for the current byte were used. Writes byte array length up to
    /// 255 Following that it writes the next 255 bytes from the array.
    /// We repeat writing length up to 255 and the next 255 bytes until we reach
    /// the end of the byte array. After reaching the end of the buffer we
    /// write a 0 byte. Only write 0 if the byte array is empty.
    pub fn byte_array(&mut self, arr: &[u8]) -> Result<&mut Self, FlatEncodeError> {
        if self.used_bits != 0 {
            return Err(FlatEncodeError::BufferNotByteAligned);
        }

        self.write_blk(arr);

        Ok(self)
    }

    /// Encode a unsigned integer of 128 bits size.
    /// This is byte alignment agnostic.
    /// We encode the 7 least significant bits of the unsigned byte. If the char
    /// value is greater than 127 we encode a leading 1 followed by
    /// repeating the above for the next 7 bits and so on.
    pub fn big_word(&mut self, c: Integer) -> &mut Self {
        let mut d = c;

        loop {
            let temp: Integer = d.clone() % 128;
            let mut w: u8 = temp.try_into().unwrap();

            d >>= 7;

            if d != Integer::ZERO {
                w |= 128;
            }
            self.bits(8, w);

            if d == Integer::ZERO {
                break;
            }
        }

        self
    }

    /// Encode a string.
    /// Convert to byte array and then use byte array encoding.
    /// Uses filler to byte align the buffer, then writes byte array length up
    /// to 255. Following that it writes the next 255 bytes from the array.
    /// After reaching the end of the buffer we write a 0 byte. Only write 0
    /// byte if the byte array is empty.
    pub fn utf8(&mut self, s: &str) -> Result<&mut Self, FlatEncodeError> {
        self.bytes(s.as_bytes())
    }

    /// Encode a list of bytes with a function
    /// This is byte alignment agnostic.
    /// If there are bytes in a list then write 1 bit followed by the functions
    /// encoding. After the last item write a 0 bit. If the list is empty
    /// only encode a 0 bit.
    pub fn list_with<T>(
        &mut self,
        list: &[T],
        encoder_func: for<'r> fn(&'r mut Encoder, &T) -> Result<(), FlatEncodeError>,
    ) -> Result<&mut Self, FlatEncodeError> {
        for item in list {
            self.one();

            encoder_func(self, item)?;
        }

        self.zero();

        Ok(self)
    }

    /// A filler amount of end 0's followed by a 1 at the end of a byte.
    /// Used to byte align the buffer by padding out the rest of the byte.
    pub fn filler(&mut self) -> &mut Self {
        self.current_byte |= 1;
        self.next_word();

        self
    }

    /// Write a 0 bit into the current byte.
    /// Write out to buffer if last used bit in the current byte.
    fn zero(&mut self) {
        if self.used_bits == 7 {
            self.next_word();
        } else {
            self.used_bits += 1;
        }
    }

    /// Write a 1 bit into the current byte.
    /// Write out to buffer if last used bit in the current byte.
    fn one(&mut self) {
        if self.used_bits == 7 {
            self.current_byte |= 1;
            self.next_word();
        } else {
            self.current_byte |= 128 >> self.used_bits;
            self.used_bits += 1;
        }
    }

    /// Write the current byte out to the buffer and begin next byte to write
    /// out. Add current byte to the buffer and set current byte and used
    /// bits to 0.
    fn next_word(&mut self) {
        self.buffer.push(self.current_byte);

        self.current_byte = 0;
        self.used_bits = 0;
    }

    /// Writes byte array length up to 255
    /// Following that it writes the next 255 bytes from the array.
    /// After reaching the end of the buffer we write a 0 byte. Only write 0 if
    /// the byte array is empty. This is byte alignment agnostic.
    fn write_blk(&mut self, arr: &[u8]) {
        let chunks = arr.chunks(255);

        for chunk in chunks {
            self.buffer.push(chunk.len() as u8);
            self.buffer.extend(chunk);
        }
        self.buffer.push(0_u8);
    }
}
