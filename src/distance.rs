use std::fmt;
use std::ops::Add;

#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub(crate) enum Distance {
    Distance(u64),
    Infinity
}

// methods:
impl Distance {
    pub fn as_km(&self) -> u64 {
        match self {
            Distance::Distance(d) => *d,
            Distance::Infinity => {panic!("Distance is infinity")},
        }
    }
}

// static functions:
impl Distance {
    pub fn from_km(d: u64) -> Distance {
        Distance::Distance(d)
    }

    pub fn zero() -> Distance {
        Distance::Distance(0)
    }
}

impl Add for Distance {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match self {
            Distance::Infinity => Distance::Infinity,
            Distance::Distance(d1) =>
                match other {
                    Distance::Infinity => Distance::Infinity,
                    Distance::Distance(d2) =>
                        Distance::Distance(d1 + d2)
                }
        }
    }
}

impl std::iter::Sum<Self> for Distance {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(Distance::zero(), |a, b| a + b)
    }
}


impl fmt::Display for Distance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Distance::Distance(d) => write!(f, "{}km", d),
            Distance::Infinity => write!(f, "INF km"),
        }
    }
}
