use crate::{
    elements::{ElementType, ElementValue},
    errors, Direction, Hypergraph,
};

/// # Remove
///
/// Remove elements.
impl<N, E, H, L, Ty> Hypergraph<N, E, H, L, Ty> {
    /// Removes the element with id `id`.
    ///
    /// Returns true if the element was removed, otherwise `false`.
    pub fn remove(
        &mut self,
        id: impl AsRef<[usize]>,
    ) -> Result<ElementValue<N, E, H, L>, errors::RemoveError> {
        let id = id.as_ref();
        if !self.contains(&id) {
            Err(errors::NoElement(id.to_vec()))?
        }
        // Never fails since id refers to an element
        let element = match self.element_type(&id).unwrap() {
            ElementType::Edge => {
                let value = self.remove_edge(id)?;
                ElementValue::Edge { value }
            }
            ElementType::Hypergraph => {
                let value = self.remove_subhypergraph(id)?;
                ElementValue::Hypergraph { value }
            }
            ElementType::Link => {
                let value = self.remove_link(id)?;
                ElementValue::Link { value }
            }
            ElementType::Node => {
                let value = self.remove_node(id)?;
                ElementValue::Node { value }
            }
        };
        Ok(element)
    }

    pub fn remove_edge(&mut self, id: impl AsRef<[usize]>) -> Result<E, errors::RemoveError> {
        let id = id.as_ref();
        if !self.contains_edge(&id) {
            Err(errors::NoEdge(id.to_vec()))?
        }

        // Removing links, except the first two links (so that the edge is still alive)
        let edge_links = self.links_of(id).unwrap().clone(); // Never fails since id refers to an edge
        for (link_id, _) in edge_links.into_iter().skip(2) {
            self.remove_link(link_id)?;
        }

        // Getting the edge value
        let local_id = id.last().unwrap(); // Never fails since id is not empty
        let hypergraph = self.hypergraph_of_mut(id).unwrap(); // Never fails since id refers to an edge
        let raw_edges = hypergraph.raw_edges_mut();
        let (edge_value, edge_links) = raw_edges.remove(local_id).unwrap(); // Never fails since id refers to an edge

        // Removing the remaining two links
        // We need to remove by hand since the edge is no longer an element of the hypergraph
        for (link_id, direction) in edge_links {
            let local_id = link_id.last().unwrap(); // Never fails since link_id is not empty
            match direction {
                Direction::Incoming => {
                    let (_, source_id, _) = self
                        .hypergraph_of_mut(id)
                        .unwrap() // Never fails since id refers to a link
                        .raw_links_mut()
                        .remove(local_id)
                        .unwrap(); // Never fails since id refers to a link
                    self.remove_link_from_unchecked(link_id, source_id);
                }
                Direction::Outgoing => {
                    let (_, _, target_id) = self
                        .hypergraph_of_mut(id)
                        .unwrap() // Never fails since id refers to a link
                        .raw_links_mut()
                        .remove(local_id)
                        .unwrap(); // Never fails since id refers to a link
                    self.remove_link_from_unchecked(link_id, target_id);
                }
            }
        }

        Ok(edge_value)
    }

    pub fn remove_subhypergraph(
        &mut self,
        id: impl AsRef<[usize]>,
    ) -> Result<Option<H>, errors::RemoveError> {
        let id = id.as_ref();
        if !self.contains_subhypergraph(&id) {
            Err(errors::NoHypergraph(id.to_vec()))?
        }
        // Remove all links
        let subhypergraph_links = self.links_of(id).unwrap().clone(); // Never fails since id refers to a hypergraph
        for (link_id, _) in subhypergraph_links {
            self.remove_link(link_id)?;
        }
        let id = id.to_vec();
        // Remove everything inside the hypergraph
        for local_id in self
            .subhypergraph(&id)
            .unwrap() // Never fails since id refers to a hypergraph
            .ids()
            .skip(1)
            .collect::<Vec<_>>()
        {
            let mut gloabl_id = id.clone();
            gloabl_id.extend(local_id);
            println!("Removing id {:?}", gloabl_id);
            self.remove(gloabl_id)?;
        }
        // Removing the hypergraph and receiving its value
        let local_id = id.last().unwrap(); // Never fails since id is not empty
        let (subhypergraph, _) = self
            .hypergraph_of_mut(&id)
            .unwrap() // Never fails since id refers to a hypergraph
            .raw_hypergraphs_mut()
            .remove(local_id)
            .unwrap(); // Never fails since id refers to a hypergraph
        Ok(subhypergraph.value)
    }

    /// Remove the link with id `link_id` from the list of links of the element `id`.  
    ///
    /// # Errors
    ///
    /// If `id` does is not a link.
    pub fn remove_link(
        &mut self,
        id: impl AsRef<[usize]>,
    ) -> Result<Option<L>, errors::RemoveError> {
        let id = id.as_ref();

        // Errors
        {
            if !self.contains_link(&id) {
                Err(errors::NoLink(id.to_vec()))?
            }
            let (source_id, target_id) = self.link_endpoints(id).unwrap(); // Never fails since id refers to a link
            if !self.contains_linkable(source_id) {
                Err(errors::NoElement(source_id.clone()))?
            }
            if !self.contains_linkable(target_id) {
                Err(errors::NoElement(target_id.clone()))?
            }
        }

        let local_id = id.last().unwrap(); // Never fails since id is not empty
        let (link_value, source_id, target_id) = self
            .hypergraph_of_mut(id)
            .unwrap() // Never fails since id refers to a link
            .raw_links_mut()
            .remove(local_id)
            .unwrap(); // Never fails since id refers to a link
        self.remove_link_from_unchecked(&id, source_id);
        self.remove_link_from_unchecked(&id, target_id);
        Ok(link_value)
    }

    /// Removes the link with id `link_id` from the list of links of the element `id`.  
    ///
    /// # Panics
    ///
    /// - `id` is empty.
    /// - `id` does not refers to a linkable element.
    /// - `link_id` is not among the links of `id`.
    fn remove_link_from_unchecked(
        &mut self,
        link_id: impl AsRef<[usize]>,
        id: impl AsRef<[usize]>,
    ) {
        let id = id.as_ref();
        let link_id = link_id.as_ref();
        let local_id = id.last().expect("empty id"); // Panics if id is empty
        let element_type = self.element_type(id).expect("id is not a valid element"); // Panics if id is not a valid element
        let hypergraph = self.hypergraph_of_mut(&id).unwrap(); // Never fails since id refers to an element
        match element_type {
            ElementType::Edge => {
                let raw_edges = hypergraph.raw_edges_mut();
                let (_, edge_links) = raw_edges.get_mut(local_id).unwrap(); // Never fails since id refers to an edge
                let link_index = edge_links
                    .iter()
                    .position(|(l_id, _)| link_id == l_id)
                    .expect("link_id is not among the links of id");
                edge_links.remove(link_index);
                if edge_links.len() < 2 {
                    self.remove_edge(id).unwrap(); // Never fails since id refers to an edge
                }
            }
            ElementType::Hypergraph => {
                let raw_hypergraphs = hypergraph.raw_hypergraphs_mut();
                let (_, hyperraph_links) = raw_hypergraphs.get_mut(local_id).unwrap(); // Never fails since id refers to a hypergraph
                let link_index = hyperraph_links
                    .iter()
                    .position(|(link_id, _)| link_id == id)
                    .expect("link_id is not among the links of id");
                hyperraph_links.remove(link_index);
            }
            ElementType::Link => {
                panic!("id refers to a link! (link_id {:?}, id {:?})", link_id, id);
            }
            ElementType::Node => {
                let raw_nodes = hypergraph.raw_nodes_mut();
                let (_, node_links) = raw_nodes.get_mut(local_id).unwrap(); // Never fails since id refers to a node
                let link_index = node_links
                    .iter()
                    .position(|(l_id, _)| link_id == l_id)
                    .expect("link_id is not among the links of id");
                node_links.remove(link_index);
            }
        }
    }

    pub fn remove_node(&mut self, id: impl AsRef<[usize]>) -> Result<N, errors::RemoveError> {
        let id = id.as_ref();
        if !self.contains_node(&id) {
            Err(errors::NoNode(id.to_vec()))?
        }
        let local_id = id.last().unwrap(); // Never fails since id is not empty
        for (link_id, _) in self.links_of(id).unwrap().clone() {
            self.remove_link(link_id)?;
        }
        let hypergraph = self.hypergraph_of_mut(id).unwrap(); // Never fails since id refers to a node
        let raw_nodes = hypergraph.raw_nodes_mut();
        let (node_value, _) = raw_nodes.remove(local_id).unwrap(); // Never fails since id refers to a node
        Ok(node_value)
    }

    /// Removes the first element matching `value`.
    pub fn remove_element_by_value(
        &mut self,
        value: ElementValue<&N, &E, &H, &L>,
    ) -> Result<(), errors::FindError> {
        let id = self.find_element_by_value(value)?;
        self.remove(id).unwrap(); // Never fails since id refers to a valid element
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use test_case::test_case;

    #[test]
    fn remove() {
        let mut h = Hypergraph::new();
        h.add_node("zero", []).unwrap();
        h.add_node("one", []).unwrap();
        h.add_edge([0], [1], "two", []).unwrap();
        h.add_link([0], [2], "five", []).unwrap();
        h.add_hypergraph("six", []).unwrap();

        println!("{:#?}", h);
        assert_eq!(
            h.remove([5]),
            Ok(ElementValue::Link {
                value: Some("five")
            })
        );

        assert_eq!(h.remove([2]), Ok(ElementValue::Edge { value: "two" }));

        assert_eq!(h.remove([0]), Ok(ElementValue::Node { value: "zero" }));
        println!("{:#?}", h);
        assert_eq!(
            h.remove([6]),
            Ok(ElementValue::Hypergraph { value: Some("six") })
        );

        assert_eq!(h.ids().collect::<Vec<_>>(), vec![vec![], vec![1]]);
    }
}
