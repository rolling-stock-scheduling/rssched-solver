pub mod local_improver;

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

pub trait LocalSearchable: Send + Sync + Clone + Ord {
    fn neighborhood(&self) -> Box<dyn Iterator<Item = Self> + Send + Sync>;
}

enum SearchResult<S> {
    Improvement(EvaluatedSolution<S>),
    NoImprovement(EvaluatedSolution<S>),
}

impl<S> SearchResult<S> {
    fn unwrap(self) -> EvaluatedSolution<S> {
        match self {
            SearchResult::Improvement(solution) => solution,
            SearchResult::NoImprovement(solution) => solution,
        }
    }

    fn as_ref(&self) -> &EvaluatedSolution<S> {
        match self {
            SearchResult::Improvement(solution) => solution,
            SearchResult::NoImprovement(solution) => solution,
        }
    }
}

use SearchResult::{Improvement, NoImprovement};

pub struct LocalSearch<S: LocalSearchable> {
    objective: Arc<Objective<S>>,
    initial_solution: S,
}

impl<S: LocalSearchable> LocalSearch<S> {
    pub fn initialize(initial_solution: S, objective: Arc<Objective<S>>) -> Self {
        Self {
            objective,
            initial_solution,
        }
    }
}

impl<S: LocalSearchable> Solver<S> for LocalSearch<S> {
    fn solve(&self) -> EvaluatedSolution<S> {
        let start_time = stdtime::Instant::now();
        // if there is no start schedule, create new schedule, where each vehicle has exactly one tour and the demand is covered.
        let init_solution = self.objective.evaluate(self.initial_solution.clone());

        let _minimizer = Minimizer::new(self.objective.clone());

        let recursion_depth = 0;
        let recursion_width = 5;

        let _take_first = TakeFirstRecursion::new(
            recursion_depth,
            Some(recursion_width),
            self.objective.clone(),
        );

        let _take_any = TakeAnyParallelRecursion::new(
            recursion_depth,
            Some(recursion_width),
            self.objective.clone(),
        );

        self.find_local_optimum(
            init_solution,
            // _minimizer.clone(),
            // _take_first.clone(),
            _take_any.clone(),
            true,
            Some(start_time),
        )
        .unwrap()
    }
}

impl<S: LocalSearchable> LocalSearch<S> {
    fn find_local_optimum(
        &self,
        start_solution: EvaluatedSolution<S>,
        local_improver: impl LocalImprover<S>,
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
