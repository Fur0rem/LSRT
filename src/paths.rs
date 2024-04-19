use std::collections::HashSet;

use crate::attack;
use crate::distance_matrix::{DistanceMatrixArray, SquareMatrix};
use crate::graph::{Graph, Neighbour, Node, Tick};
use crate::time_varying_graph::TimeVaryingGraph;

use std::ops::{Index, IndexMut};

const DEBUG: bool = false;

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
	pub time_taken: Tick,
}

pub struct SubPathsPyramid<'a> {
	pub graph: &'a Graph,
	pub subpaths: Vec<SubPath>,
}

impl<'a> SubPathsPyramid<'a> {
	pub fn print(&self) {
		let mut idx = 0;
		let mut prof = 0;
		for subpath in &self.subpaths {
			print!("{}--{}-->{} ({}) ", subpath.start, subpath.total_weight, subpath.end, subpath.time_taken);
			if idx == prof {
				println!();
				prof += 1;
				idx = 0;
			} else {
				idx += 1;
			}
		}
	}
	pub fn get_prof(idx: usize) -> usize {
		if idx == 0 {
			return 0;
		}
		if idx == 1 {
			return 1;
		}
		//stack exchange get inverse of formula for sum of intergers from 1 to n?
		return (((2 * idx) as f64 + 0.25).sqrt() - 0.5) as usize;
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
		// 1 has parents (None, 0), 2 has parents (0, None), 3 has parents (None, 1), 4 has parents (1, 2), 5 has parents (2, None)
		// Structure :
		// 0
		// 1 2
		// 3 4 5
		let prof = Self::get_prof(idx);
		let (minr, maxr) = Self::get_range(prof);
		if idx == 1 {
			return (None, Some(0));
		}
		if idx == 2 {
			return (Some(0), None);
		}

		let left_parent = if idx == minr { None } else { Some(idx - prof - 1) };
		let right_parent = if idx == maxr { None } else { Some(idx - prof) };
		return (left_parent, right_parent);
	}

	pub fn leaves_range(&self) -> std::ops::Range<usize> {
		// The leaves are the last elements of the pyramid
		if self.subpaths.len() < 1 {
			return 0..0;
		}
		if self.subpaths.len() == 1 {
			return 0..1;
		}
		if self.subpaths.len() == 3 {
			return 1..3;
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
	pub fn get_range(prof: usize) -> (usize, usize) {
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

	pub fn invalidate_path(&self, dma: &mut DistanceMatrixArray, attack: &attack::DynamicAttack) {
		// look through all the leaves, if any of them is in the attack, invalidate the path by writing 0 in the matrix,
		// then propagate the change to the root
		let mut to_invalidate = vec![];
		'outer: for (i, subpath) in self.leaves().iter().enumerate() {
			for blocked_link in &attack.blocked_links {
				if blocked_link.from == subpath.start && blocked_link.to == subpath.end {
					to_invalidate.push(i);
					continue 'outer;
				}
			}
		}

		if to_invalidate.is_empty() {
			return;
		}

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
				let weight = dma[(0, self.subpaths[i].start, self.subpaths[i].end)];
				let max_time_reachable = attack.max_time.saturating_sub(weight);
				for time in 0..max_time_reachable {
					dma[(time, self.subpaths[i].start, self.subpaths[i].end)] = 0;
				}
			}
			to_invalidate = new_to_invalidate;
		}
	}

	pub fn validate_path(&self, dma: &mut DistanceMatrixArray, blocked_links: &SquareMatrix<bool>) {
		if self.subpaths.is_empty() {
			return;
		}
		if dma[(0, self.subpaths[0].start, self.subpaths[0].end)] != 0 {
			return;
		}
		// look through all the leaves, if none of them is in the attack, validate the path by writing the weight in the matrix,
		for subpath in self.leaves() {
			if blocked_links[(subpath.start as usize, subpath.end as usize)] {
				continue;
			}
			for time in 0..dma.max_time {
				dma[(time as Tick, subpath.start, subpath.end)] = subpath.total_weight;
			}
		}
	}

	// FIXME : broken indices for the pyramid
	pub fn unvalidate_path(
		&self, dma: &mut DistanceMatrixArray, attack: &attack::DynamicAttack, already_done: &mut SquareMatrix<bool>,
		times_matrix: &SquareMatrix<Option<&'a Vec<Tick>>>,
	) {
		if self.subpaths.is_empty() {
			return;
		}
		if dma[(0, self.subpaths[0].start, self.subpaths[0].end)] == 0 {
			return;
		}
		if already_done[(self.subpaths[0].start as usize, self.subpaths[0].end as usize)] {
			return;
		}
		if DEBUG {
			println!("1");
			self.print();
		}

		// This is a vec which, for each subpath, contains a vector of intervals at which the path is invalidated
		let mut intervals_to_invalidate = vec![HashSet::new(); self.subpaths.len()];
		// look through all the leaves, write their invalidation intervals in the intervals_to_invalidate vec, then propagate the change to the root
		for i in self.leaves_range() {
			let subpath = &self.subpaths[i];
			//println!("Invalidating path {}--{}-->{}", subpath.start, subpath.total_weight, subpath.end);
			if let Some(times) = times_matrix[(subpath.start as usize, subpath.end as usize)] {
				for time in times {
					for delta in 0..attack.delta {
						let idx = *time as usize * attack.delta as usize + delta as usize;
						if idx >= dma.max_time {
							continue;
						}
						intervals_to_invalidate[i].insert(idx as Tick);
					}
				}
			}
		}

		if DEBUG {
			println!("2");
			self.print();
		}

		// for all the leaves that have been invalidated, propagate the change to their parents
		// go through the pyramid from the leaves to the root

		for i in (0..self.subpaths.len()).rev() {
			let (left_parent, right_parent) = SubPathsPyramid::get_idx_parents(i);
			// if already_done[(self.subpaths[i].start as usize, self.subpaths[i].end as usize)] { continue; }
			if let Some(left_parent) = left_parent {
				let mut to_push = vec![];
				for time in &intervals_to_invalidate[i] {
					to_push.push(time.saturating_sub((self.subpaths[i].time_taken).saturating_sub(self.subpaths[left_parent].time_taken)));
					if DEBUG {
						println!("left parent : {}--{}-->{}", self.subpaths[left_parent].start, self.subpaths[left_parent].total_weight, self.subpaths[left_parent].end);
						println!("invalidating at {} = {} - ({} - {})", 
							time.saturating_sub((self.subpaths[i].time_taken).saturating_sub(self.subpaths[left_parent].time_taken)),
							*time, self.subpaths[i].time_taken, self.subpaths[left_parent].time_taken);
					}
				}
				for time in to_push {
					intervals_to_invalidate[left_parent].insert(time);
				}
			}
			if let Some(right_parent) = right_parent {
				let mut to_push = vec![];
				for time in &intervals_to_invalidate[i] {
					to_push.push(*time);
				}
				for time in to_push {
					intervals_to_invalidate[right_parent].insert(time);
				}
			}
			already_done[(self.subpaths[i].start as usize, self.subpaths[i].end as usize)] = true;
			for time in &intervals_to_invalidate[i] {
				dma[(*time as Tick, self.subpaths[i].start, self.subpaths[i].end)] = 0;
			}
			if false && DEBUG {
				let mut aff=vec![];
				for time in &intervals_to_invalidate[i] {
					aff.push(time);
				}
				println!("Temps de {}--{}-->{}", self.subpaths[i].start, self.subpaths[i].total_weight, self.subpaths[i].end);
				aff.sort();
				for time in &aff {
					println!("{}", *time);
				}
			}
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

	fn get_range(prof: usize) -> (usize, usize) {
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

// ---------------------------------------------------------------------

#[derive(Clone)]
pub struct SubPathCell {
	pub empty: bool,
	pub subpath: SubPath,
	pub left_neighbours: Vec<bool>, // left_parents := {(neighbours, subpath.end) | neighbours in left_neighbours}
	pub right_neighbours: Vec<bool>,
}

impl Default for SubPathCell {
	fn default() -> Self {
		SubPathCell {
			empty: true,
			subpath: SubPath { start: 0, end: 0, total_weight: 0, time_taken: 0 },
			left_neighbours: vec![],
			right_neighbours: vec![],
		}
	}
}

impl SubPathCell {
	fn is_initialized(&self) -> bool {
		return !self.empty;
	}

	fn init(&mut self, subpath: &SubPath, size: usize) {
		self.empty=false;
		self.subpath=subpath.clone();
		self.left_neighbours.resize(size, false);
		self.right_neighbours.resize(size, false);
	}

	fn add(&mut self, subpath: &SubPath, (left_parent, right_parent): (Option<usize>, Option<usize>), size: usize) { // WARNING : not pyramid's idx
		if !self.is_initialized() {
			self.init(subpath, size);
		}
		if let Some(left_parent) = left_parent {
			self.left_neighbours[left_parent] = true;
		}
		if let Some(right_parent) = right_parent {
			self.right_neighbours[right_parent] = true;
		}
	}
}

#[derive(Clone)]
pub struct PathMatrix<'a> {
	pub graph: &'a Graph,
	pub paths: SquareMatrix<SubPathCell>, // Option ?
	pub in_leaves: SquareMatrix<bool>,
	pub leaves: Vec<(usize, usize)>,
}

impl<'a> Index<(Node, Node)> for PathMatrix<'a> {
	type Output = SubPathCell;

	fn index(&self, index: (Node, Node)) -> &Self::Output {
		return &self.paths[(index.0 as usize, index.1 as usize)];
	}
}

impl<'a> IndexMut<(Node, Node)> for PathMatrix<'a> {
	fn index_mut(&mut self, index: (Node, Node)) -> &mut Self::Output {
		return &mut self.paths[(index.0 as usize, index.1 as usize)];
	}
}

impl<'a> PathMatrix<'a> {
	pub fn new<'b>(graph: &'b Graph) -> PathMatrix<'b> {
		return PathMatrix { graph, paths: SquareMatrix::new(graph.nb_nodes().into() , SubPathCell::default()), in_leaves: SquareMatrix::new(graph.nb_nodes().into(), false), leaves: vec![] }
	}

	fn idx_of(subpath: &SubPath) -> (usize, usize) {
		return (subpath.start as usize, subpath.end as usize);
	}

	pub fn add(&mut self, subpath: &SubPath, (left_parent, right_parent): (Option<&SubPath>, Option<&SubPath>), is_leave: bool) {
		/*
		if self[(subpath.start, subpath.end)].time_taken==0 {
			self[(subpath.start, subpath.end)]=subpath.clone(); // use another struct
		}
		*/
		let mut left = None;
		let mut right = None;
		if let Some(left_parent) = left_parent {
			left = Some(PathMatrix::idx_of(left_parent).0);
		}
		if let Some(right_parent) = right_parent {
			right = Some(PathMatrix::idx_of(right_parent).1);
		}
		let size =  self.graph.nb_nodes() as usize;
		self[(subpath.start, subpath.end)].add(subpath, (left, right), size);
		if is_leave && !self.in_leaves[(subpath.start as usize, subpath.end as usize)] {
			self.leaves.push(PathMatrix::idx_of(subpath));
			self.in_leaves[(subpath.start as usize, subpath.end as usize)]=true;
		}
	}

	pub fn unvalidate_path(
		&self, dma: &mut DistanceMatrixArray, attack: &attack::DynamicAttack, already_done: &mut SquareMatrix<bool>,
		times_matrix: &SquareMatrix<Option<&'a Vec<Tick>>>,
	) {
	 //	println!("Hey :)");

		// This is a vec which, for each subpath, contains a vector of intervals at which the path is invalidated
		let mut intervals_to_invalidate = SquareMatrix::new(self.graph.nb_nodes() as usize, HashSet::new());
		// look through all the leaves, write their invalidation intervals in the intervals_to_invalidate vec, then propagate the change to the root
		for i in &self.leaves {
			let subpath = &self.paths[*i].subpath;
			// println!("Invalidating path {}--{}-->{}", subpath.start, subpath.total_weight, subpath.end);
			if let Some(times) = times_matrix[(subpath.start as usize, subpath.end as usize)] {
				for time in times {
					for delta in 0..attack.delta {
						let idx = *time as usize * attack.delta as usize + delta as usize;
						if idx >= dma.max_time {
							continue;
						}
						intervals_to_invalidate[*i].insert(idx as Tick);
					}
				}
			}
		}

		let mut to_invalidate = self.leaves.clone();
		if DEBUG {
			println!("leaves' len : {}", to_invalidate.len());
		}
		let mut level=1;
		// for each levels of the trees
		while !to_invalidate.is_empty() {
			let mut tmp=SquareMatrix::new(self.graph.nb_nodes().into(), false);
			for i in to_invalidate {
				let i=i.clone();
				let subpath = &self.paths[i].subpath;
				if DEBUG {
					if already_done[(subpath.start as usize, subpath.end as usize)] {
						println!("Erreur already_done {}--{}-->{}", subpath.start, subpath.total_weight, subpath.end);
					}
					println!("Invalidating path {}--{}-->{}", subpath.start, subpath.total_weight, subpath.end);
				}

				
				// self.paths[i].empty = false
				for left in 0..self.paths[i].left_neighbours.len() {
					if !self.paths[i].left_neighbours[left] {
						continue;
					}
					let left_parent=&self.paths[(left, subpath.end as usize)];
					
					if DEBUG {
						println!("left parent : {}--{}-->{}", left_parent.subpath.start, left_parent.subpath.total_weight, left_parent.subpath.end);
					}

					let mut to_push = vec![];
					for time in &intervals_to_invalidate[i] {
						to_push.push(time.saturating_sub(left_parent.subpath.total_weight - subpath.total_weight));
						if DEBUG {
							println!("invalidating at {} = {} - ({} - {})", 
								time.saturating_sub(left_parent.subpath.total_weight - subpath.total_weight),
								*time,
								left_parent.subpath.total_weight, subpath.total_weight); // subpath.time_taken,
								// left_parent.subpath.time_taken);
						}
					}
					for time in to_push {
						intervals_to_invalidate[PathMatrix::idx_of(&left_parent.subpath)].insert(time);
					}
					tmp[PathMatrix::idx_of(&left_parent.subpath)]=true;
				}
				for right in 0..self.paths[i].right_neighbours.len() {
					if !self.paths[i].right_neighbours[right] {
						continue;
					}
					let right_parent = &self.paths[(subpath.start as usize, right)];
					if DEBUG {
						println!("right parent : {}--{}-->{}", right_parent.subpath.start, right_parent.subpath.total_weight, right_parent.subpath.end);
					}
					let mut to_push = vec![];
					for time in &intervals_to_invalidate[i] {
						to_push.push(*time);
					}
					for time in to_push {
						intervals_to_invalidate[PathMatrix::idx_of(&right_parent.subpath)].insert(time);
					}
					tmp[PathMatrix::idx_of(&right_parent.subpath)]=true;
				}
				already_done[(subpath.start as usize, subpath.end as usize)] = true;
				// TODO : Suite
				for time in &intervals_to_invalidate[i] {
					dma[(*time as Tick, subpath.start, subpath.end)] = 0;
				}
				if false && DEBUG {
					let mut aff=vec![];
					for time in &intervals_to_invalidate[i] {
						aff.push(time);
					}
					println!("Temps de {}--{}-->{}", subpath.start, subpath.total_weight, subpath.end);
					aff.sort();
					for time in &aff {
						println!("{}", *time);
					}
				}
			}
			to_invalidate=vec![];
			for i in 0..tmp.dim() {
				for j in 0..tmp.dim() {
					if tmp[(i,j)] {
						to_invalidate.push((i,j));
					}
				}
			}
			if DEBUG {
				println!("len at level {level} : {}", to_invalidate.len());
			}
			level+=1;
		}

		/*
		for i in (0..self.subpaths.len()).rev() {
			let (left_parent, right_parent) = SubPathsPyramid::get_idx_parents(i);
			// if already_done[(self.subpaths[i].start as usize, self.subpaths[i].end as usize)] { continue; }
			if let Some(left_parent) = left_parent {
				let mut to_push = vec![];
				for time in &intervals_to_invalidate[i] {
					to_push.push(time.saturating_sub((self.subpaths[i].time_taken).saturating_sub(self.subpaths[left_parent].time_taken)));
				}
				for time in to_push {
					intervals_to_invalidate[left_parent].insert(time);
				}
			}
			if let Some(right_parent) = right_parent {
				let mut to_push = vec![];
				for time in &intervals_to_invalidate[i] {
					to_push.push(*time);
				}
				for time in to_push {
					intervals_to_invalidate[right_parent].insert(time);
				}
			}
			already_done[(self.subpaths[i].start as usize, self.subpaths[i].end as usize)] = true;
			for time in &intervals_to_invalidate[i] {
				dma[(*time as Tick, self.subpaths[i].start, self.subpaths[i].end)] = 0;
			}
		}
		*/
	}
}

// ---------------------------------------------------------------------
