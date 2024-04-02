use solution::Schedule;
use solver_framework::local_search::LocalSearch;

/*pub type Solution = EvaluatedSolution<Schedule>;


let segment_limit = Duration::new("3:00:00");
let overhead_threshold = Duration::new("0:05:00"); // tours of real-vehicle-providers are not splitted at nodes under these duration
let swap_factory = LimitedExchanges::new(
    Some(segment_limit),
    // None,
    Some(overhead_threshold),
    // None,
    false,
    self.network.clone(),
); */

type RollingStockLocalSearch = LocalSearch<Schedule>;
