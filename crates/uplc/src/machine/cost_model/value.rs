use rug::integer::BorrowInteger;

use crate::constant::Integer;

pub fn integer_ex_mem(i: &Integer) -> i64 {
    if *i == 0 {
        1
    } else {
        (integer_log2(i.as_abs()) / 64) + 1
    }
}

fn integer_log2(i: BorrowInteger<'_>) -> i64 {
    if i.is_zero() {
        return 0;
    }

    (i.significant_bits() - 1) as i64
}

pub fn byte_string_ex_mem(b: &[u8]) -> i64 {
    if b.is_empty() {
        1
    } else {
        ((b.len() as i64 - 1) / 8) + 1
    }
}

pub fn string_ex_mem(s: &str) -> i64 {
    s.chars().count() as i64
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::integer_log2;

    #[test]
    fn integer_log2_oracle() {
        // Values come from the Haskell implementation
        assert_eq!(integer_log2(rug::Integer::from(0).as_abs()), 0);
        assert_eq!(integer_log2(rug::Integer::from(1).as_abs()), 0);
        assert_eq!(integer_log2(rug::Integer::from(42).as_abs()), 5);

        assert_eq!(
            integer_log2(
                rug::Integer::from_str("18446744073709551615")
                    .unwrap()
                    .as_abs()
            ),
            63
        );
        assert_eq!(
            integer_log2(
                rug::Integer::from_str("999999999999999999999999999999")
                    .unwrap()
                    .as_abs()
            ),
            99
        );
        assert_eq!(
            integer_log2(
                rug::Integer::from_str("170141183460469231731687303715884105726")
                    .unwrap()
                    .as_abs()
            ),
            126
        );
        assert_eq!(
            integer_log2(
                rug::Integer::from_str("170141183460469231731687303715884105727")
                    .unwrap()
                    .as_abs()
            ),
            126
        );
        assert_eq!(
            integer_log2(
                rug::Integer::from_str("170141183460469231731687303715884105728")
                    .unwrap()
                    .as_abs()
            ),
            127
        );
        assert_eq!(
            integer_log2(
                rug::Integer::from_str("340282366920938463463374607431768211458")
                    .unwrap()
                    .as_abs()
            ),
            128
        );
        assert_eq!(
            integer_log2(
                rug::Integer::from_str("999999999999999999999999999999999999999999")
                    .unwrap()
                    .as_abs()
            ),
            139
        );
        assert_eq!(
            integer_log2(
                rug::Integer::from_str("999999999999999999999999999999999999999999999999999999999999999999999999999999999999")
                    .unwrap()
                    .as_abs()
            ),
            279
        );
    }
}
