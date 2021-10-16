# TODO

## Hypergraph

### Conversions

- Map
- map_nodes
- map_node(&mut self, index, function: FnMut(&mut N)) -> &mut Self
- map_edges
- map_edge
- map_links
- map_link
- map_hypergraphs
- map_hypergraph
- dot
- into_graph(&self) -> petgraph::Graph<&Vec<usize>, &Vec<usize>>
- into_hypergraph(&self) -> hypergraph::Hypegraph<&Vec<usize>, &Vec<usize>>

### [Extend](https://doc.rust-lang.org/nightly/core/iter/trait.Extend.html)

Increase your current hypergraph by other elements. 

- extend()
- extend_with_nodes()
- extend_with_edges()
- extend_with_links()
- extend_with_hypergraphs()

### Filtering

- filter_ref(&self) -> Hypergraph<&*>
- filter_ref_mut(&mut self) -> Hypergraph<&mut *>
- filter(self) -> Hypergraph<*>

### Find

- Needs (PartialEq)
- Each method with different constrains
- Better than with specific ids iterator (see Get)
  - EdgeIds
  - HypergraphIds
  - LinkIds
  - NodeIds

### Getters

- neighbors (iterator over ids)
  - NeighborWalk
- neighbors_mut (iterator)
  - 
- neighbors_directed (iterator)
- neighbors_directed_mut (iterator)
- [x] Ids (iterator)
  - EdgeIds
  - HypergraphIds
  - LinkIds
  - NodeIds
- References (iterator)
- References_mut (iterator)
- externals

### Git

- merge
- [diff](https://github.com/petgraph/petgraph/issues/320)

### Index

- Index
- IndexMut
- Vec<usize>

### Optimizations

- Shrink to fit

### Removal

- remove_element_by_value
  - Need comparison
    - Use get::ids

### Walkers

Check out [Walker](https://docs.rs/petgraph/0.6.0/petgraph/visit/trait.Walker.html) and [Walker trait](https://github.com/petgraph/petgraph/issues/13)

This is to visit without borrowing from the data-structure.

I propose to simply clone the ids (`Vec<ID>`).

# Traits

Implement as much as possible of `petgraph::visit` traits
