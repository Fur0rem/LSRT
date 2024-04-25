//! Optimised implementation of computing the efficiency of multiple attacks.

use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::benchmark;
use crate::time_varying_graph::PhantomEdgesMap;
use crate::{
	distance_matrix::DistanceMatrixArray,
	graph::Graph,
	time_varying_graph::{graph_to_temporal, temporal_dijkstra_write},
	Attack, DynamicAttack, StaticAttack,
};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

/// A series of [`DynamicAttack`]s.
pub struct DynamicAttackSeries<'a> {
	/// The series of attacks.
	pub series: Vec<(&'a str, DynamicAttack)>,
	/// The graph on which the attacks are performed.
	pub graph: &'a Graph,
}

impl<'a> DynamicAttackSeries<'a> {
	/// Compute the efficiency and reachables for each attack in the series.
	pub fn efficiencies_reachables(directory: &str, files: &[&'a str], graph: &'a Graph) -> Vec<(&'a str, (f64, u64))> {
		// Read the first file to get the max time
		let file_name = format!("{}{}", directory, files[0]);
		let file = File::open(file_name).unwrap();
		let mut reader = BufReader::new(file);
		let mut line = String::new();
		reader.read_line(&mut line).unwrap();
		line.clear();
		reader.read_line(&mut line).unwrap();
		let line = line.split_whitespace().collect::<Vec<&str>>();
		let max_tick = line[2];
		let max_tick = max_tick.parse().unwrap();

		// Compute the distance matrix for the unattacked graph
		let dst_mat = graph.all_pairs_distances();
		// Then, we recopy the distance matrix to all the attacks
		// We invalidate all the paths except those who took more than max_tick to reach the destination
		let dma = DistanceMatrixArray::propagated_and_unvalidated(&dst_mat, max_tick);

		let results = files
			.par_iter()
			.map(|budget| {
				benchmark!(format!("(Dynamic) Recomputing distances for {}", budget), {
					// Open the file
					let file_name = format!("{}{}", directory, budget);
					let file = std::fs::File::open(file_name).unwrap();
					let mut reader = BufReader::new(file);
					let mut line = String::new();
					reader.read_line(&mut line).unwrap();

					// Create the attack, graph, and distance matrix array
					let mut dma = dma.clone();
					let attack = DynamicAttack::from_file(&mut reader);
					let time_graph = graph_to_temporal(graph, max_tick, &attack.blocked_links(), attack.delta());
					let edges_map = PhantomEdgesMap::from_graph(&time_graph);
					// Early drop of the attack to free memory
					core::mem::drop(attack);
					for t in 0..time_graph.max_tick() {
						for i in 0..time_graph.nb_nodes() {
							// Doing the j in reverse seems to improve slightly the performance
							// Probably because nodes that are far in real life apart also have far apart indexes
							for j in (0..time_graph.nb_nodes()).rev() {
								// Not 0 -> already computed
								if dma[(t, i, j)] == 0 {
									temporal_dijkstra_write(&time_graph, i, j, t, &mut dma, &edges_map);
								}
							}
						}
					}
					let (efficiency, reachables) = dma.efficiency_reachables(max_tick);
					std::mem::drop(dma);

					(*budget, (efficiency, reachables))
				})
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
	pub fn efficiencies_reachables(directory: &str, files: &[&'a str], graph: &'a Graph) -> Vec<(&'a str, (f64, u64))> {
		let results = files
			.par_iter()
			.map(|budget| {
				let attack = StaticAttack::from_file_name(format!("{}{}", directory, budget), graph);
				let efficiency = benchmark!(format!("(Static) Efficiency for {}", budget), {
					attack.efficiency_reachables(graph)
				});

				(*budget, efficiency)
			})
			.collect();

		return results;
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
		let file = File::open(file_name).unwrap();
		let mut reader = BufReader::new(file);
		let mut line = String::new();
		reader.read_line(&mut line).unwrap();

		if line.starts_with('S') {
			return StaticAttackSeries::efficiencies_reachables(directory, files, graph);
		} else if line.starts_with('D') {
			return DynamicAttackSeries::efficiencies_reachables(directory, files, graph);
		} else {
			panic!("Invalid attack file");
		}
	}
}
