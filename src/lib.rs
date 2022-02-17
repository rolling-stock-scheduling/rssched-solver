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
use base_types::NodeId;


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

    // for id in network.service_nodes_ids() {
        // let node = network.node(id);
        // println!("node: {}", node);
        // println!("start_time: {}", node.start_time());
        // println!("end_time: {}", node.end_time());
        // println!("successor: ");
        // for succ in network.all_successors(id) {
            // println!("\tin {}: {} start {}", locations.travel_time(node.end_location(), network.node(succ).start_location()), succ, network.node(succ).start_time());
        // }
        // println!("predecessor:");
        // for pred in network.all_predecessors(id) {
            // println!("\tin {}: {} end: {}", locations.travel_time(network.node(pred).end_location(), node.start_location()), pred, network.node(pred).end_time());
        // }
        // println!("");
    // }



    // let mut schedule = Schedule::initialize(&locations, &units, &network);
    // let mut counter = 0;

    // while schedule.has_uncovered_nodes() {
        // let node_id : NodeId = schedule.uncovered_iter().next().unwrap();

        // for unit in units.iter() {
            // let unit_id = unit.id();
            // if let Ok(replacement) = schedule.assign_test(unit_id, vec!(node_id)) {
                // if replacement.is_empty() {
                    // schedule.assign(unit_id, vec!(node_id)).unwrap();
                    // break;
                // }
            // }
        // }
        // if counter == 1000 {
            // break;
        // }
        // counter += 1;
    // }
    // schedule.print();


    let mut schedule = Schedule::initialize(&locations, &units, &network);
    for unit in units.iter() {
        let unit_id = unit.id();
        let mut node = network.start_node_id_of(unit_id);
        let mut new_node_opt = schedule.uncovered_successors(node).find(|&n| schedule.assign_test(unit_id,vec!(n)).is_ok());
        while new_node_opt.is_some() {
            node = new_node_opt.unwrap();
            schedule.assign(unit_id, vec!(node)).unwrap();
            new_node_opt = schedule.uncovered_successors(node).find(|&n| schedule.assign_test(unit_id,vec!(n)).is_ok());
        }
    }
    schedule.print();


    println!("penalty: {}", schedule.total_cover_penalty());

}
