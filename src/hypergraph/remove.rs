use crate::{
    elements::{ElementType, ElementValue},
    errors,
    Hypergraph,
};

/// # Remove
///
/// Remove elements.
impl<N, E, H, L, Ty> Hypergraph<N, E, H, L, Ty> {
    /// Removes the element with id `id`.
    ///
    /// Returns true if the element was removed, otherwise `false`.
    pub fn remove(&mut self, id: impl AsRef<[usize]>) -> Result<ElementValue<N, E, H, L>, errors::RemoveError> {
        let id = id.as_ref();
        if !self.contains(&id) {
            Err(errors::NoElement(id.to_vec()))?
        }
        let element = match self.element_type(&id).unwrap() // Never fails since id refers to an element
        {    
            ElementType::Edge => {
                let value = self.remove_edge(id)?;
                ElementValue::Edge{value}
            }
            ElementType::Hypergraph => {
                let value = self.remove_subhypergraph(id)?;
                ElementValue::Hypergraph{value}
            }
            ElementType::Link => {
                let value = self.remove_link(id)?;
                ElementValue::Link{value}
            }
            ElementType::Node => {
                let value = self.remove_node(id)?;
                ElementValue::Node{value}
            }
        };
        Ok(element)
    }

    pub fn remove_edge(&mut self, id: impl AsRef<[usize]>) -> Result<E, errors::RemoveError> {
        let id = id.as_ref();
        if !self.contains_edge(&id) {
            Err(errors::NoEdge(id.to_vec()))?
        }
        let local_id = id.last().unwrap(); // Never fails since id is not empty
        let hypergraph = self.hypergraph_of_mut(id).unwrap(); // Never fails since id refers to an edg
        let raw_edges = hypergraph.raw_edges_mut();
        let (edge_value, edge_links) = raw_edges.remove(local_id).unwrap(); // Never fails since id refers to an edge
        for (link_id, _) in edge_links {
            self.remove_link(link_id)?;
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
        let local_id = id.last().unwrap(); // Never fails since id is not empty
        let hypergraph = self.hypergraph_of_mut(id).unwrap(); // Never fails since id refers to a hypergraph
        let raw_hypergraphs = hypergraph.raw_hypergraphs_mut();
        let (subhypergraph, subhypergraph_links) = raw_hypergraphs.remove(local_id).unwrap(); // Never fails since id refers to a hypergraph
        for (link_id, _) in subhypergraph_links {
            self.remove_link(link_id)?;
        }
        let id = id.to_vec();
        for local_id in subhypergraph.ids() {
            let mut gloabl_id = id.clone();
            gloabl_id.extend(local_id);
            self.remove(gloabl_id);
        }
        Ok(subhypergraph.value)
    }

    pub fn remove_link(
        &mut self,
        id: impl AsRef<[usize]>,
    ) -> Result<Option<L>, errors::RemoveError> {
        let id = id.as_ref();

        if !self.contains_link(&id) {
            Err(errors::NoLink(id.to_vec()))?
        }
        let local_id = id.last().unwrap(); // Never fails since id is not empty
        let hypergraph = self.hypergraph_of_mut(id).unwrap(); // Never fails since id refers to a link
        let raw_links = hypergraph.raw_links_mut();
        let (link_value, source_id, target_id) = raw_links.remove(local_id).unwrap(); // Never fails since id refers to a link
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
    fn remove_link_from_unchecked(&mut self, link_id: impl AsRef<[usize]>, id: impl AsRef<[usize]>) {
        let id = id.as_ref();
        let link_id = link_id.as_ref();
        let local_id = id.last().expect("empty id"); // Panics if id is empty
        let element_type = self.element_type(id).expect("id is not a valid element"); // Panics if id is not a valid element
        let hypergraph = self.hypergraph_of_mut(&id).unwrap(); // Never fails since id refers to an element
        match element_type {
            ElementType::Edge => {
                let raw_edges = hypergraph.raw_edges_mut();
                let (_, edge_links) = raw_edges.get_mut(local_id).unwrap(); // Never fails since id refers to an edge
                let link_index = edge_links.iter().position(|(l_id, _)| link_id == l_id).expect("link_id is not among the links of id");
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
                    .position(|(link_id, _)| link_id == id).expect("link_id is not among the links of id");
                hyperraph_links.remove(link_index);
            }
            ElementType::Link => {
                panic!("id refers to a link! (link_id {:?}, id {:?})", link_id, id);
            }
            ElementType::Node => {
                let raw_nodes = hypergraph.raw_nodes_mut();
                let (_, node_links) = raw_nodes.get_mut(local_id).unwrap(); // Never fails since id refers to a node
                let link_index = node_links.iter().position(|(l_id, _)| link_id == l_id).expect("link_id is not among the links of id");
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
        let hypergraph = self.hypergraph_of_mut(id).unwrap(); // Never fails since id refers to a node
        let raw_nodes = hypergraph.raw_nodes_mut();
        let (node_value, node_links) = raw_nodes.remove(local_id).unwrap(); // Never fails since id refers to a node
        for (link_id, _) in node_links {
            self.remove_link(link_id)?;
        }
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