
use crate::distance::Distance;
use crate::time::Duration;

/// objective value of schedule (to be minimized)
#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub(crate) struct ObjectiveValue {
    overhead_time: Duration, // idle_time + dead_head_time for the tours only (its minimal if all service trips are covered
    number_of_dummy_units : usize,
    dummy_overhead_time: Duration, // idle_time + dead_head_time of dummy tours.
    // coupling_conflicts: u32,
    maintenance_distance_violation: Distance, // linear combination of duration violation / distance violaten
    maintenance_duration_violation: Duration, // linear combination of duration violation / distance violaten
    // maintenance_sum:
    // unsatisfied_recommendation: u32, // number of unsatisfied recommended activity links (given by the reference plan)
    dead_head_distance: Distance // total dead_head_distance traveled
}

impl ObjectiveValue {
    pub fn print(&self) {
        println!("* overhead_time: {}", self.overhead_time);
        println!("* number_of_dummy_units: {}", self.number_of_dummy_units);
        println!("* dummy_overhead_time: {}", self.dummy_overhead_time);
        println!("* maintenance_distance_violation: {}", self.maintenance_distance_violation);
        println!("* maintenance_duration_violation: {}", self.maintenance_duration_violation);
        println!("* dead_head_distance: {}", self.dead_head_distance);
    }

    pub fn new(overhead_time: Duration,
               number_of_dummy_units: usize,
               dummy_overhead_time: Duration,
               maintenance_distance_violation: Distance,
               maintenance_duration_violation: Duration,
               dead_head_distance: Distance) -> ObjectiveValue {
        ObjectiveValue {
            overhead_time,
            number_of_dummy_units,
            dummy_overhead_time,
            maintenance_distance_violation,
            maintenance_duration_violation,
            dead_head_distance
        }
    }
}

