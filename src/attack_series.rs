//! Optimised computation for multiple attacks on a single graph.

use std::fs::File;
use std::io::{BufRead, BufReader};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::Tick;
use crate::{
	distance_matrix::{DistanceMatrixArray, SquareMatrix},
	graph::Graph,
	time_varying_graph::{graph_to_temporal, temporal_dijkstra, temporal_dijkstra_write, PhantomEdgesMap},
	Attack, DynamicAttack, StaticAttack,
};

use crate::benchmark;

/// A series of dynamic attacks on a graph.
pub struct DynamicAttackSeries<'a> {
	pub series: Vec<(&'a str, DynamicAttack)>,
	pub graph: &'a Graph,
}

impl<'a> DynamicAttackSeries<'a> {
	/// Returns the maximum tick of the series.
	/// It assumes that all the attacks have the same maximum tick.
	pub fn max_tick(&self) -> Tick {
		self.series[0].1.max_tick()
	}

	/// Reads a series of dynamic attacks from a directory and a list of files.
	pub fn read_attacks(directory: &str, files: &[&'a str], graph: &'a Graph) -> Self {
		let series = files
			.par_iter()
			.map(|budget| {
				let file_name = format!("{directory}{budget}");
				let file = File::open(file_name).unwrap();
				let mut reader = BufReader::new(file);
				let mut line = String::new();
				reader.read_line(&mut line).unwrap();
				(*budget, DynamicAttack::from_file(&mut reader))
			})
			.collect();
		return Self { series, graph };
	}

	/// Computes the efficiency and reachables for each attack in the series.
	/// This function uses a naive approach to compute the distances, it calls the [`temporal_dijkstra`] function for each pair of nodes and each tick.
	/// It is not recommended to use as it is slower than [`efficiencies_write`][DynamicAttackSeries::efficiencies_write]
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
								let path = temporal_dijkstra(&time_graph, i, j, t, &edges_map);
								dma[(t, i, j)] = path.cost();
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

	/// Computes the efficiency and reachables for each attack in the series.
	/// This function uses a more efficient approach to compute the distances, it calls the [`temporal_dijkstra_write`] function for each pair of nodes and each tick.
	/// However, it is faster than [`efficiencies_bourrin`][DynamicAttackSeries::efficiencies_bourrin] since it writes more informations in the distance matrix per call.
	/// Therefore, for most iterations, the function will not even be called.
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
								temporal_dijkstra_write(&time_graph, i, j, t, &mut dma, &edges_map);
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

	/// Computes the efficiency and reachables for each attack in the series.
	/// This function first computes the paths for the unattacked graph, then for each path, it calculates whether it was impacted by the attack.
	/// If it was, it invalidates the path and recomputes it.
	/// This function is slower than [`efficiencies_write`][DynamicAttackSeries::efficiencies_write] since despite having to compute 3 times less paths originally for small attacks,
	/// [`temporal_dijkstra_write`] writes so much information per call that it doesn't make a difference.
	pub fn efficiencies_invalidation(&self) -> Vec<(&'a str, (f64, u64))> {
		let (dst_mat, paths) = self.graph.all_pairs_distances_and_paths(self.max_tick());
		self.series
			.par_iter()
			.map(|(budget, attack)| {
				benchmark!(format!("(Dynamic) Recomputing distances for {}", budget), {
					let time_graph = graph_to_temporal(self.graph, self.max_tick(), &attack.blocked_links(), attack.delta());
					let edges_map = PhantomEdgesMap::from_graph(&time_graph);
					let mut already_done = SquareMatrix::new(false, self.graph.nb_nodes().into());
					let mut dma = DistanceMatrixArray::propagated(&dst_mat, time_graph.max_tick());

					benchmark!("Invalidate paths", {
						for path in &paths {
							path.invalidate_path(&time_graph, &mut already_done, &mut dma, &dst_mat, &edges_map);
						}
					});

					let nb_zeros = dma.count_zeros();
					let total_size = dma.total_size();
					println!("Nb zeros: {nb_zeros} / {total_size} for budget {budget}");

					benchmark!("Recomputing everything", {
						for t in 0..time_graph.max_tick() {
							for i in 0..time_graph.nb_nodes() {
								// Doing the j in reverse seems to improve slightly the performance
								// Probably because nodes that are far in real
								// life apart also have far apart indexes
								for j in (0..time_graph.nb_nodes()).rev() {
									// Not 0 -> already computed
									if dma[(t, i, j)] == 0 {
										temporal_dijkstra_write(&time_graph, i, j, t, &mut dma, &edges_map);
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

	/// Computes the efficiency and reachables for each attack in the series.
	/// This function does the same as [`efficiencies_write`][DynamicAttackSeries::efficiencies_write], but it uses the dependencies of the nodes to compute the distances.
	/// We call A a dependency of B if there is an edge A -> B.
	/// If the distance of a dependency has not been computed, it will compute it, and then compute the distance to the node.
	/// The benchmarks we did show a speedup from 1.7x to 2.1x compared to [`efficiencies_write`][DynamicAttackSeries::efficiencies_write].
	pub fn efficiencies_dependencies(&self) -> Vec<(&'a str, (f64, u64))> {
		let dst_mat = self.graph.all_pairs_distances();

		// Invert the graph to get the dependencies
		// Which is a list of nodes that are outgoing from a node
		let mut dependencies = vec![vec![]; self.graph.nb_nodes().into()];
		for node in 0..self.graph.nb_nodes() {
			for edge in self.graph.neighbours(node) {
				dependencies[edge.node as usize].push(node);
			}
		}

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

								// We look at all the nodes that could lead to j
								let mut min_distance = Tick::MAX;
								for dep in &dependencies[j as usize] {
									let distance = dma[(t, i, *dep)];
									if distance == 0 {
										temporal_dijkstra_write(&time_graph, i, *dep, t, &mut dma, &edges_map);
									}
									// Issue with [(i, i)]
									let distance = if i == *dep { 0 } else { dma[(t, i, *dep)] };

									let time_to_inspect = t.saturating_add(distance);
									if time_to_inspect >= time_graph.max_tick() {
										continue;
									}
									let edge_weight = edges_map.get_edge(*dep, j).unwrap().weights[usize::from(time_to_inspect)];
									let weight = distance.saturating_add(edge_weight);
									if weight < min_distance {
										min_distance = weight;
									}
								}
								dma[(t, i, j)] = min_distance;
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

	/// Just a test function to check that the 3 methods
	/// ([`efficiencies_bourrin`][DynamicAttackSeries::efficiencies_bourrin], [`efficiencies_write`][DynamicAttackSeries::efficiencies_write], [`efficiencies_invalidation`][DynamicAttackSeries::efficiencies_invalidation])
	/// give the same results.
	pub fn efficiencies_reachables_test(&self) -> Vec<(&'a str, (f64, u64))> {
		/*println!("Computing dynamic attack series bourrin");
		let results_bourrin = self.efficiencies_bourrin();
		println!("Computing dynamic attack series write");
		let results_write = self.efficiencies_write();
		println!("Computing dynamic attack series invalidation");
		let results_invalidation = self.efficiencies_invalidation();
		println!("Computing without series");
		let results_no_series: Vec<_> = self
			.series
			.iter()
			.map(|(_, attack)| attack.efficiency_reachables(self.graph))
			.collect();
		println!("Computing dependencies");
		let results_dependencies = self.efficiencies_dependencies();
		for i in 0..results_bourrin.len() {
			let (budget, (efficiency_bourrin, reachables_bourrin)) = results_bourrin[i];
			let (_, (efficiency_write, reachables_write)) = results_write[i];
			let (_, (efficiency_invalidation, reachables_invalidation)) = results_invalidation[i];
			let (_, (efficiency_dependencies, reachables_dependencies)) = results_dependencies[i];
			let (efficiency_no_series, reachables_no_series) = results_no_series[i];
			assert_eq!(efficiency_bourrin, efficiency_write);
			assert_eq!(efficiency_bourrin, efficiency_invalidation);
			assert_eq!(reachables_bourrin, reachables_write);
			assert_eq!(reachables_bourrin, reachables_invalidation);
			assert_eq!(efficiency_bourrin, efficiency_no_series);
			assert_eq!(reachables_bourrin, reachables_no_series);
			assert_eq!(efficiency_bourrin, efficiency_dependencies);
			assert_eq!(reachables_bourrin, reachables_dependencies);
			println!("{budget}: Efficiency: {efficiency_bourrin} / Reachables: {reachables_bourrin}");
		}*/

		let results_write = benchmark!("Dynamic Attack Series Write", { self.efficiencies_write() });

		let results_dependencies = benchmark!("Dynamic Attack Series Dependencies", { self.efficiencies_dependencies() });

		for i in 0..results_write.len() {
			let (budget, (efficiency_write, reachables_write)) = results_write[i];
			let (_, (efficiency_dependencies, reachables_dependencies)) = results_dependencies[i];
			assert_eq!(efficiency_write, efficiency_dependencies);
			assert_eq!(reachables_write, reachables_dependencies);
			println!("{budget}: Efficiency: {efficiency_write} / Reachables: {reachables_write}");
		}

		return results_write;
	}

	/// Computes the efficiency and reachables for each attack in the series.
	/// It uses the most efficient method to compute the distances.
	/// It is recommended to use this function.
	pub fn efficiencies_reachables(&self) -> Vec<(&'a str, (f64, u64))> {
		self.efficiencies_dependencies()
	}
}

/// A series of static attacks on a graph.
pub struct StaticAttackSeries<'a> {
	/// The series of attacks.
	pub series: Vec<(&'a str, StaticAttack)>,
	/// The graph on which the attacks are performed.
	pub graph: &'a Graph,
}

impl<'a> StaticAttackSeries<'a> {
	/// Reads a series of static attacks from a directory and a list of files.
	pub fn read_attacks(directory: &str, files: &[&'a str], graph: &'a Graph) -> Self {
		let series = files
			.iter()
			.map(|budget| {
				let file_name = format!("{directory}{budget}");
				let file = File::open(file_name).unwrap();
				let mut reader = BufReader::new(file);
				let mut line = String::new();
				reader.read_line(&mut line).unwrap();
				(*budget, StaticAttack::from_file(&mut reader, graph))
			})
			.collect();
		return Self { series, graph };
	}

	/// Computes the efficiency and reachables for each attack in the series.
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

/// A series of attacks on a graph.
/// The attacks can either be static or dynamic, but not both.
pub enum AttackSeries<'a> {
	Static(StaticAttackSeries<'a>),
	Dynamic(DynamicAttackSeries<'a>),
}

impl<'a> AttackSeries<'a> {
	/// Reads a series of attacks from a directory and a list of files.
	/// It assumes that all the files are of the same type (either static or dynamic).
	pub fn read_attacks(directory: &str, files: &[&'a str], graph: &'a Graph) -> Self {
		// Open the first file to check if it is a static or dynamic attack
		let file_name = format!("{}{}", directory, files[0]);
		let file = File::open(file_name).unwrap();
		let mut reader = BufReader::new(file);
		let mut line = String::new();
		reader.read_line(&mut line).unwrap();

		return match line.trim() {
			"S" => AttackSeries::Static(StaticAttackSeries::read_attacks(directory, files, graph)),
			"D" => AttackSeries::Dynamic(DynamicAttackSeries::read_attacks(directory, files, graph)),
			_ => panic!("Invalid attack type"),
		};
	}

	/// Computes the efficiency and reachables for each attack in the series.
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
