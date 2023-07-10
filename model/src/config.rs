use crate::base_types::{Cost, Distance, Duration};

pub struct Config {
    pub durations_between_activities: ConfigDurationsBetweenActivities,
    pub objective: ConfigObjective,
    pub maintenance: ConfigMaintenance,
}

pub struct ConfigDurationsBetweenActivities {
    pub minimal: Duration,
    pub turn: Duration, // Wende
    pub dead_head_trip: Duration,
    pub coupling: Duration,
    pub event: Duration,
}

pub struct ConfigObjective {
    pub cost_of_used_vehicle: Cost,
    pub cost_of_violated_activity_link: Cost,
    pub continuous_idle_time: ConfigContinuousIdleTime,
    pub bathtub: ConfigBathtub,
}

pub struct ConfigContinuousIdleTime {
    pub minimum: Duration,
    pub exponent: f32,
    pub cost_factor: f32,
}

pub struct ConfigBathtub {
    pub marginal_cost_per_deceeded_km: Cost,
    pub marginal_cost_per_exceeded_km: Cost,
    pub marginal_cost_per_deceeded_second: Cost,
    pub marginal_cost_per_exceeded_second: Cost,
}

pub struct ConfigMaintenance {
    pub duration: Duration,
    pub distance: Distance,
    pub bathtub_limits: ConfigBathtubLimits,
}

pub struct ConfigBathtubLimits {
    pub distance_upper_limit: Distance,
    pub distance_lower_limit: Distance,
    pub duration_upper_limit: Duration,
    pub duration_lower_limit: Duration,
}
