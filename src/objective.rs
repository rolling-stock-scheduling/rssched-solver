
use crate::distance::Distance;
use crate::base_types::Penalty;


/// objective value of schedule (to be minimized)
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub(crate) struct Objective {
    not_covered: Penalty, // number of service trips not covered (some might be partially non-covered)
    coupling_conflicts: u32,
    maintenance_violation: f32, // linear combination of duration violation / distance violaten
    unsatisfied_recommendation: u32, // number of unsatisfied recommended activity links (given by the reference plan)
    dead_head_distance: Distance // total dead_head_distance traveled
}

