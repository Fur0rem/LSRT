use crate::graph::{Graph, Time, Weight};

pub fn print_dma(dma: &[DistanceMatrix]) {
	for t in 0..dma.len() {
		println!("Time : {}", t);
		for i in 0..dma[t].distances.len() {
			for j in 0..dma[t].distances[i].len() {
				let distance_to_str = if dma[t][i][j] == Weight::MAX {
					"∞".to_string()
				}
				else {
					dma[t][i][j].to_string()
				};
				print!("{} ", distance_to_str);
			}
			println!();
		}
	}
}

/// Computes the sum of the inverse of the distances in the distance matrix
/// Which is a measure of the efficiency of the time varying graph
pub fn sum_dma(dma: &[DistanceMatrix], max_time: Time) -> (f64, u64) {
	let mut sum = 0.0;
	let mut reachables = 0;
	let distances_len = dma[0_usize].distances.len();
	for t in 0..max_time {
		for i in 0..distances_len {
			for j in 0..distances_len {
				let distance = dma[t as usize][i][j];

				// If the distance is 0 or the distance is greater than the max time -> 1/infinity = 0
				if distance + t < max_time && distance != 0 {
					sum += 1.0 / (distance as f64);
					reachables += 1;
				}
			}
		}
	}

	// Normalise the sum
	sum -= distances_len as f64;
	sum /= dma.len() as f64 * (distances_len * (distances_len - 1)) as f64;

	return (sum, reachables);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistanceMatrix {
	pub distances: Vec<Vec<Weight>>,
}

impl DistanceMatrix {
	pub fn new(graph: &Graph) -> DistanceMatrix {
		let mut distances = vec![vec![u16::MAX; graph.nb_nodes()]; graph.nb_nodes()];
		for i in 0..graph.nb_nodes() {
			distances[i][i] = 1;
		}
		DistanceMatrix { distances }
	}

	pub fn init(graph: &Graph) -> DistanceMatrix {
		let mut distances = vec![vec![0; graph.nb_nodes()]; graph.nb_nodes()];
		for i in 0..graph.nb_nodes() {
			distances[i][i] = 1;
		}
		DistanceMatrix { distances }
	}
}

impl std::ops::Index<usize> for DistanceMatrix {
	type Output = Vec<Weight>;

	fn index(&self, index: usize) -> &Self::Output {
		return &self.distances[index];
	}
}
impl std::ops::IndexMut<usize> for DistanceMatrix {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		return &mut self.distances[index];
	}
}
