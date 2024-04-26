use std::cmp::Ordering;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::num::ParseIntError;
use std::ops::{Index, IndexMut};

use crate::distance_matrix::DistanceMatrix;
use crate::paths::{Path, SubPathsPyramid};

use std::collections::BinaryHeap;

/// A neighbour is a node and the weight of the edge to this node
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Neighbour {
	pub node: Node,
	pub weight: Tick,
}

/// A directed graph is represented as an adjacency list
/// A node can be used as an index in the adjacency list and the value is a list of its neighbours
/// A graph can only hold 65535 nodes, since the Node type is u16, however as mentioned before, Paris has 9596 nodes, so it should be enough
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
	pub edges: Vec<Vec<Neighbour>>,
}

use crate::{Node, Tick};

impl Graph {
	/// Returns the number of nodes in the graph
	pub fn nb_nodes(&self) -> Node {
		self.edges.len() as Node
	}

	/// Creates a graph from a file in the sparse row format
	/// The sparse row format is a text file with 4 lines
	/// The first line contains the number of elements in the array and the number of rows
	/// The second line contains the weights of the elements
	/// The third line contains the column index of the elements
	/// The fourth line contains the row index of the elements
	/// The graph cannot be empty
	pub fn from_sparse_row_format(filename: &str) -> Result<Self, Box<dyn Error>> {
		let file = File::open(filename)?;
		let mut reader = BufReader::new(file);

		// The first line contains the number of elements in the array
		let mut line = String::new();
		reader.read_line(&mut line)?;
		let mut sizes = line.split_whitespace();
		let size_elem_array: usize = sizes.next().ok_or("Number of elements missing")?.parse()?;
		let size_row_array: usize = sizes.next().ok_or("Number of rows missing")?.parse()?;

		// Error handling in case of bad data
		if size_elem_array == 0 || size_row_array == 0 {
			return Err("Empty graph".into());
		}
		if size_row_array > Node::MAX.into() {
			return Err("Too many nodes".into());
		}

		line.clear();

		// The second line contains the weights of the elements
		reader.read_line(&mut line)?;
		let weights = line.split_whitespace();
		let weights = weights.map(|x| x.parse::<Tick>()).collect::<Result<Vec<Tick>, ParseIntError>>()?;
		line.clear();

		// The third line contains the column index of the elements
		reader.read_line(&mut line)?;
		let col_indices = line
			.split_whitespace()
			.map(|x| x.parse::<Node>())
			.collect::<Result<Vec<Node>, ParseIntError>>()?;
		line.clear();

		// The fourth line contains the row index of the elements
		reader.read_line(&mut line)?;
		let row_indices = line
			.split_whitespace()
			.map(|x| x.parse::<usize>())
			.collect::<Result<Vec<usize>, ParseIntError>>()?;
		let mut edges = vec![vec![]; size_row_array];

		// Create the adjacency list
		for i in 0..size_row_array {
			let row_start = row_indices[i];
			let row_end = if size_row_array - 1 == i {
				size_elem_array
			}
			else {
				row_indices[i + 1]
			};
			for j in row_start..row_end {
				edges[i].push(Neighbour {
					node: col_indices[j],
					weight: weights[j],
				});
			}
		}

		Ok(Self { edges })
	}

	/// Prints the graph
	pub fn print(&self) {
		for (i, row) in self.edges.iter().enumerate() {
			print!("{i}: ");
			for neighbour in row {
				print!("{}--{}-->{} ", i, neighbour.weight, neighbour.node);
			}
			println!();
		}
	}

	/// Removes an edge between two nodes in the graph
	/// Returns None if the edge does not exist
	/// Returns Some(()) if the edge was removed
	pub fn remove_edge(&mut self, from: Node, to: Node) -> Option<()> {
		let mut i = 0;
		let from = usize::from(from);
		while i < self.edges[from].len() {
			if self.edges[from][i].node == to {
				self.edges[from].remove(i);
				return Some(());
			}
			i += 1;
		}
		return None;
	}

	/// Creates a graph from a list of neighbours
	pub fn from_neighbours(neighbours: Vec<Vec<Neighbour>>) -> Self {
		Self { edges: neighbours }
	}

	pub fn get_edge(&self, from: Node, to: Node) -> Option<&Neighbour> {
		self.edges[usize::from(from)].iter().find(|x| x.node == to)
	}
}

/// A DijkstraState is a node and the distance to this node
/// It is used as a candidate in the dijkstra algorithm
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct DijkstraState {
	pub node: Node,
	pub distance: Tick,
}

/// DijkstraState is ordered by distance
impl Ord for DijkstraState {
	fn cmp(&self, other: &Self) -> Ordering {
		other.distance.cmp(&self.distance)
	}
}

impl PartialOrd for DijkstraState {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

pub struct DijkstraInformation<T>(Vec<T>);

impl<T> Index<Node> for DijkstraInformation<T> {
	type Output = T;

	fn index(&self, index: Node) -> &Self::Output {
		&self.0[usize::from(index)]
	}
}

impl<T> IndexMut<Node> for DijkstraInformation<T> {
	fn index_mut(&mut self, index: Node) -> &mut Self::Output {
		&mut self.0[usize::from(index)]
	}
}

impl<T> DijkstraInformation<T> {
	pub fn new(default: T, size: Node) -> Self
	where
		T: Clone,
	{
		Self(vec![default; usize::from(size)])
	}
}

/// [Dijkstra's algorithm](https://en.wikipedia.org/wiki/Dijkstra%27s_algorithm) to find the shortest [`Path`] between two [`Node`]s
pub fn dijkstra(graph: &Graph, start: Node, end: Node) -> Path {
	if start == end {
		return Path {
			graph,
			start,
			steps: vec![Neighbour { node: start, weight: 1 }],
		};
	}
	// Initialize the heap with the start node, and the distances and previous vectors
	let mut heap = BinaryHeap::new();
	let mut previous = DijkstraInformation::new(None, graph.nb_nodes());
	let mut distances = DijkstraInformation::new(Tick::MAX, graph.nb_nodes());
	heap.push(DijkstraState { node: start, distance: 0 });
	distances[start] = 0;

	// While there are nodes to visit, pop the node with the smallest distance
	while let Some(DijkstraState { node, distance }) = heap.pop() {
		if node == end {
			break;
		}
		if distance > distances[node] {
			continue;
		}
		// For each neighbour, update the distance if it is smaller, and push it to the heap
		for neighbour in &graph.edges[usize::from(node)] {
			let new_distance = distance + neighbour.weight;
			if new_distance < distances[neighbour.node] {
				distances[neighbour.node] = new_distance;
				heap.push(DijkstraState {
					node: neighbour.node,
					distance: new_distance,
				});
				previous[neighbour.node] = Some(node);
			}
		}
	}

	// Reconstruct the path from the previous vector
	let mut steps = vec![];
	let mut current = end;
	while let Some(prev) = previous[current] {
		steps.push(Neighbour {
			node: current,
			weight: distances[current] - distances[prev],
		});
		current = prev;
	}
	steps.reverse();
	return Path { graph, start, steps };
}

/// Dijkstra algorithm to find the shortest distance between two nodes
/// It writes all the information found in the distance matrix, which is the path from start to end, all the subpaths composing it, and the distances between the start and all visited nodes
/// This is useful to avoid recomputing paths included in one another when computing all pairs distances
/// This is meant to be called for every node in the graph, since it does extra calculations to avoid recomputing paths
pub fn dijkstra_write(graph: &Graph, start: Node, end: Node, dst_mat: &mut DistanceMatrix) {
	if start == end {
		dst_mat[(start, end)] = 1;
		return;
	}
	// Initialize the heap with the start node, and the distances and previous vectors
	let mut heap = BinaryHeap::new();
	let mut previous = DijkstraInformation::new(None, graph.nb_nodes());
	let mut distances = DijkstraInformation::new(Tick::MAX, graph.nb_nodes());
	let mut visited = DijkstraInformation::new(false, graph.nb_nodes());
	heap.push(DijkstraState { node: start, distance: 0 });
	distances[start] = 0;

	// While there are nodes to visit, pop the node with the smallest distance
	while let Some(DijkstraState { node, distance }) = heap.pop() {
		if node == end {
			break;
		}
		if visited[node] {
			continue;
		}
		if distance > distances[node] {
			continue;
		}
		visited[node] = true;

		// We write the distance from the start to the node visited, since we know it is the shortest path, and avoids recomputing it
		if start != node {
			dst_mat[(start, node)] = distance;
		}
		for neighbour in &graph.edges[usize::from(node)] {
			if visited[neighbour.node] {
				continue;
			}
			let new_distance = distance + neighbour.weight;
			if new_distance < distances[neighbour.node] {
				distances[neighbour.node] = new_distance;
				heap.push(DijkstraState {
					node: neighbour.node,
					distance: new_distance,
				});
				previous[neighbour.node] = Some(node);
			}
		}
	}

	// Reconstruct the path from the previous vector
	let mut path = vec![];
	let mut current = end;
	if previous[current].is_none() {
		dst_mat[(start, end)] = Tick::MAX;
		return;
	}
	while let Some(prev) = previous[current] {
		path.push(Neighbour {
			node: current,
			weight: distances[current] - distances[prev],
		});
		current = prev;
	}
	path.reverse();
	let path = Path { graph, start, steps: path };

	// Compute the subpaths of the path
	let subpaths = SubPathsPyramid::from_path(graph, &path);
	for subpath in &subpaths.subpaths {
		dst_mat[(subpath.start, subpath.end)] = subpath.total_weight;
	}
}

impl Graph {
	/// Computes all the shortest paths between every pair of nodes
	pub fn all_pairs_paths(&self) -> Vec<SubPathsPyramid> {
		let mut paths = vec![];
		let mut dst_mat = DistanceMatrix::zeroed(self);
		for node in 0..self.nb_nodes() {
			for node2 in 0..self.nb_nodes() {
				// If the value is already computed, skip
				if dst_mat[(node, node2)] != 0 {
					continue;
				}

				// Compute the path from node to node2
				let path = dijkstra(self, node, node2);

				// If the path is too long, set the value to infinity (max of possible weight)
				let subpaths = SubPathsPyramid::from_path(self, &path);
				for subpath in &subpaths.subpaths {
					dst_mat[(subpath.start, subpath.end)] = subpath.total_weight;
				}
				paths.push(subpaths);
			}
		}

		return paths;
	}

	/// Computes the distance matrix of the graph
	/// If you only need the distance between every pair of nodes, this is faster than all_pairs_paths
	pub fn all_pairs_distances(&self) -> DistanceMatrix {
		let mut dst_mat = DistanceMatrix::zeroed(self);

		for node in 0..self.nb_nodes() {
			for node2 in (0..self.nb_nodes()).rev() {
				// If the value is already computed, skip
				if dst_mat[(node, node2)] != 0 {
					continue;
				}
				dijkstra_write(self, node, node2, &mut dst_mat);
			}
		}

		return dst_mat;
	}

	/// Computes the distance matrix of the graph and all the shortest paths between every pair of nodes
	/// If you need both the distance matrix and the paths, this is faster than calling all_pairs_distances and all_pairs_paths
	pub fn all_pairs_distances_and_paths(&self, max_time: Tick) -> (DistanceMatrix, Vec<SubPathsPyramid>) {
		let mut dst_mat = DistanceMatrix::zeroed(self);
		let mut paths = vec![];

		for node in 0..self.nb_nodes() {
			for node2 in 0..self.nb_nodes() {
				// If the value is already computed, skip
				if dst_mat[(node, node2)] != 0 {
					continue;
				}

				// Compute the path from node to node2
				let path = dijkstra(self, node, node2);

				// If the path is too long, set the value to infinity (max of possible weight)
				let subpaths = SubPathsPyramid::from_path(self, &path);
				for subpath in &subpaths.subpaths {
					if subpath.total_weight > max_time {
						dst_mat[(subpath.start, subpath.end)] = Tick::MAX;
						continue;
					}
					dst_mat[(subpath.start, subpath.end)] = subpath.total_weight;
				}
				paths.push(subpaths);
			}
		}

		return (dst_mat, paths);
	}
}
