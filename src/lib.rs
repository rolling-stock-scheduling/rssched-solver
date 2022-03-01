mod base_types;
mod time;
mod distance;
mod utilities;

mod locations;
mod units;
mod network;

mod schedule;

mod solver;
mod modifications;

use solver::Solver;
use solver::greedy_1::Greedy1;

use network::Network;
use units::Units;
use locations::Locations;

use schedule::Schedule;
use schedule::path::Segment;

use std::rc::Rc;

pub fn run() {

    // load instance: All the objects are static and are multiple times referenced;
    // network also references locations
    let loc= Rc::new(Locations::load_from_csv("relationen.csv"));
    let units = Rc::new(Units::load_from_csv("fahrzeuggruppen.csv", loc.clone()));
    let nw = Rc::new(Network::load_from_csv("kundenfahrten.csv", "wartungsfenster.csv","endpunkte.csv", loc.clone(), units.clone()));



    // execute greedy_1 algorithms (going through units and pick nodes greedily)
    let greedy_1 = Greedy1::initialize(loc.clone(), units.clone(), nw.clone());
    let schedule = greedy_1.solve();

    schedule.write_to_csv("leistungsketten.csv").unwrap();



    // print some properties of the resulting schedule to the terminal:
    schedule.print();
    println!("total distance: {}", schedule.total_distance());
    println!("total overhead time: {} (MIN: {})", schedule.total_overhead_time(), nw.minimal_overhead());


    // manual_test(units.clone(), schedule);

}












//////////////////////////////////////////
//////////// manual test /////////////////
//////////////////////////////////////////


fn manual_test(units: Rc<Units>, schedule: Schedule) {
    let unit1 = *units.get_all().first().unwrap();
    let unit2 = *units.get_all().last().unwrap();

    let tour1 = schedule.tour_of(unit1);
    let tour2 = schedule.tour_of(unit2);

    let dummy1 = *schedule.all_dummy_units().first().unwrap();
    let dummy2 = *schedule.all_dummy_units().last().unwrap();
    println!("dummy2: {}", dummy2);


    let from = 5;
    let to = 10;
    let segment = Segment::new(*tour1.nodes_iter().nth(from).unwrap(),*tour1.nodes_iter().nth(to).unwrap());

    // let new_schedule = schedule.override_reassign(segment, unit1, unit2).unwrap();
    let new_schedule = schedule.override_reassign_all(dummy2,unit2).unwrap();
    let dummy3 = *new_schedule.all_dummy_units().first().unwrap();
    let dummy4 = *new_schedule.all_dummy_units().last().unwrap();
    let newest_schedule = new_schedule.override_reassign_all(dummy3, dummy4).unwrap();


    println!("BEFORE:");
    tour1.print();
    println!("MIDDLE:");
    new_schedule.tour_of(unit1).print();
    println!("AFTER:");
    newest_schedule.tour_of(unit1).print();
    println!("BEFORE:");
    tour2.print();
    println!("MIDDLE:");
    new_schedule.tour_of(unit2).print();
    println!("AFTER:");
    newest_schedule.tour_of(unit2).print();

    println!("\n\nBEFORE dummy_tours:");
    for dummy in schedule.all_dummy_units(){
        println!("dummy-unit: {}", dummy);
        schedule.tour_of(dummy).print();
        for n in schedule.tour_of(dummy).nodes_iter() {
            println!("{}", schedule.covered_by(*n));
        }
    }
    println!("\n\nMIDDLE dummy_tours:");
    for dummy in new_schedule.all_dummy_units(){
        println!("dummy-unit: {}", dummy);
        new_schedule.tour_of(dummy).print();
    }
    println!("\n\nAFTER dummy_tours:");
    for dummy in newest_schedule.all_dummy_units(){
        println!("dummy-unit: {}", dummy);
        newest_schedule.tour_of(dummy).print();
    }

}
