// #[cfg(feature = "num-bigint")]
// use num_bigint::{BigInt, BigUint, ToBigInt};

use crate::constant::Integer;
use ibig::{ibig, ops::UnsignedAbs, IBig, UBig};

pub trait ZigZag {
    type Zag;

    fn zigzag(self) -> Self::Zag;
    fn unzigzag(zag: Self::Zag) -> Self;
}

impl ZigZag for Integer {
    type Zag = UBig;

    fn zigzag(self) -> Self::Zag {
        if self >= ibig!(0) {
            // For non-negative numbers, just multiply by 2 (left shift by 1)
            self.clone().unsigned_abs() << 1
        } else {
            // For negative numbers: -(2 * n) - 1
            // First multiply by 2
            let double: Integer = self.clone() << 1;

            // Then negate and subtract 1
            let zagged: IBig = -double - 1;

            zagged.unsigned_abs()
        }
    }

    fn unzigzag(zag: Self::Zag) -> Self {
        let significant_bits = zag.clone() >> 1;
        let significant_bits = IBig::from(significant_bits);

        let exp: UBig = zag.clone() & 1;
        let exp: i32 = exp.try_into().unwrap();
        let exp = -exp;

        significant_bits ^ exp
    }
}
