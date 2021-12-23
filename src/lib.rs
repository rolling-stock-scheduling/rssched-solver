mod network;
use network::Network;

mod time;

mod placeholder;

pub fn run() {
    // let a : Node = Node::Start;
    let network: Network = Network::initialize();

    println!("{}", network);
}
