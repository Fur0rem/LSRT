//! Distance matrices and distance matrix arrays to compute distances in graphs.

use crate::graph::Graph;
use crate::{Node, Tick};
use std::ops::{Index, IndexMut};

/// A matrix of size NxN
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SquareMatrix<T> {
	/// 2D square array flattened
	pub matrix: Vec<T>,
	/// size of one side of the square
	dim: usize,
}

impl<T> SquareMatrix<T> {
	/// Returns the dimension of the square matrix
	pub const fn dim(&self) -> usize {
		self.dim
	}

	/// Returns a new square matrix with the default value of the type
	pub fn default(dim: usize) -> Self
	where
		T: Default + Clone,
	{
		let size = dim * dim;
		let matrix = vec![T::default(); size];
		return Self { matrix, dim };
	}

	/// Returns a new square matrix a value specified by the user
	pub fn new(dim: usize, default_value: T) -> Self
	where
		T: Clone,
	{
		let size = dim * dim;
		let matrix = vec![default_value; size];
		return Self { matrix, dim };
	}
}

impl<T> Index<(usize, usize)> for SquareMatrix<T> {
	type Output = T;

	fn index(&self, index: (usize, usize)) -> &Self::Output {
		return &self.matrix[index.0 * self.dim + index.1];
	}
}

impl<T> IndexMut<(usize, usize)> for SquareMatrix<T> {
	fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
		return &mut self.matrix[index.0 * self.dim + index.1];
	}
}

/// A distance matrix to stock the distances between nodes in a graph.
pub type DistanceMatrix = SquareMatrix<Tick>;

impl Index<(Node, Node)> for DistanceMatrix {
	type Output = Tick;

	fn index(&self, index: (Node, Node)) -> &Self::Output {
		return &self.matrix[index.0 as usize * self.dim + index.1 as usize];
	}
}

impl IndexMut<(Node, Node)> for DistanceMatrix {
	fn index_mut(&mut self, index: (Node, Node)) -> &mut Self::Output {
		return &mut self.matrix[index.0 as usize * self.dim + index.1 as usize];
	}
}

impl DistanceMatrix {
	/// Fills the diagonal of the matrix with 1
	fn fill_diagonal(&mut self, graph: &Graph) {
		for i in 0..graph.nb_nodes() {
			self[(i, i)] = 1;
		}
	}

	pub fn nb_nodes(&self) -> Node {
		self.dim as Node
	}

	/// Returns the distance matrix with all distances set to the maximum value and the diagonal set to 1
	pub fn maxed(graph: &Graph) -> Self {
		let size = graph.nb_nodes() as usize;
		let distances = vec![Tick::MAX; size * size];
		let mut dm = DistanceMatrix {
			matrix: distances,
			dim: size,
		};
		dm.fill_diagonal(graph);
		return dm;
	}

	/// Returns the distance matrix with all distances set to 0 and the diagonal set to 1
	pub fn zeroed(graph: &Graph) -> Self {
		let size = graph.nb_nodes() as usize;
		let distances = vec![0; size * size];
		let mut dm = DistanceMatrix {
			matrix: distances,
			dim: size,
		};
		dm.fill_diagonal(graph);
		return dm;
	}

	/// Computes the efficiency and reachables of the distance matrix
	/// It's treated as a distance matrix array with all the distance matrices being this one.
	pub fn efficiency_reachables(&self, max_tick: Tick) -> (f64, u64) {
		let mut sum = 0.0;
		let mut reachables = 0;
		for i in 0..self.nb_nodes() {
			for j in 0..self.nb_nodes() {
				let distance = self[(i, j)];
				// If the distance is 0 or the distance is greater than the max time -> 1/infinity = 0
				if distance < max_tick && distance != 0 {
					let difference = max_tick - distance + 1;
					sum += (difference as f64) / (distance as f64); // Sum from 1 to max_tick of 1/distance = (max_tick - distance)/distance
					reachables += difference as u64;
				}
			}
		}

		// Normalise the sum
		sum -= self.nb_nodes() as f64 * max_tick as f64;
		sum /= max_tick as f64 * (self.nb_nodes() * (self.nb_nodes() - 1)) as f64;
		reachables -= self.nb_nodes() as u64 * max_tick as u64;

		return (sum, reachables);
	}

	/// Prints the distance matrix
	pub fn print(&self) {
		for i in 0..self.nb_nodes() {
			for j in 0..self.nb_nodes() {
				let distance = self[(i, j)];
				let distance_to_str = if distance == Tick::MAX || distance == 0 {
					"∞".to_string()
				} else {
					distance.to_string()
				};
				print!("{} ", distance_to_str);
			}
			println!();
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistanceMatrixArray {
	// 3D array of distance matrices flattened
	pub distances: Vec<Tick>,
	nb_nodes: Node,
	pub max_tick: Tick,
}

impl std::ops::Index<(Tick, Node, Node)> for DistanceMatrixArray {
	type Output = Tick;

	fn index(&self, index: (Tick, Node, Node)) -> &Self::Output {
		let dim = usize::from(self.nb_nodes);
		let time = usize::from(index.0);
		let node_from = usize::from(index.1);
		let node_to = usize::from(index.2);
		return &self.distances[time * dim * dim + node_from * dim + node_to];
	}
}

impl std::ops::IndexMut<(Tick, Node, Node)> for DistanceMatrixArray {
	fn index_mut(&mut self, index: (Tick, Node, Node)) -> &mut Self::Output {
		let dim = usize::from(self.nb_nodes);
		let time = usize::from(index.0);
		let node_from = usize::from(index.1);
		let node_to = usize::from(index.2);
		return &mut self.distances[time * dim * dim + node_from * dim + node_to];
	}
}

impl DistanceMatrixArray {
	pub fn fill_1_on_diagonals(&mut self) {
		for t in 0..self.max_tick as Tick {
			for i in 0..self.nb_nodes as Node {
				self[(t, i, i)] = 1;
			}
		}
	}

	pub fn maxed(nb_nodes: Node, max_tick: Tick) -> Self {
		let size = nb_nodes as usize;
		let distances = vec![Tick::MAX; size * size * max_tick as usize];
		let mut dma = Self {
			distances,
			nb_nodes,
			max_tick,
		};
		dma.fill_1_on_diagonals();
		return dma;
	}

	pub fn zeroed(nb_nodes: Node, max_tick: Tick) -> Self {
		let size = nb_nodes as usize;
		let distances = vec![0; size * size * max_tick as usize];
		let mut dma = Self {
			distances,
			nb_nodes,
			max_tick,
		};
		dma.fill_1_on_diagonals();
		return dma;
	}

	pub fn propagated(dst_matrix: &DistanceMatrix, max_tick: Tick) -> Self {
		let mut new_matrix = Self::zeroed(dst_matrix.nb_nodes(), max_tick);
		for t in 0..max_tick {
			for i in 0..dst_matrix.nb_nodes() {
				for j in 0..dst_matrix.nb_nodes() {
					new_matrix[(t, i as Node, j as Node)] = dst_matrix[(i, j)];
				}
			}
		}
		return new_matrix;
	}

	pub fn propagated_and_unvalidated(dst_matrix: &DistanceMatrix, max_tick: Tick) -> Self {
		let mut dma = Self::zeroed(dst_matrix.nb_nodes(), max_tick);
		for i in 0..dst_matrix.nb_nodes() {
			for j in 0..dst_matrix.nb_nodes() {
				let weight = dst_matrix[(i, j)];
				let max_tick_reachable = max_tick.saturating_sub(weight).saturating_add(2);
				for t in max_tick_reachable..max_tick {
					dma[(t, i, j)] = weight;
				}
			}
		}
		return dma;
	}

	pub fn efficiency_reachables(&self, max_tick: Tick) -> (f64, u64) {
		let mut sum = 0.0;
		let mut reachables = 0;
		for t in 0..self.max_tick {
			for i in 0..self.nb_nodes {
				for j in 0..self.nb_nodes {
					let distance = self[(t, i, j)];
					// If the distance is 0 or the distance is greater than the max time -> 1/infinity = 0
					if distance.saturating_add(t) <= max_tick && distance != 0 {
						sum += 1.0 / (distance as f64);
						reachables += 1;
					}
				}
			}
		}
		// Normalise the sum
		sum -= self.nb_nodes as f64 * max_tick as f64;
		sum /= max_tick as f64 * (self.nb_nodes * (self.nb_nodes - 1)) as f64;
		reachables -= self.nb_nodes as u64 * max_tick as u64;

		return (sum, reachables);
	}

	pub fn print(&self) {
		for t in 0..self.max_tick {
			println!("Time : {}", t);
			for i in 0..self.nb_nodes {
				for j in 0..self.nb_nodes {
					let distance = self[(t, i, j)];
					let distance_to_str = if distance == Tick::MAX {
						"∞".to_string()
					} else {
						distance.to_string()
					};
					// Pad the string to 4 characters
					print!("{:4} ", distance_to_str);
				}
				println!();
			}
		}
	}
}
