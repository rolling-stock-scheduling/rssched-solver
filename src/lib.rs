mod base_types;
mod time;
mod distance;
mod utilities;
mod train_formation;

mod locations;
mod units;
mod network;

mod objective;
mod schedule;


mod solver;

use solver::Solver;
use solver::greedy_1::Greedy1;

use network::Network;
use units::Units;
use locations::Locations;


pub fn run() {

    // load instance: All the objects are static and are multiple times referenced;
    // network also references locations
    let locations = Locations::load_from_csv("relationen.csv");
    let units = Units::load_from_csv("fahrzeuggruppen.csv", &locations);
    let network: Network = Network::load_from_csv("kundenfahrten.csv", "wartungsfenster.csv","endpunkte.csv", &locations, &units);



    // for location in locations.get_all_locations() {
        // println!("{}", location);
    // }
    // println!("{}", network);


    // execute greedy_1 algorithms (going through units and pick nodes greedily)

    let greedy_1 = Greedy1::initialize(&locations, &units, &network);
    let schedule = greedy_1.solve();


    schedule.write_to_csv("leistungsketten.csv").unwrap();



    // print some properties of the resulting schedule to the terminal:

    schedule.print();

    println!("total distance: {}", schedule.total_distance());

    println!("uncovered nodes (penalty: {}):", schedule.total_cover_penalty());

    for node in schedule.uncovered_nodes(){
        println!("\t{}", network.node(node));
    }

}
