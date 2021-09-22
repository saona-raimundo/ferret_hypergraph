use crate::Hypergraph;
/// A “walker” object that can be used to step through a hypergraph without borrowing it.
///
/// Created with [`.detach()`](struct.IdIter.html#method.detach).
#[derive(Debug)]
pub struct IdWalk {
    next_id: Option<Vec<usize>>,
}
impl IdWalk {
    pub fn new(next_id: impl Into<Option<Vec<usize>>>) -> Self {
        IdWalk {
            next_id: next_id.into(),
        }
    }
    /// Step to the next id in the walk for `hypergraph`.
    ///
    /// The next id is always other than the starting point where `self` was created.
    fn next<N, E, H, L, Ty>(
        &mut self,
        hypergraph: &Hypergraph<N, E, H, L, Ty>,
    ) -> Option<Vec<usize>> {
        match &self.next_id {
            None => None,
            Some(id) => {
                if hypergraph.contains(id) {
                    let mut next = hypergraph.next_id(id);
                    core::mem::swap(&mut next, &mut self.next_id);
                    next
                } else {
                    // Update to the next valid id in hypergraph
                    self.next_id = hypergraph.next_id(id);
                    self.next(hypergraph)
                }
            }
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
        let mut id_walk = IdWalk::new(vec![]);

        assert_eq!(id_walk.next(&h).unwrap(), vec![]);

        for i in 0..7 {
            assert_eq!(id_walk.next(&h).unwrap(), vec![i]);
        }
        assert_eq!(id_walk.next(&h), None);
    }
}
