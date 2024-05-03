//! A singular attack.
//!
//! Attacks are separated into 2 categories:
//! 1. Static attacks : the [`BlockedLink`]s are fixed over the whole time
//! 2. Dynamic attacks : the [`BlockedLink`]s change over time
//! We did this to optimise the computation time for static attacks, as the [`DistanceMatrix`] is the same for all ticks.

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
	/// Computes the distance matrix for the attack.
	fn all_pairs_distances(&self, graph: &Graph) -> AttackResult;
	/// Computes the efficiency and reachables for the attack.
	fn efficiency_reachables(&self, graph: &Graph) -> (f64, u64) {
		let distance_matrix = self.all_pairs_distances(graph);
		match distance_matrix {
			AttackResult::Static(dm) => dm.efficiency_reachables(self.max_tick()),
			AttackResult::Dynamic(dma) => dma.efficiency_reachables(self.max_tick()),
		}
	}
	/// Returns the max tick of the attack.
	fn max_tick(&self) -> Tick;
	/// Returns the blocked links of the attack.
	fn blocked_links(&self) -> Cow<Vec<BlockedLink>>;
	/// Returns the delta of the attack.
	fn delta(&self) -> Tick;
}

/// A blocked link from an attack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockedLink {
	/// The node from which the link is deleted.
	pub from: Node,
	/// The node to which the link is deleted.
	pub to: Node,
	/// The times at which the link is deleted, they are sorted and unique.
	/// Note : A [`Time`] is different from a [`Tick`]
	pub times: Vec<Time>,
}

/// A dynamic attack.
#[derive(Debug)]
pub struct DynamicAttack {
	/// The blocked links.
	pub blocked_links: Vec<BlockedLink>,
	/// The delta of the attack.
	pub delta: Tick,
	/// The max tick of the attack.
	pub max_tick: Tick,
}

/// A static attack.
#[derive(Debug)]
pub struct StaticAttack {
	/// The blocked links.
	pub blocked_links: Vec<Vec<Node>>,
	/// The delta of the attack.
	pub delta: Tick,
	/// The max tick of the attack.
	pub max_tick: Tick,
}

impl dyn Attack {
	/// Creates an attack from a file.
	/// It reads the first line to determine if the attack is static or dynamic.
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
	/// Creates a static attack from a file.
	/// The file must contain in the second line the max tick and delta, then for the rest the blocked links.
	/// This assumes the first line has already been read and contains S, therefore the reader is at the second line.
	/// Then it reads as many lines as there are nodes, and each line contains which of its neighbors are blocked.
	/// For example, if the first line is `S`, the second line is `100 1`, and the third line is `1 2 3`, it means that the attack lasts 100 ticks, the delta is 1, and the first node has its links to 1, 2 and 3 blocked.
	pub fn from_file(reader: &mut BufReader<File>, graph: &Graph) -> Self {
		let mut line = String::new();
		reader.read_line(&mut line).expect("Empty attack file");
		let (max_tick, delta) = scanf!(line.trim(), "{} {}", Tick, Tick).expect("Invalid attack format : Expected <max tick> <delta>");

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

	/// Creates a static attack from a file name.
	/// It opens the file and calls `from_file`.
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

	fn delta(&self) -> Tick {
		return self.delta;
	}

	fn blocked_links(&self) -> Cow<Vec<BlockedLink>> {
		let mut blocked_links = vec![];
		for i in 0..self.blocked_links.len() {
			for j in 0..self.blocked_links[i].len() {
				blocked_links.push(BlockedLink {
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
	/// Creates a dynamic attack from a file.
	/// The file must contain in the first line the number of edges, delta and max tick, then for the rest the blocked links.
	/// A line has format: `x (node1,node2) t0 t1 t2 t3 ...` where x is the number of times the link is blocked, node1 and node2 are the nodes of the link, and t0, t1, t2, ... are the times at which the link is blocked.
	/// It assumes the times are sorted in ascending order and unique.
	/// It assumes the first line has already been read and contains D, therefore the reader is at the second line.
	pub fn from_file(reader: &mut BufReader<File>) -> Self {
		let mut line = String::new();
		reader.read_line(&mut line).unwrap();
		let (nb_edges, delta, max_tick) =
			scanf!(line.trim(), "{} {} {}", usize, Tick, Tick).expect("Invalid attack file : Expected <nb_edges> <delta> <max_tick>");

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

			blocked_links.push(BlockedLink { from, to, times });
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

	fn delta(&self) -> Tick {
		return self.delta;
	}

	fn blocked_links(&self) -> Cow<Vec<BlockedLink>> {
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
