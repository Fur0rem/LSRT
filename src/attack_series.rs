use std::io::BufRead;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
	distance_matrix::{DistanceMatrixArray, SquareMatrix},
	graph::Graph,
	time_varying_graph::{graph_to_temporal, temporal_dijkstra, temporal_dijkstra_write, PhantomEdgesMap},
	Attack, DynamicAttack, StaticAttack,
};
use crate::{Node, Tick};

use crate::benchmark;

pub struct DynamicAttackSeries<'a> {
	pub series: Vec<(&'a str, DynamicAttack)>,
	pub graph: &'a Graph,
}

impl<'a> DynamicAttackSeries<'a> {
	pub fn max_tick(&self) -> Tick {
		self.series[0].1.max_tick()
	}

	pub fn read_attacks(directory: &str, files: &[&'a str], graph: &'a Graph) -> Self {
		let series = files
			.iter()
			.map(|budget| {
				let file_name = format!("{}{}", directory, budget);
				let file = std::fs::File::open(file_name).unwrap();
				let mut reader = std::io::BufReader::new(file);
				let mut line = String::new();
				reader.read_line(&mut line).unwrap();
				(*budget, DynamicAttack::from_file(&mut reader))
			})
			.collect();
		return Self { series, graph };
	}

	pub fn efficiencies_bourrin(&self) -> Vec<(&'a str, (f64, u64))> {
		self.series
			.par_iter()
			.map(|(budget, attack)| {
				benchmark!(format!("(Dynamic) Recomputing distances for {}", budget), {
					let time_graph = graph_to_temporal(self.graph, self.max_tick(), &attack.blocked_links(), attack.delta());
					let edges_map = PhantomEdgesMap::from_graph(&time_graph);
					let mut dma = DistanceMatrixArray::zeroed(self.graph.nb_nodes(), self.max_tick());
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
								let path = temporal_dijkstra(&time_graph, i as Node, j as Node, t, &edges_map);
								dma[(t, i, j)] = path.cost() as Tick;
							}
						}
					}
					let (efficiency, reachables) = dma.efficiency_reachables(self.max_tick());

					std::mem::drop(dma);
					(*budget, (efficiency, reachables))
				})
			})
			.collect()
	}

	pub fn efficiencies_write(&self) -> Vec<(&'a str, (f64, u64))> {
		// First, compute the paths for the unattacked graph
		let dst_mat = self.graph.all_pairs_distances();

		self.series
			.par_iter()
			.map(|(budget, attack)| {
				benchmark!(format!("(Dynamic) Recomputing distances for {}", budget), {
					let time_graph = graph_to_temporal(self.graph, self.max_tick(), &attack.blocked_links(), attack.delta());
					let edges_map = PhantomEdgesMap::from_graph(&time_graph);
					let mut dma = DistanceMatrixArray::propagated_and_unvalidated(&dst_mat, time_graph.max_tick());
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
								temporal_dijkstra_write(&time_graph, i as Node, j as Node, t, &mut dma, &edges_map);
							}
						}
					}
					let (efficiency, reachables) = dma.efficiency_reachables(self.max_tick());

					std::mem::drop(dma);
					(*budget, (efficiency, reachables))
				})
			})
			.collect()
	}

	pub fn efficiencies_invalidation(&self) -> Vec<(&'a str, (f64, u64))> {
		let (dst_mat, paths) = self.graph.all_pairs_distances_and_paths(self.max_tick());
		self.series
			.par_iter()
			.map(|(budget, attack)| {
				benchmark!(format!("(Dynamic) Recomputing distances for {}", budget), {
					let time_graph = graph_to_temporal(self.graph, self.max_tick(), &attack.blocked_links(), attack.delta());
					let edges_map = PhantomEdgesMap::from_graph(&time_graph);
					let mut already_done = SquareMatrix::new(self.graph.nb_nodes().into(), false);
					let mut dma = DistanceMatrixArray::propagated(&dst_mat, time_graph.max_tick());

					benchmark!("Invalidate paths", {
						for path in &paths {
							path.invalidate_path(&time_graph, &mut already_done, &mut dma, &dst_mat, &edges_map);
						}
					});

					let mut nb_zeros = 0;
					for t in 0..self.max_tick() {
						for i in 0..time_graph.nb_nodes() {
							for j in 0..time_graph.nb_nodes() {
								if dma[(t, i, j)] == 0 {
									nb_zeros += 1;
								}
							}
						}
					}

					println!(
						"Nb zeros: {} / {} for budget {}",
						nb_zeros,
						time_graph.max_tick() as usize * time_graph.nb_nodes() as usize * time_graph.nb_nodes() as usize,
						budget
					);

					benchmark!("Recomputing everything", {
						for t in 0..time_graph.max_tick() {
							for i in 0..time_graph.nb_nodes() {
								// Doing the j in reverse seems to improve slightly the performance
								// Probably because nodes that are far in real
								// life apart also have far apart indexes
								for j in (0..time_graph.nb_nodes()).rev() {
									// Not 0 -> already computed
									if dma[(t, i, j)] == 0 {
										temporal_dijkstra_write(&time_graph, i as Node, j as Node, t, &mut dma, &edges_map);
									}
								}
							}
						}
					});

					let (efficiency, reachables) = dma.efficiency_reachables(self.max_tick());

					std::mem::drop(dma);
					(*budget, (efficiency, reachables))
				})
			})
			.collect()
	}

	pub fn efficiencies_reachables(&self) -> Vec<(&'a str, (f64, u64))> {
		println!("Computing dynamic attack series bourrin");
		let results_bourrin = self.efficiencies_bourrin();
		println!("Computing dynamic attack series write");
		let results_write = self.efficiencies_write();
		println!("Computing dynamic attack series invalidation");
		let results_invalidation = self.efficiencies_invalidation();
		for i in 0..results_bourrin.len() {
			let (budget, (efficiency_bourrin, reachables_bourrin)) = results_bourrin[i];
			let (_, (efficiency_write, reachables_write)) = results_write[i];
			let (_, (efficiency_invalidation, reachables_invalidation)) = results_invalidation[i];
			assert_eq!(efficiency_bourrin, efficiency_write);
			assert_eq!(efficiency_bourrin, efficiency_invalidation);
			assert_eq!(reachables_bourrin, reachables_write);
			assert_eq!(reachables_bourrin, reachables_invalidation);
			println!(
				"{}: Efficiency: {} / Reachables: {}",
				budget, efficiency_bourrin, reachables_bourrin
			);
		}
		return results_bourrin;
	}
}

pub struct StaticAttackSeries<'a> {
	pub series: Vec<(&'a str, StaticAttack)>,
	pub graph: &'a Graph,
}

impl<'a> StaticAttackSeries<'a> {
	pub fn read_attacks(directory: &str, files: &[&'a str], graph: &'a Graph) -> Self {
		let series = files
			.iter()
			.map(|budget| {
				let file_name = format!("{}{}", directory, budget);
				let file = std::fs::File::open(file_name).unwrap();
				let mut reader = std::io::BufReader::new(file);
				let mut line = String::new();
				reader.read_line(&mut line).unwrap();
				(*budget, StaticAttack::from_file(&mut reader, graph))
			})
			.collect();
		return Self { series, graph };
	}

	pub fn efficiencies_reachables(&self) -> Vec<(&'a str, (f64, u64))> {
		self.series
			.par_iter()
			.map(|(budget, attack)| {
				(
					*budget,
					benchmark!(format!("(Static) Efficiency for {}", budget), {
						attack.efficiency_reachables(self.graph)
					}),
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
	pub fn read_attacks(directory: &str, files: &[&'a str], graph: &'a Graph) -> Self {
		// Open the first file to check if it is a static or dynamic attack
		let file_name = format!("{}{}", directory, files[0]);
		let file = std::fs::File::open(file_name).unwrap();
		let mut reader = std::io::BufReader::new(file);
		let mut line = String::new();
		reader.read_line(&mut line).unwrap();

		if line.starts_with('S') {
			AttackSeries::Static(StaticAttackSeries::read_attacks(directory, files, graph))
		}
		else if line.starts_with('D') {
			AttackSeries::Dynamic(DynamicAttackSeries::read_attacks(directory, files, graph))
		}
		else {
			panic!("Invalid attack file");
		}
	}

	pub fn efficiencies_reachables(&self) -> Vec<(&'a str, (f64, u64))> {
		let nb_threads = std::env::var("LSRT_NB_THREADS");
		if let Ok(nb_threads) = nb_threads {
			let nb_threads = nb_threads.parse().unwrap();
			rayon::ThreadPoolBuilder::new().num_threads(nb_threads).build_global().unwrap();
		}
		match self {
			AttackSeries::Static(series) => series.efficiencies_reachables(),
			AttackSeries::Dynamic(series) => series.efficiencies_reachables(),
		}
	}
}
