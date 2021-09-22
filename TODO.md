# TODO

## Hypergraph

### Removal

- remove
  - edge
    - edge
    - links
    - edge neighbor of 
      - nodes
      - hypergraphs
  - hypergraph
    - hypergraph
    - links
    - hypergraph neighbor of edges
    - edges with insufficient links
  - link
    - link
    - source & target neighbors
    - edges with insufficient links
  - node
    - node
    - links
    - node neighbor of edges
    - edges with insufficient links
- remove_element_by_value
  - Need comparison
  - Use get::ids
- Recall all the invariants
  - Edges need at least two vertices
  - Edges include links (that might be in other hypergraphs)
  - Links need source and target
  - Hypergraph includes its content

### Getters

- neighbors (iterator over ids)
  - NeighborWalk
- neighbors_mut (iterator)
  - 
- neighbors_directed (iterator)
- neighbors_directed_mut (iterator)
- Ids (iterator)
- References (iterator)
- References_mut (iterator)
- externals

### Walkers

Check out [Walker](https://docs.rs/petgraph/0.6.0/petgraph/visit/trait.Walker.html) and [Walker trait](https://github.com/petgraph/petgraph/issues/13)

This is to visit without borrowing from the data-structure.

I propose to simply clone the ids (`Vec<ID>`).

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

### Optimizations

- Shrink to fit

### Index

- Index
- IndexMut
- Vec<usize>

### Filtering

- filter_ref(&self) -> Hypergraph<&*>
- filter_ref_mut(&mut self) -> Hypergraph<&mut *>
- filter(self) -> Hypergraph<*>

### Git

- merge
- [diff](https://github.com/petgraph/petgraph/issues/320)

### [Extend](https://doc.rust-lang.org/nightly/core/iter/trait.Extend.html)

Increase your current hypergraph by other elements. 

- extend()
- extend_with_nodes()
- extend_with_edges()
- extend_with_links()
- extend_with_hypergraphs()

### Visualization

- draw()
  Check out [petgraph_evcxr](https://docs.rs/petgraph-evcxr/0.2.0/src/petgraph_evcxr/lib.rs.html#23-45)

# Traits

Implement as much as possible of `petgraph::visit` traits
