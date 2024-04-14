use crate::base_types::Meter;
use std::fmt;
use std::ops::{Add, Sub};

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug)]
pub enum Distance {
    Distance(Meter),
    Infinity,
}

// methods:
impl Distance {
    pub fn in_meter(&self) -> Result<Meter, &str> {
        match self {
            Distance::Distance(d) => Ok(*d),
            Distance::Infinity => Err("Distance is infinity"),
        }
    }

    /// Returns max{self-other, 0}
    pub fn sub_max_zero(self, other: Distance) -> Distance {
        match self {
            Distance::Infinity => Distance::Infinity,
            Distance::Distance(d1) => match other {
                Distance::Infinity => Distance::ZERO,
                Distance::Distance(d2) => {
                    if d1 < d2 {
                        Distance::ZERO
                    } else {
                        Distance::Distance(d1 - d2)
                    }
                }
            },
        }
    }
}

// static functions:
impl Distance {
    pub const ZERO: Distance = Distance::Distance(0);

    pub fn from_meter(m: Meter) -> Distance {
        Distance::Distance(m)
    }

    pub fn from_km(km: f32) -> Distance {
        Distance::from_meter((km * 1000.0) as Meter)
    }

    pub fn from_km_str(km_string: &str) -> Distance {
        // could be an int string ("1000") or a float string ("1000.0")
        let km: f32 = if km_string.contains('.') {
            km_string.parse().unwrap()
        } else {
            km_string.parse::<i32>().unwrap() as f32
        };
        Distance::from_km(km)
    }
}

impl Add for Distance {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match self {
            Distance::Infinity => Distance::Infinity,
            Distance::Distance(d1) => match other {
                Distance::Infinity => Distance::Infinity,
                Distance::Distance(d2) => Distance::Distance(d1 + d2),
            },
        }
    }
}

impl Sub for Distance {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        match self {
            Distance::Infinity => Distance::Infinity,
            Distance::Distance(d1) => match other {
                Distance::Infinity => panic!("Cannot subtract Distance::Infinity"),
                Distance::Distance(d2) => {
                    assert!(d1 >= d2, "Cannot subtract {} from {}", d2, d1);
                    Distance::Distance(d1 - d2)
                }
            },
        }
    }
}

impl std::iter::Sum<Self> for Distance {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(Distance::ZERO, |a, b| a + b)
    }
}

impl fmt::Display for Distance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Distance::Distance(d) => {
                let m = d % 1000;
                let km = (d - m) / 1000;
                write!(f, "{}.{:03}km", km, m)
            }
            Distance::Infinity => write!(f, "INF km"),
        }
    }
}
