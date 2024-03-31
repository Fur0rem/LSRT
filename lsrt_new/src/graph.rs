use std::cmp::Ordering;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::paths::{Path, SubPathsPyramid};

use std::collections::BinaryHeap;

use crate::distance_matrix::DistanceMatrix;

pub type Node = u16;
pub type Weight = u16;
pub type Time = u16;

/// A neighbour is a node and the weight of the edge to this node
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Neighbour {
	pub node: Node,
	pub weight: Weight,
}

/// A graph is represented as an adjacency list
/// The index of the vector is the node, and the value is a vector of Neighbour
/// The Neighbour contains the node and the weight of the edge to this node
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
	pub edges: Vec<Vec<Neighbour>>,
}

impl Graph {

    /// Returns the number of nodes in the graph
    pub fn nb_nodes(&self) -> usize {
        self.edges.len()
    }

	// TODO : change the python file to make it a adjacency list to begin with
	pub fn from_sparse_row_format(filename: &str) -> Result<Graph, Box<dyn Error>> {
		let file = File::open(filename)?;
		let mut reader = BufReader::new(file);

		// The first line contains the number of elements in the array
		let mut line = String::new();
		reader.read_line(&mut line)?;
		let mut sizes = line.split_whitespace();
		let size_elem_array: usize = sizes.next().unwrap().parse()?;
		let size_row_array: usize = sizes.next().unwrap().parse()?;
		line.clear();

		// The second line contains the weights of the elements
		reader.read_line(&mut line)?;
		let weights = line.split_whitespace();
		let weights = weights.map(|x| x.parse::<Weight>().unwrap()).collect::<Vec<Weight>>();
		line.clear();

		// The third line contains the column index of the elements
		reader.read_line(&mut line)?;
		let col_index = line.split_whitespace();
		let col_index = col_index.map(|x| x.parse::<Node>().unwrap()).collect::<Vec<Node>>();
		line.clear();

		// The fourth line contains the row index of the elements
		reader.read_line(&mut line)?;
		let row_index = line.split_whitespace();
		let row_index = row_index.map(|x| x.parse::<usize>().unwrap()).collect::<Vec<usize>>();
		let mut edges = vec![vec![]; size_row_array];

		// Create the adjacency list
		for i in 0..size_row_array {
			let row_start = row_index[i];
			let row_end = if size_row_array - 1 == i {
				size_elem_array
			}
			else {
				row_index[i + 1]
			};
			for j in row_start..row_end {
				edges[i].push(Neighbour {
					node: col_index[j],
					weight: weights[j],
				});
			}
		}

		Ok(Graph {
			edges,
		})
	}

	pub fn print(&self) {
		for (i, row) in self.edges.iter().enumerate() {
			print!("{}: ", i);
			for neighbour in row {
				print!("{}--{}-->{} ", i, neighbour.weight, neighbour.node);
			}
			println!();
		}
	}

	pub fn remove_link(&mut self, from: Node, to: Node) {
		let mut i = 0;
		while i < self.edges[from as usize].len() {
			if self.edges[from as usize][i].node == to {
				self.edges[from as usize].remove(i);
				return;
			}
			i += 1;
		}
	}

	pub fn from_neighbours(neighbours: Vec<Vec<(Node, Weight)>>) -> Graph {
		let mut edges = vec![];
		for neighbours_of_node in neighbours {
			let mut node_edges = vec![];
			for (node, weight) in neighbours_of_node {
				node_edges.push(Neighbour { node, weight });
			}
			edges.push(node_edges);
		}
		Graph {
			edges,
		}
	}
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct DijkstraState {
	pub node: Node,
	pub distance: u16,
}

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

// FIXME : probably wrong cause distance is unused
pub fn dijkstra(graph: &Graph, start: Node, end: Node) -> Path {
	let mut dist = vec![u16::MAX; graph.nb_nodes()];
	let mut prev = vec![u16::MAX; graph.nb_nodes()];
	let mut visited = vec![false; graph.nb_nodes()];
	dist[start as usize] = 0;
	let mut heap = BinaryHeap::new();
	heap.push(DijkstraState { node: start, distance: 0 });

	while let Some(DijkstraState { node, distance: _ }) = heap.pop() {
		if visited[node as usize] {
			continue;
		}
		visited[node as usize] = true;
		for neighbour in &graph.edges[node as usize] {
			if dist[node as usize] == u16::MAX {
				continue;
			}
			let alt = dist[node as usize] + neighbour.weight;
			if alt < dist[neighbour.node as usize] {
				dist[neighbour.node as usize] = alt;
				prev[neighbour.node as usize] = node;
				heap.push(DijkstraState {
					node: neighbour.node,
					distance: alt,
				});
			}
		}
	}
	let mut steps = vec![];
	let mut node = end;
	while node != start {
		let prev_node = prev[node as usize];
		if prev_node == u16::MAX {
			break;
		}
		let weight = dist[node as usize] - dist[prev_node as usize];
		steps.push(Neighbour { node, weight });
		node = prev_node;
	}
	steps.reverse();
	Path {
		steps,
		start: start as Node,
		graph,
	}
}

impl Graph {

    /// Computes the distance matrix of the graph
    /// The distance matrix is initialised with 0's and the diagonal is filled with 1's
    /// 0 means the value has not been computed yet
    /// It does dijkstra for each pair of nodes, and stores the result in a matrix
    /// For each path computed, it also stores all the subpaths present in the path
    /// This should reduce the number of dijkstra calls, since a path of length n has n(n+1)/2 subpaths
	pub fn all_pairs_distances(&self, max_time: Weight) -> DistanceMatrix {
		let mut dst_mat = DistanceMatrix::init(self);

		for node in 0..self.nb_nodes() {
			for node2 in 0..self.nb_nodes() {

                // If the value is already computed, skip
				if dst_mat[node][node2] != 0 {
					continue;
				}

                // Compute the path from node to node2
				let path = dijkstra(self, node as Node, node2 as Node);

                // If the path is too long, set the value to infinity (max of possible weight)
				let subpaths = SubPathsPyramid::from_path(self, &path);
				for subpath in &subpaths.subpaths {
					if subpath.total_weight > max_time {
						dst_mat[subpath.start as usize][subpath.end as usize] = u16::MAX;
						continue;
					}
					dst_mat[subpath.start as usize][subpath.end as usize] = subpath.total_weight;
				}
			}
		}

		return dst_mat;
	}
}
