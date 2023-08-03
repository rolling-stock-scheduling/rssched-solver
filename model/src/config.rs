use crate::base_types::{Distance, Duration};

pub struct Config {
    pub durations_between_activities: ConfigDurationsBetweenActivities,
    pub default_maximal_formation_length: Distance,
}

pub struct ConfigDurationsBetweenActivities {
    pub minimal: Duration,
    pub dead_head_trip: Duration,
}

impl Config {
    pub fn new(
        minimal_duration: Duration,
        dead_head_trip_duration: Duration,
        default_maximal_formation_length: Distance,
    ) -> Config {
        Config {
            durations_between_activities: ConfigDurationsBetweenActivities {
                minimal: minimal_duration,
                dead_head_trip: dead_head_trip_duration,
            },
            default_maximal_formation_length,
        }
    }
}
