mod tour;
use tour::Tour;

mod path;
use path::Path;

mod objective;
use objective::Objective;

pub(crate) mod train_formation;
use train_formation::TrainFormation;

use crate::network::Network;
use crate::network::nodes::Node;
use crate::units::Units;
use crate::locations::Locations;
use crate::distance::Distance;
use crate::base_types::{NodeId,UnitId,Penalty};

use std::collections::VecDeque;
use im::{HashSet,HashMap};

use std::fmt;
use std::error::Error;
use crate::base_types::PENALTY_ZERO;

use std::rc::Rc;


// this represents a solution to the rolling stock problem.
// It should be an immutable object. So whenever a modification is applied a copy of the
// schedule is create.
#[derive(Clone)]
pub(crate) struct Schedule {
    tours: HashMap<UnitId, Tour>,
    covered_by: HashMap<NodeId, TrainFormation>,

    // non-covered or only partially covered service nodes are stored seperately for quick objective_value conputation
    uncovered_nodes: HashSet<NodeId>,
    objective_value: Option<Objective>,

    loc: Rc<Locations>,
    units: Rc<Units>,
    nw: Rc<Network>,
}



// methods
impl Schedule {
    pub(crate) fn tour_of(&self, unit: UnitId) -> &Tour {
        self.tours.get(&unit).unwrap()
    }

    pub(crate) fn assign_path(&self, unit: UnitId, path: Path) -> Result<Schedule,String> {
        let new_nodes = path.clone();

        let mut tours = self.tours.clone(); // lazy clone
        let tour = tours.get(&unit).unwrap();

        // insert sequence into tour
        let (new_tour, removed_nodes) = tour.insert(path)?;

        tours.insert(unit, new_tour);

        // update covered_by:
        let mut covered_by = self.covered_by.clone(); // lazy clone
        for node in removed_nodes.iter() {
            covered_by.get_mut(node).unwrap().remove(unit);
        }
        for node in new_nodes.iter() {
            covered_by.get_mut(node).unwrap().add(unit);
        }

        // update uncovered_nodes:
        let mut uncovered_nodes = self.uncovered_nodes.clone(); // lazy clone
        for node in removed_nodes.iter() {
            if self.cover_penalty_of(*node) != PENALTY_ZERO {
                uncovered_nodes.insert(*node);
            }
        }
        for node in new_nodes.iter() {
            if self.cover_penalty_of(*node) == PENALTY_ZERO {
                uncovered_nodes.remove(node);
            }
        }
        Ok(Schedule{tours,covered_by,uncovered_nodes,objective_value:None,loc:self.loc.clone(),units:self.units.clone(),nw:self.nw.clone()})
    }

    pub(crate) fn assign_node(&self, unit: UnitId, node: NodeId) -> Result<Schedule,String> {
        self.assign_path(unit, Path::new(vec!(node),self.loc.clone(),self.nw.clone()))
    }

    /// simulates inserting the node_sequence into the tour of unit. Return all nodes (as a Path) that would
    /// have been removed from the tour.
    pub(crate) fn conflict_path(&self, unit: UnitId, path: Path) -> Result<Path,String> {
        let tour: Tour = self.tours.get(&unit).unwrap().clone();
        let result = tour.insert(path)?;
        Ok(result.1)
    }

    pub(crate) fn conflict_node(&self, unit: UnitId, node: NodeId) -> Result<Path, String> {
        self.conflict_path(unit, Path::new(vec!(node), self.loc.clone(), self.nw.clone()))
    }

    fn cover_penalty_of(&self, node: NodeId) -> Penalty {
        self.nw.node(node).cover_penalty(self.covered_by.get(&node).unwrap())
    }

    pub(crate) fn total_cover_penalty(&self) -> Penalty {
        self.uncovered_nodes.iter().map(|&n| self.cover_penalty_of(n)).sum()
    }

    pub(crate) fn total_distance(&self) -> Distance {
        self.tours.values().map(|t| t.distance()).sum()
    }

    pub(crate) fn uncovered_nodes(&self) -> Vec<NodeId> {
        let mut list: Vec<NodeId> = self.uncovered_nodes.iter().cloned().collect();
        list.sort_by(|&n1,&n2| self.nw.node(n1).cmp_start_time(self.nw.node(n2)));
        list
    }

    pub(crate) fn has_uncovered_nodes(&self) -> bool {
        self.uncovered_nodes.len() > 0
    }

    pub(crate) fn uncovered_successors(&self, node: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.nw.all_successors(node).filter(|n| self.uncovered_nodes.contains(n))
    }

    pub(crate) fn write_to_csv(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let mut writer = csv::WriterBuilder::new().delimiter(b';').from_path(path)?;
        writer.write_record(&["fahrzeuggruppeId","sortierZeit","typ","bpAb","bpAn","kundenfahrtId","endpunktId","wartungsfensterId"])?;
        for unit in self.units.get_all() {
            let tour = self.tours.get(&unit).unwrap();
            for node_id in tour.nodes_iter() {
                let node = self.nw.node(*node_id);
                if let Node::Start(_) = node {
                    continue;
                }
                let fahrzeuggruppen_id = format!("{}",unit);
                let sortier_zeit = node.start_time().as_iso();
                let typ = String::from(match node {
                    Node::Service(_) => "KUNDENFAHRT",
                    Node::Maintenance(_) => "WARTUNG",
                    Node::End(_) => {"ENDPUNKT"},
                    _ => ""
                });

                let bp_ab = format!("{}", node.start_location());
                let bp_an = format!("{}", node.start_location());

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

            tours.insert(unit_id, Tour::new(unit_id, vec!(start_node, end_node),loc.clone(),nw.clone()));

            covered_by.insert(start_node, TrainFormation::new(vec!(unit_id), units.clone()));
            covered_by.insert(end_node, TrainFormation::new(vec!(unit_id), units.clone()));
        }

        for node in nw.service_nodes() {
            covered_by.insert(node, TrainFormation::new(vec!(), units.clone()));
        }
        for node in nw.maintenance_nodes() {
            covered_by.insert(node, TrainFormation::new(vec!(), units.clone()));
        }

        assert!(end_nodes.is_empty(), "There are more end_nodes than units!");
        let mut uncovered_nodes: HashSet<NodeId> = HashSet::new();
        uncovered_nodes.extend(nw.service_nodes());
        uncovered_nodes.extend(nw.maintenance_nodes());


        Schedule{tours, covered_by, uncovered_nodes, objective_value:None, loc, units, nw}
    }
}
