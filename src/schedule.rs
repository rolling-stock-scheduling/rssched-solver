mod tour;
use tour::Tour;

use crate::network::Network;
use crate::units::Units;
use crate::locations::Locations;
use crate::base_types::UnitId;

use std::collections::HashMap;

use std::fmt;

pub(crate) struct Schedule<'a> {
    tours: HashMap<UnitId, Tour>,
    locations: &'a Locations,
    units: &'a Units,
    network: &'a Network<'a>
}


impl<'a> Schedule<'a> {
    pub(crate) fn initialize(locations: &'a Locations, units: &'a Units, network: &'a Network) -> Schedule<'a> {

        let mut tours : HashMap<UnitId, Tour> = HashMap::with_capacity(units.len());
        for unit in units.iter() {
            let unit_id = unit.get_id();
            tours.insert(unit_id, Tour::new(unit_id, vec!(network.start_node_of(unit_id).id())));
        }

        Schedule{tours, locations, units, network}
    }

    pub(crate) fn print(&self) {
        println!("** schedule with {} tours:", self.tours.len());
        for (unit, tour) in self.tours.iter() {
            print!("\ttour of {}:", unit);
            tour.print(self.locations, self.network);
        }
    }
}


impl<'a> fmt::Display for Schedule<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "** schedule with {} tours:", self.tours.len())?;
        for tour in self.tours.values() {
            writeln!(f, "\t{}", tour)?;
        }
        Ok(())
    }
}
