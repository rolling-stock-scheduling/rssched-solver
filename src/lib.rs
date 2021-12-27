mod network;
use network::Network;


mod time;

mod location;

mod vehicle;
use vehicle::{Vehicle, VehicleType};

mod distance;
use distance::Distance;

mod placeholder;

pub fn run() {
    // let a : Node = Node::Start;

    let vehicles = vec!(Vehicle::new(0, VehicleType::Giruno, Distance::from_km(300)),
                        Vehicle::new(1, VehicleType::FVDosto, Distance::from_km(25000)),
                        Vehicle::new(2, VehicleType::Astoro, Distance::from_km(0)));
    let (stations, dead_head_distances) = location::create_locations();
    let network: Network = Network::initialize(&stations, &vehicles);
    println!("{}", network);


    println!("Deadhead-distance from {} to {}: {}.", stations[0], stations[1], dead_head_distances.dist(&stations[0], &stations[1]));
    println!("Deadhead-distance from {} to {}: {}.", stations[2], stations[1], dead_head_distances.dist(&stations[2], &stations[1]));

}
