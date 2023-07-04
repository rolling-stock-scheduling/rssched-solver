use crate::base_types::Cost;
use crate::config::Config;
use crate::distance::Distance;
use crate::time::Duration;
use core::cmp::Ordering;

use std::fmt::{self, Display};
use std::ops::Sub;
use std::sync::Arc;

/// objective value of schedule (to be minimized)
#[derive(Copy, Clone)]
pub(crate) struct ObjectiveValue {
    overhead_time: Duration, // idle_time + dead_head_time for the tours only (its minimal if all service trips are covered)
    number_of_dummy_units: usize,
    dummy_overhead_time: Duration, // idle_time + dead_head_time of dummy tours.
    // coupling_conflicts: u32,
    maintenance_penalty: MaintenancePenalty, // proporital combination of duration violation and distance violation
    maintenance_distance_violation: Distance,
    maintenance_duration_violation: Duration,
    soft_objective_cost: Cost,
    // unsatisfied_recommendation: u32, // number of unsatisfied recommended activity links (given by the reference plan)
    dead_head_distance: Distance, // total dead_head_distance traveled
    continuous_idle_time_cost: Cost,
    maintenance_distance_bathtub_cost: Cost,
    maintenance_duration_bathtub_cost: Cost,
}

impl ObjectiveValue {
    /// print objective value in comparison to the given objective value
    pub fn print(&self, objective_value_for_comparison: Option<&ObjectiveValue>) {
        println!(
            "* overhead_time: {} {}",
            self.overhead_time,
            self.print_difference(
                self.overhead_time,
                objective_value_for_comparison.map(|obj| obj.overhead_time)
            )
        );
        println!(
            "* number_of_dummy_units: {} {}",
            self.number_of_dummy_units,
            self.print_difference(
                self.number_of_dummy_units,
                objective_value_for_comparison.map(|obj| obj.number_of_dummy_units)
            )
        );
        println!(
            "* dummy_overhead_time: {} {}",
            self.dummy_overhead_time,
            self.print_difference(
                self.dummy_overhead_time,
                objective_value_for_comparison.map(|obj| obj.dummy_overhead_time)
            )
        );
        println!(
            "* maintenance_violation: {} {}",
            self.maintenance_penalty,
            self.print_difference(
                self.maintenance_penalty,
                objective_value_for_comparison.map(|obj| obj.maintenance_penalty)
            )
        );
        println!(
            "     - maintenance_distance_violation: {} {}",
            self.maintenance_distance_violation,
            self.print_difference(
                self.maintenance_distance_violation,
                objective_value_for_comparison.map(|obj| obj.maintenance_distance_violation)
            )
        );
        println!(
            "     - maintenance_duration_violation: {} {}",
            self.maintenance_duration_violation,
            self.print_difference(
                self.maintenance_duration_violation,
                objective_value_for_comparison.map(|obj| obj.maintenance_duration_violation)
            )
        );
        println!(
            "* soft_objective_cost: {:2.1} {}",
            self.soft_objective_cost,
            self.print_difference(
                self.soft_objective_cost,
                objective_value_for_comparison.map(|obj| obj.soft_objective_cost)
            )
        );
        println!(
            "    - dead_head_distance: {} {}",
            self.dead_head_distance,
            self.print_difference(
                self.dead_head_distance,
                objective_value_for_comparison.map(|obj| obj.dead_head_distance)
            )
        );
        println!(
            "    - continuous_idle_time_cost: {:2.1} {}",
            self.continuous_idle_time_cost,
            self.print_difference(
                self.continuous_idle_time_cost,
                objective_value_for_comparison.map(|obj| obj.continuous_idle_time_cost)
            )
        );
        println!(
            "    - maintenance_distance_bathtub_cost: {:2.1} {}",
            self.maintenance_distance_bathtub_cost,
            self.print_difference(
                self.maintenance_distance_bathtub_cost,
                objective_value_for_comparison.map(|obj| obj.maintenance_distance_bathtub_cost)
            )
        );
        println!(
            "    - maintenance_duration_bathtub_cost: {:2.1} {}",
            self.maintenance_duration_bathtub_cost,
            self.print_difference(
                self.maintenance_duration_bathtub_cost,
                objective_value_for_comparison.map(|obj| obj.maintenance_duration_bathtub_cost)
            )
        );
    }

    /// method for printing the difference between two values in green or red depending on the sign
    fn print_difference<T>(&self, value: T, value_for_comparison: Option<T>) -> String
    where
        T: Display + PartialOrd + Sub,
        <T as Sub>::Output: Display,
    {
        match value_for_comparison {
            Some(value_for_comparison) => {
                if value > value_for_comparison {
                    format!("(\x1b[0;31m+{:2.1}\x1b[0m)", value - value_for_comparison)
                } else if value < value_for_comparison {
                    format!("(\x1b[0;32m-{:2.1}\x1b[0m)", value_for_comparison - value)
                } else {
                    format!("")
                }
            }
            None => format!(""),
        }
    }

    pub fn new(
        overhead_time: Duration,
        number_of_dummy_units: usize,
        dummy_overhead_time: Duration,
        maintenance_distance_violation: Distance,
        maintenance_duration_violation: Duration,
        dead_head_distance: Distance,
        continuous_idle_time_cost: Cost,
        maintenance_distance_bathtub_cost: Cost,
        maintenance_duration_bathtub_cost: Cost,
        config: Arc<Config>,
    ) -> ObjectiveValue {
        let soft_objective_cost = dead_head_distance.as_km_cost()
            + continuous_idle_time_cost
            + maintenance_distance_bathtub_cost
            + maintenance_duration_bathtub_cost;

        ObjectiveValue {
            overhead_time,
            number_of_dummy_units,
            dummy_overhead_time,
            maintenance_distance_violation,
            maintenance_duration_violation,
            maintenance_penalty: MaintenancePenalty::new(
                maintenance_duration_violation,
                maintenance_distance_violation,
                config,
            ),
            soft_objective_cost,
            dead_head_distance,
            continuous_idle_time_cost,
            maintenance_distance_bathtub_cost,
            maintenance_duration_bathtub_cost,
        }
    }
}

impl ObjectiveValue {
    pub fn cmp_with_threshold(&self, other: &Self, threshold: Cost) -> Ordering {
        self.overhead_time
            .cmp(&other.overhead_time)
            .then(self.number_of_dummy_units.cmp(&other.number_of_dummy_units))
            .then(self.dummy_overhead_time.cmp(&other.dummy_overhead_time))
            .then(self.maintenance_penalty.cmp(&other.maintenance_penalty))
            .then(match self.soft_objective_cost - other.soft_objective_cost {
                diff if diff > threshold => Ordering::Greater,
                diff if diff < -threshold => Ordering::Less,
                _ => Ordering::Equal,
            })
    }
}

impl Ord for ObjectiveValue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.overhead_time
            .cmp(&other.overhead_time)
            .then(self.number_of_dummy_units.cmp(&other.number_of_dummy_units))
            .then(self.dummy_overhead_time.cmp(&other.dummy_overhead_time))
            .then(self.maintenance_penalty.cmp(&other.maintenance_penalty))
            .then(
                self.soft_objective_cost
                    .partial_cmp(&other.soft_objective_cost)
                    .unwrap(),
            )
    }
}

impl PartialOrd for ObjectiveValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ObjectiveValue {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Eq for ObjectiveValue {}

#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub(crate) struct MaintenancePenalty {
    penalty: f32,
}

impl MaintenancePenalty {
    pub(crate) fn new(
        maintenance_duration_violation: Duration,
        maintenance_distance_violation: Distance,
        config: Arc<Config>,
    ) -> MaintenancePenalty {
        let penalty = maintenance_duration_violation.in_min() as f32
            / config.maintenance.duration.in_min() as f32
            + maintenance_distance_violation.in_meter() as f32
                / config.maintenance.distance.in_meter() as f32;
        MaintenancePenalty { penalty }
    }
}

impl Eq for MaintenancePenalty {}

impl Ord for MaintenancePenalty {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Sub for MaintenancePenalty {
    type Output = MaintenancePenalty;

    fn sub(self, other: Self) -> Self::Output {
        MaintenancePenalty {
            penalty: self.penalty - other.penalty,
        }
    }
}

impl fmt::Display for MaintenancePenalty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:3.2}%", self.penalty * 100.0)
    }
}

pub(crate) fn compute_idle_time_cost(idle_time: Duration, config: &Arc<Config>) -> Cost {
    let para = &config.objective.continuous_idle_time;
    para.cost_factor
        * ((std::cmp::max(idle_time, para.minimum) - para.minimum).in_min() as Cost / 60.0)
            .powf(para.exponent)
}
