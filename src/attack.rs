//! A singular attack.
//!
//! Attacks are separated into 2 categories:
//! 1. Static attacks : the blocked links are fixed over the whole time
//! 2. Dynamic attacks : the blocked links change over time
//! We did this to optimise the computation time for static attacks, as the distance matrix is the same for all times.

use std::{
	borrow::Cow,
	fs::File,
	io::{BufRead, BufReader},
};

use sscanf::scanf;

use crate::{
	distance_matrix::{DistanceMatrix, DistanceMatrixArray},
	time_varying_graph::temporal_dijkstra_write,
};
use crate::{graph::Graph, paths::TemporalSubPathsPyramid};
use crate::{time_varying_graph::graph_to_temporal, Time};
use crate::{Node, Tick};

/// The result of an attack.
/// For static attacks, it is a distance matrix.
/// For dynamic attacks, it is a distance matrix array, which contains the distance matrix for each time.
#[derive(Debug)]
pub enum AttackResult {
	Static(DistanceMatrix),
	Dynamic(DistanceMatrixArray),
}

/// Trait for an attack.
/// An attack is a set of blocked links, and contains its delta and max time.
/// Any attack must implement this trait to have its efficiency computed.
pub trait Attack: std::fmt::Debug {
	// efficiency + reachables
	fn all_pairs_distances(&self, graph: &Graph) -> AttackResult;
	fn efficiency(&self, graph: &Graph) -> (f64, u64) {
		let distance_matrix = self.all_pairs_distances(graph);
		match distance_matrix {
			AttackResult::Static(dm) => dm.efficiency_reachables(self.max_tick()),
			AttackResult::Dynamic(dma) => dma.efficiency_reachables(self.max_tick()),
		}
	}

	fn max_tick(&self) -> Tick;
	fn blocked_links(&self) -> Cow<Vec<DeletedLink>>;
	fn delta(&self) -> u8;
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
	pub delta: u8,
	pub max_tick: Tick,
}

#[derive(Debug)]
pub struct StaticAttack {
	pub blocked_links: Vec<Vec<Node>>,
	pub delta: u8,
	pub max_tick: Tick,
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
		// Format : first line contains max_tick and delta
		// Then, the list of blocked links separated by a space
		let mut line = String::new();
		reader.read_line(&mut line).unwrap();
		let (max_tick, delta) = scanf!(line.trim(), "{} {}", Tick, u8).unwrap();

		// while there is new line, read it and append to the result
		let mut blocked_links = Vec::with_capacity(graph.nb_nodes() as usize);
		for _ in 0..graph.nb_nodes() {
			let mut line = String::new();
			reader.read_line(&mut line).unwrap();
			let blocks = line.split_whitespace().map(|x| x.parse().unwrap()).collect();
			blocked_links.push(blocks);
		}

		return Self {
			blocked_links,
			delta,
			max_tick,
		};
	}
}

impl Attack for StaticAttack {
	fn max_tick(&self) -> Tick {
		return self.max_tick;
	}

	fn delta(&self) -> u8 {
		return self.delta;
	}

	fn blocked_links(&self) -> Cow<Vec<DeletedLink>> {
		let mut blocked_links = vec![];
		for i in 0..self.blocked_links.len() {
			for j in 0..self.blocked_links[i].len() {
				blocked_links.push(DeletedLink {
					from: i as Node,
					to: self.blocked_links[i][j],
					times: (0..self.max_tick).collect(),
				});
			}
		}
		return Cow::Owned(blocked_links);
	}

	fn all_pairs_distances(&self, graph: &Graph) -> AttackResult {
		// Create a graph without the links that got blocked
		let mut new_graph = graph.clone();
		for i in 0..self.blocked_links.len() {
			for j in (0..self.blocked_links[i].len()).rev() {
				new_graph.remove_edge(i as Node, self.blocked_links[i][j]);
			}
		}

		// Compute the distance matrix
		let distance_matrix = new_graph.all_pairs_distances();
		return AttackResult::Static(distance_matrix);
	}
}

// TODO : ew it's ugly af
impl DynamicAttack {
	pub fn from_file(reader: &mut BufReader<File>) -> Self {
		let mut line = String::new();
		reader.read_line(&mut line).unwrap();
		let (nb_edges, delta, max_tick) = scanf!(line.trim(), "{} {} {}", usize, u8, Tick).unwrap();

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
			delta,
			max_tick: max_tick as Tick,
		};
	}
}

impl Attack for DynamicAttack {
	fn max_tick(&self) -> Tick {
		return self.max_tick;
	}

	fn delta(&self) -> u8 {
		return self.delta;
	}

	fn blocked_links(&self) -> Cow<Vec<DeletedLink>> {
		return Cow::Borrowed(&self.blocked_links);
	}

	fn all_pairs_distances(&self, graph: &Graph) -> AttackResult {
		// Create the time varying graph
		let time_graph = graph_to_temporal(graph, self.max_tick(), &self.blocked_links(), self.delta());
		let mut dma = DistanceMatrixArray::zeroed(graph.nb_nodes(), self.max_tick());
		// Compute all distance matrices
		for t in 0..time_graph.max_tick() {
			for i in 0..time_graph.nb_nodes() {
				// Doing the j in reverse seems to improve slightly the performance
				// Probably because nodes that are far in real life apart also have far apart indexes
				for j in (0..time_graph.nb_nodes()).rev() {
					// Not 0 -> already computed
					if dma[(t, i, j)] != 0 {
						continue;
					}
					temporal_dijkstra_write(&time_graph, i as Node, j as Node, t, &mut dma);
				}
			}
			//println!("{}/{}", t, self.max_tick);
		}

		return AttackResult::Dynamic(dma);
	}
}
