use crate::attack;
use crate::distance_matrix::DistanceMatrixArray;
use crate::graph::{Graph, Neighbour, Node, Tick};
use crate::time_varying_graph::TimeVaryingGraph;

pub struct Path<'a> {
	pub graph: &'a Graph,
	pub start: Node,
	pub steps: Vec<Neighbour>,
}

impl Path<'_> {
	pub fn cost(&self) -> Tick {
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
}

#[derive(Debug, Clone)]
pub struct SubPath {
	pub start: Node,
	pub end: Node,
	pub total_weight: Tick,
}

pub struct SubPathsPyramid<'a> {
	pub graph: &'a Graph,
	pub subpaths: Vec<SubPath>,
}

impl<'a> SubPathsPyramid<'a> {
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

	pub fn leaves(&'a self) -> &'a [SubPath] {
		// The leaves are the last elements of the pyramid
		if self.subpaths.is_empty() {
			return &self.subpaths;
		}
		if self.subpaths.len() == 1 {
			return &self.subpaths;
		}

		let prof = Self::get_prof(self.subpaths.len() - 1);
		let (minr, maxr) = Self::get_range(prof);
		return &self.subpaths[minr..=maxr];
	}

	fn get_range(prof: usize) -> (usize, usize) {
		if prof == 0 {
			return (0, 0);
		}
		let most_right = ((prof + 1) * prof) / 2;
		return (most_right - prof, most_right - 1);
	}

	pub fn from_path<'b>(graph: &'b Graph, path: &Path) -> SubPathsPyramid<'b> {
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

	pub fn invalidate_path(&self, dma: &mut DistanceMatrixArray, attack: &attack::DynamicAttack) {
		// look through all the leaves, if any of them is in the attack, invalidate the path by writing 0 in the matrix,
		// then propagate the change to the root
		let mut to_invalidate = vec![];
		for (i, subpath) in self.subpaths.iter().enumerate() {
			for blocked_link in &attack.blocked_links {
				if blocked_link.from == subpath.start && blocked_link.to == subpath.end {
					to_invalidate.push(i);
					// clear the value in the matrix
					for time in 0..attack.max_time {
						dma[(time, subpath.start, subpath.end)] = 0;
					}
					break;
				}
			}
		}

		if to_invalidate.is_empty() {
			return;
		}
		//println!("Invalidating path : {:?}", to_invalidate);

		// for all the leaves that have been invalidated, propagate the change to their parents
		while !to_invalidate.is_empty() {
			let mut new_to_invalidate = vec![];
			for i in to_invalidate {
				let (left_parent, right_parent) = SubPathsPyramid::get_idx_parents(i);
				if let Some(left_parent) = left_parent {
					if dma[(0, self.subpaths[left_parent].start, self.subpaths[left_parent].end)] != 0 {
						new_to_invalidate.push(left_parent);
					}
				}
				if let Some(right_parent) = right_parent {
					if dma[(0, self.subpaths[right_parent].start, self.subpaths[right_parent].end)] != 0 {
						new_to_invalidate.push(right_parent);
					}
				}
				for time in 0..attack.max_time {
					dma[(time, self.subpaths[i].start, self.subpaths[i].end)] = 0;
				}
			}
			to_invalidate = new_to_invalidate;
		}
	}
}

/// A path in a time varying graph
#[derive(Debug, Clone)]
pub struct TemporalPath<'a> {
	pub time_graph: &'a TimeVaryingGraph,
	pub start: Node,
	pub steps: Vec<Neighbour>,
	pub start_time: Tick,
}

impl<'a> TemporalPath<'a> {
	pub fn cost(&self) -> Tick {
		if self.steps.is_empty() {
			return Tick::MAX;
		}
		let mut current_time = self.start_time;
		let mut total_weight = 0;
		for step in &self.steps {
			if current_time as usize >= self.time_graph.max_time as usize {
				return Tick::MAX;
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
		if self.cost() == Tick::MAX {
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
	pub total_weight: Tick,
	pub start_time: Tick,
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
					total_weight: Tick::MAX,
					start_time: wait,
				};
			} else {
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
