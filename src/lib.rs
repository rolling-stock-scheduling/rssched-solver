mod time;
mod distance;

mod location;
mod vehicle;

mod network;

mod schedule;



use network::Network;
use distance::Distance;
use vehicle::{Vehicle, VehicleType};
use schedule::Schedule;
use time::Duration;





mod placeholder;

pub fn run() {

    let vehicles = vec!(Vehicle::new(0, VehicleType::Giruno, Distance::from_km(300), Duration::new("500:00")),
                        Vehicle::new(1, VehicleType::FVDosto, Distance::from_km(25000), Duration::new("50:00")),
                        Vehicle::new(2, VehicleType::Astoro, Distance::from_km(0), Duration::new("30000:00")));
    let (stations, dead_head_distances) = location::create_locations();
    let network: Network = Network::initialize(&stations, &vehicles);
    println!("{}", network);

    for node in network.all_nodes_iter() {
        println!("node: {}", node);
        println!("start_time: {}", node.start_time());
        println!("end_time: {}", node.end_time());
    }



    println!("Deadhead-distance from {} to {}: {}.", stations[0], stations[1], dead_head_distances.dist(&stations[0], &stations[1]));
    println!("Deadhead-distance from {} to {}: {}.", stations[2], stations[1], dead_head_distances.dist(&stations[2], &stations[1]));

    let first_schedule = Schedule::initialize(&vehicles, &network);

    // println!("{}", first_schedule)
    first_schedule.print(&dead_head_distances);
}
