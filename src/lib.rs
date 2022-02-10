mod time;
mod distance;
mod base_types;

mod locations;
mod units;

mod network;

mod objective;
mod schedule;

mod utilities;

use network::Network;
use units::Units;
use schedule::Schedule;
use locations::Locations;


pub fn run() {

    // load instance: All the objects are static and are multiple times referenced;
    // network also references locations
    let locations = Locations::load_from_csv("relationen.csv");
    let units = Units::load_from_csv("fahrzeuggruppen.csv", &locations);
    let network: Network = Network::initialize(&locations, &units, "kundenfahrten.csv", "wartungsfenster.csv","endpunkte.csv");



    for location in locations.get_all_locations() {
        println!("{}", location);
    }
    println!("{}", network);

    for node in network.all_nodes_iter() {
        println!("node: {}", node);
        println!("start_time: {}", node.start_time());
        println!("end_time: {}", node.end_time());
        // println!("successor: ");
        // for succ in network.all_successors(node) {
            // println!("\tin {}: {}", locations.travel_time(node.end_location(), succ.start_location()), succ);
        // }
        // println!("predecessor:");
        // for pred in network.all_predecessors(node) {
            // println!("\tin {}: {}", locations.travel_time(pred.end_location(), node.start_location()), pred);
        // }
        println!("");
    }



    // println!("Deadhead-measures from {} to {}: distance: {}; travel_time: {}.", stations[0], stations[1], locations.distance(&stations[0], &stations[1]), locations.travel_time(&stations[0], &stations[1]));
    // println!("Deadhead-measures from {} to {}: distance: {}; travel_time: {}.", stations[2], stations[1], locations.distance(&stations[2], &stations[1]), locations.travel_time(&stations[2], &stations[1]));

    let first_schedule = Schedule::initialize(&locations, &units, &network);

    // println!("{}", first_schedule)
    first_schedule.print();
}
