//! Time varying graphs and algorithms to work on them.

use crate::attack::BlockedLink;
use crate::distance_matrix::{DistanceMatrixArray, SquareMatrix};
use crate::graph::{DijkstraInformation, DijkstraState};
use crate::graph::{Graph, Neighbour};
use crate::paths::{TemporalPath, TemporalSubPathsPyramid};
use crate::{Node, Tick};

use std::collections::BinaryHeap;
use std::ops::Index;

/// A phantom edge is used to replace a blocked edge with an edge with an equivalent weight + waiting time
/// This allows us to remove the time dimension from the graph, and use classical algorithms on it
/// The weight of the edge is now a vector of weights, where we can index the weight by the time
#[derive(Debug, Clone)]
pub struct PhantomEdge {
	pub neighbour: Node,
	pub weights: Vec<Tick>,
}

impl Index<Tick> for PhantomEdge {
	type Output = Tick;

	fn index(&self, index: Tick) -> &Self::Output {
		return &self.weights[usize::from(index)];
	}
}

/// The TimeVaryingGraph is a graph where the edges can be blocked at certain times
/// The graph is represented as an adjacency list, where each edge is a PhantomEdge
#[derive(Debug)]
pub struct TimeVaryingGraph {
	/// The maximum tick of the simulation
	max_tick: Tick,
	/// The adjacency list of the graph
	nodes: Vec<Vec<PhantomEdge>>,
	// A map to get the PhantomEdge from the graph from the node indexes for fast access
	// One day God will give me a safe way to do self-referential structs in Rust, that day is not today
	//edges_map: SquareMatrix<Option<*const PhantomEdge>>,
}

pub struct PhantomEdgesMap<'a> {
	edges_map: SquareMatrix<Option<&'a PhantomEdge>>,
}

impl<'a> PhantomEdgesMap<'a> {
	pub fn from_graph(time_graph: &'a TimeVaryingGraph) -> Self {
		let mut edges_map = SquareMatrix::new(None, time_graph.nb_nodes().into());
		for (node, neighbours) in time_graph.nodes.iter().enumerate() {
			for neighbour in neighbours {
				edges_map[(node, neighbour.neighbour.into())] = Some(neighbour);
			}
		}
		return PhantomEdgesMap { edges_map };
	}

	/// Get the PhantomEdge from the graph from the node indexes
	pub fn get_edge(&'a self, from: Node, to: Node) -> Option<&'a PhantomEdge> {
		let edge = self.edges_map[(from.into(), to.into())];
		return edge;
	}
}

impl TimeVaryingGraph {
	/// Returns the number of nodes in the graph
	pub fn nb_nodes(&self) -> Node {
		self.nodes.len() as Node
	}

	pub const fn max_tick(&self) -> Tick {
		self.max_tick
	}

	pub fn from_kind(graph: &Graph, max_tick: Tick, deleted_edges: &Vec<BlockedLink>, delta: u8) -> Self {
		let phantoms = create_phantom_edges(graph, max_tick, deleted_edges, delta);
		return Self { max_tick, nodes: phantoms };
	}

	pub fn from_aggressive(graph: &Graph, max_tick: Tick, deleted_edges: &Vec<BlockedLink>, delta: u8) -> Self {
		let phantoms = create_phantom_edges_aggressive(graph, max_tick, deleted_edges, delta);
		return Self { max_tick, nodes: phantoms };
	}

	pub fn get_neighbours(&self, node: Node) -> &Vec<PhantomEdge> {
		return &self.nodes[usize::from(node)];
	}
}

/// Writes the information from the Dijkstra's algorithm to the `DistanceMatrixArray`
/// This function is used to avoid recomputing paths when we already have the information from previous Dijkstra's
pub fn temporal_dijkstra_write(
	time_graph: &TimeVaryingGraph, start: Node, end: Node, time_start: Tick, dma: &mut DistanceMatrixArray, edges_map: &PhantomEdgesMap,
) {
	// Initialize the distance, previous node and visited arrays
	/*let max_node_index = time_graph.nb_nodes() as usize;
	let mut dist = vec![Tick::MAX; max_node_index];
	let mut prev = vec![Node::MAX; max_node_index];
	let mut visited = vec![false; max_node_index];
	dist[start as usize] = 0;
	let mut heap = BinaryHeap::new();
	heap.push(DijkstraState { node: start, distance: 0 });*/

	let mut dist = DijkstraInformation::new(Tick::MAX, time_graph.nb_nodes());
	let mut prev = DijkstraInformation::new(Node::MAX, time_graph.nb_nodes());
	let mut visited = DijkstraInformation::new(false, time_graph.nb_nodes());
	let mut heap = BinaryHeap::new();
	dist[start] = 0;
	heap.push(DijkstraState { node: start, distance: 0 });

	// Dijkstra's algorithm
	while let Some(DijkstraState { node, distance }) = heap.pop() {
		if visited[node] {
			continue;
		}
		if node == end {
			break;
		}
		if distance.saturating_add(time_start) >= time_graph.max_tick {
			dma[(time_start, start, end)] = Tick::MAX;
			break;
		}
		if node != start {
			dma[(time_start, start, node)] = distance;
		}
		visited[node] = true;
		for phantom_edge in time_graph.get_neighbours(node) {
			let neighbour = &phantom_edge.neighbour;
			let weight = phantom_edge[distance + time_start];
			if weight == Tick::MAX {
				continue;
			}
			let new_distance = distance + weight;
			if new_distance < dist[*neighbour] {
				dist[*neighbour] = new_distance;
				prev[*neighbour] = node;
				heap.push(DijkstraState {
					node: *neighbour,
					distance: new_distance,
				});
			}
		}
	}

	// Write all the information you can get from the Dijkstra's algorithm to the DistanceMatrixArray
	if dist[end] == Tick::MAX {
		dma[(time_start, start, end)] = Tick::MAX;
		return;
	}
	// Reconstruction of the path
	let mut node = end;
	let mut steps = vec![];
	while node != start {
		let prev_node = prev[node];
		if prev_node == Node::MAX {
			break;
		}
		let phantom_edge = edges_map.get_edge(prev_node, node).unwrap();
		let neighbour_node = phantom_edge.neighbour;
		steps.push(Neighbour {
			node: neighbour_node,
			weight: phantom_edge[dist[prev_node] + time_start],
		});
		node = prev_node;
	}
	steps.reverse();
	let path = TemporalPath {
		steps,
		time_graph,
		start,
		start_time: time_start,
	};
	let pyramid = TemporalSubPathsPyramid::from_path(time_graph, &path);
	for subpath in &pyramid.subpaths {
		let (from, to, time_start, total_length) = (subpath.start, subpath.end, subpath.start_time, subpath.total_weight);
		dma[(time_start, from, to)] = total_length;
	}
}

/// Creates the phantom edges for the time varying graph by computing how much time we need to wait to have it avalaible again
pub fn create_phantom_edges(graph: &Graph, max_tick: Tick, deleted_edges: &Vec<BlockedLink>, delta: u8) -> Vec<Vec<PhantomEdge>> {
	let mut phantoms = vec![];
	let max_tick = usize::from(max_tick);
	let delta = usize::from(delta);

	for (node, neighbours) in graph.edges.iter().enumerate() {
		let node = node as Node;
		let mut edges = vec![];

		// For all the edges, we create a phantom edge
		for neighbour in neighbours {
			let weight = neighbour.weight;
			let mut weights = vec![weight; max_tick];
			for deleted_edge in deleted_edges {
				if deleted_edge.from == node && deleted_edge.to == neighbour.node {
					for t in &deleted_edge.times {
						let t: usize = (*t).into();
						let t_with_delta = t * delta;
						for td in t_with_delta..t_with_delta + delta {
							if td < max_tick {
								weights[td] = 0;
							}
						}
					}
					let last_elem_idx = weights.len() - 1;
					if weights[last_elem_idx] == 0 {
						weights[last_elem_idx] = Tick::MAX;
					}
					for i in (0..max_tick - 1).rev() {
						if weights[i] == 0 {
							weights[i] = weights[i + 1].saturating_add(1);
						}
					}

					break;
				}
			}
			edges.push(PhantomEdge {
				neighbour: neighbour.node,
				weights,
			});
		}
		phantoms.push(edges);
	}

	return phantoms;
}

pub fn create_phantom_edges_aggressive(
	graph: &Graph, max_tick: Tick, deleted_edges: &Vec<BlockedLink>, delta: u8,
) -> Vec<Vec<PhantomEdge>> {
	let mut phantoms = vec![];

	for (node, neighbours) in graph.edges.iter().enumerate() {
		let node = node as Node;
		let mut edges = vec![];

		// For all the edges, we create a phantom edge
		for neighbour in neighbours {
			let weight = neighbour.weight;
			let mut weights = vec![weight; max_tick.into()];
			for deleted_edge in deleted_edges {
				if deleted_edge.from == node && deleted_edge.to == neighbour.node {
					for t in &deleted_edge.times {
						let t: usize = (*t).into();
						let delta: usize = delta.into();
						let t_with_delta = t * delta;
						for td in t_with_delta..t_with_delta + delta {
							if td < max_tick.into() {
								weights[td] = 0;
							}
						}
					}
					// For all the elements in the array, see if we have time to get through (more non-zero elements in a row than weight)
					let mut i = usize::from(max_tick - 1);
					while i > 0 {
						if weights[i] != 0 {
							let mut j = i;
							let mut count = 0;
							while j > 0 && weights[j] != 0 {
								count += 1;
								j -= 1;
							}
							if count < weight.into() {
								for k in j..=i {
									weights[k] = 0;
								}
							}
							i = j;
						}
						else {
							i -= 1;
						}
					}

					// If the last element is 0, we set it to MAX
					let last_elem_idx = weights.len() - 1;
					if weights[last_elem_idx] == 0 {
						weights[last_elem_idx] = Tick::MAX;
					}
					for i in (0..usize::from(max_tick - 1)).rev() {
						if weights[i] == 0 {
							weights[i] = weights[i + 1].saturating_add(1);
						}
					}

					break;
				}
			}
			edges.push(PhantomEdge {
				neighbour: neighbour.node,
				weights,
			});
		}
		phantoms.push(edges);
	}

	return phantoms;
}

pub fn graph_to_temporal(graph: &Graph, max_tick: Tick, deleted_edges: &Vec<BlockedLink>, delta: u8) -> TimeVaryingGraph {
	#[cfg(feature = "aggressive")]
	let phantoms = create_phantom_edges_aggressive(graph, max_tick, deleted_edges, delta);
	#[cfg(not(feature = "aggressive"))]
	let phantoms = create_phantom_edges(graph, max_tick, deleted_edges, delta); // replace with your non-aggressive function
	return TimeVaryingGraph { max_tick, nodes: phantoms };
}

pub fn temporal_dijkstra<'a>(
	time_graph: &'a TimeVaryingGraph, start: Node, end: Node, time_start: Tick, edges_map: &'a PhantomEdgesMap,
) -> TemporalPath<'a> {
	// Initialize the distance, previous node and visited arrays
	/*let max_node_index = time_graph.nb_nodes() as usize;
	let mut dist = vec![Tick::MAX; max_node_index];
	let mut prev = vec![Node::MAX; max_node_index];
	let mut visited = vec![false; max_node_index];*/
	let mut dist = DijkstraInformation::new(Tick::MAX, time_graph.nb_nodes());
	let mut prev = DijkstraInformation::new(Node::MAX, time_graph.nb_nodes());
	let mut visited = DijkstraInformation::new(false, time_graph.nb_nodes());
	dist[start] = 0;
	let mut heap = BinaryHeap::new();
	heap.push(DijkstraState { node: start, distance: 0 });

	// Dijkstra's algorithm
	while let Some(DijkstraState { node, distance }) = heap.pop() {
		if visited[node] {
			continue;
		}
		if node == end {
			break;
		}
		if distance.saturating_add(time_start) >= time_graph.max_tick {
			break;
		}
		visited[node] = true;
		for phantom_edge in &time_graph.nodes[usize::from(node)] {
			let neighbour = &phantom_edge.neighbour;
			let weight = phantom_edge[distance + time_start];
			if weight == Tick::MAX {
				continue;
			}
			let new_distance = distance + weight;
			if new_distance < dist[*neighbour] {
				dist[*neighbour] = new_distance;
				prev[*neighbour] = node;
				heap.push(DijkstraState {
					node: *neighbour,
					distance: new_distance,
				});
			}
		}
	}

	// Reconstruction of the path
	let mut node = end;
	let mut steps = vec![];
	while node != start {
		let prev_node = prev[node];
		if prev_node == Node::MAX {
			break;
		}
		let phantom_edge = edges_map.get_edge(prev_node, node).unwrap();
		let neighbour_node = phantom_edge.neighbour;
		steps.push(Neighbour {
			node: neighbour_node,
			weight: phantom_edge[dist[prev_node] + time_start],
		});
		node = prev_node;
	}

	steps.reverse();
	let path = TemporalPath {
		steps,
		time_graph,
		start,
		start_time: time_start,
	};
	return path;
}
