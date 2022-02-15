mod time;
mod distance;
mod base_types;

mod locations;
mod units;

mod network;

mod objective;
mod schedule;

mod utilities;

mod train_formation;

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

    // for id in network.all_nodes_ids() {
        // let node = network.node(id);
        // println!("node: {}", node);
        // println!("start_time: {}", node.start_time());
        // println!("end_time: {}", node.end_time());
        // println!("successor: ");
        // for succ in network.all_successors(node) {
            // println!("\tin {}: {}", locations.travel_time(node.end_location(), succ.start_location()), succ);
        // }
        // println!("predecessor:");
        // for pred in network.all_predecessors(node) {
            // println!("\tin {}: {}", locations.travel_time(pred.end_location(), node.start_location()), pred);
        // }
        // println!("");
    // }



    let mut first_schedule = Schedule::initialize(&locations, &units, &network);

    let unit_id = units.iter().next().unwrap().get_id();
    println!("Unit: {}", unit_id);
    for node_id in network.service_nodes_ids() {
        if network.can_reach(network.start_node_id_of(unit_id),node_id) && network.can_reach(node_id,first_schedule.get_tour_of(unit_id).last_node()) {
            first_schedule.assign(unit_id, vec!(node_id));
        }
    }
    first_schedule.print();

    println!("penalty: {}", first_schedule.total_cover_penalty());

    // for node in network.all_nodes_ids() {
        // println!("{}: \t{}", node, first_schedule.covered_by.get(&node).unwrap());
    // }

    // println!("{}", first_schedule)
    // first_schedule.print();
}
