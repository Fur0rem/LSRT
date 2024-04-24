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
}

pub struct StaticAttackSeries<'a> {
	pub series: Vec<(&'a str, StaticAttack)>,
	pub graph: &'a Graph,
}

impl<'a> StaticAttackSeries<'a> {
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
