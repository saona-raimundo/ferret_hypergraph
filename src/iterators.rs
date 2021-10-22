use crate::{traits, Hypergraph};

#[derive(Debug)]
pub struct WalkIter<'a, N, E, H, L, Ty, Walker> {
    walker: Walker,
    hypergraph: &'a Hypergraph<N, E, H, L, Ty>,
}

impl<'a, N, E, H, L, Ty, Walker> WalkIter<'a, N, E, H, L, Ty, Walker> {
    pub fn new(walker: Walker, hypergraph: &'a Hypergraph<N, E, H, L, Ty>) -> Self {
        WalkIter { walker, hypergraph }
    }

    pub fn detach(self) -> Walker {
        self.walker
    }
}

impl<'a, N, E, H, L, Ty, Walker> Iterator for WalkIter<'a, N, E, H, L, Ty, Walker>
where
    Walker: traits::Walker<'a, N, E, H, L, Ty>,
{
    type Item = Walker::Item;
    fn next(&mut self) -> std::option::Option<Walker::Item> {
        self.walker.walk_next(self.hypergraph)
    }
}
