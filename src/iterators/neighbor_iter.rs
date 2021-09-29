use thiserror::Error;

use crate::{errors, walkers::NeighborWalk, Direction, Hypergraph};

/// Iterator over the neighbors of a linkable element of a hypergraph.
///
/// Iterator element type is `ElementLinkable<N, E, H>`.
///
/// Created with [`.neighbors()`][1].
///
/// [1]: struct.G\Hypergraph.html#method.ids
#[derive(Debug, Clone)]
pub struct NeighborIter<'a, N, E, H, L, Ty> {
    /// Reference to search neighbor values
    hypergraph: &'a Hypergraph<N, E, H, L, Ty>,
    /// Direction to accept
    direction: Direction,
    /// All links to visit
    source_id: Vec<usize>,
    /// Link counter
    next_link: usize,
}

impl<'a, N, E, H, L, Ty> Iterator for NeighborIter<'a, N, E, H, L, Ty> {
    type Item = &'a Vec<usize>;
    fn next(&mut self) -> std::option::Option<<Self as Iterator>::Item> {
        let links = self.hypergraph.links_of(&self.source_id).unwrap(); // Never fails since source_id refers to a linkable element
        if self.next_link >= links.len() {
            None
        } else {
            let result = match links.get(self.next_link) {
                Some((link_id, direction)) => {
                    if direction == &self.direction {
                        let element_linkable_id = match direction {
                            Direction::Outgoing => {
                                let (_, target) = self.hypergraph.link_endpoints(link_id).unwrap(); // Never fails since link exists
                                target
                            }
                            Direction::Incoming => {
                                let (source, _) = self.hypergraph.link_endpoints(link_id).unwrap(); // Never fails since link exists
                                source
                            }
                        };
                        Some(element_linkable_id)
                    } else {
                        self.next_link += 1;
                        return self.next();
                    }
                }
                None => None,
            };
            self.next_link += 1;
            result
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("Failed to create an iterator over neighbors because the desired source_id does not corresponds to an existing linkable element.")]
pub struct NewError(#[from] errors::NoElementLinkable);

impl<'a, N, E, H, L, Ty> NeighborIter<'a, N, E, H, L, Ty> {
    pub fn new(
        hypergraph: &'a Hypergraph<N, E, H, L, Ty>,
        source_id: impl AsRef<[usize]>,
        direction: Direction,
    ) -> Result<Self, NewError> {
        if !hypergraph.contains_linkable(&source_id) {
            Err(errors::NoElementLinkable(source_id.as_ref().to_vec()))?
        } else {
            let next_link = 0;
            Ok(NeighborIter {
                hypergraph,
                direction,
                source_id: source_id.as_ref().to_vec(),
                next_link,
            })
        }
    }
    pub fn detach(self) -> NeighborWalk {
        NeighborWalk::new(self.direction, self.next_link, self.source_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next() {}
}
