use std::fmt;
use std::ops::Add;
use crate::base_types::Meter;

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub(crate) enum Distance {
    Distance(Meter),
    Infinity
}

// methods:
impl Distance {
    pub fn in_meter(&self) -> Meter {
        match self {
            Distance::Distance(d) => *d,
            Distance::Infinity => {panic!("Distance is infinity")},
        }
    }
}

// static functions:
impl Distance {
    pub fn from_meter(m: Meter) -> Distance {
        Distance::Distance(m)
    }

    pub fn from_km(km: f32) -> Distance {
        Distance::from_meter((km * 1000.0) as Meter)
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
            Distance::Distance(d) => {
                let m = d % 1000;
                let km = (d - m)/1000;
                write!(f, "{}.{:03}km", km, m)},
            Distance::Infinity => write!(f, "INF km"),
        }
    }
}
