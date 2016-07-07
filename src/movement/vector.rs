use std::ops::*;
use super::q8p24::Q8p24;

#[derive(Copy, Clone, Debug)]
pub struct Vector{
    pub x: Q8p24,
    pub y: Q8p24
}

impl Vector {
    /// Create a new vecotr
    pub fn new(x: Q8p24, y: Q8p24) -> Vector {
        Vector {
            x: x,
            y: y,
        }
    }

    /// truncate any of the values below the decimal point
    pub fn trunc(self) -> Vector {
        Vector {
            x: self.x.trunc().into(),
            y: self.y.trunc().into()
        }
    }

    /// give only the points beyond the decimal point
    pub fn fract(self) -> Vector {
        Vector {
            x: self.x.fract(),
            y: self.y.fract(),
        }
    }
}

impl Add for Vector {
    type Output = Vector;
    fn add(self, rhs: Vector) -> Vector {
        Vector{
            x: self.x + rhs.x,
            y: self.y + rhs.y
        }
    }
}

impl Sub for Vector {
    type Output = Vector;
    fn sub(self, rhs: Vector) -> Vector {
        Vector{
            x: self.x - rhs.x,
            y: self.y - rhs.y
        }
    }
}

impl<T> From<(T, T)> for Vector
    where Q8p24: From<T>
{
    fn from((x, y): (T, T)) -> Vector {
        Vector::new(x.into(), y.into())
    }
}
