mod time;
mod distance;
mod base_types;

mod locations;
mod unit;

mod network;

mod schedule;

use network::Network;
use distance::Distance;
use unit::{Unit, UnitType};
use schedule::Schedule;
use time::Duration;
use locations::Locations;



mod placeholder;

pub fn run() {



    let units = vec!(Unit::new(0, UnitType::Giruno, Distance::from_km(300.0), Duration::new("500:00")),
                        Unit::new(1, UnitType::FVDosto, Distance::from_km(25000.0), Duration::new("50:00")),
                        Unit::new(2, UnitType::Astoro, Distance::from_km(0.0), Duration::new("30000:00")));
    let locations = Locations::load_from_csv(String::from("relationen.csv"));

    for station in locations.get_all_stations() {
        println!("{}", station);
    }

    let network: Network = Network::initialize(&locations, &units);
    println!("{}", network);

    for node in network.all_nodes_iter() {
        println!("node: {}", node);
        println!("start_time: {}", node.start_time());
        println!("end_time: {}", node.end_time());
        println!("successor: ");
        for succ in network.all_successors(node) {
            println!("succ: {}", succ);
            println!("\tin {}: {}", locations.travel_time(node.end_location(), succ.start_location()), succ);
        }
        println!("predecessor:");
        for pred in network.all_predecessors(node) {
            println!("\tin {}: {}", locations.travel_time(pred.end_location(), node.start_location()), pred);
        }
        println!("");
    }



    // println!("Deadhead-measures from {} to {}: distance: {}; travel_time: {}.", stations[0], stations[1], locations.distance(&stations[0], &stations[1]), locations.travel_time(&stations[0], &stations[1]));
    // println!("Deadhead-measures from {} to {}: distance: {}; travel_time: {}.", stations[2], stations[1], locations.distance(&stations[2], &stations[1]), locations.travel_time(&stations[2], &stations[1]));

    let first_schedule = Schedule::initialize(&units, &network);

    // println!("{}", first_schedule)
    first_schedule.print(&locations);
}
