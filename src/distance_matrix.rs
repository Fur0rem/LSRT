use crate::graph::{Graph, Node, Tick};
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SquareMatrix<T> {
	// 2D square array flattened
	pub matrix: Vec<T>,
	// size of one side of the square
	dim: usize,
}

impl<T> SquareMatrix<T> {
	pub fn dim(&self) -> usize {
		self.dim
	}

	pub fn default(dim: usize) -> SquareMatrix<T>
	where
		T: Default + Clone,
	{
		let size = dim * dim;
		let matrix = vec![T::default(); size];
		return SquareMatrix { matrix, dim };
	}

	pub fn new(dim: usize, default_value: T) -> SquareMatrix<T>
	where
		T: Clone,
	{
		let size = dim * dim;
		let matrix = vec![default_value; size];
		return SquareMatrix { matrix, dim };
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
	fn fill_diagonal(&mut self, graph: &Graph) {
		for i in 0..graph.nb_nodes() {
			self[(i, i)] = 1;
		}
	}

	pub fn maxed(graph: &Graph) -> DistanceMatrix {
		let size = graph.nb_nodes() as usize;
		let distances = vec![Tick::MAX; size * size];
		let mut dm = DistanceMatrix {
			matrix: distances,
			dim: size,
		};
		dm.fill_diagonal(graph);
		return dm;
	}

	pub fn zeroed(graph: &Graph) -> DistanceMatrix {
		let size = graph.nb_nodes() as usize;
		let distances = vec![0; size * size];
		let mut dm = DistanceMatrix {
			matrix: distances,
			dim: size,
		};
		dm.fill_diagonal(graph);
		return dm;
	}

	pub fn efficiency_reachables(&self, max_time: Tick) -> (f64, u64) {
		let mut sum = 0.0;
		let mut reachables = 0;
		for i in 0..self.dim {
			for j in 0..self.dim {
				let distance = self[(i as Node, j as Node)];
				// If the distance is 0 or the distance is greater than the max time -> 1/infinity = 0
				if distance <= max_time && distance != 0 {
					let difference = max_time - distance + 1;
					sum += (difference as f64) / (distance as f64); // Sum from 1 to max_time of 1/distance = (max_time - distance)/distance
					reachables += difference as u64;
				}
			}
		}

		// Normalise the sum
		sum -= self.dim as f64 * max_time as f64;
		sum /= max_time as f64 * (self.dim * (self.dim - 1)) as f64;
		reachables -= self.dim as u64 * max_time as u64;

		return (sum, reachables);
	}

	pub fn print(&self) {
		for i in 0..self.dim {
			for j in 0..self.dim {
				let distance = self[(i as Node, j as Node)];
				let distance_to_str = if distance == Tick::MAX {
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
	dim: usize,
	pub max_time: usize,
}

impl std::ops::Index<(Tick, Node, Node)> for DistanceMatrixArray {
	type Output = Tick;

	fn index(&self, index: (Tick, Node, Node)) -> &Self::Output {
		return &self.distances[index.0 as usize * self.dim * self.dim + index.1 as usize * self.dim + index.2 as usize];
	}
}

impl std::ops::IndexMut<(Tick, Node, Node)> for DistanceMatrixArray {
	fn index_mut(&mut self, index: (Tick, Node, Node)) -> &mut Self::Output {
		return &mut self.distances[index.0 as usize * self.dim * self.dim + index.1 as usize * self.dim + index.2 as usize];
	}
}

impl DistanceMatrixArray {
	pub fn fill_1_on_diagonals(&mut self) {
		for t in 0..self.max_time as Tick {
			for i in 0..self.dim as Node {
				self[(t, i, i)] = 1;
			}
		}
	}

	pub fn maxed(nb_nodes: Node, max_time: Tick) -> Self {
		let size = nb_nodes as usize;
		let distances = vec![Tick::MAX; size * size * max_time as usize];
		let mut dma = DistanceMatrixArray {
			distances,
			dim: size,
			max_time: max_time as usize,
		};
		dma.fill_1_on_diagonals();
		return dma;
	}

	pub fn zeroed(nb_nodes: Node, max_time: Tick) -> Self {
		let size = nb_nodes as usize;
		let distances = vec![0; size * size * max_time as usize];
		let mut dma = DistanceMatrixArray {
			distances,
			dim: size,
			max_time: max_time as usize,
		};
		dma.fill_1_on_diagonals();
		return dma;
	}

	pub fn propagated(dst_matrix: &DistanceMatrix, max_time: Tick) -> Self {
		let mut new_matrix = DistanceMatrixArray::zeroed(dst_matrix.dim() as Node, max_time);
		for t in 0..max_time {
			for i in 0..dst_matrix.dim() {
				for j in 0..dst_matrix.dim() {
					new_matrix[(t, i as Node, j as Node)] = dst_matrix[(i as Node, j as Node)];
				}
			}
		}
		return new_matrix;
	}

	pub fn efficiency_reachables(&self, max_time: Tick) -> (f64, u64) {
		let mut sum = 0.0;
		let mut reachables = 0;
		let distances_len = (self.distances.len() as f64 / max_time as f64).sqrt() as usize;
		for t in 0..max_time {
			for i in 0..distances_len {
				for j in 0..distances_len {
					let distance = self[(t, i as Node, j as Node)];
					// If the distance is 0 or the distance is greater than the max time -> 1/infinity = 0
					if distance + t <= max_time && distance != 0 {
						sum += 1.0 / (distance as f64);
						reachables += 1;
					}
				}
			}
		}
		// Normalise the sum
		sum -= distances_len as f64 * max_time as f64;
		sum /= max_time as f64 * (distances_len * (distances_len - 1)) as f64;
		reachables -= distances_len as u64 * max_time as u64;

		return (sum, reachables);
	}

	pub fn print(&self, max_time: Tick) {
		let distances_len = (self.distances.len() as f64 / max_time as f64).sqrt() as usize;
		for t in 0..max_time {
			println!("Time : {}", t);
			for i in 0..distances_len {
				for j in 0..distances_len {
					let distance = self[(t, i as Node, j as Node)];
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
