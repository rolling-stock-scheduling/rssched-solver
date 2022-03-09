
use crate::distance::Distance;
use crate::time::Duration;

/// objective value of schedule (to be minimized)
#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub(crate) struct Objective {
    overhead_time: Duration, // idle_time + dead_head_time for the tours only (its minimal if all service trips are covered
    number_of_dummy_units : usize,
    dummy_overhead_time: Duration, // idle_time + dead_head_time of dummy tours.
    // coupling_conflicts: u32,
    // maintenance_violation: f32, // linear combination of duration violation / distance violaten
    // unsatisfied_recommendation: u32, // number of unsatisfied recommended activity links (given by the reference plan)
    dead_head_distance: Distance // total dead_head_distance traveled
}

impl Objective {
    pub fn print(&self) {
        println!("* overhead_time: {}", self.overhead_time);
        println!("* number_of_dummy_units: {}", self.number_of_dummy_units);
        println!("* dummy_overhead_time: {}", self.dummy_overhead_time);
        println!("* dead_head_distance: {}", self.dead_head_distance);
    }

    pub fn new(overhead_time: Duration, number_of_dummy_units: usize, dummy_overhead_time: Duration, dead_head_distance: Distance) -> Objective {
        Objective{
            overhead_time,
            number_of_dummy_units,
            dummy_overhead_time,
            dead_head_distance
        }
    }
}

