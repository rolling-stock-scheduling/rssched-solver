mod base_types;
mod time;
mod distance;
mod utilities;

mod locations;
mod units;
mod network;

mod schedule;


mod solver;

use solver::Solver;
use solver::greedy_1::Greedy1;

use network::Network;
use units::Units;
use locations::Locations;

use std::rc::Rc;

pub fn run() {

    // load instance: All the objects are static and are multiple times referenced;
    // network also references locations
    let loc= Rc::new(Locations::load_from_csv("relationen.csv"));
    let units = Rc::new(Units::load_from_csv("fahrzeuggruppen.csv", loc.clone()));
    let nw = Rc::new(Network::load_from_csv("kundenfahrten.csv", "wartungsfenster.csv","endpunkte.csv", loc.clone(), units.clone()));



    // for location in locations.get_all_locations() {
        // println!("{}", location);
    // }
    // println!("{}", network);


    // execute greedy_1 algorithms (going through units and pick nodes greedily)

    let greedy_1 = Greedy1::initialize(loc.clone(), units.clone(), nw.clone());
    let schedule = greedy_1.solve();


    schedule.write_to_csv("leistungsketten.csv").unwrap();



    // print some properties of the resulting schedule to the terminal:

    schedule.print();

    println!("total distance: {}", schedule.total_distance());

    println!("uncovered nodes (penalty: {}):", schedule.total_cover_penalty());

    for node in schedule.uncovered_nodes(){
        println!("\t{}", nw.node(node));
    }

}
