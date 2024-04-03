use super::LocalImprover;
use crate::local_search::Neighborhood;
use objective_framework::EvaluatedSolution;
use objective_framework::Objective;
use std::sync::Arc;

pub struct Minimizer<S: Send + Sync + Clone + Ord> {
    neighborhood: Arc<dyn Neighborhood<S>>,
    objective: Arc<Objective<S>>,
}

impl<S: Send + Sync + Clone + Ord> Minimizer<S> {
    pub fn new(
        neighborhood: Arc<dyn Neighborhood<S>>,
        objective: Arc<Objective<S>>,
    ) -> Minimizer<S> {
        Minimizer {
            neighborhood,
            objective,
        }
    }
}

impl<S: Send + Sync + Clone + Ord> LocalImprover<S> for Minimizer<S> {
    fn improve(&self, solution: &EvaluatedSolution<S>) -> Option<EvaluatedSolution<S>> {
        let best_neighbor_opt = self
            .neighborhood
            .neighbors_of(solution.solution())
            .map(|neighbor| self.objective.evaluate(neighbor))
            .min_by(|s1, s2| {
                s1.objective_value()
                    .partial_cmp(s2.objective_value())
                    .unwrap()
            });
        match best_neighbor_opt {
            Some(best_neighbor) => {
                if best_neighbor.objective_value() < solution.objective_value() {
                    Some(best_neighbor)
                } else {
                    None // no improvement found
                }
            }
            None => {
                println!("\x1b[31mWARNING: NO SWAP POSSIBLE.\x1b[0m");
                None
            }
        }
    }
}
