use std::{collections::HashSet, io::BufRead};

use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};

use crate::{
	distance_matrix::{DistanceMatrixArray, SquareMatrix},
	graph::{dijkstra, Graph, Node, Tick},
	paths::{Path, TemporalSubPathsPyramid},
	time_varying_graph::{self, graph_to_temporal, temporal_dijkstra, temporal_dijkstra_write},
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

	/*pub fn efficiency(&self) -> Vec<(&str, (f64, u64))> {
		// First, compute the paths for the unattacked graph
		let max_time = self.series[0].1.max_time(); // The max time is the same for all attacks
											//let (dst_mat, paths) = self.graph.all_pairs_distances_and_paths(max_time);
		let dst_mat = self.graph.all_pairs_distances(max_time);

		// Then, we recopy the distance matrix to all the attacks
		let mut dmas = vec![DistanceMatrixArray::propagated(&dst_mat, max_time); self.series.len()];

		let results = dmas
			.iter_mut()
			.enumerate()
			.map(|(e, dma)| {
				// We invalidate all the paths except those who took more than max_time to reach the destination
				for i in 0..self.graph.nb_nodes() {
					for j in 0..self.graph.nb_nodes() {
						let weight = dst_mat[(i, j)];
						let max_time_reachable = max_time.saturating_sub(weight);
						for t in 0..max_time_reachable {
							dma[(t, i, j)] = 0;
						}
					}
				}

				dma.fill_1_on_diagonals();

				// Then, we recompute the paths for the attacked graph
				let attack = &self.series[e].1;
				(
					self.series[e].0,
					benchmark!("recomputing everything", {
						let time_graph = graph_to_temporal(self.graph, max_time, &attack.blocked_links(), attack.delta());
						for t in 0..time_graph.max_time {
							for i in 0..time_graph.nb_nodes() {
								// Doing the j in reverse seems to improve slightly the performance
								// Probably because nodes that are far in real life apart also have far apart indexes
								for j in (0..time_graph.nb_nodes()).rev() {
									// Not 0 -> already computed
									if dma[(t, i, j)] != 0 {
										continue;
									}
									temporal_dijkstra_write(&time_graph, i as Node, j as Node, t, dma);
								}
							}
							//println!("{}/{}", t, self.max_time);
						}
						//dma.print(max_time);
						dma.efficiency_reachables(max_time)
					}),
				)
			})
			.collect();

		return results;
	}*/

	pub fn efficiency(&self) -> Vec<(&str, (f64, u64))> {
		// First, compute the paths for the unattacked graph
		let max_time = self.series[0].1.max_time(); // The max time is the same for all attacks
		let (dst_mat, paths) = self.graph.all_pairs_distances_and_paths(max_time);
		let mut dmas = vec![DistanceMatrixArray::propagated(&dst_mat, max_time); self.series.len()];

		let results = dmas
			.par_iter_mut()
			.enumerate()
			.map(|(e, dma)| {
				// Then, we recompute the paths for the attacked graph
				let attack = &self.series[e].1;
				let mut already_done = SquareMatrix::new(self.graph.nb_nodes() as usize, false);
				let mut times_matrix = SquareMatrix::new(self.graph.nb_nodes() as usize, None);
				// push the blocked links times
				let blocked_links = attack.blocked_links();
				for link in blocked_links.iter() {
					times_matrix[(link.from as usize, link.to as usize)] = Some(&link.times);
				}

				benchmark!("unvalidating paths", {
					for path in paths.iter() {
						path.unvalidate_path(dma, &attack, &mut already_done, &times_matrix);
					}
				});

				// Count the number of 0's
				/*let mut count = 0;
				for t in 0..max_time {
					for i in 0..self.graph.nb_nodes() {
						for j in 0..self.graph.nb_nodes() {
							if dma[(t, i, j)] == 0 {
								count += 1;
							}
						}
					}
				}

				let backup_dma = dma.clone();

				println!(
					"Zeroes: {}/{}",
					count,
					self.graph.nb_nodes() as usize * self.graph.nb_nodes() as usize * max_time as usize
				);*/

				(
					self.series[e].0,
					benchmark!("recomputing everything", {
						let time_graph = graph_to_temporal(self.graph, max_time, &attack.blocked_links(), attack.delta());
						for t in 0..time_graph.max_time {
							for i in 0..time_graph.nb_nodes() {
								// Doing the j in reverse seems to improve slightly the performance
								// Probably because nodes that are far in real life apart also have far apart indexes
								for j in (0..time_graph.nb_nodes()).rev() {
									// Not 0 -> already computed
									if dma[(t, i, j)] != 0 {
										continue;
									}
									temporal_dijkstra_write(&time_graph, i as Node, j as Node, t, dma);
								}
							}
						}
						dma.efficiency_reachables(max_time)
					}),
				)
			})
			.collect();

		return results;
	}
}
