
use crate::distance::Distance;
use crate::time::Duration;
use crate::config::Config;
use core::cmp::Ordering;

use std::sync::Arc;
use std::fmt;

/// objective value of schedule (to be minimized)
#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub(crate) struct ObjectiveValue {
    overhead_time: Duration, // idle_time + dead_head_time for the tours only (its minimal if all service trips are covered
    number_of_dummy_units : usize,
    dummy_overhead_time: Duration, // idle_time + dead_head_time of dummy tours.
    // coupling_conflicts: u32,
    maintenance_penalty: MaintenancePenalty, // proporital combination of duration violation and distance violation
    // maintenance_sum:
    // unsatisfied_recommendation: u32, // number of unsatisfied recommended activity links (given by the reference plan)
    dead_head_distance: Distance, // total dead_head_distance traveled
    maintenance_distance_violation: Distance,
    maintenance_duration_violation: Duration,
}

impl ObjectiveValue {
    pub fn print(&self) {
        println!("* overhead_time: {}", self.overhead_time);
        println!("* number_of_dummy_units: {}", self.number_of_dummy_units);
        println!("* dummy_overhead_time: {}", self.dummy_overhead_time);
        println!("* maintenance_violation: {} ({}; {})", self.maintenance_penalty, self.maintenance_distance_violation, self.maintenance_duration_violation);
        println!("* dead_head_distance: {}", self.dead_head_distance);
    }

    pub fn new(overhead_time: Duration,
               number_of_dummy_units: usize,
               dummy_overhead_time: Duration,
               maintenance_distance_violation: Distance,
               maintenance_duration_violation: Duration,
               dead_head_distance: Distance,
               config: Arc<Config>) -> ObjectiveValue {
        ObjectiveValue {
            overhead_time,
            number_of_dummy_units,
            dummy_overhead_time,
            maintenance_distance_violation,
            maintenance_duration_violation,
            maintenance_penalty : MaintenancePenalty::new(maintenance_duration_violation, maintenance_distance_violation, config),
            dead_head_distance
        }
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub(crate) struct MaintenancePenalty {
    penalty: f32
}

impl MaintenancePenalty {
    pub(crate) fn new(maintenance_duration_violation: Duration, maintenance_distance_violation: Distance, config: Arc<Config>) -> MaintenancePenalty {
        let penalty = maintenance_duration_violation.in_min() as f32 / config.maintenance.duration.in_min() as f32
            + maintenance_distance_violation.in_meter() as f32 / config.maintenance.distance.in_meter() as f32;
        MaintenancePenalty{penalty}
    }
}

impl Eq for MaintenancePenalty {}

impl Ord for MaintenancePenalty {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl fmt::Display for MaintenancePenalty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:3.2}%", self.penalty * 100.0)
    }
}
