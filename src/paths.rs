use crate::distance_matrix::{DistanceMatrix, DistanceMatrixArray, SquareMatrix};
use crate::graph::{Graph, Neighbour};
use crate::time_varying_graph::{PhantomEdgesMap, TimeVaryingGraph};
use crate::{Node, Tick};

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

pub fn merge_sorted(vec1: &[Tick], vec2: &[Tick], offset_vec_1: Tick, max_time: Tick) -> Vec<Tick> {
	let mut res = Vec::with_capacity(vec1.len() + vec2.len());
	// assume both vecs are sorted, merge them in a sorted way
	let mut i = 0;
	let mut j = 0;
	while i < vec1.len() && j < vec2.len() {
		let val1 = vec1[i].saturating_sub(offset_vec_1);
		let val2 = vec2[j];
		match val1.cmp(&val2) {
			std::cmp::Ordering::Less => {
				res.push(val1);
				i += 1;
			}
			std::cmp::Ordering::Equal => {
				res.push(val1);
				i += 1;
				j += 1;
			}
			std::cmp::Ordering::Greater => {
				res.push(val2);
				j += 1;
			}
		}

		if *res.last().unwrap() > max_time {
			return res;
		}
	}
	while i < vec1.len() {
		let val1 = vec1[i].saturating_sub(offset_vec_1);
		res.push(val1);
		i += 1;
	}
	while j < vec2.len() {
		res.push(vec2[j]);
		j += 1;
	}
	return res;
}

#[derive(Debug, Clone)]
pub struct SubPath {
	pub start: Node,
	pub end: Node,
	pub total_weight: Tick,
	pub time_taken: Tick,
}

pub struct SubPathsPyramid<'a> {
	pub graph: &'a Graph,
	pub subpaths: Vec<SubPath>,
}

// 1 has parents (None, 0), 2 has parents (0, None), 3 has parents (None, 1), 4 has parents (1, 2), 5 has parents (2, None)
// Structure :
// 0
// 1 2
// 3 4 5
impl<'a> SubPathsPyramid<'a> {
	pub fn print(&self) {
		let mut idx = 0;
		let mut prof = 0;
		for subpath in &self.subpaths {
			print!("{}--{}-->{} ", subpath.start, subpath.total_weight, subpath.end);
			if idx == prof {
				println!();
				prof += 1;
				idx = 0;
			}
			else {
				idx += 1;
			}
		}
	}
	pub fn get_prof(idx: usize) -> usize {
		if idx <= 1 {
			return idx;
		}
		//stack exchange get inverse of formula for sum of intergers from 1 to n?
		return (((2 * idx) as f64 + 0.25).sqrt() - 0.5).floor() as usize;
	}

	pub fn get_idx_left_son(idx: usize) -> usize {
		return idx + Self::get_prof(idx) + 1;
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
		let left_parent = if idx == minr { None } else { Some(idx - prof - 1) };
		let right_parent = if idx == maxr { None } else { Some(idx - prof) };
		return (left_parent, right_parent);
	}

	pub fn get_idx_children(&self, idx: usize) -> Option<(usize, usize)> {
		let (left_child, right_child) = (Self::get_idx_left_son(idx), Self::get_idx_right_son(idx));
		if left_child >= self.subpaths.len() {
			return None;
		}
		return Some((right_child, left_child));
	}

	pub fn leaves_range(&self) -> std::ops::Range<usize> {
		// The leaves are the last elements of the pyramid
		if self.subpaths.is_empty() {
			return 0..0;
		}
		let prof = Self::get_prof(self.subpaths.len() - 1);
		let (minr, maxr) = Self::get_range(prof);
		return minr..maxr + 1;
	}

	pub fn leaves(&'a self) -> &'a [SubPath] {
		// The leaves are the last elements of the pyramid
		if self.subpaths.len() <= 1 {
			return &self.subpaths;
		}

		let prof = Self::get_prof(self.subpaths.len() - 1);
		let (minr, maxr) = Self::get_range(prof);
		return &self.subpaths[minr..=maxr];
	}

	/// Inclusive range
	pub const fn get_range(prof: usize) -> (usize, usize) {
		if prof == 0 {
			return (0, 0);
		}
		let most_right = ((prof + 2) * (prof + 1)) / 2;
		return (most_right - prof - 1, most_right - 1);
	}

	pub fn from_path<'b>(graph: &'b Graph, path: &Path) -> SubPathsPyramid<'b> {
		if path.steps.is_empty() {
			return SubPathsPyramid { graph, subpaths: vec![] };
		}
		if path.steps.len() == 1 {
			return SubPathsPyramid {
				graph,
				subpaths: vec![SubPath {
					start: path.start,
					end: path.steps[0].node,
					total_weight: path.steps[0].weight,
					time_taken: 0,
				}],
			};
		}
		let nb_elems = (path.steps.len() * (path.steps.len() + 1)) / 2;
		let mut elems = vec![
			SubPath {
				start: 0,
				end: 0,
				total_weight: 0,
				time_taken: 0,
			};
			nb_elems
		];

		let mut prev_node = path.start;
		let mut leaf_index = ((path.steps.len() - 1) * (path.steps.len())) / 2;
		let mut wait = 0;
		for step in &path.steps {
			elems[leaf_index] = SubPath {
				start: prev_node,
				end: step.node,
				total_weight: step.weight,
				time_taken: wait,
			};
			prev_node = step.node;
			leaf_index += 1;
			wait += step.weight;
		}

		let leaf_index = ((path.steps.len() - 1) * (path.steps.len())) / 2;
		for p in (0..SubPathsPyramid::get_prof(leaf_index)).rev() {
			let (minr, maxr) = SubPathsPyramid::get_range(p);
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
					time_taken: left.time_taken,
				};
				elems[i] = new_path;
			}
		}

		return SubPathsPyramid { graph, subpaths: elems };
	}

	/// Invalidate the path in the distance matrix array because an attack impacted the path
	///
	/// How to use this function :
	/// ```rust
	/// let graph = Graph::from_file(&mut reader);
	/// let attack = DynamicAttack::from_file(&mut reader);
	/// let (dst_mat, paths) = graph.all_pairs_distances_and_paths(max_tick);
	/// let dma = DistanceMatrixArray::propagated_and_infinitied(&dst_mat, max_tick);
	/// let time_graph = graph_to_temporal(graph, max_tick, &attack.blocked_links(), attack.delta());
	/// let mut already_done = SquareMatrix::new(graph.nb_nodes().into(), false);
	/// for path in &paths {
	///     path.invalidate_path(&time_graph, &mut already_done, &mut dma, &dst_mat);
	/// }
	/// ```
	///
	/// Note: This approach is far slower than just recomputing everything, the method used in [the actual algorithm][`crate::attack_series::AttackSeries::efficiencies_reachables`]
	/// for computing the distances and reachables.
	pub fn invalidate_path(
		&self, time_varying_graph: &TimeVaryingGraph, already_done: &mut SquareMatrix<bool>, dma: &mut DistanceMatrixArray,
		original_dst_mat: &DistanceMatrix, edges_map: &PhantomEdgesMap,
	) {
		if self.subpaths.is_empty() {
			return;
		}
		let total = &self.subpaths[0];
		let (from, to) = (total.start, total.end);
		if already_done[(from.into(), to.into())] {
			return;
		}

		let mut times_to_invalidate = vec![vec![]; self.subpaths.len()];
		let mut total_invalidations = vec![];
		// Invalidate the leaves
		let mut to_invalidate = vec![];
		for leaf in self.leaves_range().rev() {
			let subpath = &self.subpaths[leaf];
			let phantom_edge = edges_map.get_edge(subpath.start, subpath.end).unwrap();
			let unattacked_edge = self.graph.get_edge(subpath.start, subpath.end).unwrap();
			let mut invalid_times = vec![];
			for t in 0..time_varying_graph
				.max_tick()
				.saturating_sub(original_dst_mat[(subpath.start, subpath.end)] - 1)
			{
				if phantom_edge[t] != unattacked_edge.weight {
					invalid_times.push(t);
				}
			}
			if !invalid_times.is_empty() {
				to_invalidate.push(leaf);
				times_to_invalidate[leaf] = invalid_times;
				total_invalidations.push(leaf);
			}
		}

		// Propagate the invalidations to their parents layer by layer
		while !to_invalidate.is_empty() {
			let mut next_invalidate: Vec<usize> = vec![];
			for i in to_invalidate {
				if let Some((right_child, left_child)) = self.get_idx_children(i) {
					// gather all the invalid times from your children
					let right_children_offset = self.subpaths[right_child].time_taken - self.subpaths[i].time_taken;
					let mut invalid_times_right = Vec::with_capacity(times_to_invalidate[right_child].len());
					for t in &times_to_invalidate[right_child] {
						if *t >= right_children_offset {
							invalid_times_right.push(t - right_children_offset);
						}
					}
					let invalid_times_left = &times_to_invalidate[left_child];
					let invalid_times = merge_sorted(
						&invalid_times_right,
						invalid_times_left,
						right_children_offset,
						time_varying_graph.max_tick(),
					);

					times_to_invalidate[i] = invalid_times;
				}

				if times_to_invalidate[i].is_empty() {
					continue;
				}

				total_invalidations.push(i);

				let (left_parent, right_parent) = Self::get_idx_parents(i);
				if let Some(right_parent) = right_parent {
					let length = next_invalidate.len();
					if length == 0 || next_invalidate[length - 1] != right_parent {
						next_invalidate.push(right_parent);
					}
				}
				if let Some(left_parent) = left_parent {
					next_invalidate.push(left_parent);
				}
			}
			to_invalidate = next_invalidate;
		}

		// Invalidate the paths
		for i in total_invalidations {
			let invalid_times = &times_to_invalidate[i];
			let subpath = &self.subpaths[i];
			for &tick in invalid_times {
				// If we can't find a path short enough to fit in the time budget, we can't use it
				if subpath.total_weight.saturating_add(tick) > time_varying_graph.max_tick() {
					break;
				}
				dma[(tick, subpath.start, subpath.end)] = 0;
			}
			already_done[(subpath.start.into(), subpath.end.into())] = true;
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
			if current_time >= self.time_graph.max_tick() {
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
	pub fn get_prof(idx: usize) -> usize {
		return SubPathsPyramid::get_prof(idx);
	}

	pub fn get_idx_left_son(idx: usize) -> usize {
		return SubPathsPyramid::get_idx_left_son(idx);
	}

	pub fn get_idx_right_son(idx: usize) -> usize {
		return SubPathsPyramid::get_idx_right_son(idx);
	}

	pub fn get_idx_parents(idx: usize) -> (Option<usize>, Option<usize>) {
		return SubPathsPyramid::get_idx_parents(idx);
	}

	const fn get_range(prof: usize) -> (usize, usize) {
		return SubPathsPyramid::get_range(prof);
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
			if wait > graph.max_tick() {
				elems[leaf_index] = TemporalSubPath {
					start: prev_node,
					end: step.node,
					total_weight: Tick::MAX,
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

		if path.steps.len() == 1 {
			return TemporalSubPathsPyramid { graph, subpaths: elems };
		}

		// Fill the rest of the pyramid
		let leaf_index = ((path.steps.len() - 1) * (path.steps.len())) / 2;
		for p in (0..TemporalSubPathsPyramid::get_prof(leaf_index)).rev() {
			let (minr, maxr) = TemporalSubPathsPyramid::get_range(p);
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
