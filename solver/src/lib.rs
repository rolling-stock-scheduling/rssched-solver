pub mod local_search;
pub mod min_cost_flow_solver;
pub mod objective;
pub mod one_node_per_tour;

use objective_framework::{EvaluatedSolution, Objective};
use std::sync::Arc;

pub trait Solver<I, S> {
    fn initialize(instance: I, objective: Arc<Objective<S>>) -> Self;

    fn solve(&self) -> EvaluatedSolution<S>;
}
