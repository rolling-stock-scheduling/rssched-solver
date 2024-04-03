use heuristic_framework::local_search::LocalSearchable;

use crate::Schedule;

mod swap_factory;
mod swaps;

impl LocalSearchable for Schedule {
    fn neighborhood(&self) -> Box<dyn Iterator<Item = Self> + Send + Sync> {
        Box::new(vec![].into_iter())
    }
}
