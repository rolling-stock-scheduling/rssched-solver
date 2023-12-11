use std::cmp::Ordering;

use crate::ObjectiveValue;

#[derive(Clone)]
pub struct EvaluatedSolution<S> {
    solution: S,
    objective_value: ObjectiveValue,
}

impl<S> EvaluatedSolution<S> {
    pub fn new(solution: S, objective_value: ObjectiveValue) -> EvaluatedSolution<S> {
        EvaluatedSolution {
            solution,
            objective_value,
        }
    }

    pub fn solution(&self) -> &S {
        &self.solution
    }

    pub fn objective_value(&self) -> &ObjectiveValue {
        &self.objective_value
    }
}

impl<S> Ord for EvaluatedSolution<S> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.objective_value.cmp(&other.objective_value)
    }
}

impl<S> PartialOrd for EvaluatedSolution<S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<S> PartialEq for EvaluatedSolution<S> {
    fn eq(&self, other: &Self) -> bool {
        self.objective_value == other.objective_value
    }
}

impl<S> Eq for EvaluatedSolution<S> {}
