#![allow(clippy::needless_return)]
#![allow(clippy::needless_range_loop)]
// #![warn(clippy::uninlined_format_args)]
//#![warn(clippy::as_conversions)]
// #![warn(clippy::cast_lossless)]
// #![warn(clippy::unnecessary_cast)]
// #![warn(clippy::self_assignment)]
// #![warn(clippy::self_named_constructors)]
// #![warn(clippy::cast_possible_wrap)]
// #![warn(clippy::unwrap_used)]
//#![warn(clippy::cast_precision_loss)]
//#![warn(clippy::as_conversions)]
//#![warn(clippy::cast_possible_truncation)]
//#![warn(clippy::min_ident_chars)]

//! # Link Stream Robustness Tool
//! This is a tool to calculate the robustness of a network to a series of attacks.
//! The network is represented as a graph, and the attacks are represented as a series of edges that are removed from the graph.
//! The tool can calculate the robustness of the network to a series of attacks, and output the results in a file.
//! The two units measured are the efficiency and the reachables.
//! The efficiency is the average shortest path length between all pairs of nodes in the network.
//! The reachables is the number of nodes that are reachable from all other nodes in the network.
//!
//! Each attack containts a set of blocked links, and has a delta and a max tick.
//! The max tick is the maximum tick the attack can last, which is the diameter of the network.
//! The delta is the number of simulation ticks between each step of the attack.
//! So with a delta of 4, if a link is not blocked at time 0, and blocked at time 1, it will be not blocked at ticks 0, 1, 2, 3, and blocked at ticks 4, 5, 6, 7.
//!
//! The tool can measure those using two modes, which affects when you can cross a link of weight N at tick t:
//! 1. Default : If the link isn't blocked at tick t, you can cross it.
//! 2. Aggressive : The link needs to not be blocked from tick t to t + N to be crossed.

pub mod graph;
use graph::Graph;

pub mod attack;
use attack::*;

pub mod paths;

pub mod time_varying_graph;

pub mod distance_matrix;

pub mod attack_series;
use attack_series::AttackSeries;

use std::io::Write;

mod tests;

/// A Node in the graph.
/// Paris has 9596 nodes, so u16 should be enough
pub type Node = u16;
/// A Tick is a unit of time in the graph, it represents one step in the simulation.
/// Paris has a diameter of 1404 ticks, so u16 should be enough.
/// A tick of 0 or `Tick::MAX` represents an infinite weight, or a removed edge.
pub type Tick = u16;
/// A Time is a unit of time in the graph, it represents one step in the attack.
/// Each attack has a delta, which is the number of ticks for one time.
/// So Time <= Tick
pub type Time = u16;

#[macro_export]
/// Prints the time elapsed in a block of code.
macro_rules! benchmark {
	($name:expr, $block:expr) => {{
		let start = std::time::Instant::now();
		let result = $block;
		let duration = start.elapsed();
		println!("Time elapsed in {}() is: {:?}", $name, duration);
		result
	}};
}

/// The name of the directory containing the files for the project.
const FILES_DIR_NAME: &str = "data";

/// Main function of the program.
///
/// The program takes 3 arguments:
/// 1. The name of the city
/// 2. The name of the attack
/// 3. All the different budget files for the attack
/// The program will look inside the data directory for the graph and attack files, which should be organized as follows:
/// ``` plaintext
/// data/
/// ├── City
/// │   ├── attack1
/// │   │   ├── 10
/// │   │   ├── 2
/// │   │   └── 20
/// │   ├── attack2
/// │   │   ├── 10
/// │   │   ├── 2
/// │   │   └── 20
/// │   ├── attack3
/// │   │   ├── 10
/// │   │   ├── 2
/// │   │   └── 20
/// │   └── graph
/// └── OtherCity
///     └── ...
/// lsrt (executable)
/// ```
/// You can run the program like this:
/// ./lsrt City attack1 "10 2 20"
/// The program will output the results in the data/City/attack1/result file.
/// You should put all budgets in one string, separated by whitespaces (spaces, \r, \n, \t, etc), and bash can be tricky with whitespaces, so be careful with that.
pub fn main() {
	// Collect args
	let args: Vec<String> = std::env::args().collect();
	println!("Args: {args:?}");

	// Check if the number of arguments is correct
	if args.len() != 4 {
		eprintln!("Usage: {} <graph> <attack> <attack_files>", args[0]);
		std::process::exit(1);
	}

	// Load graph and attack files
	let graph_filename = format!("{FILES_DIR_NAME}/{}/graph", &args[1]);
	let attack_directory = format!("{FILES_DIR_NAME}/{}/{}/", &args[1], &args[2]);
	let attack_files: Vec<&str> = args[3].split_ascii_whitespace().collect();

	// Create the result file
	let result_file = format!("{FILES_DIR_NAME}/{}/{}/result", &args[1], &args[2]);
	let result_file = std::fs::File::create(result_file).unwrap();
	let mut result_file = std::io::BufWriter::new(result_file);

	// Run all the attacks
	benchmark!("Series of attack", {
		let graph = Graph::from_sparse_row_format(&graph_filename).unwrap();
		//let results = AttackSeries::efficiencies_reachables(&attack_directory, &attack_files, &graph);
		let attack_series = AttackSeries::read_attacks(&attack_directory, &attack_files, &graph);
		let results = attack_series.efficiencies_reachables();
		for (budget, (efficiency, reachables)) in results {
			writeln!(result_file, "{budget} {efficiency} {reachables}").unwrap();
		}
	});
}
