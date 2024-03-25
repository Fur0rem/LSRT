use crate::graph::{Weight, Graph, Neighbour, Time, Node, DistanceMatrix};
use crate::attack::{TimeVaryingEdge, TimeVaryingGraph};

pub struct Path<'a> {
    pub graph: &'a Graph,
    pub start: Node,
    pub steps: Vec<&'a Neighbour>,
}


impl Path<'_> {

    pub fn cost(&self) -> Weight {
        self.steps.iter().map(|x| x.weight).sum()
    }

    pub fn print(&self) {
        //print the start, then each step, then the end
        print!("{} ", self.start);
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

    fn get_prof(idx : usize) -> usize {
        //stack exchange get inverse of formula for sum of intergers from 1 to n?
        return (((2 * idx) as f64 + 0.25).sqrt() - 0.5) as usize + 1;
    }

    fn get_idx_left_son(idx : usize) -> usize {
        return idx + Self::get_prof(idx);
    }

    pub fn get_idx_right_son(idx : usize) -> usize {
        return Self::get_idx_left_son(idx) + 1;
    }

    pub fn get_idx_parents(idx : usize) -> (Option<usize>, Option<usize>) {
        if idx == 0 {
            return (None, None);
        }
        let prof = Self::get_prof(idx);
        let (minr, maxr) = Self::get_range(prof);
        let left_parent = if idx == minr {
            None
        } else {
            Some(idx - prof)
        };
        let right_parent = if idx == maxr {
            None
        } else {
            Some(idx - prof + 1)
        };
        return (left_parent, right_parent);
    }

    fn get_range(prof : usize) -> (usize, usize) {
        if prof == 0 {
            return (0,0)
        }
        let truc = ((prof + 1) * prof) / 2;
        return (truc - prof, truc - 1);
    }

    // TODO : clean this up
    pub fn from_path<'a>(graph: &'a Graph, path: &Path) -> SubPathsPyramid<'a> {
        if path.steps.is_empty() {
            return SubPathsPyramid {
                graph,
                subpaths: vec![],
            }
        }
        let nb_elems = (path.steps.len() * (path.steps.len() + 1)) / 2;
        let mut elems = vec![ SubPath {
            start: 0,
            end: 0,
            total_weight: 0,
        } ; nb_elems];

        let mut prev_node = path.start;
        let mut leaf_index = ((path.steps.len() - 1) * (path.steps.len()))/ 2;
        for step in &path.steps {
            elems[leaf_index] = SubPath {
                start: prev_node,
                end: step.node,
                total_weight: step.weight,
            };
            prev_node = step.node;
            leaf_index += 1;
        }

        let leaf_index = ((path.steps.len() - 1) * (path.steps.len()))/ 2;

        //println!("{:?}", elems);

        for p in (0..SubPathsPyramid::get_prof(leaf_index)-1).rev() {
            let (minr, maxr) = SubPathsPyramid::get_range(p+1);
            for i in minr..=maxr {
                let il = SubPathsPyramid::get_idx_left_son(i);
                let ir = SubPathsPyramid::get_idx_right_son(i);
    
                let shared = SubPathsPyramid::get_idx_left_son(ir);
                let shared_len = if shared < nb_elems {
                    elems[shared].total_weight
                } else {
                    0
                };
                let left = &elems[il];
                let right = &elems[ir];
                //println!("left {} + right {} - shared {} with len {}", left.total_length, right.total_length, shared, shared_len);
                // let new_path = SubPath {
                //     from : left.from,
                //     to : right.to,
                //     time_start : 0,
                //     // TODO : fix that because they get doubled up
                //     total_length : (left.total_length + right.total_length) - shared_len
                // };
                //println!("total_weight : {} + {} - {}",
                    //left.total_weight, right.total_weight, shared_len);
                //println!(" = {}", (left.total_weight + right.total_weight) - shared_len);
                let new_path = SubPath {
                    start: left.start,
                    end: right.end,
                    total_weight: (left.total_weight + right.total_weight) - shared_len,
                };
                //println!("elems[{i}] = {new_path:?} = {il} + {ir}");
                elems[i] = new_path;
            }
        }
        //println!("Done {:?}", elems);
        
        return SubPathsPyramid {
            graph,
            subpaths: elems,
        }
    }
}


fn invalidate_node_and_parents(idx : usize, pyramid : &mut SubPathsPyramid, dst_mat_del: &mut Vec<DistanceMatrix>, max_time: u64) {
    if pyramid.subpaths[idx].total_weight == Weight::MAX {
        return;
    }
    for t in 0..max_time {
        let old_val = dst_mat_del[t as usize][pyramid.subpaths[idx].start][pyramid.subpaths[idx].end as usize];
        if old_val != Weight::MAX {
            //println!("Invalidating path from {} to {} at time {} with total length {}", pyramid.subpaths[idx].start, pyramid.subpaths[idx].to, t, pyramid.subpaths[idx].total_length);
            dst_mat_del[t as usize][pyramid.subpaths[idx].start][pyramid.subpaths[idx].end as usize] = 0;
        }
    }
    pyramid.subpaths[idx].total_weight = Weight::MAX;
    let (left_parent, right_parent) = SubPathsPyramid::get_idx_parents(idx);
    if let Some(left_parent) = left_parent {
        //println!("Invalidating left parent {}", left_parent);
        invalidate_node_and_parents(left_parent, pyramid, dst_mat_del, max_time);
    }
    if let Some(right_parent) = right_parent {
        //println!("Invalidating right parent {}", right_parent);
        invalidate_node_and_parents(right_parent, pyramid, dst_mat_del, max_time);
    }
}
fn invalidate_path_pyramid(pyramids: &mut Vec<SubPathsPyramid>, blocked_links: Vec<Vec<((Node, Node), Vec<Time>)>>, max_time: u64, dst_mat_del: &mut Vec<DistanceMatrix>) {
    //let mut i = 0;
    for pyramid in pyramids {
        //println!("Invalidating path {:?}", path.fields);
        let (leaf_min_index, leaf_max_index) = SubPathsPyramid::get_range(SubPathsPyramid::get_prof(pyramid.subpaths.len()-1));
        //println!("Leaf min index {} and max index {}", leaf_min_index, leaf_max_index);
        for i in leaf_min_index..=leaf_max_index {
            let subpath = &pyramid.subpaths[i];
            /*for t in 0..max_time {
                if deleted_edges.links[subpath.from as usize][subpath.to as usize].contains(&t) {
                    // invalidate the distance matrix
                    //println!("Invalidating path from {} to {} at time {} with total length {}", subpath.from, subpath.to, t, subpath.total_length);
                    invalidate_node_and_parents(i, path, dst_mat_del, max_time);
                    break;
                }
            }*/
            /*for t in &deleted_edges.links[subpath.from as usize][subpath.to as usize] {
                invalidate_node_and_parents(i, path, dst_mat_del, max_time);
                break;
            }*/
            // TODO : why did i store the (Node, Node) again?
            if !(&blocked_links[subpath.start as usize][subpath.end as usize].1.is_empty()) {
                invalidate_node_and_parents(i, pyramid, dst_mat_del, max_time);
            }
        }
        //i += 1;
        //println!("Invalidated path {}", i);
    }
}

/*pub struct TemporalPath<'a> {
    pub graph: &'a TimeVaryingGraph,
    pub start: Node,
    pub steps: Vec<&'a (Neighbour, TimeVaryingEdge)>,
    pub start_time: Time,
}*/

pub struct TemporalPath<'a> {
    pub graph: &'a  TimeVaryingGraph,
    pub start: Node,
    pub steps: Vec<&'a (Neighbour, TimeVaryingEdge)>,
    pub start_time: Time,
}


impl<'a> TemporalPath<'a> {

    pub fn cost(&self) -> Weight {
        self.steps.iter().map(|x| x.0.weight).sum()
    }

    pub fn print(&self) {
        //print the start, then each step, then the end
        print!("{} ", self.start);
        for step in &self.steps {
            print!("--{}-->{} ", step.0.weight, step.0.node);
        }
        println!();
    }

    pub fn empty(graph: &'a TimeVaryingGraph) -> Self {
        TemporalPath {
            graph,
            start: 0,
            steps: vec![],
            start_time: 0,
        }
    }

}


// TODO : mix it with the other subpaths pyramid
#[derive(Debug, Clone)]
pub struct TemporalSubPath {
    pub start: Node,
    pub end: Node,
    pub total_weight: Weight,
}

pub struct TemporalSubPathsPyramid<'a> {
    pub graph: &'a TimeVaryingGraph,
    pub subpaths: Vec<SubPath>,
}

impl TemporalSubPathsPyramid<'_> {

    fn get_prof(idx : usize) -> usize {
        //stack exchange get inverse of formula for sum of intergers from 1 to n?
        return (((2 * idx) as f64 + 0.25).sqrt() - 0.5) as usize + 1;
    }

    fn get_idx_left_son(idx : usize) -> usize {
        return idx + Self::get_prof(idx);
    }

    pub fn get_idx_right_son(idx : usize) -> usize {
        return Self::get_idx_left_son(idx) + 1;
    }

    pub fn get_idx_parents(idx : usize) -> (Option<usize>, Option<usize>) {
        if idx == 0 {
            return (None, None);
        }
        let prof = Self::get_prof(idx);
        let (minr, maxr) = Self::get_range(prof);
        let left_parent = if idx == minr {
            None
        } else {
            Some(idx - prof)
        };
        let right_parent = if idx == maxr {
            None
        } else {
            Some(idx - prof + 1)
        };
        return (left_parent, right_parent);
    }

    fn get_range(prof : usize) -> (usize, usize) {
        if prof == 0 {
            return (0,0)
        }
        let truc = ((prof + 1) * prof) / 2;
        return (truc - prof, truc - 1);
    }

    // TODO : clean this up
    pub fn from_path<'a>(graph: &'a TimeVaryingGraph, path: &TemporalPath) -> TemporalSubPathsPyramid<'a> {
        if path.steps.is_empty() {
            return TemporalSubPathsPyramid {
                graph,
                subpaths: vec![],
            }
        }
        let nb_elems = (path.steps.len() * (path.steps.len() + 1)) / 2;
        let mut elems = vec![ SubPath {
            start: 0,
            end: 0,
            total_weight: 0,
        } ; nb_elems];

        let mut prev_node = path.start;
        let mut leaf_index = ((path.steps.len() - 1) * (path.steps.len()))/ 2;
        for step in &path.steps {
            elems[leaf_index] = SubPath {
                start: prev_node,
                end: step.0.node,
                total_weight: step.0.weight,
            };
            prev_node = step.0.node;
            leaf_index += 1;
        }

        let leaf_index = ((path.steps.len() - 1) * (path.steps.len()))/ 2;

        //println!("{:?}", elems);

        for p in (0..SubPathsPyramid::get_prof(leaf_index)-1).rev() {
            let (minr, maxr) = SubPathsPyramid::get_range(p+1);
            for i in minr..=maxr {
                let il = SubPathsPyramid::get_idx_left_son(i);
                let ir = SubPathsPyramid::get_idx_right_son(i);
    
                let shared = SubPathsPyramid::get_idx_left_son(ir);
                let shared_len = if shared < nb_elems {
                    elems[shared].total_weight
                } else {
                    0
                };
                let left = &elems[il];
                let right = &elems[ir];
                //println!("left {} + right {} - shared {} with len {}", left.total_length, right.total_length, shared, shared_len);
                // let new_path = SubPath {
                //     from : left.from,
                //     to : right.to,
                //     time_start : 0,
                //     // TODO : fix that because they get doubled up
                //     total_length : (left.total_length + right.total_length) - shared_len
                // };
                //println!("total_weight : {} + {} - {}",
                    //left.total_weight, right.total_weight, shared_len);
                //println!(" = {}", (left.total_weight + right.total_weight) - shared_len);
                let new_path = SubPath {
                    start: left.start,
                    end: right.end,
                    total_weight: (left.total_weight + right.total_weight) - shared_len,
                };
                //println!("elems[{i}] = {new_path:?} = {il} + {ir}");
                elems[i] = new_path;
            }
        }
        //println!("Done {:?}", elems);
        
        return TemporalSubPathsPyramid {
            graph,
            subpaths: elems,
        }
    }
}


fn invalidate_temporal_node_and_parents(idx : usize, pyramid : &mut TemporalSubPathsPyramid, dst_mat_del: &mut Vec<DistanceMatrix>, max_time: Time) {
    if pyramid.subpaths[idx].total_weight == Weight::MAX {
        return;
    }
    for t in 0..max_time {
        let old_val = dst_mat_del[t as usize][pyramid.subpaths[idx].start][pyramid.subpaths[idx].end as usize];
        if old_val != Weight::MAX {
            //println!("Invalidating path from {} to {} at time {} with total length {}", pyramid.subpaths[idx].start, pyramid.subpaths[idx].to, t, pyramid.subpaths[idx].total_length);
            dst_mat_del[t as usize][pyramid.subpaths[idx].start][pyramid.subpaths[idx].end as usize] = 0;
        }
    }
    pyramid.subpaths[idx].total_weight = Weight::MAX;
    let (left_parent, right_parent) = SubPathsPyramid::get_idx_parents(idx);
    if let Some(left_parent) = left_parent {
        //println!("Invalidating left parent {}", left_parent);
        invalidate_temporal_node_and_parents(left_parent, pyramid, dst_mat_del, max_time);
    }
    if let Some(right_parent) = right_parent {
        //println!("Invalidating right parent {}", right_parent);
        invalidate_temporal_node_and_parents(right_parent, pyramid, dst_mat_del, max_time);
    }
}

pub fn invalidate_temporal_path_pyramid(pyramids: &mut Vec<TemporalSubPathsPyramid>, blocked_links: Vec<Vec<((Node, Node), Vec<Time>)>>, max_time: Time, dst_mat_del: &mut Vec<DistanceMatrix>) {
    //let mut i = 0;
    for pyramid in pyramids {
        //println!("Invalidating path {:?}", path.fields);
        let (leaf_min_index, leaf_max_index) = SubPathsPyramid::get_range(SubPathsPyramid::get_prof(pyramid.subpaths.len()-1));
        //println!("Leaf min index {} and max index {}", leaf_min_index, leaf_max_index);
        for i in leaf_min_index..=leaf_max_index {
            let subpath = &pyramid.subpaths[i];
            /*for t in 0..max_time {
                if deleted_edges.links[subpath.from as usize][subpath.to as usize].contains(&t) {
                    // invalidate the distance matrix
                    //println!("Invalidating path from {} to {} at time {} with total length {}", subpath.from, subpath.to, t, subpath.total_length);
                    invalidate_node_and_parents(i, path, dst_mat_del, max_time);
                    break;
                }
            }*/
            /*for t in &deleted_edges.links[subpath.from as usize][subpath.to as usize] {
                invalidate_node_and_parents(i, path, dst_mat_del, max_time);
                break;
            }*/
            // TODO : why did i store the (Node, Node) again?
            if !(&blocked_links[subpath.start as usize][subpath.end as usize].1.is_empty()) {
                invalidate_temporal_node_and_parents(i, pyramid, dst_mat_del, max_time);
            }
        }
        //i += 1;
        //println!("Invalidated path {}", i);
    }
}