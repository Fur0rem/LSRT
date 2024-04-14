use crate::graph::{Graph, Node, Tick};

#[derive(Clone, PartialEq, Eq)]
pub struct DistanceMatrix {
	// 2D square array flattened
	pub distances: Vec<Tick>,
	// size of one side of the square
	dim: usize,
}

impl std::ops::Index<(Node, Node)> for DistanceMatrix {
	type Output = Tick;

	fn index(&self, index: (Node, Node)) -> &Self::Output {
		return &self.distances[index.0 as usize * self.dim + index.1 as usize];
	}
}

impl std::ops::IndexMut<(Node, Node)> for DistanceMatrix {
	fn index_mut(&mut self, index: (Node, Node)) -> &mut Self::Output {
		return &mut self.distances[index.0 as usize * self.dim + index.1 as usize];
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
		let mut dm = DistanceMatrix { distances, dim: size };
		dm.fill_diagonal(graph);
		return dm;
	}

	pub fn zeroed(graph: &Graph) -> DistanceMatrix {
		let size = graph.nb_nodes() as usize;
		let distances = vec![0; size * size];
		let mut dm = DistanceMatrix { distances, dim: size };
		dm.fill_diagonal(graph);
		return dm;
	}

	pub fn efficiency_reachables(&self, max_time: Tick) -> (f64, u64) {
		let mut sum = 0.0;
		let mut reachables = 0;
		let distances_len = (self.distances.len() as f64).sqrt() as usize;
		for i in 0..distances_len {
			for j in 0..distances_len {
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
		sum -= distances_len as f64 * max_time as f64;
		sum /= max_time as f64 * (distances_len * (distances_len - 1)) as f64;
		reachables -= distances_len as u64 * max_time as u64;

		return (sum, reachables);
	}

	pub fn print(&self) {
		let distances_len = (self.distances.len() as f64).sqrt() as usize;
		for i in 0..distances_len {
			for j in 0..distances_len {
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

pub struct DistanceMatrixArray {
	// 3D array of distance matrices flattened
	pub distances: Vec<Tick>,
	dim: usize,
	max_time: usize,
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
	fn fill_diagonals(&mut self, graph: &Graph, max_time: Tick) {
		for t in 0..max_time {
			for i in 0..graph.nb_nodes() {
				self[(t, i, i)] = 1;
			}
		}
	}

	pub fn maxed(graph: &Graph, max_time: Tick) -> DistanceMatrixArray {
		let size = graph.nb_nodes() as usize;
		let distances = vec![Tick::MAX; size * size * max_time as usize];
		let mut dma = DistanceMatrixArray {
			distances,
			dim: size,
			max_time: max_time as usize,
		};
		dma.fill_diagonals(graph, max_time);
		return dma;
	}

	pub fn zeroed(graph: &Graph, max_time: Tick) -> DistanceMatrixArray {
		let size = graph.nb_nodes() as usize;
		let distances = vec![0; size * size * max_time as usize];
		let mut dma = DistanceMatrixArray {
			distances,
			dim: size,
			max_time: max_time as usize,
		};
		dma.fill_diagonals(graph, max_time);
		return dma;
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
					print!("{} ", distance_to_str);
				}
				println!();
			}
		}
	}
}
