use std::fmt;

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
}

impl fmt::Display for Distance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Distance::Distance(d) => write!(f, "{}km", d),
            Distance::Infinity => write!(f, "INF km"),
        }
    }
}
