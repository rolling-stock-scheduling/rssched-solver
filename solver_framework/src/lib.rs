use objective_framework::{EvaluatedSolution, Objective};
use std::sync::Arc;

pub trait Solver<I, S> {
    fn initialize(instance: I, objective: Arc<Objective<S>>) -> Self;

    fn solve(&self) -> EvaluatedSolution<S>;
}
