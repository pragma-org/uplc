use bumpalo::collections::Vec as BumpVec;
use minicbor::data::{IanaTag, Tag};

use crate::data::PlutusData;

use super::Ctx;

impl<'a, 'b> minicbor::decode::Decode<'b, Ctx<'a>> for &'a PlutusData<'a> {
    fn decode(
        decoder: &mut minicbor::Decoder<'b>,
        ctx: &mut Ctx<'a>,
    ) -> Result<Self, minicbor::decode::Error> {
        let typ = decoder.datatype()?;

        match typ {
            minicbor::data::Type::Tag => {
                let mut probe = decoder.probe();

                let tag = probe.tag()?;

                if matches!(tag.as_u64(), 121..=127 | 1280..=1400 | 102) {
                    let x = decoder.tag()?.as_u64();

                    return match x {
                        121..=127 => {
                            let mut fields = BumpVec::new_in(ctx.arena);

                            for x in decoder.array_iter_with(ctx)? {
                                fields.push(x?);
                            }

                            let fields = ctx.arena.alloc(fields);

                            let data = PlutusData::constr(ctx.arena, x - 121, fields);

                            Ok(data)
                        }
                        1280..=1400 => {
                            let mut fields = BumpVec::new_in(ctx.arena);

                            for x in decoder.array_iter_with(ctx)? {
                                fields.push(x?);
                            }

                            let fields = ctx.arena.alloc(fields);

                            let data = PlutusData::constr(ctx.arena, (x - 1280) + 7, fields);

                            Ok(data)
                        }
                        102 => {
                            let mut fields = BumpVec::new_in(ctx.arena);

                            let count = decoder.array()?;
                            if count != Some(2) {
                                return Err(minicbor::decode::Error::message(format!(
                                    "expected array of length 2 following plutus data tag 102",
                                )));
                            }

                            let discriminator_i128: i128 = decoder.int()?.into();
                            let discriminator: u64 = match u64::try_from(discriminator_i128) {
                                Ok(n) => n,
                                Err(e) => {
                                    return Err(minicbor::decode::Error::message(format!(
                                        "could not cast discriminator from plutus data tag 102 into u64: {discriminator_i128}",
                                    )));
                                }
                            };

                            for x in decoder.array_iter_with(ctx)? {
                                fields.push(x?);
                            }

                            let fields = ctx.arena.alloc(fields);

                            let data = PlutusData::constr(ctx.arena, discriminator, fields);

                            Ok(data)
                        }
                        _ => {
                            let e = minicbor::decode::Error::message(format!(
                                "unknown tag for plutus data tag: {tag}",
                            ));

                            Err(e)
                        }
                    };
                }

                match tag.try_into() {
                    Ok(x @ IanaTag::PosBignum | x @ IanaTag::NegBignum) => {
                        let _ = decoder.tag()?;
                        let mut bytes = BumpVec::new_in(ctx.arena);

                        for chunk in decoder.bytes_iter()? {
                            let chunk = chunk?;

                            bytes.extend_from_slice(chunk);
                        }

                        let integer = ctx.arena.alloc(num::BigInt::from_bytes_be(
                            if x == IanaTag::PosBignum {
                                num_bigint::Sign::Plus
                            } else {
                                num_bigint::Sign::Minus
                            },
                            &bytes,
                        ));

                        Ok(PlutusData::integer(ctx.arena, integer))
                    }

                    _ => {
                        let e = minicbor::decode::Error::message(format!(
                            "unknown tag for plutus data tag: {tag}",
                        ));

                        Err(e)
                    }
                }
            }
            minicbor::data::Type::Map | minicbor::data::Type::MapIndef => {
                let mut fields = BumpVec::new_in(ctx.arena);

                for x in decoder.map_iter_with(ctx)? {
                    let x = x?;

                    fields.push(x);
                }

                let fields = ctx.arena.alloc(fields);

                Ok(PlutusData::map(ctx.arena, fields))
            }
            minicbor::data::Type::Bytes | minicbor::data::Type::BytesIndef => {
                let mut bs = BumpVec::new_in(ctx.arena);

                for chunk in decoder.bytes_iter()? {
                    let chunk = chunk?;

                    bs.extend_from_slice(chunk);
                }

                let bs = ctx.arena.alloc(bs);

                Ok(PlutusData::byte_string(ctx.arena, bs))
            }
            minicbor::data::Type::Array | minicbor::data::Type::ArrayIndef => {
                let mut fields = BumpVec::new_in(ctx.arena);

                for x in decoder.array_iter_with(ctx)? {
                    fields.push(x?);
                }

                let fields = ctx.arena.alloc(fields);

                Ok(PlutusData::list(ctx.arena, fields))
            }
            minicbor::data::Type::U8
            | minicbor::data::Type::U16
            | minicbor::data::Type::U32
            | minicbor::data::Type::U64
            | minicbor::data::Type::I8
            | minicbor::data::Type::I16
            | minicbor::data::Type::I32
            | minicbor::data::Type::I64
            | minicbor::data::Type::Int => {
                let i: i128 = decoder.int()?.into();

                Ok(PlutusData::integer_from(ctx.arena, i))
            }
            any => {
                let e = minicbor::decode::Error::message(format!(
                    "bad cbor data type ({any:?}) for plutus data"
                ));

                Err(e)
            }
        }
    }
}

fn encode_bytestring<'a, W: minicbor::encode::Write>(e: &'a mut minicbor::Encoder<W>, bs: &[u8]) -> Result<&'a mut minicbor::Encoder<W>, minicbor::encode::Error<W::Error>> {
    const CHUNK_SIZE: usize = 64;

    if bs.len() <= 64 {
        e.bytes(bs)?;
    } else {
        e.begin_bytes()?;

        for b in bs.chunks(CHUNK_SIZE) {
            e.bytes(b)?;
        }

        e.end()?;
    }
    Ok(e)
}

impl<C> minicbor::encode::Encode<C> for PlutusData<'_> {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            PlutusData::Constr { tag, fields } => {
                if *tag < 7 {
                    e.tag(Tag::new(*tag + 121))?;
                } else if *tag <= 127 {
                    e.tag(Tag::new((*tag - 7) + 1280))?;
                } else {
                    e.tag(Tag::new(102))?;
                    e.array(2)?;
                    e.u64(*tag);
                }

                // defaultEncodeList in Codec.Serialise emits definite in case of 0-length list
                // https://github.com/well-typed/cborg/blob/1e9d079d382f237a1a282e268eecce2b395acb9c/serialise/src/Codec/Serialise/Class.hs#L165-L171
                if fields.len() == 0 {
                    e.array(0)?;
                } else {
                    // TODO: figure out if we need to care about def vs indef
                    // The encoding implementation in plutus-core uses indefinite here,
                    // though both forms are accepted when decoding
                    // https://github.com/IntersectMBO/plutus/blob/9538fc9829426b2ecb0628d352e2d7af96ec8204/plutus-core/plutus-core/src/PlutusCore/Data.hs#L198
                    e.begin_array()?;
                    for f in fields.iter() {
                        f.encode(e, ctx)?;
                    }
                    e.end()?;
                }
            }
            // stolen from pallas
            // we use definite array to match the approach used by haskell's plutus
            // implementation https://github.com/input-output-hk/plutus/blob/9538fc9829426b2ecb0628d352e2d7af96ec8204/plutus-core/plutus-core/src/PlutusCore/Data.hs#L152
            PlutusData::Map(map) => {
                let len: u64 = map
                    .len()
                    .try_into()
                    .expect("setting map length should work fine");

                e.map(len)?;

                for (k, v) in map.iter() {
                    k.encode(e, ctx)?;
                    v.encode(e, ctx)?;
                }
            }
            PlutusData::Integer(n) => {
                let (sign, digits) = n.to_u64_digits();
                match sign {
                    num_bigint::Sign::Plus => {
                        if digits.len() == 1 {
                            e.u64(digits[0])?;
                        } else {
                            e.tag(Tag::new(2))?;
                            let (_sign, bytes) = n.to_bytes_be();
                            encode_bytestring(e, &bytes)?;
                        }
                    }
                    num_bigint::Sign::Minus => {
                        if digits.len() == 1 {
                            let integer = minicbor::data::Int::try_from(digits[0] as i128).unwrap();
                            e.int(integer)?;
                        } else {
                            e.tag(Tag::new(3))?;
                            let (_sign, bytes) = n.to_bytes_be();
                            encode_bytestring(e, &bytes)?;
                        }
                    }
                    num_bigint::Sign::NoSign => {
                        e.u8(0)?;
                    }
                }
            }
            // we match the haskell implementation by encoding bytestrings longer than 64
            // bytes as indefinite lists of bytes
            PlutusData::ByteString(bs) => {
                encode_bytestring(e, bs)?;
            }
            PlutusData::List(_) => todo!(),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binder::DeBruijn;
    use crate::flat::decode;
    use bumpalo::Bump;

    #[test]
    fn encode_empty_record() {
        let d = PlutusData::Constr {
            tag: 0,
            fields: &[],
        };
        let mut v = vec![];
        minicbor::encode(d, &mut v);
        assert_eq!(hex::encode(v), "d87980");
    }

    #[test]
    fn encode_record() {
        let b1 = PlutusData::ByteString(&[0x00]);
        let b2 = PlutusData::ByteString(&[0x00, 0x01]);
        let d = PlutusData::Constr {
            tag: 1,
            fields: &[
                &b1,
                &b2,
            ],
        };
        let mut v = vec![];
        minicbor::encode(d, &mut v);
        assert_eq!(hex::encode(v), "d87a9f4100420001ff");
    }

    #[test]
    fn encode_record_integer() {
        let zero = num::BigInt::from(0);
        let one = num::BigInt::from(1);
        let d = PlutusData::Constr {
            tag: 128,
            fields: &[
                &PlutusData::Integer(&zero),
                &PlutusData::Integer(&one),
            ],
        };
        let mut v = vec![];
        minicbor::encode(d, &mut v);
        assert_eq!(hex::encode(v), "d8668218809f0001ff");
    }

    #[test]
    fn encode_cbor_data_bigint() {
        let big = num::BigInt::from_bytes_be(
            num_bigint::Sign::Plus,
            &hex::decode("033b2e3c9fd0803ce7ffffff").unwrap()
        );
        let d = PlutusData::Constr {
            tag: 0,
            fields: &[
                &PlutusData::Integer(&big),
            ],
        };
        let mut v = vec![];
        minicbor::encode(d, &mut v);
        assert_eq!(hex::encode(v), "d8799fc24c033b2e3c9fd0803ce7ffffffff");
    }

}
