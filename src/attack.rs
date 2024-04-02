use std::{
	borrow::Cow,
	fs::File,
	io::{BufRead, BufReader},
};

use sscanf::scanf;

use crate::distance_matrix::{print_dma, sum_dma, sum_dma_static, DistanceMatrix};
use crate::time_varying_graph::{graph_to_temporal, temporal_dijkstra};
use crate::{
	graph::{Graph, Node, Time},
	paths::TemporalSubPathsPyramid,
};

pub enum AttackResult {
	Static(DistanceMatrix),
	Dynamic(Vec<DistanceMatrix>),
}

pub trait Attack: std::fmt::Debug {
	// efficiency + reachables
	fn all_pairs_distances(&self, graph: &Graph) -> AttackResult;
	fn efficiency(&self, graph: &Graph) -> (f64, u64) {
		let distance_matrix = self.all_pairs_distances(graph);
		match distance_matrix {
			AttackResult::Static(dm) => sum_dma_static(&dm, self.max_time()),
			AttackResult::Dynamic(dma) => sum_dma(&dma, self.max_time()),
		}
	}
		
	fn max_time(&self) -> Time;
	fn blocked_links(&self) -> Cow<Vec<DeletedLink>>;
	fn delta(&self) -> Time;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletedLink {
	pub from: Node,
	pub to: Node,
	pub times: Vec<Time>,
}

#[derive(Debug)]
pub struct DynamicAttack {
	pub blocked_links: Vec<DeletedLink>,
	pub delta: Time,
	pub max_time: Time,
}

#[derive(Debug)]
pub struct StaticAttack {
	pub blocked_links: Vec<Vec<Node>>,
	pub delta: Time,
	pub max_time: Time,
}

impl dyn Attack {
	pub fn from_file(file: &str, graph: &Graph) -> Box<dyn Attack> {
		// Format : D for dynamic, S for static
		let file = File::open(file).unwrap();
		let mut reader = BufReader::new(file);
		let mut line = String::new();
		reader.read_line(&mut line).unwrap();

		if line.starts_with('S') {
			return Box::new(StaticAttack::from_file(&mut reader, graph));
		}
		else if line.starts_with('D') {
			return Box::new(DynamicAttack::from_file(&mut reader));
		}

		panic!("Invalid attack file");
	}
}

impl StaticAttack {
	pub fn from_file(reader: &mut BufReader<File>, graph: &Graph) -> Self {
		// Format : first line contains max_time and delta
		// Then, the list of blocked links separated by a space
		let mut line = String::new();
		reader.read_line(&mut line).unwrap();
		let (max_time, delta) = scanf!(line.trim(), "{} {}", u16, u16).unwrap();

		// while there is new line, read it and append to the result
		let mut blocked_links = Vec::with_capacity(graph.nb_nodes());
		for _ in 0..graph.nb_nodes() {
			let mut line = String::new();
			reader.read_line(&mut line).unwrap();
			let blocks = line.split_whitespace().map(|x| x.parse().unwrap()).collect();
			blocked_links.push(blocks);
		}

		return Self {
			blocked_links,
			delta,
			max_time,
		};
	}
}

impl Attack for StaticAttack {
	fn max_time(&self) -> Time {
		return self.max_time;
	}

	fn delta(&self) -> Time {
		return self.delta;
	}

	fn blocked_links(&self) -> Cow<Vec<DeletedLink>> {
		let mut blocked_links = vec![];
		for i in 0..self.blocked_links.len() {
			for j in 0..self.blocked_links[i].len() {
				blocked_links.push(DeletedLink {
					from: i as Node,
					to: self.blocked_links[i][j],
					times: (0..self.max_time).collect(),
				});
			}
		}
		return Cow::Owned(blocked_links);
	}

	fn all_pairs_distances(&self, graph: &Graph) -> AttackResult {
		// Create a graph without the links that got blocked
		let mut new_graph = graph.clone();
		for i in 0..self.blocked_links.len() {
			for j in 0..self.blocked_links[i].len() {
				new_graph.remove_link(i as Node, self.blocked_links[i][j]);
			}
		}

		// Compute the distance matrix
		let distance_matrix = new_graph.all_pairs_distances(self.max_time);
		return AttackResult::Static(distance_matrix);
	}
}

// TODO : ew it's ugly af
impl DynamicAttack {
	pub fn from_file(reader: &mut BufReader<File>) -> Self {
		let mut line = String::new();
		reader.read_line(&mut line).unwrap();
		let (nb_edges, delta, max_time) = scanf!(line.trim(), "{} {} {}", usize, u16, u16).unwrap();

		// Then for each line, nb_blocked (node1,node2) <the times at which it is blocked>
		// For example : 4 (0,2) 5 9 17 18
		let mut blocked_links = vec![];
		for _ in 0..nb_edges {
			let mut line = String::new();
			reader.read_line(&mut line).unwrap();
			let mut iter = line.split_whitespace();
			let nb_blocked = iter.next().unwrap();
			let from = iter.next().unwrap();
			let to = iter.next().unwrap();

			let nb_blocked = nb_blocked.parse().unwrap();
			// remove the paranthesis and comma
			let from: usize = from[1..from.len() - 1].parse().unwrap();
			let to: usize = to[0..to.len() - 1].parse().unwrap();

			let mut times = vec![];
			for _ in 0..nb_blocked {
				let time = iter.next().unwrap().parse().unwrap();
				times.push(time);
			}

			blocked_links.push(DeletedLink {
				from: from as Node,
				to: to as Node,
				times,
			});
		}

		return Self {
			blocked_links,
			delta: delta as Time,
			max_time: max_time as Time,
		};
	}
}

impl Attack for DynamicAttack {
	fn max_time(&self) -> Time {
		return self.max_time;
	}

	fn delta(&self) -> Time {
		return self.delta;
	}

	fn blocked_links(&self) -> Cow<Vec<DeletedLink>> {
		return Cow::Borrowed(&self.blocked_links);
	}

	fn all_pairs_distances(&self, graph: &Graph) -> AttackResult {		
		// Create the time varying graph
		let time_graph = graph_to_temporal(graph, self.max_time(), &self.blocked_links(), self.delta());
		let distance_matrix = DistanceMatrix::init(graph);
		let mut dma = vec![distance_matrix.clone(); self.max_time as usize];
		// Compute all distance matrices
		for t in 0..time_graph.max_time {
			for i in 0..time_graph.nodes.len() {
				for j in 0..time_graph.nodes.len() {
					// Not 0 -> already computed
					if dma[t as usize][i][j] != 0 {
						continue;
					}

					// Find the path from i to j at time t
					let path = temporal_dijkstra(&time_graph, i as Node, j as Node, time_graph.max_time, t);

					// Write all the subpaths used in the main path
					let subpaths = TemporalSubPathsPyramid::from_path(&time_graph, &path);
					for subpath in &subpaths.subpaths {
						let (from, to, time_start, total_length) = (subpath.start, subpath.end, subpath.start_time, subpath.total_weight);

						if time_start >= time_graph.max_time as u16 {
							continue;
						}
						dma[time_start as usize][from as usize][to as usize] = total_length;
					}
				}
			}
		}

		return AttackResult::Dynamic(dma);
	}
}
