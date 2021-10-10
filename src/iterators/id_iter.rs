use crate::{walkers::IdWalk, Hypergraph};

/// Iterator over the ids of a hypergraph.
///
/// Iterator element type is `Vec<usize>`.
///
/// Created with [`.ids()`][1].
///
/// [1]: ../../struct.Hypergraph.html#method.ids
#[derive(Debug)]
pub struct IdIter<'a, N, E, H, L, Ty> {
    hypergraph: &'a Hypergraph<N, E, H, L, Ty>,
    next: Option<Vec<usize>>,
}

impl<'a, N, E, H, L, Ty> Iterator for IdIter<'a, N, E, H, L, Ty> {
    type Item = Vec<usize>;
    fn next(&mut self) -> std::option::Option<<Self as Iterator>::Item> {
        match &self.next {
            Some(id) => {
                let mut next = self.hypergraph.next_id(id);
                core::mem::swap(&mut next, &mut self.next);
                next
            }
            None => None,
        }
    }
}
impl<'a, N, E, H, L, Ty> IdIter<'a, N, E, H, L, Ty> {
    pub fn new(hypergraph: &'a Hypergraph<N, E, H, L, Ty>) -> Self {
        IdIter {
            hypergraph,
            next: Some(vec![]),
        }
    }
    pub fn detach(self) -> IdWalk {
        IdWalk::new(self.next)
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
        let id_iter = IdIter::new(&h);

        assert_eq!(
            id_iter.collect::<Vec<_>>(),
            vec![
                vec![],
                vec![0],
                vec![1],
                vec![2],
                vec![3],
                vec![4],
                vec![5],
                vec![6]
            ]
        );
    }
}
