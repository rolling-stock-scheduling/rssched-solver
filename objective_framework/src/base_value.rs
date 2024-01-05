use std::{
    fmt,
    iter::Sum,
    ops::{Add, Sub},
};

use time::Duration;

/// A single value of an indicator. E.g., count of things, durations, costs
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum BaseValue {
    Integer(i64), // cannot handle negative values
    Float(f64),
    Duration(Duration), // cannot handle negative values
    Maximum,
    Zero,
}

impl BaseValue {
    pub fn print_difference(self, other: BaseValue) -> String {
        if self == other {
            return String::new();
        }
        match (self, other) {
            (BaseValue::Integer(a), BaseValue::Integer(b)) => {
                BaseValue::print_difference_in_value(a, b)
            }
            (BaseValue::Float(a), BaseValue::Float(b)) => {
                BaseValue::print_difference_in_value(a, b)
            }
            (BaseValue::Duration(a), BaseValue::Duration(b)) => {
                BaseValue::print_difference_in_value(a, b)
            }
            (BaseValue::Maximum, _) => String::new(),
            (_, BaseValue::Maximum) => String::new(),
            (new_value, BaseValue::Zero) => format!("(\x1b[0;31m+{:2.1}\x1b[0m)", new_value),
            (BaseValue::Zero, old_value) => format!("(\x1b[0;32m-{:2.1}\x1b[0m)", old_value),
            _ => panic!("Cannot subtract {:?} and {:?}", self, other),
        }
    }

    /// method for printing the difference between two values in green or red depending on the sign
    fn print_difference_in_value<V>(value: V, value_for_comparison: V) -> String
    where
        V: fmt::Display + PartialOrd + Sub,
        <V as Sub>::Output: fmt::Display,
    {
        if value > value_for_comparison {
            format!("(\x1b[0;31m+{:2.1}\x1b[0m)", value - value_for_comparison)
        } else if value < value_for_comparison {
            format!("(\x1b[0;32m-{:2.1}\x1b[0m)", value_for_comparison - value)
        } else {
            String::new()
        }
    }
}

impl Add for BaseValue {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (BaseValue::Integer(a), BaseValue::Integer(b)) => BaseValue::Integer(a + b),
            (BaseValue::Float(a), BaseValue::Float(b)) => BaseValue::Float(a + b),
            (BaseValue::Duration(a), BaseValue::Duration(b)) => BaseValue::Duration(a + b),
            (BaseValue::Maximum, _) => BaseValue::Maximum,
            (_, BaseValue::Maximum) => BaseValue::Maximum,
            (BaseValue::Zero, value) => value,
            (value, BaseValue::Zero) => value,
            _ => panic!("Cannot add {:?} and {:?}", self, other),
        }
    }
}

impl Sub for BaseValue {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        match (self, other) {
            (BaseValue::Integer(a), BaseValue::Integer(b)) => BaseValue::Integer(a - b),
            (BaseValue::Float(a), BaseValue::Float(b)) => BaseValue::Float(a - b),
            (BaseValue::Duration(a), BaseValue::Duration(b)) => BaseValue::Duration(a - b),
            (BaseValue::Maximum, _) => BaseValue::Maximum,
            (value, BaseValue::Zero) => value,
            (BaseValue::Zero, BaseValue::Integer(a)) => BaseValue::Integer(-a),
            (BaseValue::Zero, BaseValue::Float(a)) => BaseValue::Float(-a),
            _ => panic!("Cannot sub {:?} and {:?}", self, other),
        }
    }
}

impl Sum<Self> for BaseValue {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(BaseValue::Zero, |a, b| a + b)
    }
}

impl fmt::Display for BaseValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BaseValue::Integer(i) => write!(f, "{}", i),
            BaseValue::Float(c) => write!(f, "{}", c),
            BaseValue::Duration(d) => write!(f, "{}", d),
            BaseValue::Maximum => write!(f, "MAX"),
            BaseValue::Zero => write!(f, "0"),
        }
    }
}
