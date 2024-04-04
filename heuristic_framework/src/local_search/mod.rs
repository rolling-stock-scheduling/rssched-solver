pub mod local_improver;
mod search_result;

use std::sync::Arc;
use std::time as stdtime;

use crate::Solver;
use local_improver::LocalImprover;
use objective_framework::EvaluatedSolution;
use objective_framework::Objective;

#[allow(unused_imports)]
use local_improver::Minimizer;

#[allow(unused_imports)]
use local_improver::TakeFirstRecursion;

#[allow(unused_imports)]
use local_improver::TakeAnyParallelRecursion;

use search_result::SearchResult;
use search_result::SearchResult::{Improvement, NoImprovement};

/// A local search neighborhood that provides for each solution a iterator over all neighbors.
/// The provides solution, as well as the Neighborhood instance must live as long as the iterator.
/// (Note that the iterator highly depends on the current_solution and that the Neighborhood may
/// have some attributes which goes into the iterator.)
pub trait Neighborhood<S>: Send + Sync {
    fn neighbors_of<'a>(
        &'a self,
        current_solution: &'a S,
    ) -> Box<dyn Iterator<Item = S> + Send + Sync + 'a>;
}

pub struct LocalSearchSolver<S> {
    neighborhood: Arc<dyn Neighborhood<S>>,
    objective: Arc<Objective<S>>,
    local_improver: Option<Box<dyn LocalImprover<S>>>,
}

impl<S> LocalSearchSolver<S> {
    pub fn initialize(
        neighborhood: Arc<dyn Neighborhood<S>>,
        objective: Arc<Objective<S>>,
    ) -> Self {
        Self {
            neighborhood,
            objective,
            local_improver: None,
        }
    }

    /// This method is used to set the local improver to be used in the local search.
    /// They can be found in the local_improver module.
    pub fn with_local_improver(
        neighborhood: Arc<dyn Neighborhood<S>>,
        objective: Arc<Objective<S>>,
        local_improver: Box<dyn LocalImprover<S>>,
    ) -> Self {
        Self {
            neighborhood,
            objective,
            local_improver: Some(local_improver),
        }
    }
}

impl<S> Solver<S> for LocalSearchSolver<S> {
    fn solve(&self, initial_solution: S) -> EvaluatedSolution<S> {
        let start_time = stdtime::Instant::now();
        let init_solution = self.objective.evaluate(initial_solution);

        // default local improver is Minimizer
        let minimizer: Box<dyn LocalImprover<S>> = Box::new(Minimizer::new(
            self.neighborhood.clone(),
            self.objective.clone(),
        ));

        self.find_local_optimum(
            init_solution,
            self.local_improver.as_ref().unwrap_or(&minimizer).as_ref(),
            true,
            Some(start_time),
        )
        .unwrap()
    }
}

impl<S> LocalSearchSolver<S> {
    fn find_local_optimum(
        &self,
        start_solution: EvaluatedSolution<S>,
        local_improver: &dyn LocalImprover<S>,
        verbose: bool,
        start_time: Option<stdtime::Instant>,
    ) -> SearchResult<S> {
        let mut result = NoImprovement(start_solution);
        while let Some(new_solution) = local_improver.improve(result.as_ref()) {
            if verbose {
                self.objective.print_objective_value_with_comparison(
                    new_solution.objective_value(),
                    result.as_ref().objective_value(),
                );
                if let Some(start_time) = start_time {
                    println!(
                        "elapsed time for local search: {:0.2}sec",
                        stdtime::Instant::now()
                            .duration_since(start_time)
                            .as_secs_f32()
                    );
                }
                println!();
            }
            result = Improvement(new_solution);
        }
        result
    }
}
