use std;
use std::ops::*;

const FIXED_POINT: i32 = 1 << 24;
const MAX: Q8p24 = Q8p24(std::i32::MAX);
const MIN: Q8p24 = Q8p24(std::i32::MIN);

use std::fmt::{self, Formatter, Debug};

/// a int with 8 bits and 24 fixed point
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Q8p24(pub i32);

impl Q8p24 {
    /// get the value without the fractional bits
    pub fn trunc(self) -> Q8p24 {
        Q8p24(self.0 & !(FIXED_POINT-1))
    }

    /// get the fractional bits
    pub fn fract(self) -> Q8p24 {
        Q8p24(self.0 & (FIXED_POINT-1))
    }
}

impl Add for Q8p24 {
    type Output = Q8p24;
    fn add(self, rhs: Q8p24) -> Q8p24 {
        Q8p24(self.0.saturating_add(rhs.0))
    }
}

impl Sub for Q8p24 {
    type Output = Q8p24;
    fn sub(self, rhs: Q8p24) -> Q8p24 {
        Q8p24(self.0.saturating_sub(rhs.0))
    }
}

impl Mul for Q8p24 {
    type Output = Q8p24;
    fn mul(self, rhs: Q8p24) -> Q8p24 {
        let temp = (self.0 as i64).saturating_mul(rhs.0 as i64);
        Q8p24((temp >> 24) as i32)
    }
}

impl Div for Q8p24 {
    type Output = Q8p24;
    fn div(self, rhs: Q8p24) -> Q8p24 {
        let temp = ((self.0 as i64) * FIXED_POINT as i64) / rhs.0 as i64;
        Q8p24(temp as i32)
    }
}

impl Debug for Q8p24 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let x: f64 = (*self).into();
        write!(f, "{:?}", x)
    }
}

impl From<f64> for Q8p24 {
    fn from(f: f64) -> Q8p24 {
        if f > 127. {
            MAX
        } else if f < -128. {
            MIN
        } else {
            let x = f * FIXED_POINT as f64;
            Q8p24(x.round() as i32)
        }
    }
}

impl From<f32> for Q8p24 {
    fn from(f: f32) -> Q8p24 {
        if f > 127. {
            MAX
        } else if f < -128. {
            MIN
        } else {
            let x = f * FIXED_POINT as f32;
            Q8p24(x.round() as i32)
        }
    }
}

impl From<i8> for Q8p24 {
    fn from(f: i8) -> Q8p24 {
        Q8p24((f as i32) << 24)
    }
}

impl From<Q8p24> for f64 {
    fn from(f: Q8p24) -> f64 {
        f.0 as f64 / FIXED_POINT as f64
    }
}

impl From<Q8p24> for f32 {
    fn from(f: Q8p24) -> f32 {
        f.0 as f32 / FIXED_POINT as f32
    }
}

impl From<Q8p24> for i8 {
    fn from(f: Q8p24) -> i8 {
        (f.0 >> 24) as i8
    }
}

#[cfg(test)]
mod test {
    use quickcheck::{QuickCheck, Testable};
    use super::Q8p24;

    fn quickcheck<A: Testable>(a: A) {
        QuickCheck::new().tests(100).quickcheck(a)
    }


    fn apx_eq(a: f64, b: f64) -> bool {
        let delta = a - b;
        delta < (1. / (1 << 18) as f64)
    }

    #[test]
    fn into() {
        fn into(x: f64) -> bool {
            if x > 127. || x < -127. {
                return true;
            }

            let y: Q8p24 = x.into();
            let z: f64 = y.into();
            apx_eq(x, z)
        }

        quickcheck(into as fn(f64) -> bool);

        for i in -127..128 {
            into(i as f64);
        }
    }

    #[test]
    fn add() {
        fn add(x: f64, y: f64) -> bool {
            let (xi, yi): (Q8p24, Q8p24) = (x.into(), y.into());
            let (x, y): (f64, f64) = (x.into(), y.into());
            let zy = xi + yi;
            let z = x + y;
            if z.abs() >= 127. {
                return true;
            }
            apx_eq(zy.into(), z)
        }

        quickcheck(add as fn(f64, f64) -> bool);
    }

    #[test]
    fn sub() {
        fn sub(x: f64, y: f64) -> bool {
            let (xi, yi): (Q8p24, Q8p24) = (x.into(), y.into());
            let zy = xi - yi;
            let z = x - y;
            if z.abs() >= 127. {
                return true;
            }
            apx_eq(zy.into(), z)
        }

        quickcheck(sub as fn(f64, f64) -> bool);
    }

    #[test]
    fn mul() {
        fn mul(x: f64, y: f64) -> bool {
            let (xi, yi): (Q8p24, Q8p24) = (x.into(), y.into());
            let zi = xi * yi;
            let z = x * y;
            if z.abs() >= 127. {
                return true;
            }
            apx_eq(zi.into(), z)
        }

        quickcheck(mul as fn(f64, f64) -> bool);
    }
}