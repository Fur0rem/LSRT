use crate::distance_matrix::{DistanceMatrixArray, SquareMatrix};
use crate::graph::DijkstraState;
use crate::graph::{Graph, Neighbour, Node, Tick};
use crate::paths::{TemporalPath, TemporalSubPathsPyramid};
use crate::DeletedLink;

use std::collections::BinaryHeap;

#[derive(Debug, Clone)]
pub struct TimeVaryingEdge {
	pub weights: Vec<Tick>,
}

/// A phantom edge is used to replace a blocked edge with an edge with an equivalent weight + waiting time
/// This allows us to remove the time dimension from the graph, and use classical algorithms on it
/// The weight of the edge is now a vector of weights, where we can index the weight by the time
#[derive(Debug, Clone)]
pub struct PhantomEdge {
	pub neighbour: Node,
	pub weights: TimeVaryingEdge,
}

/// The TimeVaryingGraph is a graph where the edges can be blocked at certain times
/// The graph is represented as an adjacency list, where each edge is a PhantomEdge
#[derive(Debug)]
pub struct TimeVaryingGraph {
	pub max_time: Tick,
	pub nodes: Vec<Vec<PhantomEdge>>,
	// One day God will give me a safe way to do self-referential structs in Rust, that day is not today
	pub edges_map: SquareMatrix<Option<*const PhantomEdge>>,
}

impl TimeVaryingGraph {
	/// Returns the number of nodes in the graph
	pub fn nb_nodes(&self) -> Node {
		self.nodes.len() as Node
	}

	pub fn from(graph: &Graph, max_time: Tick, deleted_edges: &Vec<DeletedLink>, delta: u8) -> Self {
		let phantoms = create_phantom_edges(graph, max_time, deleted_edges, delta);
		let mut edges_map = SquareMatrix::new(graph.nb_nodes() as usize, None);
		for (node, neighbours) in phantoms.iter().enumerate() {
			for neighbour in neighbours {
				edges_map[(node, neighbour.neighbour as usize)] = Some(neighbour as *const PhantomEdge);
			}
		}
		return TimeVaryingGraph {
			max_time,
			nodes: phantoms,
			edges_map,
		};
	}
}

/// Writes the information from the Dijkstra's algorithm to the DistanceMatrixArray
/// This function is used to avoid recomputing paths when we already have the information from previous Dijkstra's
fn write_info_to_dma(
	dma: &mut DistanceMatrixArray, time_graph: &TimeVaryingGraph, start: Node, end: Node, time_start: Tick, dist: &[Tick], prev: &[Node],
) {
	if dist[end as usize] == Tick::MAX {
		dma[(time_start, start, end)] = Tick::MAX;
		return;
	}
	// Reconstruction of the path
	let mut node = end;
	let mut steps = vec![];
	while node != start {
		let prev_node = prev[node as usize];
		if prev_node == Node::MAX {
			break;
		}
		let phantom_edge = time_graph.edges_map[(prev_node as usize, node as usize)].unwrap();
		let phantom_edge = unsafe { &*phantom_edge };
		let tve = &phantom_edge.weights;
		let neighbour_node = phantom_edge.neighbour;
		steps.push(Neighbour {
			node: neighbour_node,
			weight: tve.weights[(dist[prev_node as usize] + time_start as Tick) as usize],
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

pub fn temporal_dijkstra_write(time_graph: &TimeVaryingGraph, start: Node, end: Node, time_start: Tick, dma: &mut DistanceMatrixArray) {
	// Initialize the distance, previous node and visited arrays
	let max_node_index = time_graph.nb_nodes() as usize;
	let mut dist = vec![Tick::MAX; max_node_index];
	let mut prev = vec![Node::MAX; max_node_index];
	let mut visited = vec![false; max_node_index];
	dist[start as usize] = 0;
	let mut heap = BinaryHeap::new();
	heap.push(DijkstraState { node: start, distance: 0 });

	// Dijkstra's algorithm
	while let Some(DijkstraState { node, distance }) = heap.pop() {
		if visited[node as usize] {
			continue;
		}
		if node == end {
			break;
		}
		if distance.saturating_add(time_start) >= time_graph.max_time {
			dma[(time_start, start, end)] = Tick::MAX;
			break;
		}
		if node != start {
			dma[(time_start, start, node)] = distance;
		}
		visited[node as usize] = true;
		for phantom_edge in &time_graph.nodes[node as usize] {
			let edge = &phantom_edge.weights;
			let neighbour = &phantom_edge.neighbour;
			let weight = edge.weights[distance as usize + time_start as usize];
			if weight == Tick::MAX {
				continue;
			}
			let new_distance = distance + weight;
			if new_distance < dist[*neighbour as usize] {
				dist[*neighbour as usize] = new_distance;
				prev[*neighbour as usize] = node;
				heap.push(DijkstraState {
					node: *neighbour,
					distance: new_distance,
				});
			}
		}
	}

	// Write all the information you can get from the Dijkstra's algorithm to the DistanceMatrixArray
	write_info_to_dma(dma, time_graph, start, end, time_start, &dist, &prev);
}

/// Creates the phantom edges for the time varying graph by computing how much time we need to wait to have it avalaible again
pub fn create_phantom_edges(graph: &Graph, max_time: Tick, deleted_edges: &Vec<DeletedLink>, delta: u8) -> Vec<Vec<PhantomEdge>> {
	let mut phantoms = vec![];

	for (node, neighbours) in graph.edges.iter().enumerate() {
		let mut edges = vec![];

		// For all the edges, we create a phantom edge
		for neighbour in neighbours {
			let weight = neighbour.weight;
			let mut weights = vec![weight; max_time as usize];
			for deleted_edge in deleted_edges {
				if deleted_edge.from == node as Node && deleted_edge.to == neighbour.node {
					for t in &deleted_edge.times {
						let t = *t as usize;
						let t_with_delta = t * delta as usize;
						for td in t_with_delta..t_with_delta + delta as usize {
							if td < max_time as usize {
								weights[td] = 0;
							}
						}
					}
					let last_elem_idx = weights.len() - 1;
					if weights[last_elem_idx] == 0 {
						weights[last_elem_idx] = Tick::MAX;
					}
					for i in (0..max_time as usize - 1).rev() {
						if weights[i] == 0 {
							weights[i] = weights[i + 1].saturating_add(1);
						}
					}

					break;
				}
			}
			edges.push(PhantomEdge {
				neighbour: neighbour.node,
				weights: TimeVaryingEdge { weights },
			});
		}
		phantoms.push(edges);
	}

	return phantoms;
}

pub fn graph_to_temporal(graph: &Graph, max_time: Tick, deleted_edges: &Vec<DeletedLink>, delta: u8) -> TimeVaryingGraph {
	let phantoms = create_phantom_edges(graph, max_time, deleted_edges, delta);
	let mut edges_map = SquareMatrix::new(graph.nb_nodes() as usize, None);
	for (node, neighbours) in phantoms.iter().enumerate() {
		for neighbour in neighbours {
			edges_map[(node, neighbour.neighbour as usize)] = Some(neighbour as *const PhantomEdge);
		}
	}
	return TimeVaryingGraph {
		max_time,
		nodes: phantoms,
		edges_map,
	};
}

pub fn temporal_dijkstra(time_graph: &TimeVaryingGraph, start: Node, end: Node, time_start: Tick) -> TemporalPath {
	// Initialize the distance, previous node and visited arrays
	let max_node_index = time_graph.nb_nodes() as usize;
	let mut dist = vec![Tick::MAX; max_node_index];
	let mut prev = vec![Node::MAX; max_node_index];
	let mut visited = vec![false; max_node_index];
	dist[start as usize] = 0;
	let mut heap = BinaryHeap::new();
	heap.push(DijkstraState { node: start, distance: 0 });

	// Dijkstra's algorithm
	while let Some(DijkstraState { node, distance }) = heap.pop() {
		if visited[node as usize] {
			continue;
		}
		if node == end {
			break;
		}
		if distance + time_start >= time_graph.max_time {
			break;
		}
		visited[node as usize] = true;
		for phantom_edge in &time_graph.nodes[node as usize] {
			let edge = &phantom_edge.weights;
			let neighbour = &phantom_edge.neighbour;
			let weight = edge.weights[distance as usize + time_start as usize];
			if weight == Tick::MAX {
				continue;
			}
			let new_distance = distance + weight;
			if new_distance < dist[*neighbour as usize] {
				dist[*neighbour as usize] = new_distance;
				prev[*neighbour as usize] = node;
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
		let prev_node = prev[node as usize];
		if prev_node == Node::MAX {
			break;
		}
		let phantom_edge = time_graph.edges_map[(prev_node as usize, node as usize)].unwrap();
		let phantom_edge = unsafe { &*phantom_edge };
		let tve = &phantom_edge.weights;
		let neighbour_node = phantom_edge.neighbour;
		steps.push(Neighbour {
			node: neighbour_node,
			weight: tve.weights[(dist[prev_node as usize] + time_start as Tick) as usize],
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
