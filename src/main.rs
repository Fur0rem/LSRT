#![allow(clippy::needless_return)]
#![allow(dead_code)]
#![allow(unused_imports)]

mod graph;
use graph::Graph;

mod attack;
use attack::*;

mod paths;
mod tests;

mod time_varying_graph;

mod distance_matrix;

mod attack_series;
use attack_series::AttackSeries;

use rayon::prelude::*;
use std::io::Write;

// TODO : export this somehow
macro_rules! benchmark {
	($name:expr, $block:expr) => {{
		let start = std::time::Instant::now();
		let result = $block;
		let duration = start.elapsed();
		println!("Time elapsed in {}() is: {:?}", $name, duration);
		result
	}};
}

fn main() {
	// Collect args
	let args: Vec<String> = std::env::args().collect();
	println!("Args: {:?}", args);

	const FILES_DIR_NAME: &str = "data";
	let graph_filename = format!("{FILES_DIR_NAME}/{}/graph", &args[1]);
	let attack_directory = format!("{FILES_DIR_NAME}/{}/{}/", &args[1], &args[2]);
	let attack_files: Vec<&str> = args[3].split_ascii_whitespace().collect();

	let result_file = format!("{FILES_DIR_NAME}/{}/{}/result", &args[1], &args[2]);
	let result_file = std::fs::File::create(result_file).unwrap();
	let mut result_file = std::io::BufWriter::new(result_file);

	benchmark!("Series of attack", {
		let graph = Graph::from_sparse_row_format(&graph_filename).unwrap();
		let results = AttackSeries::efficiencies(&attack_directory, &attack_files, &graph);
		//let results = attack_series.efficiency();
		for (budget, (efficiency, reachables)) in results {
			writeln!(result_file, "{} {} {}", budget, efficiency, reachables).unwrap();
		}
	});
}
