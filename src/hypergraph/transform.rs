use crate::{Hypergraph, Main, Sub};

/// # Add
///
/// A graph that can be extended with further nodes and edges
impl<N, E, H, L, Ty> Hypergraph<N, E, H, L, Ty> {
    /// Utility method to transform into a Hypergraph of class `Sub`.
    pub fn into_sub(self) -> Hypergraph<N, E, H, L, Sub> {
        Hypergraph {
            value: self.value,
            edges: self.edges,
            nodes: self.nodes,
            links: self.links,
            hypergraphs: self.hypergraphs,
            next_id: self.next_id,
            class: Sub,
        }
    }

    /// Pre-appends `location` to all absolute ids.
    ///
    /// # Remarks
    ///
    /// This is useful when extending from other hypergraph.
    pub(crate) fn preappend_id(&mut self, location: impl AsRef<[usize]>) -> &mut Self {
        let location = location.as_ref().to_vec();

        // Nodes, Edges and Hypergraphs
        let mut local_ids = self.raw_nodes().keys().cloned().collect::<Vec<_>>();
        local_ids.extend_from_slice(&self.raw_edges().keys().cloned().collect::<Vec<_>>());
        local_ids.extend_from_slice(&self.raw_hypergraphs().keys().cloned().collect::<Vec<_>>());
        for local_id in local_ids {
            let links = self.links_of_mut([local_id]).unwrap(); // Never fails since local_id is valid
            for (link, _) in links {
                let mut new_link = location.clone();
                new_link.extend_from_slice(link);
                *link = new_link;
            }
        }
        // Links
        for (_, source, target) in self.raw_links_mut().values_mut() {
            let mut new_source = location.clone();
            new_source.extend_from_slice(source);
            *source = new_source;
            let mut new_target = location.clone();
            new_target.extend_from_slice(target);
            *target = new_target;
        }

        // Recursive call
        for local_id in self.raw_hypergraphs().keys().cloned().collect::<Vec<_>>() {
            let subhypergraph = self.subhypergraph_mut([local_id]).unwrap(); // Never fails since local_id is valid
            subhypergraph.preappend_id(&location);
        }
        self
    }
}

impl<N, E, H, L> From<Hypergraph<N, E, H, L, Main>> for Hypergraph<N, E, H, L, Sub> {
    fn from(source: Hypergraph<N, E, H, L, Main>) -> Self {
        Hypergraph {
            value: source.value,
            edges: source.edges,
            nodes: source.nodes,
            links: source.links,
            hypergraphs: source.hypergraphs,
            next_id: source.next_id,
            class: Sub,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Direction;

    #[test]
    fn preappend_id() {
        let mut h = Hypergraph::new();
        h.add_node("zero");
        h.add_node("one");
        h.add_edge([0], [1], "two").unwrap();
        h.add_link([0], [2], "three").unwrap();
        h.add_hypergraph("six");
        h.preappend_id([1]);

        assert_eq!(
            h.links_of([0]).unwrap(),
            &vec![
                (vec![1, 3], Direction::Outgoing),
                (vec![1, 5], Direction::Outgoing)
            ]
        );
        assert_eq!(
            h.links_of([1]).unwrap(),
            &vec![(vec![1, 4], Direction::Incoming)]
        );
        assert_eq!(
            h.links_of([2]).unwrap(),
            &vec![
                (vec![1, 3], Direction::Incoming),
                (vec![1, 4], Direction::Outgoing),
                (vec![1, 5], Direction::Incoming),
            ]
        );
    }
}
