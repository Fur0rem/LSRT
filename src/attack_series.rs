use std::{collections::HashSet, io::BufRead};

use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};

use crate::{
	distance_matrix::{DistanceMatrixArray, SquareMatrix},
	graph::{dijkstra, Graph},
	paths::{Path, TemporalSubPathsPyramid},
	time_varying_graph::{self, graph_to_temporal, temporal_dijkstra, temporal_dijkstra_write},
	Attack, DynamicAttack, StaticAttack,
};
use crate::{Node, Tick};

use crate::benchmark;

pub struct DynamicAttackSeries<'a> {
	pub series: Vec<(&'a str, DynamicAttack)>,
	pub graph: &'a Graph,
}

impl<'a> DynamicAttackSeries<'a> {
	/*pub fn from_directory(directory: &str, files: &[&'a str], graph: &'a Graph) -> DynamicAttackSeries<'a> {
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
		DynamicAttackSeries { series, graph }
	}

	pub fn efficiency(&self) -> Vec<(&str, (f64, u64))> {
		// First, compute the paths for the unattacked graph
		let max_tick = self.series[0].1.max_tick(); // The max time is the same for all attacks
											//let (dst_mat, paths) = self.graph.all_pairs_distances_and_paths(max_tick);
		let dst_mat = self.graph.all_pairs_distances(max_tick);

		// Then, we recopy the distance matrix to all the attacks
		// We invalidate all the paths except those who took more than max_tick to reach the destination
		let mut dmas = vec![DistanceMatrixArray::propagated_and_unvalidated(&dst_mat, max_tick); self.series.len()];

		let results = dmas
			.par_iter_mut()
			.enumerate()
			.map(|(e, dma)| {
				// Then, we recompute the paths for the attacked graph
				let attack = &self.series[e].1;
				(
					self.series[e].0,
					benchmark!(format!("(Dynamic) Recomputing distances for {}", self.series[e].0), {
						let time_graph = graph_to_temporal(self.graph, max_tick, &attack.blocked_links(), attack.delta());
						for t in 0..time_graph.max_tick {
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
						dma.efficiency_reachables(max_tick)
					}),
				)
			})
			.collect();

		return results;
	}*/

	pub fn efficiencies(directory: &str, files: &[&'a str], graph: &'a Graph) -> Vec<(&'a str, (f64, u64))> {
		// First, compute the paths for the unattacked graph
		// Read the first file to get the max time
		let file_name = format!("{}{}", directory, files[0]);
		let file = std::fs::File::open(file_name).unwrap();
		let mut reader = std::io::BufReader::new(file);
		let mut line = String::new();
		reader.read_line(&mut line).unwrap();
		line.clear();
		reader.read_line(&mut line).unwrap();
		let line = line.split_whitespace().collect::<Vec<&str>>();
		let max_tick = line[2];
		let max_tick = max_tick.parse().unwrap();

		//println!("Graph diameter: {}", max_tick);

		// Compute the distance matrix for the unattacked graph
		let dst_mat = graph.all_pairs_distances();

		// Then, we recopy the distance matrix to all the attacks
		// We invalidate all the paths except those who took more than max_tick to reach the destination
		files
			.par_iter()
			.map(|budget| {
				benchmark!(format!("(Dynamic) Recomputing distances for {}", budget), {
					let file_name = format!("{}{}", directory, budget);
					let file = std::fs::File::open(file_name).unwrap();
					let mut reader = std::io::BufReader::new(file);
					let mut line = String::new();
					reader.read_line(&mut line).unwrap();
					let attack = DynamicAttack::from_file(&mut reader);
					let mut dma = DistanceMatrixArray::propagated_and_unvalidated(&dst_mat, max_tick);
					let time_graph = graph_to_temporal(graph, max_tick, &attack.blocked_links(), attack.delta());
					std::mem::drop(attack);
					for t in 0..time_graph.max_tick() {
						for i in 0..time_graph.nb_nodes() {
							// Doing the j in reverse seems to improve slightly the performance
							// Probably because nodes that are far in real
							// life apart also have far apart indexes
							for j in (0..time_graph.nb_nodes()).rev() {
								// Not 0 -> already computed
								if dma[(t, i, j)] != 0 {
									continue;
								}
								temporal_dijkstra_write(&time_graph, i as Node, j as Node, t, &mut dma);
							}
						}
					}
					let (efficiency, reachables) = dma.efficiency_reachables(max_tick);

					std::mem::drop(dma);
					(*budget, (efficiency, reachables))
				})
			})
			.collect()
	}

	pub fn efficiency_invalidation(&self) -> Vec<(&str, (f64, u64))> {
		// First, compute the paths for the unattacked graph
		let max_tick = self.series[0].1.max_tick(); // The max time is the same for all attacks
		let (dst_mat, paths) = self.graph.all_pairs_distances_and_paths(max_tick);
		let mut dmas = vec![DistanceMatrixArray::propagated(&dst_mat, max_tick); self.series.len()];

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
						path.unvalidate_path(dma, attack, &mut already_done, &times_matrix);
					}
				});

				// Count the number of 0's
				/*let mut count = 0;
				for t in 0..max_tick {
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
					self.graph.nb_nodes() as usize * self.graph.nb_nodes() as usize * max_tick as usize
				);*/

				(
					self.series[e].0,
					benchmark!("recomputing everything", {
						let time_graph = graph_to_temporal(self.graph, max_tick, &attack.blocked_links(), attack.delta());
						for t in 0..time_graph.max_tick() {
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
						dma.efficiency_reachables(max_tick)
					}),
				)
			})
			.collect();

		return results;
	}
}

pub struct StaticAttackSeries<'a> {
	pub series: Vec<(&'a str, StaticAttack)>,
	pub graph: &'a Graph,
}

impl<'a> StaticAttackSeries<'a> {
	/*pub fn from_directory(directory: &str, files: &[&'a str], graph: &'a Graph) -> StaticAttackSeries<'a> {
		let series = files
			.iter()
			.map(|budget| {
				let file_name = format!("{}{}", directory, budget);
				// Open the file with a reader that can read from it
				let file = std::fs::File::open(file_name).unwrap();
				let mut reader = std::io::BufReader::new(file);
				// read the first line and skip it
				let mut line = String::new();
				reader.read_line(&mut line).unwrap();

				let attack = StaticAttack::from_file(&mut reader, graph);
				(*budget, attack)
			})
			.collect();
		return StaticAttackSeries { series, graph };
	}

	pub fn efficiency(&self) -> Vec<(&str, (f64, u64))> {
		let efficiencies: Vec<(&str, (f64, u64))> = self
			.series
			.par_iter()
			.map(|(budget, attack)| {
				(
					*budget,
					benchmark!(format!("(Static) Efficiency for {}", budget), {
						let (efficiency, reachables) = attack.efficiency(self.graph);
						(efficiency, reachables)
					}),
				)
			})
			.collect();
		return efficiencies;
	}*/

	pub fn efficiencies(directory: &str, files: &[&'a str], graph: &'a Graph) -> Vec<(&'a str, (f64, u64))> {
		files
			.par_iter()
			.map(|budget| {
				let file_name = format!("{}{}", directory, budget);
				let file = std::fs::File::open(file_name).unwrap();
				let mut reader = std::io::BufReader::new(file);
				let mut line = String::new();
				reader.read_line(&mut line).unwrap();
				let attack = StaticAttack::from_file(&mut reader, graph);
				(
					*budget,
					benchmark!(format!("(Static) Efficiency for {}", budget), { attack.efficiency(graph) }),
				)
			})
			.collect()
	}
}

pub enum AttackSeries<'a> {
	Static(StaticAttackSeries<'a>),
	Dynamic(DynamicAttackSeries<'a>),
}

impl<'a> AttackSeries<'a> {
	/*pub fn from_directory(directory: &str, files: &[&'a str], graph: &'a Graph) -> AttackSeries<'a> {
		// Open the first file to check if it is a static or dynamic attack
		let file_name = format!("{}{}", directory, files[0]);
		let file = std::fs::File::open(file_name).unwrap();
		let mut reader = std::io::BufReader::new(file);
		let mut line = String::new();
		reader.read_line(&mut line).unwrap();

		if line.starts_with('S') {
			AttackSeries::Static(StaticAttackSeries::from_directory(directory, files, graph))
		} else if line.starts_with('D') {
			AttackSeries::Dynamic(DynamicAttackSeries::from_directory(directory, files, graph))
		} else {
			panic!("Invalid attack file");
		}
	}

	pub fn efficiency(&self) -> Vec<(&str, (f64, u64))> {
		match self {
			AttackSeries::Static(sas) => sas.efficiency(),
			AttackSeries::Dynamic(das) => das.efficiency(),
		}
	}

	pub fn efficiency_invalidation(&self) -> Vec<(&str, (f64, u64))> {
		match self {
			AttackSeries::Static(sas) => sas.efficiency(),
			AttackSeries::Dynamic(das) => das.efficiency_invalidation(),
		}
	}*/

	pub fn efficiencies(directory: &str, files: &[&'a str], graph: &'a Graph) -> Vec<(&'a str, (f64, u64))> {
		let nb_threads = std::env::var("LSRT_NB_THREADS");
		if let Ok(nb_threads) = nb_threads {
			let nb_threads = nb_threads.parse().unwrap();
			rayon::ThreadPoolBuilder::new().num_threads(nb_threads).build_global().unwrap();
		}
		// Open the first file to check if it is a static or dynamic attack
		let file_name = format!("{}{}", directory, files[0]);
		let file = std::fs::File::open(file_name).unwrap();
		let mut reader = std::io::BufReader::new(file);
		let mut line = String::new();
		reader.read_line(&mut line).unwrap();

		if line.starts_with('S') {
			StaticAttackSeries::efficiencies(directory, files, graph)
		}
		else if line.starts_with('D') {
			DynamicAttackSeries::efficiencies(directory, files, graph)
		}
		else {
			panic!("Invalid attack file");
		}
	}
}
