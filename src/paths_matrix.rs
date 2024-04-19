use std::collections::HashSet;
use std::ops::{Index, IndexMut};

use crate::distance_matrix::{DistanceMatrixArray, SquareMatrix};
use crate::graph::{Graph, Neighbour, Node, Tick};
use crate::paths::{SubPath}

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
		// println!("Hey :)");
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
		println!("Hey :)");

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
		/* vec![];
		for i in 0..self.leaves.len() {
			if self.leaves[i] {
				to_invalidate.push(i);
			}
		} */
		while !to_invalidate.is_empty() {
			let mut tmp=SquareMatrix::new(self.graph.nb_nodes().into(), false);
			for i in to_invalidate {
				let i=i.clone();
				let subpath = &self.paths[i].subpath;
				if self.paths[i].empty {
					println!("Invalidating path {}--{}-->{}", subpath.start, subpath.total_weight, subpath.end);
				}
				// self.paths[i].empty = false
				for left in 0..self.paths[i].left_neighbours.len() {
					if !self.paths[i].left_neighbours[left] {
						continue;
					}
					let left_parent=&self.paths[(left, subpath.end as usize)];
					let mut to_push = vec![];
					for time in &intervals_to_invalidate[i] {
						to_push.push(time.saturating_sub((subpath.time_taken).saturating_sub(left_parent.subpath.time_taken)));
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
			}
			to_invalidate=vec![];
			for i in 0..tmp.dim() {
				for j in 0..tmp.dim() {
					if tmp[(i,j)] {
						to_invalidate.push((i,j));
					}
				}
			}
			println!("len : {}", to_invalidate.len());
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