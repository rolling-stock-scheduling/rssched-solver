use std::{cmp::Ordering, slice::Iter};

use crate::base_value::BaseValue;

/// the hierarchical objective value of a schedule
#[derive(Clone)]
pub struct ObjectiveValue {
    objective_vector: Vec<BaseValue>,
}

impl ObjectiveValue {
    pub(super) fn new(objective_vector: Vec<BaseValue>) -> ObjectiveValue {
        ObjectiveValue { objective_vector }
    }

    pub(super) fn iter(&self) -> Iter<BaseValue> {
        self.objective_vector.iter()
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
