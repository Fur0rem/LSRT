use crate::graph::DijkstraState;
use crate::graph::{Graph, Neighbour, Node, Time, Weight};
use crate::paths::TemporalPath;
use crate::DeletedLink;

use std::collections::BinaryHeap;

#[derive(Debug, Clone)]
pub struct TimeVaryingEdge {
	pub weights: Vec<Weight>,
}

/// A phantom edge is used to replace a blocked edge with an edge with an equivalent weight + waiting time
/// This allows us to remove the time dimension from the graph, and use classical algorithms on it
/// The weight of the edge is now a vector of weights, where we can index the weight by the time
#[derive(Debug, Clone)]
pub struct PhantomEdge {
	pub node: Node,
	pub weights: TimeVaryingEdge,
}

/// The TimeVaryingGraph is a graph where the edges can be blocked at certain times
/// The graph is represented as an adjacency list, where each edge is a PhantomEdge
#[derive(Debug, Clone)]
pub struct TimeVaryingGraph {
	pub max_time: u64,
	pub nodes: Vec<Vec<PhantomEdge>>,
}

pub fn temporal_dijkstra(time_graph: &TimeVaryingGraph, start: Node, end: Node, max_time: u64, time_start: u64) -> TemporalPath<'_> {
	// Initialize the distance, previous node and visited arrays
	let max_node_index = time_graph.nodes.len();
	let mut dist = vec![Weight::MAX; max_node_index];
	let mut prev = vec![Weight::MAX; max_node_index];
	let mut visited = vec![false; max_node_index];
	dist[start as usize] = 0;
	let mut heap = BinaryHeap::new();
	heap.push(DijkstraState { node: start, distance: 0 });

	// Dijkstra's algorithm
	while let Some(DijkstraState { node, distance }) = heap.pop() {
		if visited[node as usize] {
			continue;
		}
		visited[node as usize] = true;
		for phantom_edge in &time_graph.nodes[node as usize] {
			let edge = &phantom_edge.weights;
			let neighbour = &phantom_edge.node;
			if (distance as u64 + time_start) >= max_time {
				continue;
			}
			let weight = edge.weights[distance as usize + time_start as usize];
			if weight == Weight::MAX {
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

	// Reconstruct the path
	let mut steps = vec![];
	let mut node = end;
	while node != start {
		let prev_node = prev[node as usize];
		if prev_node == u16::MAX {
			break;
		}
		let phantom_edge = time_graph.nodes[prev_node as usize].iter().find(|x| x.node == node).unwrap();
		let tve = &phantom_edge.weights;
		let neighbour_node = &phantom_edge.node;
		steps.push(Neighbour {
			node: *neighbour_node,
			weight: tve.weights[(dist[prev_node as usize] + time_start as Weight) as usize],
		});
		node = prev_node;
	}
	steps.reverse();

	TemporalPath {
		steps,
		time_graph,
		start,
		start_time: time_start as Time,
	}
}

/// Creates the phantom edges for the time varying graph by computing how much time we need to wait to have it avalaible again
pub fn create_phantom_edges(graph: &Graph, max_time: Time, deleted_edges: &Vec<DeletedLink>, delta: u16) -> Vec<Vec<PhantomEdge>> {
	let mut phantoms = vec![];

	for (node, neighbours) in graph.edges.iter().enumerate() {
		let mut edges = vec![];

		// For all the edges, we create a phantom edge
		for neighbour in neighbours {
			let weight = &neighbour.weight;
			let mut weights = vec![];

			for t in 0..max_time {
				// Check if the edge is deleted at time t, if it is, we add 1 to the weight of the phantom edge at time t
				let mut wait = 0;
				for deleted_edge in deleted_edges {
					if deleted_edge.from == node as Node && deleted_edge.to == neighbour.node {
						while deleted_edge.times.contains(&((t + wait) / delta)) {
							wait += 1;
						}
					}
				}

				// Remove weights that are too high
				let weight_t = weight + wait;
				weights.push(if weight_t + t as Time <= max_time { weight_t } else { Weight::MAX });
			}
			edges.push(PhantomEdge {
				node: neighbour.node,
				weights: TimeVaryingEdge { weights },
			});
		}
		phantoms.push(edges);
	}

	return phantoms;
}

pub fn graph_to_temporal(graph: &Graph, max_time: Time, deleted_edges: &Vec<DeletedLink>, delta: u16) -> TimeVaryingGraph {
	let phantoms = create_phantom_edges(graph, max_time, deleted_edges, delta);
	TimeVaryingGraph {
		max_time: max_time.into(),
		nodes: phantoms,
	}
}
