use std::cmp::Ordering;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

use std::collections::BinaryHeap;
use fibonacii_heap::Heap;

type Node = u16;
type Weight = u16;

// Adjacency list representation of a graph

#[derive(Debug, Clone)]
pub struct Neighbour {
    node: Node,
    weight: Weight,
}

#[derive(Debug)]
pub struct Graph {
    nb_nodes: usize,
    edges: Vec<Vec<Neighbour>>,
}

impl Graph {

    // TODO : change the python file to make it a adjacency list to begin with
    pub fn from_sparse_row_format(filename: &str) -> Result<Graph, Box<dyn Error>> {
        let file = File::open(filename)?;
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let mut sizes = line.split_whitespace();
        let size_elem_array: usize = sizes.next().unwrap().parse()?;
        let size_row_array: usize = sizes.next().unwrap().parse()?;
        line.clear();
        reader.read_line(&mut line)?;
        let weights = line.split_whitespace();
        let weights = weights.map(|x| x.parse::<Weight>().unwrap()).collect::<Vec<Weight>>();
        line.clear();
        reader.read_line(&mut line)?;
        let col_index = line.split_whitespace();
        let col_index = col_index.map(|x| x.parse::<Node>().unwrap()).collect::<Vec<Node>>();
        line.clear();
        reader.read_line(&mut line)?;
        let row_index = line.split_whitespace();
        let row_index = row_index.map(|x| x.parse::<usize>().unwrap()).collect::<Vec<usize>>();
        let mut edges = vec![vec![]; size_row_array];
        
        for i in 0..size_row_array {
            let row_start = row_index[i];
            let row_end = if size_row_array - 1 == i {
                size_elem_array
            } else {
                row_index[i + 1]
            };
            for j in row_start..row_end {
                edges[i].push(Neighbour {
                    node: col_index[j],
                    weight: weights[j],
                });
            }
        }

        Ok(Graph {
            nb_nodes: size_row_array,
            edges,
        })
    }
        
    

    pub fn print(&self) {
        for (i, row) in self.edges.iter().enumerate() {
            print!("{}: ", i);
            for neighbour in row {
                print!("{}--{}-->{} ", i, neighbour.weight, neighbour.node);
            }
            println!();
        }
    }

}

pub struct Path<'a> {
    graph: &'a Graph,
    start: Node,
    steps: Vec<&'a Neighbour>,
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

#[derive(Copy, Clone, Eq, PartialEq)]
struct DijkstraState {
    node: Node,
    distance: u16,
}

impl Ord for DijkstraState {
    fn cmp(&self, other: &Self) -> Ordering {
        other.distance.cmp(&self.distance)
    }
}

impl PartialOrd for DijkstraState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/*fn dijkstra(graph: &Graph, start: Node, end: Node) -> Path {
    let mut dist = vec![u16::MAX; graph.nb_nodes as usize];
    let mut prev = vec![u16::MAX; graph.nb_nodes as usize];
    let mut visited = vec![false; graph.nb_nodes as usize];
    dist[start as usize] = 0;
    let mut heap = BinaryHeap::new();
    heap.push(DijkstraState {
        node: start,
        distance: 0,
    });

    while let Some(DijkstraState { node, distance }) = heap.pop() {
        if visited[node as usize] {
            continue;
        }
        visited[node as usize] = true;
        // for edge in &graph.edges {
        //     if edge.from == node {
        //         if dist[node as usize] == u32::MAX {
        //             continue;
        //         }
        //         let alt = dist[node as usize] + edge.weight;
        //         if alt < dist[edge.to as usize] {
        //             dist[edge.to as usize] = alt;
        //             prev[edge.to as usize] = node;
        //             heap.push(DijkstraNode {
        //                 node: edge.to,
        //                 cost: alt,
        //             });
        //         }
        //     }
        // }
        for neighbour in &graph.edges[node as usize] {
            if dist[node as usize] == u16::MAX {
                continue;
            }
            let alt = dist[node as usize] + neighbour.weight;
            if alt < dist[neighbour.node as usize] {
                dist[neighbour.node as usize] = alt;
                prev[neighbour.node as usize] = node;
                heap.push(DijkstraState {
                    node: neighbour.node,
                    distance: alt,
                });
            }
        }
    }
    let mut steps = vec![];
    let mut node = end;
    while node != start {
        let prev_node = prev[node as usize];
        if prev_node == u16::MAX {
            break;
        }
        let weight = dist[node as usize] - dist[prev_node as usize];
        steps.push((prev_node, weight, node));
        node = prev_node;
    }
    steps.reverse();
    Path {
        steps,
        graph,
    }
}*/

pub fn dijkstra(graph: &Graph, start: Node, end: Node) -> Path {
    let mut dist = vec![u16::MAX; graph.nb_nodes as usize];
    let mut prev = vec![u16::MAX; graph.nb_nodes as usize];
    let mut visited = vec![false; graph.nb_nodes as usize];
    dist[start as usize] = 0;
    let mut heap = Heap::new();
    heap.push(DijkstraState {
        node: start,
        distance: 0,
    });

    while let Some(DijkstraState { node, distance }) = heap.pop() {
        if visited[node as usize] {
            continue;
        }
        visited[node as usize] = true;
        for neighbour in &graph.edges[node as usize] {
            if dist[node as usize] == u16::MAX {
                continue;
            }
            let alt = dist[node as usize] + neighbour.weight;
            if alt < dist[neighbour.node as usize] {
                dist[neighbour.node as usize] = alt;
                prev[neighbour.node as usize] = node;
                heap.push(DijkstraState {
                    node: neighbour.node,
                    distance: alt,
                });
            }
        }
    }
    let mut steps = vec![];
    let mut node = end;
    while node != start {
        let prev_node = prev[node as usize];
        if prev_node == u16::MAX {
            break;
        }
        let weight = dist[node as usize] - dist[prev_node as usize];
        steps.push(graph.edges[prev_node as usize].iter().find(|x| x.node == node).unwrap());
        node = prev_node;
    }
    steps.reverse();
    Path {
        steps,
        graph,
        start,
    }
}

#[derive(Debug, Clone)]
pub struct SubPath {
    start: Node,
    end: Node,
    total_weight: Weight,
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

    // FIXME : ca fait de la merde
    /*pub fn from_path<'a>(graph: &'a Graph, path: &Path) -> SubPathsPyramid<'a> {

        let nb_elems = (path.steps.len() * (path.steps.len() + 1)) / 2;
        let mut subpaths = vec![SubPath {
            start: 0,
            end: 0,
            total_weight: 0,
        }; nb_elems];
        let mut leaf_index = ((path.steps.len() - 1) * (path.steps.len())) / 2 - 1;
        let mut prev_node = path.start;
        for step in &path.steps {
            subpaths[leaf_index] = SubPath {
                start: prev_node,
                end: step.node,
                total_weight: step.weight,
            };
            prev_node = step.node;
            leaf_index -= 1;
        }
        println!("{:?}", subpaths);
        for p in (0..SubPathsPyramid::get_prof(leaf_index) - 1).rev() {
            println!("Profondeur {}", p);
            let (minr, maxr) = SubPathsPyramid::get_range(p + 1);
            for i in minr..=maxr {
                let il = SubPathsPyramid::get_idx_left_son(i);
                let ir = SubPathsPyramid::get_idx_right_son(i);
                let shared = SubPathsPyramid::get_idx_left_son(ir);
                let shared_len = if shared < nb_elems {
                    println!("shared : {}", shared);
                    println!("shared_len : {}", subpaths[shared].total_weight);
                    subpaths[shared].total_weight
                } else {
                    0
                };
                println!("i : {}, il : {}, ir : {}, shared : {}, shared_len : {}, nb_elems : {}", i, il, ir, shared, shared_len, nb_elems);
                let left = &subpaths[il];
                let right = &subpaths[ir];
                let new_path = SubPath {
                    start: left.start,
                    end: right.end,
                    total_weight: (left.total_weight + right.total_weight) - shared_len,
                };
                subpaths[i] = new_path;
            }
        }
        SubPathsPyramid {
            graph,
            subpaths,
        }
    }*/

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
        //println!("{:?}", path);
        //let leaf_index = ((path.steps.len() - 1) * (path.steps.len()))/ 2;
        //println!("Leaf start {}", leaf_index);
        // for (i, step) in path.steps.iter().enumerate() {
        //     //println!("elems[{}] = {:?}", leaf_index + i, step);
        //     elems[leaf_index + i] = SubPath {
        //         from : step.0,
        //         to : step.2,
        //         time_start : 0,
        //         total_length : step.1
        //     };
        // }

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

#[derive(Debug)]
pub struct DistanceMatrix {
    pub distances: Vec<Vec<Weight>>,
}

/*
fn newer_johnson(graph: &Graph, max_time: u64) -> (DistanceMatrix, Vec<MergedPathTree>) {
    let mut dst_mat = vec![vec![0; graph.max_node_index as usize]; graph.max_node_index as usize];
    for i in 0..graph.max_node_index {
        for j in 0..graph.max_node_index {
            if i == j {
                dst_mat[i as usize][j as usize] = 1;
            }
        }
    }
    let mut paths = vec![];
    for (node, _) in &graph.nodes {
        let mut nb_dijkstras_done = 0;
        for (node2, _) in graph.nodes.iter() {
            if node == node2 {
                continue;
            }
            if dst_mat[*node as usize][*node2 as usize] != 0 {
                continue;
            }
            nb_dijkstras_done += 1;
            let path = dijkstra(graph, *node, *node2, max_time, &mut dst_mat);
            if path.steps.is_empty() {
                dst_mat[*node as usize][*node2 as usize] = u32::MAX;
                continue;
            }
            /*dst_mat[*node as usize][*node2 as usize] = {
                let sum = path.steps.iter().map(|(_, w, _)| w).sum::<u32>();
                if sum <= max_time as u32 {
                    sum
                } else {
                    u32::MAX
                }
            };*/
            /*if (path.from == 1 && path.to == 30) {
                println!("{:?}", path);
                let subpaths = get_correlated_paths(&path, max_time, &dst_mat);
                println!("{:?}", subpaths);
            }*/
            let subpaths = get_correlated_paths_tree(&path, max_time);
            for subpath in &subpaths.fields {
                let (from, to, time_start, total_length) = (subpath.from, subpath.to, subpath.time_start, subpath.total_length);
                /*if (path.from == 1 && path.to == 30) {
                    println!("SubPath from {} to {} at time {} with total length {}", from, to, time_start, total_length);
                }*/
                if (total_length > max_time as u32) {
                    dst_mat[from as usize][to as usize] = u32::MAX;
                    continue;
                }
                if dst_mat[from as usize][to as usize] == 0 {
                    dst_mat[from as usize][to as usize] = total_length;
                } else {
                    // FIXME : en soit est ce qu'on a besoin de faire min ici? Si c'est le cas, alors on a un bug
                    if total_length != dst_mat[from as usize][to as usize] {
                        println!("SubPath from {} to {} at time {} with total length {}", from, to, time_start, total_length);
                        println!("Old value : {}", dst_mat[from as usize][to as usize]);
                        println!("New value : {}", total_length);
                    }
                    dst_mat[from as usize][to as usize] = std::cmp::min(dst_mat[from as usize][to as usize], total_length);
                }
            }
            paths.push(subpaths);
        }
        // TODO : si path pas trouvé, alors mettre a infini
        println!("Dijkstras done for node {} : {}", node, nb_dijkstras_done);
        /*if (*node < 5) {
            print_matrix(&dst_mat);
        }*/
    }
    return (dst_mat, paths);
        //print_matrix(&dst_mat);
    //}
    //return dst_mat;
} */


impl Graph {

    pub fn all_pairs_shortest_paths(&self, max_time: Weight) -> (DistanceMatrix, Vec<SubPathsPyramid>) {
        let mut dst_mat = vec![vec![0; self.nb_nodes]; self.nb_nodes];
        for i in 0..self.nb_nodes {
            dst_mat[i][i] = 1;
        }

        let mut paths = vec![];
        //for (node, _) in &graph.nodes {
        for node in 0..self.nb_nodes {
            let mut nb_dijkstras_done = 0;
            //for (node2, _) in graph.nodes.iter() {
            for node2 in 0..self.nb_nodes {
                if node == node2 {
                    continue;
                }
                // if dst_mat[*node as usize][*node2 as usize] != 0 {
                //     continue;
                // }
                if dst_mat[node as usize][node2 as usize] != 0 {
                    continue;
                }

                nb_dijkstras_done += 1;
                // let path = dijkstra(graph, *node, *node2, max_time, &mut dst_mat);
                // if path.steps.is_empty() {
                //     dst_mat[*node as usize][*node2 as usize] = u32::MAX;
                //     continue;
                // }
                let path = dijkstra(self, node as Node, node2 as Node);
                if path.steps.is_empty() {
                    dst_mat[node as usize][node2 as usize] = u16::MAX;
                    continue;
                }
                /*else {
                    path.print();
                }*/
                /*dst_mat[*node as usize][*node2 as usize] = {
                    let sum = path.steps.iter().map(|(_, w, _)| w).sum::<u32>();
                    if sum <= max_time as u32 {
                        sum
                    } else {
                        u32::MAX
                    }
                };*/
                /*if (path.from == 1 && path.to == 30) {
                    println!("{:?}", path);
                    let subpaths = get_correlated_paths(&path, max_time, &dst_mat);
                    println!("{:?}", subpaths);
                }*/
                //let subpaths = get_correlated_paths_tree(&path, max_time);

                let subpaths = SubPathsPyramid::from_path(self, &path);
                for subpath in &subpaths.subpaths {
                    /*if subpath.total_weight != 0 {
                        println!("SubPath from {} to {} with total length {}", subpath.start, subpath.end, subpath.total_weight);
                    }*/
                    //let (from, to, time_start, total_length) = (subpath.start, subpath.end, subpath.time_start, subpath.total_length);
                    /*if (path.from == 1 && path.to == 30) {
                        println!("SubPath from {} to {} at time {} with total length {}", from, to, time_start, total_length);
                    }*/
                    // if (total_length > max_time as u32) {
                    //     dst_mat[from as usize][to as usize] = u32::MAX;
                    //     continue;
                    // }
                    if subpath.total_weight > max_time {
                        dst_mat[subpath.start as usize][subpath.end as usize] = u16::MAX;
                        continue;
                    }
                    // if dst_mat[from as usize][to as usize] == 0 {
                    //     dst_mat[from as usize][to as usize] = total_length;
                    // } else {
                    //     // FIXME : en soit est ce qu'on a besoin de faire min ici? Si c'est le cas, alors on a un bug
                    //     if total_length != dst_mat[from as usize][to as usize] {
                    //         println!("SubPath from {} to {} at time {} with total length {}", from, to, time_start, total_length);
                    //         println!("Old value : {}", dst_mat[from as usize][to as usize]);
                    //         println!("New value : {}", total_length);
                    //     }
                    //     dst_mat[from as usize][to as usize] = std::cmp::min(dst_mat[from as usize][to as usize], total_length);
                    // }
                    if dst_mat[subpath.start as usize][subpath.end as usize] == 0 {
                        dst_mat[subpath.start as usize][subpath.end as usize] = subpath.total_weight;
                    } else {
                        // FIXME : en soit est ce qu'on a besoin de faire min ici? Si c'est le cas, alors on a un bug
                        // if subpath.total_weight != dst_mat[subpath.start as usize][subpath.end as usize] {
                        //     println!("SubPath from {} to {} with total length {}", subpath.start, subpath.end, subpath.total_weight);
                        //     println!("Old value : {}", dst_mat[subpath.start as usize][subpath.end as usize]);
                        //     println!("New value : {}", subpath.total_weight);
                        // }
                        dst_mat[subpath.start as usize][subpath.end as usize] = std::cmp::min(dst_mat[subpath.start as usize][subpath.end as usize], subpath.total_weight);
                    }
                }
                paths.push(subpaths);
            }
            // TODO : si path pas trouvé, alors mettre a infini
            println!("Dijkstras done for node {} : {}", node, nb_dijkstras_done);
            /*if (*node < 5) {
                print_matrix(&dst_mat);
            }*/
        }

        return (DistanceMatrix { distances: dst_mat }, paths);
    } 
}