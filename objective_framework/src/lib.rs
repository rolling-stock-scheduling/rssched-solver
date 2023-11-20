use std::{
    cmp::Ordering,
    fmt,
    iter::Sum,
    ops::{Add, Mul, Sub},
};

use time::Duration;

/// the hierarchical objective value of a schedule
#[derive(Clone)]
pub struct ObjectiveValue {
    objective_vector: Vec<ObjBaseValue>,
}

impl ObjectiveValue {
    fn new(objective_vector: Vec<ObjBaseValue>) -> ObjectiveValue {
        ObjectiveValue { objective_vector }
    }
}

impl Ord for ObjectiveValue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.objective_vector
            .iter()
            .zip(other.objective_vector.iter())
            .fold(Ordering::Equal, |acc, (value, other_value)| {
                acc.then_with(|| value.partial_cmp(other_value).unwrap())
            })
    }
}

impl PartialOrd for ObjectiveValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ObjectiveValue {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Eq for ObjectiveValue {}

/// Defines the values of a schedule that form the objective.
/// It is static throughout optimization.
/// It is a hierarchical objective, i.e., it consists of several levels.
/// Each level consists of a linear combination of indicators.
///
/// The objective is to be minimized with the most important level being the first entry of the
/// vector.
pub struct Objective<T> {
    hierarchy_levels: Vec<Level<T>>, // TODO: some level might not be suitable for ObjectiveTourValue, need flag if it is tour-based or not
}

impl<T> Objective<T> {
    pub fn evaluate(&self, solution: &T) -> ObjectiveValue {
        let objective_value_hierarchy: Vec<ObjBaseValue> = self
            .hierarchy_levels
            .iter()
            .map(|level| level.evaluate(solution))
            .collect();

        ObjectiveValue::new(objective_value_hierarchy)
    }

    pub fn new(hierarchy_levels: Vec<Level<T>>) -> Objective<T> {
        Objective { hierarchy_levels }
    }

    pub fn print_objective_value(&self, objective_value: &ObjectiveValue) {
        for (level, value) in self
            .hierarchy_levels
            .iter()
            .zip(objective_value.objective_vector.iter())
        {
            println!("{}: {}", level.to_string(), value);
        }
    }

    pub fn print_objective_value_with_comparison(
        &self,
        objective_value: &ObjectiveValue,
        comparison: &ObjectiveValue,
    ) {
        for ((level, value), comparison_value) in self
            .hierarchy_levels
            .iter()
            .zip(objective_value.objective_vector.iter())
            .zip(comparison.objective_vector.iter())
        {
            println!(
                "{}: {} {}",
                level.to_string(),
                value,
                value.print_difference(*comparison_value)
            );
        }
    }
}

/////////////////////// LEVEL ///////////////////////

/// A level of the objective hierarchy.
struct Level<T> {
    // valueType must be multiplyable with Coefficient
    summands: Vec<(ObjCoefficient, Box<dyn Indicator<T>>)>,
}

impl<T> Level<T> {
    pub fn evaluate(&self, solution: &T) -> ObjBaseValue {
        self.summands
            .iter()
            .map(|(coefficient, indicator)| coefficient * indicator.evaluate(solution))
            .sum()
    }

    pub fn new(summands: Vec<(ObjCoefficient, Box<dyn Indicator<T>>)>) -> Level<T> {
        Level { summands }
    }

    pub fn to_string(&self) -> String {
        self.summands
            .iter()
            .map(|(coefficient, indicator)| {
                if coefficient.is_one() {
                    format!("{}", indicator.name())
                } else {
                    format!("{}*{}", coefficient, indicator.name())
                }
            })
            .collect::<Vec<String>>()
            .join(" + ")
    }
}

/// An atomic aspect of the solution.
/// An indicator could be the aspect "number of dummy_tours" or "total deadhead distance", ...
trait Indicator<T> {
    fn evaluate(&self, solution: &T) -> ObjBaseValue;
    fn name(&self) -> String;
}

/// A single value of an indicator. E.g., count of things, durations, costs
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum ObjBaseValue {
    Count(i32),         // cannot handle negative values
    Duration(Duration), // cannot handle negative values
    Costs(f32),
    Maximum,
    Zero,
}

impl ObjBaseValue {
    pub fn print_difference(self, other: ObjBaseValue) -> String {
        if self == other {
            return format!("");
        }
        match (self, other) {
            (ObjBaseValue::Count(a), ObjBaseValue::Count(b)) => {
                ObjBaseValue::print_difference_in_value(a, b)
            }
            (ObjBaseValue::Duration(a), ObjBaseValue::Duration(b)) => {
                ObjBaseValue::print_difference_in_value(a, b)
            }
            (ObjBaseValue::Costs(a), ObjBaseValue::Costs(b)) => {
                ObjBaseValue::print_difference_in_value(a, b)
            }
            (ObjBaseValue::Maximum, _) => format!(""),
            (_, ObjBaseValue::Maximum) => format!(""),
            (new_value, ObjBaseValue::Zero) => format!("(\x1b[0;31m+{:2.1}\x1b[0m)", new_value),
            (ObjBaseValue::Zero, old_value) => format!("(\x1b[0;32m-{:2.1}\x1b[0m)", old_value),
            _ => panic!("Cannot subtract {:?} and {:?}", self, other),
        }
    }

    /// method for printing the difference between two values in green or red depending on the sign
    fn print_difference_in_value<T>(value: T, value_for_comparison: T) -> String
    where
        T: fmt::Display + PartialOrd + Sub,
        <T as Sub>::Output: fmt::Display,
    {
        if value > value_for_comparison {
            format!("(\x1b[0;31m+{:2.1}\x1b[0m)", value - value_for_comparison)
        } else if value < value_for_comparison {
            format!("(\x1b[0;32m-{:2.1}\x1b[0m)", value_for_comparison - value)
        } else {
            format!("")
        }
    }
}

impl Add for ObjBaseValue {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (ObjBaseValue::Count(a), ObjBaseValue::Count(b)) => ObjBaseValue::Count(a + b),
            (ObjBaseValue::Duration(a), ObjBaseValue::Duration(b)) => ObjBaseValue::Duration(a + b),
            (ObjBaseValue::Costs(a), ObjBaseValue::Costs(b)) => ObjBaseValue::Costs(a + b),
            (ObjBaseValue::Maximum, _) => ObjBaseValue::Maximum,
            (_, ObjBaseValue::Maximum) => ObjBaseValue::Maximum,
            (ObjBaseValue::Zero, value) => value,
            (value, ObjBaseValue::Zero) => value,
            _ => panic!("Cannot add {:?} and {:?}", self, other),
        }
    }
}

impl Sub for ObjBaseValue {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        match (self, other) {
            (ObjBaseValue::Count(a), ObjBaseValue::Count(b)) => ObjBaseValue::Count(a - b),
            (ObjBaseValue::Duration(a), ObjBaseValue::Duration(b)) => ObjBaseValue::Duration(a - b),
            (ObjBaseValue::Costs(a), ObjBaseValue::Costs(b)) => ObjBaseValue::Costs(a - b),
            (ObjBaseValue::Maximum, _) => ObjBaseValue::Maximum,
            (value, ObjBaseValue::Zero) => value,
            (ObjBaseValue::Zero, ObjBaseValue::Count(a)) => ObjBaseValue::Count(-a),
            (ObjBaseValue::Zero, ObjBaseValue::Costs(a)) => ObjBaseValue::Costs(-a),
            _ => panic!("Cannot sub {:?} and {:?}", self, other),
        }
    }
}

impl Sum<Self> for ObjBaseValue {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(ObjBaseValue::Zero, |a, b| a + b)
    }
}

impl fmt::Display for ObjBaseValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjBaseValue::Count(i) => write!(f, "{}", i),
            ObjBaseValue::Duration(d) => write!(f, "{}", d),
            ObjBaseValue::Costs(c) => write!(f, "{}", c),
            ObjBaseValue::Maximum => write!(f, "MAX"),
            ObjBaseValue::Zero => write!(f, "0"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum ObjCoefficient {
    Integer(i32),
    Float(f32),
}

impl ObjCoefficient {
    pub fn is_one(&self) -> bool {
        match self {
            ObjCoefficient::Integer(i) => *i == 1,
            ObjCoefficient::Float(f) => *f == 1.0,
        }
    }
}

impl Mul<ObjBaseValue> for ObjCoefficient {
    type Output = ObjBaseValue;

    fn mul(self, other: ObjBaseValue) -> ObjBaseValue {
        match self {
            ObjCoefficient::Integer(i) => match other {
                ObjBaseValue::Count(c) => ObjBaseValue::Count(i * c),
                ObjBaseValue::Duration(d) => {
                    ObjBaseValue::Duration(Duration::from_seconds(i as u32 * d.in_sec()))
                }
                ObjBaseValue::Costs(c) => ObjBaseValue::Costs(i as f32 * c),
                ObjBaseValue::Maximum => ObjBaseValue::Maximum,
                ObjBaseValue::Zero => ObjBaseValue::Zero,
            },
            ObjCoefficient::Float(f) => match other {
                ObjBaseValue::Count(c) => ObjBaseValue::Count((f * c as f32) as i32),
                ObjBaseValue::Duration(d) => {
                    ObjBaseValue::Duration(Duration::from_seconds((f * d.in_sec() as f32) as u32))
                }
                ObjBaseValue::Costs(c) => ObjBaseValue::Costs(f * c),
                ObjBaseValue::Maximum => ObjBaseValue::Maximum,
                ObjBaseValue::Zero => ObjBaseValue::Zero,
            },
        }
    }
}

// impl Mul<ObjBaseValue> for &ObjCoefficient, therefore we can use '*' even for references.
impl Mul<ObjBaseValue> for &ObjCoefficient {
    type Output = ObjBaseValue;
    fn mul(self, other: ObjBaseValue) -> ObjBaseValue {
        (*self).mul(other)
    }
}

impl fmt::Display for ObjCoefficient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjCoefficient::Integer(i) => write!(f, "{}", i),
            ObjCoefficient::Float(fl) => write!(f, "{}", fl),
        }
    }
}
