use bumpalo::collections::Vec as BumpVec;
use minicbor::data::IanaTag;
use rug::ops::NegAssign;

use crate::data::PlutusData;

use super::decode::Ctx;

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

                            let data = PlutusData::constr(ctx.arena, x - 121, fields);

                            Ok(data)
                        }
                        1280..=1400 => {
                            let mut fields = BumpVec::new_in(ctx.arena);

                            for x in decoder.array_iter_with(ctx)? {
                                fields.push(x?);
                            }

                            let data = PlutusData::constr(ctx.arena, (x - 1280) + 7, fields);

                            Ok(data)
                        }
                        102 => {
                            todo!("tagged data")
                        }
                        _ => {
                            let e = minicbor::decode::Error::message(format!(
                                "unknown tag for plutus data tag: {}",
                                tag
                            ));

                            Err(e)
                        }
                    };
                }

                match tag.try_into() {
                    Ok(x @ IanaTag::PosBignum | x @ IanaTag::NegBignum) => {
                        let mut bytes = BumpVec::new_in(ctx.arena);

                        for chunk in decoder.bytes_iter()? {
                            let chunk = chunk?;

                            bytes.extend_from_slice(chunk);
                        }

                        let integer = ctx
                            .arena
                            .alloc(rug::Integer::from_digits(&bytes, rug::integer::Order::Msf));

                        if x == IanaTag::NegBignum {
                            integer.neg_assign();
                        }

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

                Ok(PlutusData::map(ctx.arena, fields))
            }
            minicbor::data::Type::Bytes | minicbor::data::Type::BytesIndef => {
                let mut bs = BumpVec::new_in(ctx.arena);

                for chunk in decoder.bytes_iter()? {
                    let chunk = chunk?;

                    bs.extend_from_slice(chunk);
                }

                Ok(PlutusData::byte_string(ctx.arena, bs))
            }
            minicbor::data::Type::Array | minicbor::data::Type::ArrayIndef => {
                let mut fields = BumpVec::new_in(ctx.arena);

                for x in decoder.array_iter_with(ctx)? {
                    fields.push(x?);
                }

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
