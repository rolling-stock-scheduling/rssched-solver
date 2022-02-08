mod tour;
use tour::Tour;

use crate::network::Network;
use crate::unit::Unit;
use crate::locations::Locations;

use std::fmt;

pub(crate) struct Schedule<'a> {
    tours: Vec<Tour<'a>>,
}


impl<'a> Schedule<'a> {
    pub(crate) fn initialize(units: &'a Vec<Unit>, network: &'a Network<'a>) -> Schedule<'a> {
        let (start_nodes, end_nodes) = network.terminal_nodes();

        let mut tours : Vec<Tour<'a>> = Vec::with_capacity(units.len());
        for (i, unit) in units.iter().enumerate() {
            tours.push(Tour::new(unit, vec!(&start_nodes[i], &end_nodes[i])));
        }

        Schedule{tours}
    }

    pub(crate) fn print(&self, locations: &Locations) {
        println!("** schedule with {} tours:", self.tours.len());
        for (i, tour) in self.tours.iter().enumerate() {
            print!("\t{}. ", i);
            tour.print(locations);
        }
    }
}


impl<'a> fmt::Display for Schedule<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "** schedule with {} tours:", self.tours.len())?;
        for tour in self.tours.iter() {
            writeln!(f, "\t{}", tour)?;
        }
        Ok(())
    }
}
