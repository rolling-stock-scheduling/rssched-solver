pub mod local_search;

use objective_framework::EvaluatedSolution;

pub trait Solver<S> {
    fn solve(&self) -> EvaluatedSolution<S>;
}
