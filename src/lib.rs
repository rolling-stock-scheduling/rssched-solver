mod network;
use network::Network;


mod time;

mod location;
use location::Location;

mod placeholder;

pub fn run() {
    // let a : Node = Node::Start;
    let zuerich = Location::new("Zuerich");
    let basel = Location::new("Basel");
    let altstetten = Location::new("Altstetten");
    let station = vec!(&zuerich,&basel,&altstetten);
    let network: Network = Network::initialize(station);
    println!("{}", network);
}
