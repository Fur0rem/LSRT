use crate::graph::{Graph, Neighbour, Node, Time, Weight};
use crate::time_varying_graph::TimeVaryingGraph;

pub struct Path<'a> {
	pub graph: &'a Graph,
	pub start: Node,
	pub steps: Vec<Neighbour>,
}

impl Path<'_> {
	pub fn cost(&self) -> Weight {
		self.steps.iter().map(|x| x.weight).sum()
	}

	pub fn print(&self) {
		//print the start, then each step, then the end
		print!("P {} ", self.start);
		for step in &self.steps {
			print!("--{}-->{} ", step.weight, step.node);
		}
		println!();
	}

	pub fn empty(graph: &Graph) -> Path {
		Path {
			graph,
			start: 0,
			steps: vec![],
		}
	}
}

#[derive(Debug, Clone)]
pub struct SubPath {
	pub start: Node,
	pub end: Node,
	pub total_weight: Weight,
}

pub struct SubPathsPyramid<'a> {
	pub graph: &'a Graph,
	pub subpaths: Vec<SubPath>,
}

impl SubPathsPyramid<'_> {
	fn get_prof(idx: usize) -> usize {
		//stack exchange get inverse of formula for sum of intergers from 1 to n?
		return (((2 * idx) as f64 + 0.25).sqrt() - 0.5) as usize + 1;
	}

	fn get_idx_left_son(idx: usize) -> usize {
		return idx + Self::get_prof(idx);
	}

	pub fn get_idx_right_son(idx: usize) -> usize {
		return Self::get_idx_left_son(idx) + 1;
	}

	pub fn get_idx_parents(idx: usize) -> (Option<usize>, Option<usize>) {
		if idx == 0 {
			return (None, None);
		}
		let prof = Self::get_prof(idx);
		let (minr, maxr) = Self::get_range(prof);
		let left_parent = if idx == minr { None } else { Some(idx - prof) };
		let right_parent = if idx == maxr { None } else { Some(idx - (prof - 1) + 1) };
		return (left_parent, right_parent);
	}

	fn get_range(prof: usize) -> (usize, usize) {
		if prof == 0 {
			return (0, 0);
		}
		let most_right = ((prof + 1) * prof) / 2;
		return (most_right - prof, most_right - 1);
	}

	// TODO : clean this up
	pub fn from_path<'a>(graph: &'a Graph, path: &Path) -> SubPathsPyramid<'a> {
		if path.steps.is_empty() {
			return SubPathsPyramid { graph, subpaths: vec![] };
		}
		let nb_elems = (path.steps.len() * (path.steps.len() + 1)) / 2;
		let mut elems = vec![
			SubPath {
				start: 0,
				end: 0,
				total_weight: 0,
			};
			nb_elems
		];

		let mut prev_node = path.start;
		let mut leaf_index = ((path.steps.len() - 1) * (path.steps.len())) / 2;
		for step in &path.steps {
			elems[leaf_index] = SubPath {
				start: prev_node,
				end: step.node,
				total_weight: step.weight,
			};
			prev_node = step.node;
			leaf_index += 1;
		}

		let leaf_index = ((path.steps.len() - 1) * (path.steps.len())) / 2;

		for p in (0..SubPathsPyramid::get_prof(leaf_index) - 1).rev() {
			let (minr, maxr) = SubPathsPyramid::get_range(p + 1);
			for i in minr..=maxr {
				let il = SubPathsPyramid::get_idx_left_son(i);
				let ir = SubPathsPyramid::get_idx_right_son(i);

				let shared = SubPathsPyramid::get_idx_left_son(ir);
				let shared_len = if shared < nb_elems { elems[shared].total_weight } else { 0 };
				let left = &elems[il];
				let right = &elems[ir];

				let new_path = SubPath {
					start: left.start,
					end: right.end,
					total_weight: (left.total_weight + right.total_weight) - shared_len,
				};
				elems[i] = new_path;
			}
		}

		return SubPathsPyramid { graph, subpaths: elems };
	}
}

/// A path in a time varying graph
#[derive(Debug, Clone)]
pub struct TemporalPath<'a> {
	pub time_graph: &'a TimeVaryingGraph,
	pub start: Node,
	pub steps: Vec<Neighbour>,
	pub start_time: Time,
}

impl<'a> TemporalPath<'a> {
	pub fn cost(&self) -> Weight {
		if self.steps.is_empty() {
			return Weight::MAX;
		}
		let mut current_time = self.start_time;
		let mut total_weight = 0;
		for step in &self.steps {
			if current_time as usize >= self.time_graph.max_time as usize {
				return Weight::MAX;
			}
			total_weight += step.weight;
			current_time += step.weight;
		}
		return total_weight;
	}

	pub fn print(&self) {
		println!("TP {}", self.to_str());
	}

	pub fn to_str(&self) -> String {
		if self.steps.is_empty() {
			return "No path".to_string();
		}
		if self.cost() == Weight::MAX {
			return "No path".to_string();
		}
		let to = self.steps.last().unwrap().node;
		let mut res = format!("{} -> {} : {} at time {}\n", self.start, to, self.cost(), self.start_time);
		//print the start, then each step, then the end
		res.push_str(&format!("{} ", self.start));
		for step in &self.steps {
			let weight = step.weight;
			res.push_str(&format!("--{}-->{} ", weight, step.node));
		}
		res.push('\n');
		return res;
	}
}

/// A subpath from a temporal path
#[derive(Debug, Clone)]
pub struct TemporalSubPath {
	pub start: Node,
	pub end: Node,
	pub total_weight: Weight,
	pub start_time: Time,
}

impl TemporalSubPath {
	pub fn print(&self) {
		println!("TSP {}--{}-->{} @ {}", self.start, self.total_weight, self.end, self.start_time);
	}
}

/// A pyramid of subpaths
/// This is used to store all the subpaths of a path
/// And efficiently re-use already computed subpaths
/// For example, if we have a path 0--1--2--3, we can store the subpaths 0--1, 1--2, 2--3, 0--1--2, 1--2--3, 0--1--2--3
pub struct TemporalSubPathsPyramid<'a> {
	pub graph: &'a TimeVaryingGraph,
	pub subpaths: Vec<TemporalSubPath>,
}

impl TemporalSubPathsPyramid<'_> {
	fn get_prof(idx: usize) -> usize {
		//stack exchange get inverse of formula for sum of intergers from 1 to n?
		return (((2 * idx) as f64 + 0.25).sqrt() - 0.5) as usize + 1;
	}

	fn get_idx_left_son(idx: usize) -> usize {
		return idx + Self::get_prof(idx);
	}

	pub fn get_idx_right_son(idx: usize) -> usize {
		return Self::get_idx_left_son(idx) + 1;
	}

	pub fn get_idx_parents(idx: usize) -> (Option<usize>, Option<usize>) {
		if idx == 0 {
			return (None, None);
		}
		let prof = Self::get_prof(idx);
		let (minr, maxr) = Self::get_range(prof);
		let left_parent = if idx == minr { None } else { Some(idx - prof) };
		let right_parent = if idx == maxr { None } else { Some(idx - prof + 1) };
		return (left_parent, right_parent);
	}

	fn get_range(prof: usize) -> (usize, usize) {
		if prof == 0 {
			return (0, 0);
		}
		let truc = ((prof + 1) * prof) / 2;
		return (truc - prof, truc - 1);
	}

	// TODO : clean this up
	pub fn from_path<'a>(graph: &'a TimeVaryingGraph, path: &TemporalPath) -> TemporalSubPathsPyramid<'a> {
		if path.steps.is_empty() {
			return TemporalSubPathsPyramid { graph, subpaths: vec![] };
		}

		// Initialise the subpath pyramid
		let nb_elems = (path.steps.len() * (path.steps.len() + 1)) / 2;
		let mut elems = vec![
			TemporalSubPath {
				start: 0,
				end: 0,
				total_weight: 0,
				start_time: 0,
			};
			nb_elems
		];

		// Fill the leaves of the pyramid
		let mut prev_node = path.start;
		let mut leaf_index = ((path.steps.len() - 1) * (path.steps.len())) / 2;
		let mut wait = path.start_time;
		for step in &path.steps {
			if wait as usize >= graph.max_time as usize {
				elems[leaf_index] = TemporalSubPath {
					start: prev_node,
					end: step.node,
					total_weight: Weight::MAX,
					start_time: wait,
				};
			}
			else {
				elems[leaf_index] = TemporalSubPath {
					start: prev_node,
					end: step.node,
					total_weight: step.weight,
					start_time: wait,
				};
				wait += step.weight;
			}
			prev_node = step.node;
			leaf_index += 1;
		}

		// Fill the rest of the pyramid
		let leaf_index = ((path.steps.len() - 1) * (path.steps.len())) / 2;
		for p in (0..TemporalSubPathsPyramid::get_prof(leaf_index) - 1).rev() {
			let (minr, maxr) = TemporalSubPathsPyramid::get_range(p + 1);
			for i in minr..=maxr {
				let il = TemporalSubPathsPyramid::get_idx_left_son(i);
				let ir = TemporalSubPathsPyramid::get_idx_right_son(i);

				let shared = TemporalSubPathsPyramid::get_idx_left_son(ir);
				let shared_len = if shared < nb_elems { elems[shared].total_weight } else { 0 };
				let left = &elems[il];
				let right = &elems[ir];
				let new_path = TemporalSubPath {
					start: left.start,
					end: right.end,
					total_weight: (left.total_weight + right.total_weight) - shared_len,
					start_time: left.start_time,
				};

				elems[i] = new_path;
			}
		}

		return TemporalSubPathsPyramid { graph, subpaths: elems };
	}
}
