mod tour;
use tour::Tour;

pub(crate) mod path;
use path::Path;
use path::Segment;

mod objective;
use objective::Objective;

pub(crate) mod train_formation;
use train_formation::TrainFormation;

use crate::network::Network;
use crate::network::nodes::Node;
use crate::units::{Units,UnitType};
use crate::locations::Locations;
use crate::distance::Distance;
use crate::base_types::{NodeId,UnitId};
use crate::time::Duration;

use std::collections::VecDeque;
use im::HashMap;

use itertools::Itertools;
use std::fmt;
use std::error::Error;

use std::rc::Rc;


// this represents a solution to the rolling stock problem.
// It should be an immutable object. So whenever a modification is applied a copy of the
// schedule is create.
#[derive(Clone)]
pub(crate) struct Schedule {
    tours: HashMap<UnitId, Tour>,
    covered_by: HashMap<NodeId, TrainFormation>,

    // non-covered or only partially covered service nodes are stored seperately
    dummy_units: HashMap<UnitId, UnitType>,
    dummy_tours: HashMap<UnitId, Tour>,
    dummy_counter: usize,

    loc: Rc<Locations>,
    units: Rc<Units>,
    nw: Rc<Network>,
}



// methods
impl Schedule {
    pub(crate) fn tour_of(&self, unit: UnitId) -> &Tour {
        self.tours.get(&unit).unwrap_or_else(|| self.dummy_tours.get(&unit).expect(format!("{} is neither real nor dummy unit", unit).as_str()))
    }

    pub(crate) fn covered_by(&self, node: NodeId) -> &TrainFormation {
        self.covered_by.get(&node).unwrap()
    }

    pub(crate) fn type_of(&self, unit: UnitId) -> UnitType {
        self.dummy_units.get(&unit).copied().unwrap_or_else(|| self.units.get_unit(unit).unit_type())
    }

    fn is_dummy(&self, unit: UnitId) -> bool {
        self.dummy_units.contains_key(&unit)
    }

    pub(crate) fn total_overhead_time(&self) -> Duration {
        self.tours.values().map(|t| t.overhead_time()).sum()
    }

    pub(crate) fn overhead_time_of(&self, unit: UnitId) -> Duration {
        self.tours.get(&unit).unwrap().overhead_time()
    }

    pub(crate) fn total_dummy_overhead_time(&self) -> Duration {
        self.dummy_tours.values().map(|t| t.overhead_time()).sum()
    }

    pub(crate) fn total_distance(&self) -> Distance {
        self.tours.values().map(|t| t.distance()).sum()
    }

    pub(crate) fn total_dead_head_distance(&self) -> Distance {
        self.tours.values().map(|t| t.dead_head_distance()).sum()
    }

    pub(crate) fn number_of_dummy_units(&self) -> usize {
        self.dummy_tours.keys().count()
    }

    pub(crate) fn objective_value(&self) -> Objective {
        Objective::new(self.total_overhead_time(),self.number_of_dummy_units(),self.total_dummy_overhead_time(),self.total_dead_head_distance())
    }

    // returns the first (seen from head to tail) dummy_unit that covers the node.
    // If node is fully-covered by real units, None is returned.
    fn get_dummy_cover_of(&self, node: NodeId) -> Option<UnitId> {
        self.covered_by.get(&node).unwrap().iter().filter(|u| self.dummy_units.contains_key(u)).next()
    }

    pub(crate) fn uncovered_nodes(&self) -> impl Iterator<Item = (NodeId,UnitId)> + '_ {
        self.dummy_tours.iter().flat_map(|(u,t)| t.nodes_iter().map(|&n| (n,*u)))
    }

    pub(crate) fn all_dummy_units(&self) -> Vec<UnitId> {
        let mut dummy_units: Vec<UnitId> = self.dummy_units.keys().copied().collect();
        dummy_units.sort();
        dummy_units
    }

    pub(crate) fn uncovered_successors(&self, node: NodeId) -> impl Iterator<Item = (NodeId,UnitId)> + '_ {
        self.nw.all_successors(node).filter_map(|n| self.get_dummy_cover_of(n).map(|u| (n,u)))
    }

    /// Simulates inserting the node_sequence into the tour of unit. Return all nodes (as a Path) that would
    /// have been removed from the tour.
    pub(crate) fn conflict(&self, segment: Segment, receiver: UnitId) -> Result<Path,String> {
        let tour: Tour = self.tours.get(&receiver).unwrap().clone();
        let result = tour.conflict(segment)?;
        Ok(result)
    }

    pub(crate) fn conflict_single_node(&self, node: NodeId, receiver: UnitId) -> Result<Path, String> {
        self.conflict(Segment::new(node,node),receiver)
    }

    pub(crate) fn conflict_all(&self, dummy_provider: UnitId, receiver: UnitId) -> Result<Path, String> {
        let tour = self.dummy_tours.get(&dummy_provider).expect("Can only assign_all if provider is a dummy-unit.");
        self.conflict(Segment::new(tour.first_node(), tour.last_node()), receiver)
    }

    /// Reassigns a path (given by a segment and a provider) to the tour of receiver.
    /// Aborts if there are any conflicts.
    pub(crate) fn reassign(&self, segment: Segment, provider: UnitId, receiver: UnitId) -> Result<Schedule, String> {
        let path = self.tour_of(provider).sub_path(segment)?;
        if !self.conflict(segment, receiver)?.is_empty() {
            return Err(String::from("There are conflcits. Abort reassign()!"));
        }
        self.override_reassign(segment, provider, receiver).map(|tuple| tuple.0)
    }

    /// Reassigns a single node of provider to the tour of receiver.
    /// Aborts if there are any conflicts.
    pub(crate) fn reassign_single_node(&self, node: NodeId, provider: UnitId, receiver: UnitId) -> Result<Schedule,String> {
        self.reassign(Segment::new(node, node), provider, receiver)
    }

    /// Reassign the complete tour of the provider (must be dummy-unit) to the receiver.
    /// Aborts if there are any conflicts.
    pub(crate) fn reassign_all(&self, dummy_provider: UnitId, receiver: UnitId) -> Result<Schedule, String> {
        let tour = self.dummy_tours.get(&dummy_provider).expect("Can only assign_all if provider is a dummy-unit.");
        self.reassign(Segment::new(tour.first_node(), tour.last_node()), dummy_provider, receiver)
    }

    /// Reassigns a single node of provider to the tour of receiver.
    /// Conflicts are removed from the tour.
    pub(crate) fn override_reassign_single_node(&self, node: NodeId, provider: UnitId, receiver: UnitId) -> Result<(Schedule, Option<UnitId>),String> {
        self.override_reassign(Segment::new(node, node), provider, receiver)
    }

    /// Reassign the complete tour of the provider (must be dummy-unit) to the receiver.
    /// Conflicts are removed from the tour.
    pub(crate) fn override_reassign_all(&self, dummy_provider: UnitId, receiver: UnitId) -> Result<(Schedule, Option<UnitId>), String> {
        let tour = self.dummy_tours.get(&dummy_provider).expect("Can only assign_all if provider is a dummy-unit.");
        self.override_reassign(Segment::new(tour.first_node(), tour.last_node()), dummy_provider, receiver)
    }

    /// Tries to insert all nodes of provider's segment into receiver's tour.
    /// Nodes that causes conflcits are rejected and stay in provider's tour.
    /// Nodes that do not cause a conflict are reassigned to the receiver.
    pub(crate) fn fit_reassign(&self, segment: Segment, provider: UnitId, receiver: UnitId) -> Result<Schedule,String> {

        // do lazy clones of schedule:
        let mut tours = self.tours.clone();
        let mut covered_by = self.covered_by.clone();
        let mut dummy_units = self.dummy_units.clone();
        let mut dummy_tours = self.dummy_tours.clone();

        let tour_provider = self.tour_of(provider);
        let tour_receiver = self.tour_of(receiver);

        let mut new_tour_provider = tour_provider.clone();
        let mut new_tour_receiver = tour_receiver.clone();

        let path = tour_provider.sub_path(segment)?;

        let mut moved_nodes = Vec::new();

        for &node in path.iter().filter(|&n| tour_receiver.conflict_single_node(*n).map(|c| c.is_empty()).unwrap_or(false)) {
            new_tour_receiver = new_tour_receiver.insert_single_node(node)?;
            new_tour_provider = new_tour_provider.remove_single_node(node)?;
            moved_nodes.push(node);
        }

        // update reduced tour of the provider
        if new_tour_provider.len() > 0 {
            if self.is_dummy(provider) {
                dummy_tours.insert(provider, new_tour_provider);
            } else {
                tours.insert(provider, new_tour_provider);
            }
        } else {
            dummy_units.remove(&provider);
            dummy_tours.remove(&provider); // old_dummy_tour is completely removed
        }

        // update extended tour of the receiver
        if self.is_dummy(receiver) {
            dummy_tours.insert(receiver, new_tour_receiver);
        } else {
            tours.insert(receiver, new_tour_receiver);
        }

        // update covered_by:
        for node in moved_nodes.iter() {
            let new_formation = covered_by.get(node).unwrap().replace(provider, receiver);
            covered_by.insert(*node, new_formation);
        }


        Ok(Schedule{tours,covered_by,dummy_units,dummy_tours, dummy_counter: self.dummy_counter, loc:self.loc.clone(),units:self.units.clone(),nw:self.nw.clone()})
    }

    pub(crate) fn fit_reassign_all(&self, provider: UnitId, receiver: UnitId) -> Result<Schedule,String> {
        let provider_tour = self.tour_of(provider);
        self.fit_reassign(Segment::new(provider_tour.first_node(), provider_tour.last_node()), provider, receiver)
    }


    /// Remove segment from provider's tour and inserts the nodes into the tour of receiver unit.
    /// All conflicting nodes are removed from the tour and in the case that there are conflcits
    /// a new dummy tour is created.
    /// If path ends with an endnode it is replaces the old endpoint. (Path is suffix of the tour.)
    /// Otherwise the path must reach the old endnode.
    pub(crate) fn override_reassign(&self, segment: Segment, provider: UnitId, receiver: UnitId) -> Result<(Schedule, Option<UnitId>),String> {

        // do lazy clones of schedule:
        let mut tours = self.tours.clone();
        let mut covered_by = self.covered_by.clone();
        let mut dummy_units = self.dummy_units.clone();
        let mut dummy_tours = self.dummy_tours.clone();
        let mut dummy_counter = self.dummy_counter;

        let tour_provider = self.tour_of(provider);
        let tour_receiver = self.tour_of(receiver);
        let (shrinked_tour_provider, path) = tour_provider.remove(segment)?;


        // update covered_by:
        for node in path.iter() {
            let new_formation = covered_by.get(node).unwrap().replace(provider, receiver);
            covered_by.insert(*node, new_formation);
        }

        // insert path into tour
        let replaced_path = tour_receiver.conflict(segment)?;
        let new_tour_receiver = tour_receiver.insert(path)?;




        // update shrinked tour of the provider
        if shrinked_tour_provider.len() > 0 {
            if self.is_dummy(provider) {
                dummy_tours.insert(provider, shrinked_tour_provider);
            } else {
                tours.insert(provider, shrinked_tour_provider);
            }
        } else {
            dummy_units.remove(&provider);
            dummy_tours.remove(&provider); // old_dummy_tour is completely removed
        }

        // update extended tour of the receiver
        if self.is_dummy(receiver) {
            dummy_tours.insert(receiver, new_tour_receiver);
        } else {
            tours.insert(receiver, new_tour_receiver);
        }

        let mut new_dummy_opt = None;
        // insert new dummy tour consisting of conflicting nodes removed from receiver's tour
        if !replaced_path.is_empty() {

            let new_dummy = UnitId::from(format!("dummy{:05}", dummy_counter).as_str());

            new_dummy_opt = Some(new_dummy);

            for node in replaced_path.iter() {
                let new_formation = covered_by.get(node).unwrap().replace(receiver, new_dummy);
                covered_by.insert(*node, new_formation);
            }

            let new_dummy_type = self.type_of(receiver);
            let new_dummy_tour = Tour::new_dummy_by_path(new_dummy_type, replaced_path, self.loc.clone(), self.nw.clone());

            dummy_units.insert(new_dummy, new_dummy_type);
            dummy_tours.insert(new_dummy, new_dummy_tour);

            dummy_counter += 1;

        }

        Ok((Schedule{tours,covered_by,dummy_units,dummy_tours,dummy_counter, loc:self.loc.clone(),units:self.units.clone(),nw:self.nw.clone()}, new_dummy_opt))
    }


    pub(crate) fn write_to_csv(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let mut writer = csv::WriterBuilder::new().delimiter(b';').from_path(path)?;
        writer.write_record(&["fahrzeuggruppeId","sortierZeit","typ","bpAb","bpAn","kundenfahrtId","endpunktId","wartungsfensterId"])?;
        for unit in self.units.get_all() {
            let tour = self.tours.get(&unit).unwrap();
            for (prev_node_id, node_id) in tour.nodes_iter().tuple_windows() {
                let node = self.nw.node(*node_id);

                let prev_node = self.nw.node(*prev_node_id);

                let fahrzeuggruppen_id = format!("{}",unit);



                if prev_node.end_location() != node.start_location() {
                    // add dead_head_trip
                    let dhd_ab = format!("{}", prev_node.end_location());
                    let dhd_an = format!("{}", node.start_location());
                    let dhd_sortier_zeit = prev_node.end_time().as_iso();
                    let empty = String::from("");
                    writer.write_record(&[fahrzeuggruppen_id.clone(),dhd_sortier_zeit,String::from("BETRIEBSFAHRT"),dhd_ab,dhd_an,empty.clone(),empty.clone(),empty])?;
                }



                let sortier_zeit = node.start_time().as_iso();
                let typ = String::from(match node {
                    Node::Service(_) => "KUNDENFAHRT",
                    Node::Maintenance(_) => "WARTUNG",
                    Node::End(_) => {"ENDPUNKT"},
                    _ => ""
                });
                let (bp_ab, bp_an) = match node {
                    Node::End(_) => (String::from(""), format!("{}", node.start_location())),
                    _ => (format!("{}", node.start_location()), format!("{}", node.end_location()))
                };

                let long_id = format!("{}", node.id());
                let id: &str = long_id.split(':').collect::<Vec<_>>().get(1).unwrap(); // remove the "ST:", "MS:", "EP:"
                let kundenfahrt_id = String::from(match node {
                    Node::Service(_) => id,
                    _ => ""
                });
                let endpunkt_id = String::from(match node {
                    Node::End(_) => id,
                    _ => ""
                });
                let wartungsfenster_id = String::from(match node {
                    Node::Maintenance(_) => id,
                    _ => ""
                });
                writer.write_record(&[fahrzeuggruppen_id,sortier_zeit,typ,bp_ab,bp_an,kundenfahrt_id,endpunkt_id,wartungsfenster_id])?;
            }
        }

        Ok(())

    }

    pub(crate) fn print(&self) {
        println!("** schedule with {} tours:", self.tours.len());
        for unit in self.units.get_all() {
            print!("\ttour of {}:", unit);
            self.tours.get(&unit).unwrap().print();
        }
    }
}


impl fmt::Display for Schedule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "** schedule with {} tours:", self.tours.len())?;
        for unit in self.units.get_all() {
            writeln!(f, "\t{}", self.tours.get(&unit).unwrap())?;
        }
        Ok(())
    }
}

// static functions
impl Schedule {
    pub(crate) fn initialize(loc: Rc<Locations>, units: Rc<Units>, nw: Rc<Network>) -> Schedule {

        let mut tours: HashMap<UnitId, Tour> = HashMap::new();
        let mut covered_by: HashMap<NodeId, TrainFormation> = HashMap::new();

        // create trivial tours from start_point directly to end point
        // end_ponints are picked greedily from earliest to latest (unit_type must fit)
        let mut end_nodes: VecDeque<NodeId> = nw.end_nodes().collect();
        end_nodes.make_contiguous().sort_by(|&e1,&e2| nw.node(e1).start_time().cmp(&nw.node(e2).start_time()));

        for unit_id in units.get_all() {
            let unit = units.get_unit(unit_id);
            let start_node = nw.start_node_of(unit_id);
            let pos = end_nodes.iter().position(|&e| nw.node(e).unit_type() == unit.unit_type() && nw.can_reach(start_node, e)).expect(format!("No suitable end_node found for start_node: {}", start_node).as_str());
            let end_node = end_nodes.remove(pos).unwrap();

            tours.insert(unit_id, Tour::new(unit.unit_type(), vec!(start_node, end_node), loc.clone(), nw.clone()));

            covered_by.insert(start_node, TrainFormation::new(vec!(unit_id), units.clone()));
            covered_by.insert(end_node, TrainFormation::new(vec!(unit_id), units.clone()));
        }
        assert!(end_nodes.is_empty(), "There are more end_nodes than units!");

        // all service- and maintanence nodes are non covered. We create dummy_units to coverer
        // them. Each dummy_unit has a Tour of exactly one node.
        let mut dummy_units = HashMap::new();
        let mut dummy_tours = HashMap::new();
        let mut dummy_counter = 0;

        for node in nw.service_nodes().chain(nw.maintenance_nodes()) {
            let mut formation = Vec::new();
            for t in nw.node(node).demand().get_valid_types() {
                let trivial_tour = Tour::new_dummy(t, vec!(node), loc.clone(), nw.clone());
                let new_dummy_id = UnitId::from(format!("dummy{:05}", dummy_counter).as_str());

                dummy_units.insert(new_dummy_id,t);
                dummy_tours.insert(new_dummy_id,trivial_tour);

                formation.push(new_dummy_id);
                dummy_counter += 1;
            }
            covered_by.insert(node, TrainFormation::new(formation, units.clone()));
        }

        Schedule{tours, covered_by, dummy_units, dummy_tours, dummy_counter, loc, units, nw}
    }
}
