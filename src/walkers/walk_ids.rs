use crate::{traits::Walker, Hypergraph};

/// A “walker” object that can be used to step through a hypergraph without borrowing it.
///
/// Created with [`.detach()`](struct.IdIter.html#method.detach).
#[derive(Debug, Clone)]
pub struct WalkIds {
    next_id: Option<Vec<usize>>,
}
impl WalkIds {
    pub fn new() -> Self {
        Self::new_from(vec![])
    }

    pub fn new_from(next_id: impl Into<Option<Vec<usize>>>) -> Self {
        WalkIds {
            next_id: next_id.into(),
        }
    }
}

impl<'a, N, E, H, L, Ty> Walker<'a, N, E, H, L, Ty> for WalkIds {
    type Item = Vec<usize>;

    fn walk_next(&mut self, hypergraph: &'a Hypergraph<N, E, H, L, Ty>) -> Option<Self::Item> {
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
                    self.walk_next(hypergraph)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walk_next() {
        let mut h = Hypergraph::new();
        h.add_node("zero");
        h.add_node("one");
        h.add_edge([0], [1], "two").unwrap();
        h.add_link([0], [2], "three").unwrap();
        h.add_hypergraph("six").unwrap();
        let mut id_walk = WalkIds::new();

        assert_eq!(id_walk.walk_next(&h).unwrap(), vec![]);

        for i in 0..7 {
            assert_eq!(id_walk.walk_next(&h).unwrap(), vec![i]);
        }
        assert_eq!(id_walk.walk_next(&h), None);
    }
}
