# TODO

## Hypergraph

### Convert

- map
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

### Filter

- filter_ref(&self) -> Hypergraph<&*>
- filter_ref_mut(&mut self) -> Hypergraph<&mut *>
- filter(self) -> Hypergraph<*>

### Find

- Test

### Get

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

### Optimize

- Shrink to fit

### Remove

### Walk

# Traits

Implement as much as possible of `petgraph::visit` traits
