mod base_types;
mod time;
mod distance;
mod utilities;

mod locations;
mod units;
mod network;
mod config;

mod schedule;

mod solver;


use solver::Solver;
// use solver::greedy_1::Greedy1;
// use solver::greedy_2::Greedy2;
// use solver::greedy_3::Greedy3;
use solver::local_search::LocalSearch1;

use network::Network;
use units::Units;
use locations::Locations;
use config::Config;

use schedule::Schedule;

use std::sync::Arc;
use std::time as stdtime;

pub fn run(path: &str) {

    // load instance: All the objects are static and are multiple times referenced;
    // network also references locations
    let config = Arc::new(Config::from_yaml(&format!("{}{}", path, "../config.yaml")));
    let loc= Arc::new(Locations::load_from_csv(&format!("{}{}", path, "relationen.csv")));
    let units = Arc::new(Units::load_from_csv(&format!("{}{}", path, "fahrzeuggruppen.csv"), loc.clone()));
    let nw = Arc::new(Network::load_from_csv(&format!("{}{}", path, "kundenfahrten.csv"), &format!("{}{}", path, "wartungsfenster.csv"), &format!("{}{}", path, "endpunkte.csv"), config.clone(), loc.clone(), units.clone()));



    let start_time = stdtime::Instant::now();

    // execute greedy algorithm
    // let greedy_3 = Greedy3::initialize(config.clone(), units.clone(), nw.clone());
    // let final_schedule = greedy_3.solve();

    // Execute local search (which runs greedy to get an initial solution)
    let local_search_solver = LocalSearch1::initialize(config.clone(), units.clone(), nw.clone());
    let final_schedule = local_search_solver.solve();


    let end_time = stdtime::Instant::now();
    let runtime_duration = end_time.duration_since(start_time);

    println!("\n\nFinal schedule (long version):");
    final_schedule.print_long();

    println!("\n\nFinal schedule:");
    final_schedule.print();
    println!();
    let optimal = nw.minimal_overhead();
    println!("min_overhead: {}", optimal);
    final_schedule.objective_value().print();

    println!("ETH_Solution:");
    println!("Running time: {:0.2}sec", runtime_duration.as_secs_f32());

    final_schedule.write_to_csv(&format!("{}{}", path, "ETH_leistungsketten.csv")).unwrap();

    println!();
    let loaded_schedule = Schedule::load_from_csv(&format!("{}{}", path, "SBB_leistungsketten.csv"), config.clone(), units.clone(), nw.clone());
    println!("SBB_Solution:");
    loaded_schedule.objective_value().print();

}


