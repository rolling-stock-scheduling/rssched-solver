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

pub trait Neighborhood<S: Send + Sync + Clone + Ord>: Send + Sync {
    fn neighbors_of(&self, current_solution: &S) -> Box<dyn Iterator<Item = S> + Send + Sync>;
}

pub struct LocalSearch<S> {
    initial_solution: S,
    neighborhood: Arc<dyn Neighborhood<S>>,
    objective: Arc<Objective<S>>,
    local_improver: Option<Box<dyn LocalImprover<S>>>,
}

impl<S: Send + Sync + Clone + Ord> LocalSearch<S> {
    pub fn initialize(
        initial_solution: S,
        neighborhood: Arc<dyn Neighborhood<S>>,
        objective: Arc<Objective<S>>,
    ) -> Self {
        Self {
            initial_solution,
            neighborhood,
            objective,
            local_improver: None,
        }
    }

    /// This method is used to set the local improver to be used in the local search.
    /// They can be found in the local_improver module.
    pub fn with_local_improver(
        initial_solution: S,
        neighborhood: Arc<dyn Neighborhood<S>>,
        objective: Arc<Objective<S>>,
        local_improver: Box<dyn LocalImprover<S>>,
    ) -> Self {
        Self {
            initial_solution,
            neighborhood,
            objective,
            local_improver: Some(local_improver),
        }
    }
}

impl<S: Send + Sync + Clone + Ord> Solver<S> for LocalSearch<S> {
    fn solve(&self) -> EvaluatedSolution<S> {
        let start_time = stdtime::Instant::now();
        let init_solution = self.objective.evaluate(self.initial_solution.clone());

        // default local improver is TakeAnyParallelRecursion without recursion
        let take_any: Box<dyn LocalImprover<S>> = Box::new(TakeAnyParallelRecursion::new(
            0,
            Some(0),
            self.neighborhood.clone(),
            self.objective.clone(),
        ));

        self.find_local_optimum(
            init_solution,
            self.local_improver.as_ref().unwrap_or(&take_any).as_ref(),
            true,
            Some(start_time),
        )
        .unwrap()
    }
}

impl<S: Send + Sync + Clone + Ord> LocalSearch<S> {
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
