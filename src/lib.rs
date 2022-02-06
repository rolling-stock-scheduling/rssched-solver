mod time;
mod distance;

mod locations;
mod vehicle;

mod network;

mod schedule;



use network::Network;
use distance::Distance;
use vehicle::{Vehicle, VehicleType};
use schedule::Schedule;
use time::Duration;
use locations::Locations;



use time::Time;

mod placeholder;

pub fn run() {



    let vehicles = vec!(Vehicle::new(0, VehicleType::Giruno, Distance::from_km(300), Duration::new("500:00")),
                        Vehicle::new(1, VehicleType::FVDosto, Distance::from_km(25000), Duration::new("50:00")),
                        Vehicle::new(2, VehicleType::Astoro, Distance::from_km(0), Duration::new("30000:00")));
    let locations = Locations::create();

    let stations = locations.get_stations();

    let network: Network = Network::initialize(stations, &vehicles);
    println!("{}", network);

    for node in network.all_nodes_iter() {
        println!("node: {}", node);
        println!("start_time: {}", node.start_time());
        println!("end_time: {}", node.end_time());
    }



    println!("Deadhead-measures from {} to {}: distance: {}; travel_time: {}.", stations[0], stations[1], locations.distance(&stations[0], &stations[1]), locations.travel_time(&stations[0], &stations[1]));
    println!("Deadhead-measures from {} to {}: distance: {}; travel_time: {}.", stations[2], stations[1], locations.distance(&stations[2], &stations[1]), locations.travel_time(&stations[2], &stations[1]));

    let first_schedule = Schedule::initialize(&vehicles, &network);

    // println!("{}", first_schedule)
    first_schedule.print(&locations);
}
