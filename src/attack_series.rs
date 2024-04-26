use std::{collections::HashSet, io::BufRead};

use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};

use crate::{
	distance_matrix::{DistanceMatrixArray, SquareMatrix},
	graph::{dijkstra, Graph},
	paths::{self, Path, TemporalSubPathsPyramid},
	time_varying_graph::{self, graph_to_temporal, temporal_dijkstra, temporal_dijkstra_write, PhantomEdgesMap},
	Attack, DynamicAttack, StaticAttack,
};
use crate::{Node, Tick};

use crate::benchmark;

pub struct DynamicAttackSeries<'a> {
	pub series: Vec<(&'a str, DynamicAttack)>,
	pub graph: &'a Graph,
}

impl<'a> DynamicAttackSeries<'a> {
	pub fn efficiencies_bourrin(directory: &str, files: &[&'a str], graph: &'a Graph) -> Vec<(&'a str, (f64, u64))> {
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

		// Compute the distance matrix for the unattacked graph
		//let dst_mat = graph.all_pairs_distances();
		//let dma = DistanceMatrixArray::propagated_and_unvalidated(&dst_mat, max_tick);
		let dma = DistanceMatrixArray::zeroed(graph.nb_nodes().into(), max_tick);
		/*let mut nb_zeros = 0;
		for t in 0..max_tick {
			for i in 0..graph.nb_nodes() {
				for j in 0..graph.nb_nodes() {
					if dma[(t, i, j)] == 0 {
						nb_zeros += 1;
					}
				}
			}
		}
		println!(
			"Nb zeros: {} / {} for unattacked graph",
			nb_zeros,
			graph.nb_nodes() as usize * graph.nb_nodes() as usize * max_tick as usize
		);*/

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
					let mut dma = dma.clone();
					let attack = DynamicAttack::from_file(&mut reader);
					let time_graph = graph_to_temporal(graph, max_tick, &attack.blocked_links(), attack.delta());
					let edges_map = PhantomEdgesMap::from_graph(&time_graph);
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
								let path = temporal_dijkstra(&time_graph, i as Node, j as Node, t, &edges_map);
								dma[(t, i, j)] = path.cost() as Tick;
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

	pub fn efficiencies_write(directory: &str, files: &[&'a str], graph: &'a Graph) -> Vec<(&'a str, (f64, u64))> {
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

		// Compute the distance matrix for the unattacked graph
		let dst_mat = graph.all_pairs_distances();
		let dma = DistanceMatrixArray::propagated_and_unvalidated(&dst_mat, max_tick);

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
					let mut dma = dma.clone();
					let attack = DynamicAttack::from_file(&mut reader);
					let time_graph = graph_to_temporal(graph, max_tick, &attack.blocked_links(), attack.delta());
					let edges_map = PhantomEdgesMap::from_graph(&time_graph);
					std::mem::drop(attack);
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
					let (efficiency, reachables) = dma.efficiency_reachables(max_tick);

					std::mem::drop(dma);
					(*budget, (efficiency, reachables))
				})
			})
			.collect()
	}

	pub fn efficiencies_invalidation(directory: &str, files: &[&'a str], graph: &'a Graph) -> Vec<(&'a str, (f64, u64))> {
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

		// Compute the distance matrix for the unattacked graph
		let (dst_mat, paths) = graph.all_pairs_distances_and_paths(max_tick);
		let dma = DistanceMatrixArray::propagated(&dst_mat, max_tick);

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
					let mut dma = dma.clone();

					let attack = DynamicAttack::from_file(&mut reader);
					let time_graph = graph_to_temporal(graph, max_tick, &attack.blocked_links(), attack.delta());
					let edges_map = PhantomEdgesMap::from_graph(&time_graph);
					let mut already_done = SquareMatrix::new(graph.nb_nodes().into(), false);
					/*for path in &paths {
						let invalidations = path.invalidate_path_better(&time_graph, &mut already_done);
						for (node, ticks) in invalidations {
							let subpath = &path.subpaths[node];
							for tick in ticks {
								let old_value = dma[(tick, subpath.start, subpath.end)];
								if old_value.saturating_add(tick) < max_tick {
									dma[(tick, subpath.start, subpath.end)] = 0;
								}
							}
							already_done[(subpath.start.into(), subpath.end.into())] = true;
						}
					}*/

					/*benchmark!("Invalidate paths", {
						for path in &paths {
							path.invalidate_path_even_better(&time_graph, &mut already_done, &mut dma);
						}
					});*/

					benchmark!("Invalidate paths", {
						for path in &paths {
							path.invalidate_path(&time_graph, &mut already_done, &mut dma, &dst_mat, &edges_map);
						}
					});

					//dma.fill_1_on_diagonals();

					let mut nb_zeros = 0;
					for t in 0..max_tick {
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
						std::mem::drop(attack);
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
					let (efficiency, reachables) = dma.efficiency_reachables(max_tick);

					std::mem::drop(dma);
					(*budget, (efficiency, reachables))
				})
			})
			.collect()
	}

	pub fn efficiencies_reachables(directory: &str, files: &[&'a str], graph: &'a Graph) -> Vec<(&'a str, (f64, u64))> {
		println!("Computing dynamic attack series bourrin");
		let results_bourrin = DynamicAttackSeries::efficiencies_bourrin(directory, files, graph);
		println!("Computing dynamic attack series write");
		let results_write = DynamicAttackSeries::efficiencies_write(directory, files, graph);
		println!("Computing dynamic attack series invalidation");
		let results_invalidation = DynamicAttackSeries::efficiencies_invalidation(directory, files, graph);
		for i in 0..results_bourrin.len() {
			let (budget, (efficiency_bourrin, reachables_bourrin)) = results_bourrin[i];
			let (_, (efficiency_write, reachables_write)) = results_write[i];
			let (_, (efficiency_invalidation, reachables_invalidation)) = results_invalidation[i];
			assert_eq!(budget, files[i]);
			//assert_eq!(efficiency_bourrin, efficiency_write);
			//assert_eq!(efficiency_bourrin, efficiency_invalidation);
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
	pub fn efficiencies_reachables(directory: &str, files: &[&'a str], graph: &'a Graph) -> Vec<(&'a str, (f64, u64))> {
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
					benchmark!(format!("(Static) Efficiency for {}", budget), {
						attack.efficiency_reachables(graph)
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
	pub fn efficiencies_reachables(directory: &str, files: &[&'a str], graph: &'a Graph) -> Vec<(&'a str, (f64, u64))> {
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
			StaticAttackSeries::efficiencies_reachables(directory, files, graph)
		} else if line.starts_with('D') {
			DynamicAttackSeries::efficiencies_reachables(directory, files, graph)
		} else {
			panic!("Invalid attack file");
		}
	}
}
