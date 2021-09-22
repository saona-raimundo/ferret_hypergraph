use serde::{Deserialize, Serialize};

/// Edge direction.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    /// An `Outgoing` edge is an outward link *from* the current element.
    Outgoing,
    /// An `Incoming` edge is an inbound link *to* the current element.
    Incoming,
}

impl Direction {
    /// Return the opposite `Direction`.
    pub fn opposite(self) -> Direction {
        match self {
            Direction::Outgoing => Direction::Incoming,
            Direction::Incoming => Direction::Outgoing,
        }
    }
}
