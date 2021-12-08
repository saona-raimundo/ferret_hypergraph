use crate::{elements::ElementValue, errors, Hypergraph};

/// # Find
///
/// Find elements.
impl<N, E, H, L, Ty> Hypergraph<N, E, H, L, Ty> {
    /// Returns the id of the link that belongs to hypergraph `location` linking `source` and `target`.
    ///
    /// An empty `location` means the main hypergraph.
    ///
    /// Returns `None` if it does not exists.
    pub fn find_link_id<'a>(
        &self,
        source: impl AsRef<[usize]>,
        target: impl AsRef<[usize]>,
        value: impl Into<Option<&'a L>>,
        location: impl AsRef<[usize]>,
    ) -> Result<Vec<usize>, errors::FindError>
    where
        L: 'a + PartialEq,
    {
        let location = location.as_ref();
        let value = value.into();
        if !self.contains_hypergraph(location) {
            Err(errors::NoHypergraph(location.to_vec()))?
        }
        let hypergraph = self.hypergraph(location).unwrap(); // Never fails since location refers to a hypergraph
        let links = hypergraph.raw_links();
        let source = source.as_ref().to_vec();
        let target = target.as_ref().to_vec();
        for (local_id, link_full) in links {
            if (link_full.0.as_ref(), &link_full.1, &link_full.2) == (value, &source, &target) {
                let mut location = location.to_vec();
                location.push(*local_id);
                return Ok(location);
            }
        }
        Err(errors::FindError::NoLink)
    }

    pub fn find_element_by_value(
        &self,
        value: ElementValue<&N, &E, &H, &L>,
    ) -> Result<Vec<usize>, errors::FindError>
    where
        N: PartialEq,
        E: PartialEq,
        H: PartialEq,
        L: PartialEq,
    {
        match value {
            ElementValue::Edge { value } => self.find_edge_by_value(value),
            ElementValue::Hypergraph { value } => self.find_hypergraph_by_value(value),
            ElementValue::Link { value } => self.find_link_by_value(value),
            ElementValue::Node { value } => self.find_node_by_value(value),
        }
    }

    pub fn find_edge_by_value(&self, value: &E) -> Result<Vec<usize>, errors::FindError>
    where
        E: PartialEq,
    {
        let edge_ids = self.ids().filter(|id| self.contains_edge(id));
        edge_ids
            .map(|id| {
                let edge_value = self.edge_value(&id).unwrap(); // Never fails since id refers to an edge
                (id, edge_value)
            })
            .find(|(_, edge_value)| *value == **edge_value)
            .map(|(id, _)| id)
            .ok_or(errors::FindError::NoEdge)
    }

    pub fn find_hypergraph_by_value(
        &self,
        value: Option<&H>,
    ) -> Result<Vec<usize>, errors::FindError>
    where
        H: PartialEq,
    {
        let hypergraph_ids = self.ids().filter(|id| self.contains_hypergraph(id));
        hypergraph_ids
            .map(|id| {
                let hypergraph_value = self.hypergraph_value(&id).unwrap().as_ref(); // Never fails since id refers to an edge
                (id, hypergraph_value)
            })
            .find(|(_, hypergraph_value)| value == *hypergraph_value)
            .map(|(id, _)| id)
            .ok_or(errors::FindError::NoHypergraph)
    }

    pub fn find_link_by_value(&self, value: Option<&L>) -> Result<Vec<usize>, errors::FindError>
    where
        L: PartialEq,
    {
        let link_ids = self.ids().filter(|id| self.contains_link(id));
        link_ids
            .map(|id| {
                let link_value = self.link_value(&id).unwrap().as_ref(); // Never fails since id refers to an edge
                (id, link_value)
            })
            .find(|(_, link_value)| value == *link_value)
            .map(|(id, _)| id)
            .ok_or(errors::FindError::NoLink)
    }

    pub fn find_node_by_value(&self, value: &N) -> Result<Vec<usize>, errors::FindError>
    where
        N: PartialEq,
    {
        let node_ids = self.ids().filter(|id| self.contains_node(id));
        node_ids
            .map(|id| {
                let node_value = self.node_value(&id).unwrap(); // Never fails since id refers to a node
                (id, node_value)
            })
            .find(|(_, node_value)| *value == **node_value)
            .map(|(id, _)| id)
            .ok_or(errors::FindError::NoNode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use test_case::test_case;

    #[test]
    fn find_link_id() {
        // Links without values
        let mut h = Hypergraph::<&str, &str, (), _>::new();
        let node_0_id = h.add_node("zero");
        let node_1_id = h.add_node("one");
        let edge_id = h.add_edge([0], [1], "two").unwrap();

        let result = h.find_link_id(&node_0_id, &edge_id, None, []);
        assert_eq!(result, Ok(vec![3]));
        let result = h.find_link_id(&edge_id, &node_1_id, None, []);
        assert_eq!(result, Ok(vec![4]));

        // Links with values
        h.add_link(&node_0_id, &edge_id, "five").unwrap();
        h.add_link(&edge_id, &node_1_id, "six").unwrap();

        let result = h.find_link_id(&node_0_id, &edge_id, &"five", []);
        assert_eq!(result, Ok(vec![5]));
        let result = h.find_link_id(&edge_id, &node_1_id, &"six", []);
        assert_eq!(result, Ok(vec![6]));

        // Coherence with get::link_value
        let link_id = h.find_link_id(&node_0_id, &edge_id, &"five", []).unwrap();
        let link_value = h.link_value(link_id).unwrap();
        let result = h.find_link_id(&node_0_id, &edge_id, link_value, []);
        assert_eq!(result, Ok(vec![5]));
    }
}
