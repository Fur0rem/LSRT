//! A singular attack.
//!
//! Attacks are separated into 2 categories:
//! 1. Static attacks : the blocked links are fixed over the whole time
//! 2. Dynamic attacks : the blocked links change over time
//! We did this to optimise the computation time for static attacks, as the distance matrix is the same for all times.

use std::fmt::Debug;
use std::{
	borrow::Cow,
	fs::File,
	io::{BufRead, BufReader},
};

use sscanf::scanf;

use crate::graph::Graph;
use crate::{
	distance_matrix::{DistanceMatrix, DistanceMatrixArray},
	time_varying_graph::{temporal_dijkstra_write, PhantomEdgesMap},
};
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
pub trait Attack: Debug {
	// efficiency + reachables
	fn all_pairs_distances(&self, graph: &Graph) -> AttackResult;
	fn efficiency_reachables(&self, graph: &Graph) -> (f64, u64) {
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
		let file = File::open(file).expect("Failed to open attack file");
		let mut reader = BufReader::new(file);
		let mut line = String::new();
		reader.read_line(&mut line).expect("Empty attack file");

		match line.trim() {
			"D" => Box::new(DynamicAttack::from_file(&mut reader)),
			"S" => Box::new(StaticAttack::from_file(&mut reader, graph)),
			_ => panic!("Invalid attack file"),
		}
	}
}

impl StaticAttack {
	pub fn from_file(reader: &mut BufReader<File>, graph: &Graph) -> Self {
		// Format : first line contains max_tick and delta
		// Then, the list of blocked links separated by a space
		let mut line = String::new();
		reader.read_line(&mut line).expect("Empty attack file");
		let (max_tick, delta) = scanf!(line.trim(), "{} {}", Tick, u8).expect("Invalid attack format : Expected <max tick> <delta>");

		// while there is new line, read it and append to the result
		let mut blocked_links = Vec::with_capacity(graph.nb_nodes().into());
		for _ in 0..graph.nb_nodes() {
			let mut line = String::new();
			reader.read_line(&mut line).unwrap();
			let blocks = line
				.split_whitespace()
				.map(|x| x.parse())
				.collect::<Result<Vec<Node>, _>>()
				.expect("Invalid attack format : Expected a list of nodes");
			blocked_links.push(blocks);
		}

		assert!(
			blocked_links.len() < Node::MAX.into(),
			"Too many nodes, the maximum is {}",
			Node::MAX
		);

		return Self {
			blocked_links,
			delta,
			max_tick,
		};
	}

	pub fn from_file_name(file_name: String, graph: &Graph) -> Self {
		let file = File::open(file_name).unwrap();
		let mut reader = BufReader::new(file);
		let mut line = String::new();
		reader.read_line(&mut line).unwrap();
		return Self::from_file(&mut reader, graph);
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
		let (nb_edges, delta, max_tick) =
			scanf!(line.trim(), "{} {} {}", usize, u8, Tick).expect("Invalid attack file : Expected <nb_edges> <delta> <max_tick>");

		// Then for each line, nb_blocked (node1,node2) <the times at which it is blocked>
		// For example : 4 (0,2) 5 9 17 18
		let mut blocked_links = vec![];
		for _ in 0..nb_edges {
			let mut line = String::new();

			reader
				.read_line(&mut line)
				.expect("Invalid attack file : The number of lines doesn't match the number of edges");

			let (information, times) = line.split_at(
				line.find(')')
					.expect("Invalid attack file : Expected format <Length of times> (<from>, <to>) <times>")
					+ 1,
			);

			let (_nb_blocked, from, to) = scanf!(information, "{} ({}, {})", usize, Node, Node)
				.expect("Invalid attack file : Expected <Length of times> (<from>, <to>) <times>");

			let times = times
				.split_whitespace()
				.map(|x| x.parse().expect("Invalid time : Expected a number lower than Tick::MAX"))
				.collect();

			blocked_links.push(DeletedLink { from, to, times });
		}

		return Self {
			blocked_links,
			delta,
			max_tick,
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
		let edges_map = PhantomEdgesMap::from_graph(&time_graph);
		let mut dma = DistanceMatrixArray::zeroed(graph.nb_nodes(), self.max_tick());
		// Compute all distance matrices
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

		return AttackResult::Dynamic(dma);
	}
}
