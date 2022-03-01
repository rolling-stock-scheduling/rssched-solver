
use crate::distance::Distance;
use crate::time::Duration;
use crate::base_types::Penalty;


/// objective value of schedule (to be minimized)
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub(crate) struct Objective {
    overhead_time: Duration, // idle_time + dead_head_time for the tours only (its minimal if all service trips are covered
    number_of_dummy_units : usize,
    dummyer_overhead_time: Duration, // idle_time + dead_head_time of dummy tours.
    coupling_conflicts: u32,
    maintenance_violation: f32, // linear combination of duration violation / distance violaten
    unsatisfied_recommendation: u32, // number of unsatisfied recommended activity links (given by the reference plan)
    dead_head_distance: Distance // total dead_head_distance traveled
}

