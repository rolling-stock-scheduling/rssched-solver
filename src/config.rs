use crate::base_types::Cost;
use crate::distance::Distance;
use crate::time::Duration;

use yaml_rust::YamlLoader;
use yaml_rust::yaml::Yaml;

pub(crate) struct Config {
    pub durations_between_activities: ConfigDurationsBetweenActivities,
    objective : ConfigObjective,
    maintenance: ConfigMaintenance,
}

pub(crate) struct ConfigDurationsBetweenActivities {
    pub minimal: Duration,
    pub turn: Duration, // Wende
    pub dead_head_trip: Duration,
    pub coupling: Duration,
    pub event: Duration
}

pub(crate) struct ConfigObjective {
    cost_of_used_unit: Cost,
    cost_of_violated_activity_link: Cost,
    continuous_idle_time: ConfigContinuousIdleTime,
    bathtub: ConfigBathtub
}

pub(crate) struct ConfigContinuousIdleTime {
    minimum: Duration,
    exponent: f32,
    cost_factor: f32
}

pub(crate) struct ConfigBathtub {
    marginal_cost_per_deceeded_km: Cost,
    marginal_cost_per_exceeded_km: Cost,
    marginal_cost_per_deceeded_second: Cost,
    marginal_cost_per_exceeded_second: Cost
}

pub(crate) struct ConfigMaintenance {
    duration: Duration,
    distance: Distance,
    bathtub_limits: ConfigBathtubLimits
}

pub(crate) struct ConfigBathtubLimits {
    distance_upper_limit: Distance,
    distance_lower_limit: Distance,
    duration_upper_limit: Duration,
    duration_lower_limit: Duration
}

impl Config {
    pub(crate) fn from_yaml(path: &str) -> Config {

        fn dist_from_yaml(yaml: &Yaml) -> Distance {
            match yaml {
                Yaml::Real(string) => Distance::from_km_str(&string),
                Yaml::Integer(int) => Distance::from_km(*int as f32),
                _ => panic!("Not a valid distance format!")
            }
        }

        fn cost_from_yaml(yaml: &Yaml) -> Cost {
            match yaml {
                Yaml::Real(string) => string.parse().unwrap(),
                Yaml::Integer(int) => *int as f32,
                _ => panic!("Not a valid cost format!")
            }
        }

        let config_string: String = std::fs::read_to_string(path).expect("Could not find config.yaml").parse().expect("Cannot parse config.yaml");
        let config = &YamlLoader::load_from_str(&config_string).expect("Could not convert config-string Vec by YamlLoader.")[0];

        let minimal = Duration::from_iso(config["duration_between_leistungen"]["minimal"].as_str().unwrap());
        let turn = Duration::from_iso(config["duration_between_leistungen"]["wende"].as_str().unwrap());
        let dead_head_trip = Duration::from_iso(config["duration_between_leistungen"]["betriebsfahrt"].as_str().unwrap());
        let coupling = Duration::from_iso(config["duration_between_leistungen"]["kuppeln"].as_str().unwrap());
        let event = Duration::from_iso(config["duration_between_leistungen"]["event"].as_str().unwrap());


        println!("minimal: {}", minimal);
        let durations_between_activities = ConfigDurationsBetweenActivities {
            minimal,
            turn,
            dead_head_trip,
            coupling,
            event
        };


        let cost_of_used_unit = cost_from_yaml(&config["objective"]["cost_per_fahrzeuggruppe_planned"]);
        let cost_of_violated_activity_link = cost_from_yaml(&config["objective"]["cost_per_violated_reference_leistungsverknuepfung"]);

        let minimum = Duration::from_iso(config["objective"]["continuous_idle_time"]["minimum"].as_str().unwrap());
        let exponent = cost_from_yaml(&config["objective"]["continuous_idle_time"]["exponent"]);
        let cost_factor = cost_from_yaml(&config["objective"]["continuous_idle_time"]["cost_factor"]);

        let continuous_idle_time = ConfigContinuousIdleTime {
            minimum,
            exponent,
            cost_factor
        };

        let marginal_cost_per_deceeded_km = cost_from_yaml(&config["objective"]["bathtub"]["marginal_cost_per_deceeded_km"]);
        let marginal_cost_per_exceeded_km = cost_from_yaml(&config["objective"]["bathtub"]["marginal_cost_per_exceeded_km"]);


        let marginal_cost_per_deceeded_second = cost_from_yaml(&config["objective"]["bathtub"]["marginal_cost_per_deceeded_second"]);
        let marginal_cost_per_exceeded_second = cost_from_yaml(&config["objective"]["bathtub"]["marginal_cost_per_exceeded_second"]);

        let bathtub = ConfigBathtub {
            marginal_cost_per_deceeded_km,
            marginal_cost_per_exceeded_km,
            marginal_cost_per_deceeded_second,
            marginal_cost_per_exceeded_second
        };

        let objective = ConfigObjective {
            cost_of_used_unit,
            cost_of_violated_activity_link,
            continuous_idle_time,
            bathtub
        };

        let duration = Duration::from_iso(config["ivog"]["duration"].as_str().unwrap());
        let distance = dist_from_yaml(&config["ivog"]["distance"]);

        let distance_upper_limit = dist_from_yaml(&config["ivog"]["bathtub"]["distance"]["ub"]);
        let distance_lower_limit = dist_from_yaml(&config["ivog"]["bathtub"]["distance"]["lb"]);

        let duration_upper_limit = Duration::from_iso(config["ivog"]["bathtub"]["duration"]["ub"].as_str().unwrap());
        let duration_lower_limit = Duration::from_iso(config["ivog"]["bathtub"]["duration"]["lb"].as_str().unwrap());

        let bathtub_limits = ConfigBathtubLimits {
            distance_upper_limit,
            distance_lower_limit,
            duration_upper_limit,
            duration_lower_limit
        };

        let maintenance = ConfigMaintenance {
            duration,
            distance,
            bathtub_limits
        };

        Config {
            durations_between_activities,
            objective,
            maintenance
        }
    }
}

