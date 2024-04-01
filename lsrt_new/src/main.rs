#![allow(clippy::needless_return)]

mod graph;
use graph::Graph;

mod attack;
use attack::*;

mod paths;
mod tests;

mod time_varying_graph;

mod distance_matrix;

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
	benchmark!("Measurement", {
		let graph = Graph::from_sparse_row_format("../Saints_graph").unwrap();
		let attack = <dyn Attack>::from_file("../attack.0xv8q2", &graph);
		let (efficiency, reachables) = attack.efficiency(&graph);
		println!("Efficiency: {}, Reachables: {}", efficiency, reachables);
	});
}
