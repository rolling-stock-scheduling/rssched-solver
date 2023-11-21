/////////////////////// LEVEL ///////////////////////

use crate::{base_value::BaseValue, coefficient::Coefficient, indicator::Indicator};

/// A level of the objective hierarchy.
pub struct Level<S> {
    // valueType must be multiplyable with Coefficient
    summands: Vec<(Coefficient, Box<dyn Indicator<S>>)>,
}

impl<S> Level<S> {
    pub fn evaluate(&self, solution: &S) -> BaseValue {
        self.summands
            .iter()
            .map(|(coefficient, indicator)| coefficient * indicator.evaluate(solution))
            .sum()
    }

    pub fn new(summands: Vec<(Coefficient, Box<dyn Indicator<S>>)>) -> Level<S> {
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
