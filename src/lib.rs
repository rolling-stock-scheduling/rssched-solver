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

use modifications::{SwapIterator,LocalImprover,Greedy};
use time::Duration;
use std::rc::Rc;

pub fn run(path: &str) {

    // load instance: All the objects are static and are multiple times referenced;
    // network also references locations
    let loc= Rc::new(Locations::load_from_csv(&format!("{}{}", path, "relationen.csv")));
    let units = Rc::new(Units::load_from_csv(&format!("{}{}", path, "fahrzeuggruppen.csv"), loc.clone()));
    let nw = Rc::new(Network::load_from_csv(&format!("{}{}", path, "kundenfahrten.csv"), &format!("{}{}", path, "wartungsfenster.csv"), &format!("{}{}", path, "endpunkte.csv"), loc.clone(), units.clone()));






    // execute greedy_1 algorithms (going through units and pick nodes greedily)
    // let greedy_1 = Greedy1::initialize(loc.clone(), units.clone(), nw.clone());
    // let schedule = greedy_1.solve();

    // let swap_factory = AllExchanges::new();
    let local_improver = Greedy::new();

    let mut schedule = Schedule::initialize(loc.clone(), units.clone(), nw.clone());

    let optimal = nw.minimal_overhead();
    let all_duration: Duration = nw.total_useful_duration();

    while let Some(sched) = local_improver.improve(&schedule) {
        println!("");
        println!("min_overhead: {}", optimal);
        sched.objective_value().print();
        schedule = sched;
        if schedule.number_of_dummy_units() < 10 {
            for dummy in schedule.dummy_iter(){
                println!("{}: {}", dummy, schedule.tour_of(dummy));
            }
        }
        let useful_duration: Duration = schedule.real_units_iter().map(|u| schedule.tour_of(u).useful_time()).sum();
        println!("covered duration: {} of {}", useful_duration, all_duration);
    }

    println!("\nFinal:");
    schedule.print();
    schedule.objective_value().print();


    // schedule.write_to_csv(&format!("{}{}", path, "ETH_leistungsketten.csv")).unwrap();



    // print some properties of the resulting schedule to the terminal:

    // for node in nw.all_nodes() {
        // println!("{}", nw.node(node));
    // }

    // schedule.objective_value().print();

    // println!("Minimal overhead: {}", nw.minimal_overhead());

    // manual_test(units.clone(), schedule);
    // manual_swap_test(units.clone(), schedule);

}












//////////////////////////////////////////
//////////// manual test /////////////////
//////////////////////////////////////////
use crate::modifications::{Swap,PathExchange};
fn manual_swap_test(units: Rc<Units>, schedule: Schedule) {

    let unit_a = units.iter().next().unwrap();
    let unit_b = units.iter().last().unwrap();

    let tour_a = schedule.tour_of(unit_a);
    let tour_b = schedule.tour_of(unit_b);

    let node = *tour_a.nodes_iter().nth(2).unwrap();

    println!("removable: {}", tour_a.removable_single_node(node));

    let swap = PathExchange::new(Segment::new(node, node), unit_a, unit_b);

    let new_schedule = swap.apply(&schedule).unwrap();


    println!("BEFORE:");
    println!("{}: {}", unit_a, tour_a);
    println!("AFTER:");
    println!("{}: {}", unit_a, new_schedule.tour_of(unit_a));
    println!("BEFORE:");
    println!("{}: {}", unit_b, tour_b);
    println!("AFTER:");
    println!("{}: {}", unit_b, new_schedule.tour_of(unit_b));

    println!("\n\nAFTER dummy_tours:");
    for dummy in new_schedule.dummy_iter(){
        println!("{}: {}", dummy, new_schedule.tour_of(dummy));
    }

    println!("\nnew_schedule:");
    new_schedule.print();
}

fn manual_test(units: Rc<Units>, schedule: Schedule) {
    let unit1 = units.iter().next().unwrap();
    let unit2 = units.iter().last().unwrap();

    let tour1 = schedule.tour_of(unit1);
    let tour2 = schedule.tour_of(unit2);

    let dummy1 = schedule.dummy_iter().next().unwrap();
    let dummy2 = schedule.dummy_iter().last().unwrap();
    println!("dummy2: {}", dummy2);


    let from = 5;
    let to = 10;
    let segment = Segment::new(*tour1.nodes_iter().nth(from).unwrap(),*tour1.nodes_iter().nth(to).unwrap());

    // let new_schedule = schedule.override_reassign(segment, unit1, unit2).unwrap();
    let (new_schedule,_) = schedule.override_reassign_all(dummy2,unit2).unwrap();
    let dummy3 = new_schedule.dummy_iter().next().unwrap();
    let dummy4 = new_schedule.dummy_iter().last().unwrap();
    let (newest_schedule,_) = new_schedule.override_reassign_all(dummy3, dummy4).unwrap();


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
    for dummy in schedule.dummy_iter(){
        println!("dummy-unit: {}", dummy);
        schedule.tour_of(dummy).print();
        for n in schedule.tour_of(dummy).nodes_iter() {
            println!("{}", schedule.covered_by(*n));
        }
    }
    println!("\n\nMIDDLE dummy_tours:");
    for dummy in new_schedule.dummy_iter(){
        println!("dummy-unit: {}", dummy);
        new_schedule.tour_of(dummy).print();
    }
    println!("\n\nAFTER dummy_tours:");
    for dummy in newest_schedule.dummy_iter(){
        println!("dummy-unit: {}", dummy);
        newest_schedule.tour_of(dummy).print();
    }

    println!("BEFORE:");
    schedule.objective_value().print();
    println!("MIDDLE:");
    new_schedule.objective_value().print();
    println!("AFTER:");
    newest_schedule.objective_value().print();

}
