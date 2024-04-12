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

use rayon::prelude::*;
use std::io::Write;

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
	println!("{:?}", args);

	const FILES_DIR_NAME: &str = "data";
	let graph_filename = format!("{FILES_DIR_NAME}/{}/graph", &args[1]);
	let attack_directory = format!("{FILES_DIR_NAME}/{}/{}/", &args[1], &args[2]);
	println!("\n");
	println!("Graph filename: {}", graph_filename);
	println!("Attack directory: {}", attack_directory);
	let attack_files: Vec<&str> = args[3].split_ascii_whitespace().collect();

	let result_file = format!("{FILES_DIR_NAME}/{}/{}/result", &args[1], &args[2]);
	let result_file = std::fs::File::create(result_file).unwrap();
	let mut result_file = std::io::BufWriter::new(result_file);

	rayon::ThreadPoolBuilder::new().num_threads(8).build_global().unwrap();
	benchmark!("series of attack", {
		let graph = Graph::from_sparse_row_format(&graph_filename).unwrap();
		let efficiencies: Vec<f64> = attack_files
			.par_iter()
			.map(|attack_file| {
				let attack = <dyn Attack>::from_file(&format!("{}{}", attack_directory, attack_file), &graph);
				let (efficiency, reachables) = attack.efficiency(&graph);
				println!("Efficiency: {}, Reachables: {} for {}", efficiency, reachables, attack_file);
				efficiency
			})
			.collect();
		for efficiency in efficiencies {
			writeln!(result_file, "{}", efficiency).unwrap();
		}
	});
}
