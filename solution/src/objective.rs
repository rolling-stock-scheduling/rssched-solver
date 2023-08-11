// visible from outside:

use std::{
    cmp::Ordering,
    fmt,
    iter::Sum,
    ops::{Add, Mul, Sub},
};

use sbb_model::base_types::{Duration, VehicleId};

use crate::schedule::{tour::Tour, Schedule};

struct ObjectiveValue {
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

struct ObjectiveTourValue {
    objective_vector: Vec<ObjBaseValue>,
}

impl ObjectiveTourValue {
    fn new(objective_vector: Vec<ObjBaseValue>) -> ObjectiveTourValue {
        ObjectiveTourValue { objective_vector }
    }
    // compare
}
impl Ord for ObjectiveTourValue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.objective_vector
            .iter()
            .zip(other.objective_vector.iter())
            .fold(Ordering::Equal, |acc, (value, other_value)| {
                acc.then_with(|| value.partial_cmp(other_value).unwrap())
            })
    }
}

impl PartialOrd for ObjectiveTourValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ObjectiveTourValue {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Eq for ObjectiveTourValue {}

mod objective_factory {
    use sbb_model::base_types::VehicleId;

    use crate::schedule::{tour::Tour, Schedule};

    use super::{Indicator, Level, ObjBaseValue, ObjCoefficient, Objective};

    fn build_basic_objective() -> Objective {
        let number_of_dummy_tours = NumberOfDummiesIndicator::new();
        let first_level = Level::new(vec![(
            ObjCoefficient::Integer(1),
            Box::new(number_of_dummy_tours),
        )]);

        Objective::new(vec![first_level])
    }

    struct NumberOfDummiesIndicator {}

    impl NumberOfDummiesIndicator {
        fn new() -> NumberOfDummiesIndicator {
            NumberOfDummiesIndicator {}
        }
    }

    impl Indicator for NumberOfDummiesIndicator {
        fn compute_from_schedule(&self, schedule: &Schedule) -> ObjBaseValue {
            ObjBaseValue::Count(schedule.number_of_dummy_tours() as i32)
        }

        fn compute_from_tour(&self, tour: &Tour) -> ObjBaseValue {
            ObjBaseValue::Count(if tour.is_dummy() { 1 } else { 0 })
        }

        fn compute_from_tour_exchanges(
            &self,
            schedule: &Schedule,
            changes: Vec<(VehicleId, &Tour)>,
        ) -> ObjBaseValue {
            let mut count = schedule.number_of_dummy_tours() as i32;
            for (vehicle, tour) in changes {
                if schedule.is_dummy(vehicle) {
                    count += 1;
                }
                if tour.is_dummy() {
                    count -= 1;
                }
            }
            ObjBaseValue::Count(count)
        }

        fn name(&self) -> String {
            String::from("number of dummies")
        }
    }
}

/// Defines the values of a schedule that form the objective.
/// It is static throughout optimization.
/// It is a hierarchical objective, i.e., it consists of several levels.
/// Each level consists of a linear combination of indicators.
///
/// The objective is to be minimized with the most important level being the first entry of the
/// vector.
struct Objective {
    hierarchy_levels: Vec<Level>, // TODO: some level might not be suitable for ObjectiveTourValue, need flag if it is tour-based or not
}

impl Objective {
    pub fn compute_objective_value(&self, schedule: &Schedule) -> ObjectiveValue {
        let objective_value_hierarchy: Vec<ObjBaseValue> = self
            .hierarchy_levels
            .iter()
            .map(|level| level.compute_value(schedule))
            .collect();

        ObjectiveValue::new(objective_value_hierarchy)
    }
    pub fn compute_tour_objective_value(&self, tour: &Tour) -> ObjectiveTourValue {
        let objective_value_hierarchy: Vec<ObjBaseValue> = self
            .hierarchy_levels
            .iter()
            .map(|level| level.compute_tour_value(tour))
            .collect();

        ObjectiveTourValue::new(objective_value_hierarchy)
    }
    pub fn compute_from_tour_exchanges(
        &self,
        schedule: &Schedule,
        changes: Vec<(VehicleId, &Tour)>,
    ) -> ObjectiveValue {
        let objective_value_hierarchy: Vec<ObjBaseValue> = self
            .hierarchy_levels
            .iter()
            .map(|level| level.compute_value_from_tour_exchanges(schedule, changes))
            .collect();

        ObjectiveValue::new(objective_value_hierarchy)
    }

    pub fn new(hierarchy_levels: Vec<Level>) -> Objective {
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

struct Level {
    // valueType must be multiplyable with Coefficient
    summands: Vec<(ObjCoefficient, Box<dyn Indicator>)>,
}

impl Level {
    pub fn compute_value(&self, schedule: &Schedule) -> ObjBaseValue {
        self.summands
            .iter()
            .map(|(coefficient, indicator)| {
                *coefficient * indicator.compute_from_schedule(schedule)
            })
            .sum()
    }

    pub fn compute_tour_value(&self, tour: &Tour) -> ObjBaseValue {
        self.summands
            .iter()
            .map(|(coefficient, indicator)| *coefficient * indicator.compute_from_tour(tour))
            .sum()
    }

    pub fn compute_value_from_tour_exchanges(
        &self,
        schedule: &Schedule,
        changes: Vec<(VehicleId, &Tour)>,
    ) -> ObjBaseValue {
        self.summands
            .iter()
            .map(|(coefficient, indicator)| {
                *coefficient * indicator.compute_from_tour_exchanges(schedule, changes)
            })
            .sum()
    }

    pub fn new(summands: Vec<(ObjCoefficient, Box<dyn Indicator>)>) -> Level {
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

/// e.g., number of dummy_tours, total deadhead distance, ...
trait Indicator {
    fn compute_from_schedule(&self, schedule: &Schedule) -> ObjBaseValue;
    fn compute_from_tour(&self, tour: &Tour) -> ObjBaseValue;
    fn compute_from_tour_exchanges(
        &self,
        schedule: &Schedule,
        changes: Vec<(VehicleId, &Tour)>,
    ) -> ObjBaseValue;
    fn name(&self) -> String;
}

/// e.g., a count of things, duration, costs
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum ObjBaseValue {
    Count(i32),
    Duration(Duration),
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

impl fmt::Display for ObjCoefficient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjCoefficient::Integer(i) => write!(f, "{}", i),
            ObjCoefficient::Float(fl) => write!(f, "{}", fl),
        }
    }
}
