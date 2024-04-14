use std::io::BufRead;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
	distance_matrix::DistanceMatrixArray,
	graph::{Graph, Node, Tick},
	time_varying_graph::{self, graph_to_temporal, temporal_dijkstra_write},
	Attack, DynamicAttack,
};

macro_rules! benchmark {
	($name:expr, $block:expr) => {{
		let start = std::time::Instant::now();
		let result = $block;
		let duration = start.elapsed();
		println!("Time elapsed in {}() is: {:?}", $name, duration);
		result
	}};
}

pub struct AttackSeries<'a> {
	pub series: Vec<(&'a str, DynamicAttack)>,
	pub graph: &'a Graph,
}

impl<'a> AttackSeries<'a> {
	pub fn from_directory(directory: &str, files: &[&'a str], graph: &'a Graph) -> AttackSeries<'a> {
		let series = files
			.iter()
			.map(|budget| {
				let file_name = format!("{}{}", directory, budget);
				// Open the file with a reader that can read from it
				let file = std::fs::File::open(file_name).unwrap();
				let mut reader = std::io::BufReader::new(file);
				// read the first line
				let mut line = String::new();
				reader.read_line(&mut line).unwrap();
				// skip it
				let attack = DynamicAttack::from_file(&mut reader);
				(*budget, attack)
			})
			.collect();
		AttackSeries { series, graph }
	}

	/*pub fn efficiency(&self) -> Vec<(&str, f64, u64)> {
		rayon::ThreadPoolBuilder::new().num_threads(8).build_global().unwrap();
		self.series
			.par_iter()
			.map(|(budget, attack)| {
				let (efficiency, reachables) = attack.efficiency(self.graph);
				(*budget, efficiency, reachables)
			})
			.collect()
	}*/

	pub fn efficiency(&self) -> Vec<(&str, (f64, u64))> {
		// First, compute the paths for the unattacked graph
		let max_time = self.series[0].1.max_time(); // The max time is the same for all attacks
		let (dst_mat, paths) = self.graph.all_pairs_distances_and_paths(max_time);

		let efficiency_unattacked = dst_mat.efficiency_reachables(max_time);
		println!("Efficiency unattacked: {:?}", efficiency_unattacked);

		// Then, we fill the distance matrix array with the paths that are not attacked
		let mut distance_matrix_arrays = vec![DistanceMatrixArray::propagated(&dst_mat, max_time); self.series.len()];

		let mut efficiencies_unattacked = distance_matrix_arrays[0].efficiency_reachables(max_time);
		println!("Efficiency unattacked: {:?}", efficiencies_unattacked);

		//let mut dmas = vec![DistanceMatrixArray::propagated(&dst_mat, max_time); self.series.len()];

		benchmark!("invalidating paths", {
			for (i, (budget, attack)) in self.series.iter().enumerate() {
				for path in &paths {
					// check if any of the leaves is in the attack
					let mut attacked = false;
					for leaf in path.leaves() {
						for blocked_link in &*attack.blocked_links() {
							if blocked_link.from == leaf.start && blocked_link.to == leaf.end {
								attacked = true;
								break;
							}
						}
						if attacked {
							break;
						}
					}

					if attacked {
						for subpath in &path.subpaths {
							for t in 0..max_time {
								distance_matrix_arrays[i][(t, subpath.start, subpath.end)] = 0;
							}
						}
					}
				}
			}
			/*for e in 0..self.series.len() {
				for t in 0..max_time {
					for i in 0..self.graph.nb_nodes() {
						for j in 0..self.graph.nb_nodes() {
							if distance_matrix_arrays[e][(t, i, j)] != Tick::MAX {
								distance_matrix_arrays[e][(t, i, j)] = 0;
							}
						}
					}
				}
			}*/
		});

		// Then, we compute the efficiency for each attack
		for (e, (budget, attack)) in self.series.iter().enumerate() {
			/*benchmark!("invalidating paths", {
				for path in &paths {
					path.invalidate_path(&mut distance_matrix_arrays[e], attack);
				}
			});*/
			// count how many zeros there are in the distance matrix array
			let mut zeros = 0;
			for t in 0..max_time {
				for i in 0..self.graph.nb_nodes() {
					for j in 0..self.graph.nb_nodes() {
						if distance_matrix_arrays[e][(t, i, j)] == 0 {
							zeros += 1;
						}
					}
				}
			}
			println!(
				"Zeros: {}/{} for budget {}",
				zeros,
				distance_matrix_arrays[e].distances.len(),
				budget
			);
			//let budget: usize = budget.parse().unwrap();
			/*let time_varying_graph = graph_to_temporal(self.graph, max_time, &attack.blocked_links(), attack.delta());
			benchmark!("Recomputing stuff", {
				for t in 0..max_time {
					for i in 0..self.graph.nb_nodes() {
						for j in 0..self.graph.nb_nodes() {
							//if distance_matrix_arrays[e][(t, i, j)] == 0 {
							// compute the shortest path
							temporal_dijkstra_write(&time_varying_graph, i, j, t, &mut distance_matrix_arrays[e]);
							//}
						}
					}
				}
			});*/
		}

		let mut dmas = distance_matrix_arrays;

		let mut results = Vec::new();
		for (e, (budget, attack)) in self.series.iter().enumerate() {
			results.push((
				*budget,
				/*benchmark!("recomputing", {
					let time_varying_graph = graph_to_temporal(self.graph, max_time, &attack.blocked_links(), attack.delta());
					for t in 0..max_time {
						for i in 0..self.graph.nb_nodes() {
							for j in 0..self.graph.nb_nodes() {
								if distance_matrix_arrays[e][(t, i, j)] == 0 {
									// compute the shortest path
									temporal_dijkstra_write(&time_varying_graph, i, j, t, &mut distance_matrix_arrays[e]);
								}
							}
						}
					}
					distance_matrix_arrays[e].efficiency_reachables(max_time)
				}),*/
				//benchmark!("recomputing everythin", attack.efficiency(self.graph)),
				benchmark!("recomputing everything", {
					let time_graph = graph_to_temporal(self.graph, max_time, &attack.blocked_links(), attack.delta());
					for t in 0..time_graph.max_time {
						for i in 0..time_graph.nb_nodes() {
							// Doing the j in reverse seems to improve slightly the performance
							// Probably because nodes that are far in real life apart also have far apart indexes
							for j in (0..time_graph.nb_nodes()).rev() {
								// Not 0 -> already computed
								if dmas[e][(t, i, j)] != 0 {
									continue;
								}
								temporal_dijkstra_write(&time_graph, i as Node, j as Node, t, &mut dmas[e]);
							}
						}
						//println!("{}/{}", t, self.max_time);
					}
					dmas[e].efficiency_reachables(max_time)
				}),
			));
		}

		/*let results = self
		.series
		.par_iter()
		.map(|(budget, attack)| {
			let time_varying_graph = graph_to_temporal(self.graph, max_time, &attack.blocked_links(), attack.delta());
			let mut distance_matrix_array = DistanceMatrixArray::propagated(&dst_mat, max_time);
			for path in &paths {
				path.invalidate_path(&mut distance_matrix_array, attack);
			}
			for t in 0..max_time {
				for i in 0..self.graph.nb_nodes() {
					for j in 0..self.graph.nb_nodes() {
						if distance_matrix_array[(t, i, j)] == 0 {
							temporal_dijkstra_write(&time_varying_graph, i, j, t, &mut distance_matrix_array);
						}
					}
				}
			}
			(*budget, distance_matrix_array.efficiency_reachables(max_time))
		})
		.collect();*/

		return results;
	}
}
