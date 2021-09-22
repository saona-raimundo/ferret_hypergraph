use crate::{Direction, Hypergraph};

/// A “walker” object that can be used to step through a hypergraph without borrowing it.
///
/// Created with [`.detach()`](struct.NeighborIter.html#method.detach).
#[derive(Debug, Clone)]
pub struct NeighborWalk {
    /// Direction to accept
    direction: Direction,
    /// Link counter over the links of the source element
    next_link: usize,
    /// Link id and direction
    source_id: Vec<usize>,
}

impl NeighborWalk {
    pub fn new(direction: Direction, next_link: usize, source_id: impl AsRef<[usize]>) -> Self {
        NeighborWalk {
            direction,
            next_link,
            source_id: source_id.as_ref().to_vec(),
        }
    }

    /// Step to the next neighbor in the walk for `hypergraph`.
    ///
    /// The next neighbor is always other than the starting point where `self` was created.
    ///
    /// The walker advances in the neighbor count only if a link is found.
    /// Therefore, if a link is added, the walker will see all new links (even if it returned `None` before).
    ///
    fn next<'a, N, E, H, L, Ty>(
        &mut self,
        hypergraph: &'a Hypergraph<N, E, H, L, Ty>,
    ) -> Option<&'a Vec<usize>> {
        let links = hypergraph.links_of(&self.source_id).unwrap();
        match links.get(self.next_link) {
            Some((link_id, direction)) => {
                if direction == &self.direction {
                    let element_linkable_id = match direction {
                        Direction::Outgoing => {
                            let (_, target) = hypergraph.link_endpoints(link_id).unwrap(); // Never fails since link exists
                            target
                        }
                        Direction::Incoming => {
                            let (source, _) = hypergraph.link_endpoints(link_id).unwrap(); // Never fails since link exists
                            source
                        }
                    };
                    self.next_link += 1;
                    Some(element_linkable_id)
                } else {
                    self.next_link += 1;
                    return self.next(hypergraph);
                }
            }
            None => None,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next() {
        let mut h = Hypergraph::new();
        h.add_node("zero", []).unwrap();
        h.add_node("one", []).unwrap();
        h.add_edge([0], [1], "two", []).unwrap();
        h.add_link([0], [2], "three", []).unwrap();
        h.add_hypergraph("six", []).unwrap();
        let mut neighbor_walk = NeighborWalk::new(Direction::Outgoing, 0, [0]);

        assert_eq!(neighbor_walk.next(&h).unwrap(), &vec![2]);
        assert_eq!(neighbor_walk.next(&h).unwrap(), &vec![2]);
        assert_eq!(neighbor_walk.next(&h), None);
        h.add_link([0], [2], "three", []).unwrap();
        assert_eq!(neighbor_walk.next(&h).unwrap(), &vec![2]);
    }
}
