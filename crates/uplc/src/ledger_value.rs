//! Cardano multi-asset ledger values.
//!
//! A [`LedgerValue`] is a sorted, canonical representation of a Cardano multi-asset value
//! (currency symbol → token name → quantity). It is used by the `Value`-related built-in
//! functions introduced in later Plutus versions.

use bumpalo::collections::Vec as BumpVec;
use num::{Signed, Zero};

use crate::{
    arena::Arena,
    constant::{integer, Integer},
    data::PlutusData,
};

/// Errors produced when deserialising [`PlutusData`] into a [`LedgerValue`].
#[derive(thiserror::Error, Debug)]
pub enum UnValueDataError {
    /// Expected a `Map` constructor but found something else.
    #[error("non-Map constructor")]
    NonMapConstructor,
    /// Expected a `ByteString` constructor but found something else.
    #[error("non-B constructor")]
    NonByteStringConstructor,
    /// Expected an `Integer` constructor but found something else.
    #[error("non-I constructor")]
    NonIntegerConstructor,
    /// A currency symbol or token name exceeds the 32-byte limit.
    #[error("invalid key")]
    InvalidKey,
    /// An inner token map is empty.
    #[error("empty inner map")]
    EmptyInnerMap,
    /// Currency symbols are not in strictly ascending byte order.
    #[error("currency symbols not strictly ascending")]
    CurrencyNotAscending,
    /// Token names are not in strictly ascending byte order.
    #[error("token names not strictly ascending")]
    TokenNotAscending,
    /// A token quantity is zero or out of the valid range.
    #[error("invalid quantity")]
    InvalidQuantity,
}

/// Errors produced by `Value` built-in function operations.
#[derive(thiserror::Error, Debug)]
pub enum ValueError {
    /// `insertCoin` received an invalid currency symbol.
    #[error("insertCoin: invalid currency")]
    InsertCoinInvalidCurrency,
    /// `insertCoin` received an invalid token name.
    #[error("insertCoin: invalid token")]
    InsertCoinInvalidToken,
    /// `unionValue` produced a quantity outside the signed 128-bit range.
    #[error("unionValue: quantity is out of the signed 128-bit integer bounds")]
    UnionValueQuantityOutOfBounds,
    /// `valueContains` called with a first value that has negative amounts.
    #[error("valueContains: first value contains negative amounts")]
    ValueContainsFirstNegative,
    /// `valueContains` called with a second value that has negative amounts.
    #[error("valueContains: second value contains negative amounts")]
    ValueContainsSecondNegative,
    /// `scaleValue` produced a quantity outside the signed 128-bit range.
    #[error("scaleValue: quantity out of bounds")]
    ScaleValueQuantityOutOfBounds,
    /// `valueData` input exceeds the maximum allowed size.
    #[error("valueData: maximum input size ({0}) exceeded")]
    ValueDataMaxSizeExceeded(usize),
    /// Error during `unValueData` deserialisation.
    #[error("unValueData: {0}")]
    UnValueData(#[from] UnValueDataError),
    /// A quantity is outside the signed 128-bit integer bounds.
    #[error("Quantity out of signed 128-bit integer bounds")]
    QuantityOutOfBounds,
}

/// A Cardano multi-asset value, sorted by currency symbol then token name.
///
/// Entries are kept in strictly ascending order to allow efficient merging and comparison.
#[derive(Debug, PartialEq)]
pub struct LedgerValue<'a> {
    /// Currency entries, sorted by currency symbol in ascending byte order.
    pub entries: &'a [CurrencyEntry<'a>],
    /// Total number of individual token entries across all currencies.
    pub size: usize,
    /// Number of token entries with a negative quantity.
    pub negative_count: usize,
}

/// A single currency symbol and its associated token entries.
#[derive(Debug, PartialEq, Clone)]
pub struct CurrencyEntry<'a> {
    /// Currency symbol (policy ID) as raw bytes.
    pub currency: &'a [u8],
    /// Token entries under this currency, sorted by token name in ascending byte order.
    pub tokens: &'a [TokenEntry<'a>],
}

/// A single token name and its quantity within a currency.
#[derive(Debug, PartialEq, Clone)]
pub struct TokenEntry<'a> {
    /// Token name as raw bytes.
    pub name: &'a [u8],
    /// Quantity of this token (may be negative in intermediate results).
    pub quantity: &'a Integer,
}

impl<'a> LedgerValue<'a> {
    /// Returns an empty ledger value with no currency entries.
    pub fn empty(arena: &'a Arena) -> &'a LedgerValue<'a> {
        arena.alloc(LedgerValue {
            entries: &[],
            size: 0,
            negative_count: 0,
        })
    }

    /// Looks up the quantity for a given currency and token name, returning zero if absent.
    pub fn lookup_coin(&self, arena: &'a Arena, ccy: &[u8], tok: &[u8]) -> &'a Integer {
        for entry in self.entries {
            match entry.currency.cmp(ccy) {
                std::cmp::Ordering::Equal => {
                    for token in entry.tokens {
                        match token.name.cmp(tok) {
                            std::cmp::Ordering::Equal => return token.quantity,
                            std::cmp::Ordering::Greater => break,
                            _ => {}
                        }
                    }
                    return integer(arena);
                }
                std::cmp::Ordering::Greater => break,
                _ => {}
            }
        }
        integer(arena)
    }

    /// Inserts or replaces a single token quantity in the value, maintaining sort order.
    ///
    /// If `qty` is zero, the entry is removed.
    pub fn insert_coin(
        arena: &'a Arena,
        ccy: &'a [u8],
        tok: &'a [u8],
        qty: &'a Integer,
        v: &'a LedgerValue<'a>,
    ) -> &'a LedgerValue<'a> {
        let mut currency_entries = BumpVec::new_in(arena.as_bump());
        let mut found_ccy = false;

        for entry in v.entries {
            match entry.currency.cmp(ccy) {
                std::cmp::Ordering::Less => {
                    currency_entries.push(entry.clone());
                }
                std::cmp::Ordering::Equal => {
                    found_ccy = true;
                    let mut token_entries = BumpVec::new_in(arena.as_bump());
                    let mut found_tok = false;

                    for token in entry.tokens {
                        match token.name.cmp(tok) {
                            std::cmp::Ordering::Less => {
                                token_entries.push(token.clone());
                            }
                            std::cmp::Ordering::Equal => {
                                found_tok = true;
                                if !qty.is_zero() {
                                    token_entries.push(TokenEntry {
                                        name: tok,
                                        quantity: qty,
                                    });
                                }
                            }
                            std::cmp::Ordering::Greater => {
                                if !found_tok {
                                    found_tok = true;
                                    if !qty.is_zero() {
                                        token_entries.push(TokenEntry {
                                            name: tok,
                                            quantity: qty,
                                        });
                                    }
                                }
                                token_entries.push(token.clone());
                            }
                        }
                    }
                    if !found_tok && !qty.is_zero() {
                        token_entries.push(TokenEntry {
                            name: tok,
                            quantity: qty,
                        });
                    }

                    let tokens: &'a [TokenEntry<'a>] = arena.alloc(token_entries);
                    if !tokens.is_empty() {
                        currency_entries.push(CurrencyEntry {
                            currency: entry.currency,
                            tokens,
                        });
                    }
                }
                std::cmp::Ordering::Greater => {
                    if !found_ccy {
                        found_ccy = true;
                        if !qty.is_zero() {
                            let tokens: &'a [TokenEntry<'a>] = arena.alloc([TokenEntry {
                                name: tok,
                                quantity: qty,
                            }]);
                            currency_entries.push(CurrencyEntry {
                                currency: ccy,
                                tokens,
                            });
                        }
                    }
                    currency_entries.push(entry.clone());
                }
            }
        }

        if !found_ccy && !qty.is_zero() {
            let tokens: &'a [TokenEntry<'a>] = arena.alloc([TokenEntry {
                name: tok,
                quantity: qty,
            }]);
            currency_entries.push(CurrencyEntry {
                currency: ccy,
                tokens,
            });
        }

        let entries: &'a [CurrencyEntry<'a>] = arena.alloc(currency_entries);
        let (size, negative_count) = count_stats(entries);
        arena.alloc(LedgerValue {
            entries,
            size,
            negative_count,
        })
    }

    /// Merges two ledger values by summing quantities for matching currency/token pairs.
    ///
    /// Entries with a resulting zero quantity are dropped.
    pub fn union_value(
        arena: &'a Arena,
        v1: &'a LedgerValue<'a>,
        v2: &'a LedgerValue<'a>,
    ) -> Result<&'a LedgerValue<'a>, ValueError> {
        let mut currency_entries = BumpVec::new_in(arena.as_bump());
        let mut i = 0usize;
        let mut j = 0usize;

        while i < v1.entries.len() && j < v2.entries.len() {
            match v1.entries[i].currency.cmp(v2.entries[j].currency) {
                std::cmp::Ordering::Less => {
                    currency_entries.push(v1.entries[i].clone());
                    i += 1;
                }
                std::cmp::Ordering::Greater => {
                    currency_entries.push(v2.entries[j].clone());
                    j += 1;
                }
                std::cmp::Ordering::Equal => {
                    let merged = merge_tokens(arena, v1.entries[i].tokens, v2.entries[j].tokens)?;
                    if !merged.is_empty() {
                        currency_entries.push(CurrencyEntry {
                            currency: v1.entries[i].currency,
                            tokens: merged,
                        });
                    }
                    i += 1;
                    j += 1;
                }
            }
        }

        while i < v1.entries.len() {
            currency_entries.push(v1.entries[i].clone());
            i += 1;
        }
        while j < v2.entries.len() {
            currency_entries.push(v2.entries[j].clone());
            j += 1;
        }

        let entries: &'a [CurrencyEntry<'a>] = arena.alloc(currency_entries);
        let (size, negative_count) = count_stats(entries);
        Ok(arena.alloc(LedgerValue {
            entries,
            size,
            negative_count,
        }))
    }

    /// Returns `true` if every token in `v2` is present in `v1` with at least the same quantity.
    ///
    /// Both values must be non-negative; returns an error otherwise.
    pub fn value_contains(v1: &LedgerValue<'a>, v2: &LedgerValue<'a>) -> Result<bool, ValueError> {
        // 1. Check v1 for negatives
        if v1.negative_count > 0 {
            return Err(ValueError::ValueContainsFirstNegative);
        }

        // 2. Check v2 for negatives
        if v2.negative_count > 0 {
            return Err(ValueError::ValueContainsSecondNegative);
        }

        // 3. v2 has more tokens than v1, so v1 cannot contain v2
        if v1.size < v2.size {
            return Ok(false);
        }

        // 4. Sorted lockstep check that v2 is a submap of v1
        let mut v1_iter = v1.entries.iter().flat_map(|entry| {
            entry
                .tokens
                .iter()
                .map(move |token| (entry.currency, token.name, token.quantity))
        });
        let mut v1_current = v1_iter.next();

        for v2_entry in v2.entries {
            for v2_token in v2_entry.tokens {
                loop {
                    match v1_current {
                        Some((v1_ccy, v1_name, v1_qty)) => {
                            match (v1_ccy, v1_name).cmp(&(v2_entry.currency, v2_token.name)) {
                                std::cmp::Ordering::Less => {
                                    v1_current = v1_iter.next();
                                }
                                std::cmp::Ordering::Equal => {
                                    if v1_qty < v2_token.quantity {
                                        return Ok(false);
                                    }
                                    v1_current = v1_iter.next();
                                    break;
                                }
                                std::cmp::Ordering::Greater => {
                                    return Ok(false);
                                }
                            }
                        }
                        None => return Ok(false),
                    }
                }
            }
        }

        Ok(true)
    }

    /// Multiplies every quantity in the value by `scalar`.
    ///
    /// Returns an empty value if `scalar` is zero.
    pub fn scale_value(
        arena: &'a Arena,
        scalar: &'a Integer,
        v: &'a LedgerValue<'a>,
    ) -> Result<&'a LedgerValue<'a>, ValueError> {
        if scalar.is_zero() {
            return Ok(LedgerValue::empty(arena));
        }

        let mut currency_entries = BumpVec::new_in(arena.as_bump());

        for entry in v.entries {
            let mut token_entries = BumpVec::new_in(arena.as_bump());

            for token in entry.tokens {
                let result = token.quantity * scalar;
                check_quantity_range(&result)
                    .map_err(|_| ValueError::ScaleValueQuantityOutOfBounds)?;

                if !result.is_zero() {
                    let qty = arena.alloc_integer(result);
                    token_entries.push(TokenEntry {
                        name: token.name,
                        quantity: qty,
                    });
                }
            }

            let tokens: &'a [TokenEntry<'a>] = arena.alloc(token_entries);
            if !tokens.is_empty() {
                currency_entries.push(CurrencyEntry {
                    currency: entry.currency,
                    tokens,
                });
            }
        }

        let entries: &'a [CurrencyEntry<'a>] = arena.alloc(currency_entries);
        let (size, negative_count) = count_stats(entries);
        Ok(arena.alloc(LedgerValue {
            entries,
            size,
            negative_count,
        }))
    }

    /// Serialises a ledger value into a [`PlutusData`] map-of-maps representation.
    pub fn value_data(
        arena: &'a Arena,
        v: &'a LedgerValue<'a>,
    ) -> Result<&'a PlutusData<'a>, ValueError> {
        const VALUE_DATA_MAX_SIZE: usize = 40_000;

        if v.size > VALUE_DATA_MAX_SIZE {
            return Err(ValueError::ValueDataMaxSizeExceeded(VALUE_DATA_MAX_SIZE));
        }

        let mut outer_pairs = BumpVec::new_in(arena.as_bump());

        for entry in v.entries {
            let ccy_data = PlutusData::byte_string(arena, entry.currency);

            let mut inner_pairs = BumpVec::new_in(arena.as_bump());

            for token in entry.tokens {
                let tok_data = PlutusData::byte_string(arena, token.name);
                let qty_data = PlutusData::integer(arena, token.quantity);
                inner_pairs.push((tok_data as &PlutusData, qty_data as &PlutusData));
            }

            let inner_pairs: &'a [_] = arena.alloc(inner_pairs);
            let inner_map = PlutusData::map(arena, inner_pairs);
            outer_pairs.push((ccy_data as &PlutusData, inner_map as &PlutusData));
        }

        let outer_pairs: &'a [_] = arena.alloc(outer_pairs);
        Ok(PlutusData::map(arena, outer_pairs))
    }

    /// Deserialises a [`PlutusData`] map-of-maps into a [`LedgerValue`].
    ///
    /// Validates that keys are ≤ 32 bytes, in strictly ascending order, inner maps are
    /// non-empty, quantities are non-zero, and all quantities fit in a signed 128-bit range.
    pub fn un_value_data(
        arena: &'a Arena,
        d: &'a PlutusData<'a>,
    ) -> Result<&'a LedgerValue<'a>, ValueError> {
        let outer_map = match d {
            PlutusData::Map(m) => m,
            _ => return Err(UnValueDataError::NonMapConstructor.into()),
        };

        let mut currency_entries = BumpVec::new_in(arena.as_bump());
        let mut prev_ccy: Option<&[u8]> = None;

        for (key, value) in outer_map.iter() {
            let ccy = match key {
                PlutusData::ByteString(bs) => *bs,
                _ => return Err(UnValueDataError::NonByteStringConstructor.into()),
            };

            if ccy.len() > 32 {
                return Err(UnValueDataError::InvalidKey.into());
            }

            // Check strictly ascending order
            if let Some(prev) = prev_ccy {
                if prev.cmp(ccy) != std::cmp::Ordering::Less {
                    return Err(UnValueDataError::CurrencyNotAscending.into());
                }
            }
            prev_ccy = Some(ccy);

            let inner_map = match value {
                PlutusData::Map(m) => m,
                _ => return Err(UnValueDataError::NonMapConstructor.into()),
            };

            if inner_map.is_empty() {
                return Err(UnValueDataError::EmptyInnerMap.into());
            }

            let mut token_entries = BumpVec::new_in(arena.as_bump());
            let mut prev_tok: Option<&[u8]> = None;

            for (inner_key, inner_value) in inner_map.iter() {
                let tok = match inner_key {
                    PlutusData::ByteString(bs) => *bs,
                    _ => return Err(UnValueDataError::NonByteStringConstructor.into()),
                };

                if tok.len() > 32 {
                    return Err(UnValueDataError::InvalidKey.into());
                }

                // Check strictly ascending order
                if let Some(prev) = prev_tok {
                    if prev.cmp(tok) != std::cmp::Ordering::Less {
                        return Err(UnValueDataError::TokenNotAscending.into());
                    }
                }
                prev_tok = Some(tok);

                let qty = match inner_value {
                    PlutusData::Integer(i) => *i,
                    _ => return Err(UnValueDataError::NonIntegerConstructor.into()),
                };

                if qty.is_zero() {
                    return Err(UnValueDataError::InvalidQuantity.into());
                }

                check_quantity_range(qty)
                    .map_err(|_| ValueError::from(UnValueDataError::InvalidQuantity))?;

                token_entries.push(TokenEntry {
                    name: tok,
                    quantity: qty,
                });
            }

            let tokens: &'a [TokenEntry<'a>] = arena.alloc(token_entries);
            currency_entries.push(CurrencyEntry {
                currency: ccy,
                tokens,
            });
        }

        let entries: &'a [CurrencyEntry<'a>] = arena.alloc(currency_entries);
        let (size, negative_count) = count_stats(entries);
        Ok(arena.alloc(LedgerValue {
            entries,
            size,
            negative_count,
        }))
    }
}

/// Counts the total number of token entries and how many have negative quantities.
pub fn count_stats(entries: &[CurrencyEntry]) -> (usize, usize) {
    let mut total_size = 0usize;
    let mut negative_count = 0usize;
    for e in entries {
        for t in e.tokens {
            total_size += 1;
            if t.quantity.is_negative() {
                negative_count += 1;
            }
        }
    }
    (total_size, negative_count)
}

fn merge_tokens<'a>(
    arena: &'a Arena,
    t1: &[TokenEntry<'a>],
    t2: &[TokenEntry<'a>],
) -> Result<&'a [TokenEntry<'a>], ValueError> {
    let mut result = BumpVec::new_in(arena.as_bump());
    let mut i = 0usize;
    let mut j = 0usize;

    while i < t1.len() && j < t2.len() {
        match t1[i].name.cmp(t2[j].name) {
            std::cmp::Ordering::Less => {
                result.push(t1[i].clone());
                i += 1;
            }
            std::cmp::Ordering::Greater => {
                result.push(t2[j].clone());
                j += 1;
            }
            std::cmp::Ordering::Equal => {
                let sum = t1[i].quantity + t2[j].quantity;
                check_quantity_range(&sum)
                    .map_err(|_| ValueError::UnionValueQuantityOutOfBounds)?;
                if !sum.is_zero() {
                    let qty = arena.alloc_integer(sum);
                    result.push(TokenEntry {
                        name: t1[i].name,
                        quantity: qty,
                    });
                }
                i += 1;
                j += 1;
            }
        }
    }

    while i < t1.len() {
        result.push(t1[i].clone());
        i += 1;
    }
    while j < t2.len() {
        result.push(t2[j].clone());
        j += 1;
    }

    Ok(arena.alloc(result) as &'a [TokenEntry<'a>])
}

/// Check that a quantity is within the 128-bit signed range: -(2^127) to (2^127 - 1).
pub fn check_quantity_range(int: &Integer) -> Result<(), ValueError> {
    let bits = int.bits();
    if bits <= 127 {
        return Ok(());
    }
    if bits > 128 {
        return Err(ValueError::QuantityOutOfBounds);
    }
    // bits == 128: only valid if negative and exactly -(2^127)
    if !int.is_negative() {
        return Err(ValueError::QuantityOutOfBounds);
    }
    let magnitude = int.magnitude();
    use num::One;
    let two_pow_127 = num::BigUint::one() << 127;
    if *magnitude == two_pow_127 {
        Ok(())
    } else {
        Err(ValueError::QuantityOutOfBounds)
    }
}

/// Returns the approximate tree depth of the value for costing purposes.
pub fn value_max_depth(v: &LedgerValue) -> i64 {
    let outer_size = v.entries.len();
    let mut max_inner = 0usize;
    for entry in v.entries {
        if entry.tokens.len() > max_inner {
            max_inner = entry.tokens.len();
        }
    }
    let log_outer: i64 = if outer_size > 0 {
        (outer_size as f64).log2() as i64 + 1
    } else {
        0
    };
    let log_inner: i64 = if max_inner > 0 {
        (max_inner as f64).log2() as i64 + 1
    } else {
        0
    };
    log_outer + log_inner
}

/// Counts the total number of nodes in a [`PlutusData`] tree for costing purposes.
pub fn data_node_count(d: &PlutusData) -> i64 {
    let mut total: i64 = 0;
    let mut stack: Vec<&PlutusData> = vec![d];

    while let Some(current) = stack.pop() {
        total += 1;
        match current {
            PlutusData::Constr { fields, .. } => {
                for field in fields.iter() {
                    stack.push(field);
                }
            }
            PlutusData::Map(pairs) => {
                for (key, value) in pairs.iter() {
                    stack.push(key);
                    stack.push(value);
                }
            }
            PlutusData::List(items) => {
                for item in items.iter() {
                    stack.push(item);
                }
            }
            PlutusData::Integer(_) | PlutusData::ByteString(_) => {}
        }
    }

    total
}
