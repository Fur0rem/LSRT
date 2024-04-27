//! Distance matrices, distance matrix arrays, and square matrices to compute distances and store informations from graphs.

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
	pub fn new(default_value: T, dim: usize) -> Self
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
		let node_from = usize::from(index.0);
		let node_to = usize::from(index.1);
		return &self.matrix[node_from * self.dim + node_to];
	}
}

impl IndexMut<(Node, Node)> for DistanceMatrix {
	fn index_mut(&mut self, index: (Node, Node)) -> &mut Self::Output {
		let node_from = usize::from(index.0);
		let node_to = usize::from(index.1);
		return &mut self.matrix[node_from * self.dim + node_to];
	}
}

impl DistanceMatrix {
	/// Fills the diagonal of the matrix with 1
	fn fill_diagonal(&mut self, graph: &Graph) {
		for i in 0..graph.nb_nodes() {
			self[(i, i)] = 1;
		}
	}

	/// Returns the number of nodes in the distance matrix
	pub const fn nb_nodes(&self) -> Node {
		self.dim as Node
	}

	/// Returns the distance matrix with all distances set to the maximum value and the diagonal set to 1
	pub fn maxed(graph: &Graph) -> Self {
		let size = usize::from(graph.nb_nodes());
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
		let size = usize::from(graph.nb_nodes());
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
					sum += f64::from(difference) / f64::from(distance); // Sum from 1 to max_tick of 1/distance = (max_tick - distance)/distance
					reachables += u64::from(difference);
				}
			}
		}

		// Normalise the sum
		sum -= f64::from(self.nb_nodes()) * f64::from(max_tick);
		sum /= f64::from(max_tick) * f64::from(self.nb_nodes() * (self.nb_nodes() - 1));
		reachables -= u64::from(self.nb_nodes()) * u64::from(max_tick);

		return (sum, reachables);
	}

	/// Prints the distance matrix
	pub fn print(&self) {
		for i in 0..self.nb_nodes() {
			for j in 0..self.nb_nodes() {
				let distance = self[(i, j)];
				let distance_to_str = if distance == Tick::MAX || distance == 0 {
					"∞".to_string()
				}
				else {
					distance.to_string()
				};
				// Pad the string to 4 characters
				print!("{distance_to_str:4} ");
			}
			println!();
		}
	}
}

/// A distance matrix array to stock the distance matrices for each time in a graph.
/// It is used for dynamic attacks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistanceMatrixArray {
	/// 3D (Time, From, To) array of distances flattened
	distances: Vec<Tick>,
	/// Number of nodes in the graph
	nb_nodes: Node,
	/// Maximum tick in the simulation
	max_tick: Tick,
}

impl Index<(Tick, Node, Node)> for DistanceMatrixArray {
	type Output = Tick;

	fn index(&self, index: (Tick, Node, Node)) -> &Self::Output {
		let dim = usize::from(self.nb_nodes);
		let time = usize::from(index.0);
		let node_from = usize::from(index.1);
		let node_to = usize::from(index.2);
		return &self.distances[time * dim * dim + node_from * dim + node_to];
	}
}

impl IndexMut<(Tick, Node, Node)> for DistanceMatrixArray {
	fn index_mut(&mut self, index: (Tick, Node, Node)) -> &mut Self::Output {
		let dim = usize::from(self.nb_nodes);
		let time = usize::from(index.0);
		let node_from = usize::from(index.1);
		let node_to = usize::from(index.2);
		return &mut self.distances[time * dim * dim + node_from * dim + node_to];
	}
}

impl DistanceMatrixArray {
	/// Returns the number of elements in the distance matrix array
	pub fn total_size(&self) -> usize {
		usize::from(self.nb_nodes) * usize::from(self.nb_nodes) * usize::from(self.max_tick)
	}

	/// Counts the number of zeros in the distance matrix array
	pub fn count_zeros(&self) -> usize {
		let mut nb_zeros = 0;
		for t in 0..self.max_tick {
			for i in 0..self.nb_nodes {
				for j in 0..self.nb_nodes {
					if self[(t, i, j)] == 0 {
						nb_zeros += 1;
					}
				}
			}
		}
		return nb_zeros;
	}

	/// Fills the diagonal every matrix with 1
	pub fn fill_1_on_diagonals(&mut self) {
		for t in 0..self.max_tick {
			for i in 0..self.nb_nodes {
				self[(t, i, i)] = 1;
			}
		}
	}

	/// Returns a new distance matrix array with all distances set to the maximum value and the diagonal set to 1
	pub fn maxed(nb_nodes: Node, max_tick: Tick) -> Self {
		let size = usize::from(nb_nodes);
		let max_tick_usize = usize::from(max_tick);
		let distances = vec![Tick::MAX; size * size * max_tick_usize];
		let mut dma = Self {
			distances,
			nb_nodes,
			max_tick,
		};
		dma.fill_1_on_diagonals();
		return dma;
	}

	/// Returns a new distance matrix array with all distances set to 0 and the diagonal set to 1
	pub fn zeroed(nb_nodes: Node, max_tick: Tick) -> Self {
		let size = usize::from(nb_nodes);
		let max_tick_usize = usize::from(max_tick);
		let distances = vec![0; size * size * max_tick_usize];
		let mut dma = Self {
			distances,
			nb_nodes,
			max_tick,
		};
		dma.fill_1_on_diagonals();
		return dma;
	}

	/// Returns a new distance matrix array with all distances set to the distances of the distance matrix
	pub fn propagated(dst_matrix: &DistanceMatrix, max_tick: Tick) -> Self {
		let mut new_matrix = Self::zeroed(dst_matrix.nb_nodes(), max_tick);
		for t in 0..max_tick {
			for i in 0..dst_matrix.nb_nodes() {
				for j in 0..dst_matrix.nb_nodes() {
					new_matrix[(t, i, j)] = dst_matrix[(i, j)];
				}
			}
		}
		return new_matrix;
	}

	/// Returns a new distance matrix array with all zeros,
	/// except for the distances of the distance matrix which would be unreachable at the max tick
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

	/// Computes the efficiency and reachables of the distance matrix array
	pub fn efficiency_reachables(&self, max_tick: Tick) -> (f64, u64) {
		let mut sum = 0.0;
		let mut reachables = 0;
		for t in 0..self.max_tick {
			for i in 0..self.nb_nodes {
				for j in 0..self.nb_nodes {
					let distance = self[(t, i, j)];
					// If the distance is 0 or the distance is greater than the max time -> 1/infinity = 0
					if distance.saturating_add(t) <= max_tick && distance != 0 {
						sum += 1.0 / f64::from(distance);
						reachables += 1;
					}
				}
			}
		}
		// Normalise the sum
		sum -= f64::from(self.nb_nodes) * f64::from(max_tick);
		sum /= f64::from(max_tick) * f64::from(self.nb_nodes * (self.nb_nodes - 1));
		reachables -= u64::from(self.nb_nodes) * u64::from(max_tick);

		return (sum, reachables);
	}

	/// Prints the distance matrix array
	pub fn print(&self) {
		for t in 0..self.max_tick {
			println!("Time : {t}");
			for i in 0..self.nb_nodes {
				for j in 0..self.nb_nodes {
					let distance = self[(t, i, j)];
					let distance_to_str = if distance == Tick::MAX {
						"∞".to_string()
					}
					else {
						distance.to_string()
					};
					// Pad the string to 4 characters
					print!("{distance_to_str:4} ");
				}
				println!();
			}
		}
	}
}
