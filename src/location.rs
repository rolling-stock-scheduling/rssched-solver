use std::fmt;


pub(crate) enum Location {
    Location(Station),
    Infinity // distance to Infinity is always infinity
}

impl Location {
    pub(crate) fn new(name: &str) -> Location {
        Location::Location(Station{
            name : String::from(name)
        })
    }
}


impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Location::Location(s) => write!(f, "{}", s.name),
            Location::Infinity => write!(f, "INFINITY!"),
        }
    }
}


pub(crate) struct Station {
    name: String 
}
