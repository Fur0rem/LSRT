//! Contains structures and functions to work with paths in a graph or a time varying graph

use crate::distance_matrix::{DistanceMatrix, DistanceMatrixArray, SquareMatrix};
use crate::graph::{Graph, Neighbour};
use crate::time_varying_graph::{PhantomEdgesMap, TimeVaryingGraph};
use crate::{Node, Tick};

use std::cmp::Ordering;
use std::ops::RangeInclusive;

/// A path in a graph.
///
/// A path that does not exist is represented by an empty vector.
pub struct Path<'a> {
	/// The graph the path is in
	pub graph: &'a Graph,
	/// The start node of the path
	pub start: Node,
	/// The steps of the path, each step is a neighbour of the previous node.
	/// The start node is not included in the steps
	pub steps: Vec<Neighbour>,
}

impl Path<'_> {
	/// Returns the total cost of the path
	/// The cost of a path is the sum of the weights of its steps
	/// If the path does not exist, the cost is `Tick::MAX`
	pub fn cost(&self) -> Tick {
		if self.steps.is_empty() {
			return Tick::MAX;
		}
		self.steps.iter().map(|x| x.weight).sum()
	}

	/// Prints the path
	/// Prints the start, then each step, then the end
	pub fn print(&self) {
		print!("P {} ", self.start);
		for step in &self.steps {
			print!("--{}-->{} ", step.weight, step.node);
		}
		println!();
	}
}

/// Assuming the two vectors are sorted in ascending order, merges them into a single sorted vector, with the offset substracted from the first vector.
///
/// If the value of any element is greater than `max_time`, the function stops and returns the result
/// If any element of vec1 is less than offset_vec_1, it is ignored
/// If there are no duplicates in each vector respectively, the result will have no duplicates
fn merge_sorted(vec1: &[Tick], vec2: &[Tick], offset_vec_1: Tick, max_time: Tick) -> Vec<Tick> {
	let mut res = Vec::with_capacity(vec1.len() + vec2.len());
	let mut i = 0;
	let mut j = 0;

	// Merge the two vectors
	while i < vec1.len() && j < vec2.len() {
		if vec1[i] < offset_vec_1 {
			i += 1;
			continue;
		}
		let val1 = vec1[i] - offset_vec_1;
		let val2 = vec2[j];
		match val1.cmp(&val2) {
			Ordering::Less => {
				res.push(val1);
				i += 1;
			}
			Ordering::Equal => {
				res.push(val1);
				i += 1;
				j += 1;
			}
			Ordering::Greater => {
				res.push(val2);
				j += 1;
			}
		}
		if *res.last().unwrap() > max_time {
			return res;
		}
	}

	// Add the remaining elements
	while i < vec1.len() {
		let val1 = vec1[i].saturating_sub(offset_vec_1);
		if val1 > max_time {
			break;
		}
		res.push(val1);
		i += 1;
	}

	while j < vec2.len() {
		if vec2[j] > max_time {
			break;
		}
		res.push(vec2[j]);
		j += 1;
	}
	return res;
}

/// A path that is included inside another path
#[derive(Debug, Clone)]
pub struct SubPath {
	/// The start node of the subpath
	pub start: Node,
	/// The end node of the subpath
	pub end: Node,
	/// The total cost of the subpath
	pub total_cost: Tick,
	/// The time taken to reach the start of the subpath
	pub start_tick: Tick,
}

/// A pyramid of subpaths used to store all the subpaths of a path and efficiently re-use already computed subpaths.
///
/// For example, if we have a path 0--1--2--3, we can store these subpaths
/// ```plaintext
///     0--1--2--3
///  0--1--2 | 1--2--3
/// 0--1 | 1--2 | 2--3
/// ```
/// It is similar to a segment tree, but with a pyramidal structure.
/// The layers are numbered from 0 for the top, and the last layer is the bricks, similar to leaves in a perfectly balanced tree.
pub struct SubPathsPyramid<'a> {
	pub graph: &'a Graph,
	pub subpaths: Vec<SubPath>,
}

impl<'a> SubPathsPyramid<'a> {
	/// Prints the pyramid, layer by layer
	pub fn print(&self) {
		let mut idx = 0;
		let mut layer = 0;
		for subpath in &self.subpaths {
			print!("{}--{}-->{} ", subpath.start, subpath.total_cost, subpath.end);
			if idx == layer {
				println!();
				layer += 1;
				idx = 0;
			}
			else {
				idx += 1;
			}
		}
	}

	/// Returns the layer of the pyramid where the element is
	pub fn get_layer(idx: usize) -> usize {
		if idx <= 1 {
			return idx;
		}
		//stack exchange get inverse of formula for sum of intergers from 1 to n?
		return (((2 * idx) as f64 + 0.25).sqrt() - 0.5) as usize;
	}

	/// Returns the index of the left son of an element in the pyramid.
	/// It does not check if the son exists, however [`Self::get_idx_children`] does
	pub fn get_idx_left_son(idx: usize) -> usize {
		return idx + Self::get_layer(idx) + 1;
	}

	/// Returns the index of the right son of an element in the pyramid.
	/// It does not check if the son exists, however [`Self::get_idx_children`] does
	pub fn get_idx_right_son(idx: usize) -> usize {
		return Self::get_idx_left_son(idx) + 1;
	}

	/// Returns the indices of the parents of an element in the pyramid.
	/// None if the parent does not exist, because the element is at the edge of the pyramid, or if the element is the top of the pyramid
	pub fn get_idx_parents(idx: usize) -> (Option<usize>, Option<usize>) {
		if idx == 0 {
			return (None, None);
		}
		let layer = Self::get_layer(idx);
		let range = Self::get_range(layer);
		let (minr, maxr) = (*range.start(), *range.end());
		let left_parent = if idx == minr { None } else { Some(idx - layer - 1) };
		let right_parent = if idx == maxr { None } else { Some(idx - layer) };
		return (left_parent, right_parent);
	}

	/// Returns the indices of the children of an element in the pyramid.
	/// None if the child does not exist, because the element is at the bottom layer of the pyramid
	pub fn get_idx_children(&self, idx: usize) -> Option<(usize, usize)> {
		let (left_child, right_child) = (Self::get_idx_left_son(idx), Self::get_idx_right_son(idx));
		if left_child >= self.subpaths.len() {
			return None;
		}
		return Some((right_child, left_child));
	}

	/// Returns the range of the indices of the bricks of the pyramid
	pub fn bricks_range(&self) -> RangeInclusive<usize> {
		if self.subpaths.is_empty() {
			return 1..=0;
		}
		let layer = Self::get_layer(self.subpaths.len() - 1);
		return Self::get_range(layer);
	}

	/// Returns the bricks of the pyramid
	pub fn bricks(&'a self) -> &'a [SubPath] {
		// If there is only one element, it is the brick
		if self.subpaths.len() <= 1 {
			return &self.subpaths;
		}

		let layer = Self::get_layer(self.subpaths.len() - 1);
		return &self.subpaths[Self::get_range(layer)];
	}

	/// Returns the inclusive range of the indices of the elements of the pyramid at a given layer
	pub const fn get_range(layer: usize) -> RangeInclusive<usize> {
		if layer == 0 {
			return 0..=0;
		}
		let most_right = ((layer + 2) * (layer + 1)) / 2;
		return (most_right - layer - 1)..=(most_right - 1);
	}

	/// Constructs a pyramid of subpaths from a path
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
					total_cost: path.steps[0].weight,
					start_tick: 0,
				}],
			};
		}
		let nb_elems = (path.steps.len() * (path.steps.len() + 1)) / 2;
		let mut elems = vec![
			SubPath {
				start: 0,
				end: 0,
				total_cost: 0,
				start_tick: 0,
			};
			nb_elems
		];

		let mut prev_node = path.start;
		let mut brick_index = ((path.steps.len() - 1) * (path.steps.len())) / 2;
		let mut wait = 0;
		for step in &path.steps {
			elems[brick_index] = SubPath {
				start: prev_node,
				end: step.node,
				total_cost: step.weight,
				start_tick: wait,
			};
			prev_node = step.node;
			brick_index += 1;
			wait += step.weight;
		}

		let brick_index = ((path.steps.len() - 1) * (path.steps.len())) / 2;
		for p in (0..SubPathsPyramid::get_layer(brick_index)).rev() {
			for i in SubPathsPyramid::get_range(p) {
				let il = SubPathsPyramid::get_idx_left_son(i);
				let ir = SubPathsPyramid::get_idx_right_son(i);

				let shared = SubPathsPyramid::get_idx_left_son(ir);
				let shared_len = if shared < nb_elems { elems[shared].total_cost } else { 0 };
				let left = &elems[il];
				let right = &elems[ir];

				let new_path = SubPath {
					start: left.start,
					end: right.end,
					total_cost: (left.total_cost + right.total_cost) - shared_len,
					start_tick: left.start_tick,
				};
				elems[i] = new_path;
			}
		}

		return SubPathsPyramid { graph, subpaths: elems };
	}

	/// Invalidate the path in the distance matrix array because an attack impacted the path
	/// You can see an example of how to use this function in [here][`crate::attack_series::DynamicAttackSeries::efficiencies_invalidation`]
	///
	/// Note : This approach is far slower than just recomputing everything, the method used in [the actual program][crate::attack_series::DynamicAttackSeries::efficiencies_write]
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
		// Invalidate the bricks
		let mut to_invalidate = vec![];
		for brick in self.bricks_range().rev() {
			let subpath = &self.subpaths[brick];
			let phantom_edge = edges_map.get_edge(subpath.start, subpath.end).unwrap();
			let unattacked_edge = self.graph.get_edge(subpath.start, subpath.end).unwrap();
			let mut invalid_times = vec![];
			let max_time_reachable = time_varying_graph
				.max_tick()
				.saturating_sub(original_dst_mat[(subpath.start, subpath.end)] - 1);

			for t in 0..max_time_reachable {
				if phantom_edge[t] != unattacked_edge.weight {
					invalid_times.push(t);
				}
			}
			if !invalid_times.is_empty() {
				to_invalidate.push(brick);
				times_to_invalidate[brick] = invalid_times;
				total_invalidations.push(brick);
			}
		}

		// Propagate the invalidations to their parents layer by layer
		while !to_invalidate.is_empty() {
			let mut next_invalidate: Vec<usize> = vec![];
			for i in to_invalidate {
				if let Some((right_child, left_child)) = self.get_idx_children(i) {
					// gather all the invalid times from your children
					let right_children_offset = self.subpaths[right_child].start_tick - self.subpaths[i].start_tick;
					let invalid_times_right = &times_to_invalidate[right_child];
					let invalid_times_left = &times_to_invalidate[left_child];
					let invalid_times = merge_sorted(
						invalid_times_right,
						invalid_times_left,
						right_children_offset,
						time_varying_graph.max_tick(),
					);
					if invalid_times.is_empty() {
						continue;
					}
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
				if subpath.total_cost.saturating_add(tick) > time_varying_graph.max_tick() {
					break;
				}
				dma[(tick, subpath.start, subpath.end)] = 0;
			}
			already_done[(subpath.start.into(), subpath.end.into())] = true;
		}
	}
}

/// A path in a time varying graph.
#[derive(Debug, Clone)]
pub struct TemporalPath<'a> {
	pub time_graph: &'a TimeVaryingGraph,
	pub start: Node,
	pub steps: Vec<Neighbour>,
	pub start_time: Tick,
}

impl<'a> TemporalPath<'a> {
	/// Returns the total cost of the path
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

	/// Prints the path
	pub fn print(&self) {
		println!("TP {}", self.to_str());
	}

	/// Returns the path as a string
	pub fn to_str(&self) -> String {
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

/// Temporal variant of [`SubPath`]
#[derive(Debug, Clone)]
pub struct TemporalSubPath {
	pub start: Node,
	pub end: Node,
	pub total_weight: Tick,
	pub start_time: Tick,
}

impl TemporalSubPath {
	/// Prints the subpath
	pub fn print(&self) {
		println!("TSP {}--{}-->{} @ {}", self.start, self.total_weight, self.end, self.start_time);
	}
}

/// Temporal variant of [`SubPathsPyramid`].
///
/// Any function regarding indexing or getting the children of a node is the same as [`SubPathsPyramid`], so use those functions
pub struct TemporalSubPathsPyramid<'a> {
	/// The graph the subpaths are in
	pub graph: &'a TimeVaryingGraph,
	/// The subpaths in the pyramid
	pub subpaths: Vec<TemporalSubPath>,
}

impl<'a> TemporalSubPathsPyramid<'a> {
	/// Constructs a pyramid of temporal subpaths from a temporal path
	pub fn from_path(graph: &'a TimeVaryingGraph, path: &TemporalPath) -> TemporalSubPathsPyramid<'a> {
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

		// Fill the bricks of the pyramid
		let mut prev_node = path.start;
		let mut brick_index = ((path.steps.len() - 1) * (path.steps.len())) / 2;
		let mut wait = path.start_time;
		for step in &path.steps {
			if wait > graph.max_tick() {
				elems[brick_index] = TemporalSubPath {
					start: prev_node,
					end: step.node,
					total_weight: Tick::MAX,
					start_time: wait,
				};
			}
			else {
				elems[brick_index] = TemporalSubPath {
					start: prev_node,
					end: step.node,
					total_weight: step.weight,
					start_time: wait,
				};
				wait += step.weight;
			}
			prev_node = step.node;
			brick_index += 1;
		}

		if path.steps.len() == 1 {
			return TemporalSubPathsPyramid { graph, subpaths: elems };
		}

		// Fill the rest of the pyramid
		let brick_index = ((path.steps.len() - 1) * (path.steps.len())) / 2;
		for p in (0..SubPathsPyramid::get_layer(brick_index)).rev() {
			for i in SubPathsPyramid::get_range(p) {
				let il = SubPathsPyramid::get_idx_left_son(i);
				let ir = SubPathsPyramid::get_idx_right_son(i);

				let shared = SubPathsPyramid::get_idx_left_son(ir);
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
