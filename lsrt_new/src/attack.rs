use core::time;
use std::{fs::File, io::{BufRead, BufReader}};

use crate::{graph::{DistanceMatrix, Graph, Neighbour, Node, Time, Weight}, paths::{invalidate_path_pyramid, invalidate_temporal_path_pyramid, SubPathsPyramid, TemporalPath, TemporalSubPathsPyramid}};
use crate::paths::Path;

use std::collections::BinaryHeap;

macro_rules! benchmark {
    ($name:expr, $block:expr) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let duration = start.elapsed();
        println!("Time elapsed in {}() is: {:?}", $name, duration);
        result
    }};
}

use crate::graph::{DijkstraState};

use sscanf::scanf;

pub trait Attack : std::fmt::Debug {
    // efficiency + reachables
    fn efficiency(&self, graph: &Graph) -> (f64, usize);
    fn max_time(&self) -> Time;
    fn blocked_links(&self) -> &Vec<DeletedLink>;
    fn delta(&self) -> Time;
    /*
    err_code sum_dma(double * ret_sum, uint64_t * ret_reachables,  DISTANCE_MATRIX_ARRAY * dma){
    def_err_handler(!(ret_sum && dma && ret_reachables), "sum_dma", ERR_NULL);

    double sum = 0; 
    uint64_t reachables = 0 ; 
    for(uint32_t i = 0 ; i < dma->nb_matrixes ; i++){
        for(uint32_t j = 0 ; j < dma->matrixes[i].cols * dma->matrixes[i].rows ; j++){
            
            if( (dma->matrixes[i].values[j] != UINT8_MAX) && (dma->matrixes[i].values[j]) && i!=j ){
                sum += (double)((double)1/(double)(dma->matrixes[i].values[j])) ;
                reachables++;
            }
        }
    }
    sum -= dma->matrixes[0].rows ;
    sum /=  dma->nb_matrixes *(((dma->matrixes[0].rows) * (dma->matrixes[0].rows - 1) ));
    //sum normalized by time * (|N| * |N-1|) with G=(V,N)

    reachables -= dma->matrixes[0].rows ;
    //reachables /= (((dma->matrixes[0].rows) * (dma->matrixes[0].rows - 1) ));
   
    *ret_sum = sum ; 
    *ret_reachables = reachables ;
    return ERR_OK ;
    }//not tested ; should be okay though 
    */
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
    pub delta: Time,
    pub max_time: Time,
}

#[derive(Debug)]
pub struct StaticAttack {
    blocked_links: Vec<Vec<Node>>,
    pub delta: Time,
    pub max_time: Time,
}

impl dyn Attack {
    pub fn from_file(file: &str, graph: &Graph) -> Box<dyn Attack> {
        let file = File::open(file).unwrap();
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();

        if line.trim() == "S" {
            return Box::new(StaticAttack::from_file(&mut reader, graph));
        }
        else if line.trim() == "D" {
            return Box::new(DynamicAttack::from_file(&mut reader, graph));
        }

        panic!("Invalid attack file");
    }
}

impl StaticAttack {
    pub fn from_file(reader: &mut BufReader<File>, graph: &Graph) -> Self {
        // Format : first line contains number of blocked links then delta (useless)
        // Then, the list of blocked links separated by a space
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        println!("Line : {}", line);
        //let (nb_blocks, _delta) = scanf!(line, "{} {} ", usize, usize).unwrap();
        let iter = line.split_whitespace();
        let mut iter = iter.map(|x| x.parse().unwrap());
        let nb_blocks = iter.next().unwrap();
        let delta = iter.next().unwrap();
        let max_time = iter.next().unwrap();

        // while there is new line, read it and append to the result
        let mut blocked_links = Vec::with_capacity(graph.nb_nodes);
        for _ in 0..graph.nb_nodes {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let blocks = line.split_whitespace().map(|x| x.parse().unwrap()).collect();
            blocked_links.push(blocks);
        }

        return Self {
            blocked_links,
            delta,
            max_time,
        };
    }
}

impl Attack for StaticAttack {

    fn max_time(&self) -> Time {
        return self.max_time;
    }
    
    fn delta(&self) -> Time {
        return self.delta;
    }

    fn blocked_links(&self) -> &Vec<DeletedLink> {
        unimplemented!();
    }

    // Create a graph without the links that got blocked
    fn efficiency(&self, graph: &Graph) -> (f64, usize) {
        let mut new_graph = graph.clone();
        for i in 0..self.blocked_links.len() {
            for j in 0..self.blocked_links[i].len() {
                new_graph.remove_link(i as Node, self.blocked_links[i][j]);
            }
        }

        let (distance_matrix, _shortest_paths) = new_graph.all_pairs_shortest_paths(self.max_time);
        println!("{:?}", distance_matrix);
        let mut sum = 0.0;
        let mut reachables = 0;
        for t in 0..self.max_time {
            for i in 0..distance_matrix.distances.len() {
                for j in 0..distance_matrix.distances[i].len() {

                    let value = distance_matrix.distances[i][j];
                    if value > self.max_time {
                        continue;
                    }
                    let value = value + t;
                    if value > self.max_time {
                        continue;
                    }
                    sum += 1.0 / (value as f64);
                    reachables += 1;
                }
            }
        }

        let efficiency = sum / ((self.max_time as f64) * (graph.nb_nodes * (graph.nb_nodes - 1)) as f64);
        return (efficiency, reachables);
    }
}


// TODO : ew it's ugly af
impl DynamicAttack {
    pub fn from_file(reader: &mut BufReader<File>, graph: &Graph) -> Self {
        // First line : nb_edges, delta, max_time
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        //let (nb_edges, delta, max_time) = scanf!(line, "{} {} {}", usize, u16, u16).unwrap();
        let iter = line.split_whitespace();
        let mut iter = iter.map(|x| x.parse().unwrap());
        let nb_edges = iter.next().unwrap();
        let delta = iter.next().unwrap();
        let max_time = iter.next().unwrap();

        // Then for each line, nb_blocked (node1,node2) <the times at which it is blocked>
        // For example : 4 (0,2) 5 9 17 18
        let mut blocked_links = vec![];
        for _ in 0..nb_edges {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            //println!("Line : {}", line);
            //let (nb_blocked, from, to) = scanf!(line, "{} ({}, {}) ", usize, usize, usize).unwrap();
            let mut iter = line.split_whitespace();
            let nb_blocked = iter.next().unwrap();
            let from = iter.next().unwrap();
            let to = iter.next().unwrap();

            let nb_blocked = nb_blocked.parse().unwrap();
            // remove the paranthesis and comma
            let from : usize = from[1..from.len()-1].parse().unwrap();
            let to : usize = to[0..to.len()-1].parse().unwrap();


            let mut times = vec![];
            for _ in 0..nb_blocked {
                let time = iter.next().unwrap().parse().unwrap();
                times.push(time);
            }

            blocked_links.push(DeletedLink {
                from: from as Node,
                to: to as Node,
                times,
            });
        }

        return Self {
            blocked_links,
            delta: delta as Time,
            max_time: max_time as Time,
        };
    }
}

// TODO : put the tvg into their own files
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeVaryingEdge {
    weights: Vec<Weight>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeVaryingGraph {
    max_time: u64,
    nodes: Vec<Vec<(Neighbour, TimeVaryingEdge)>>,
    dst_mat_undel: DistanceMatrix,
    dst_mat_del: Vec<DistanceMatrix>,
}

// TODO : optimize to write every min distance computed
fn temporal_dijkstra<'a>(graph: &'a TimeVaryingGraph, start: Node, end: Node, max_time: u64, time: u64, max_node_index: u64, distance_matrix: &DistanceMatrix) -> TemporalPath<'a> {
    let mut dist = vec![Weight::MAX; max_node_index as usize];
    let mut prev = vec![Weight::MAX; max_node_index as usize];
    let mut visited = vec![false; max_node_index as usize];
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
        for (neighbour, edge) in &graph.nodes[node as usize] {
            if (distance as u64 + time) >= max_time {
                continue;
            }
            let weight = edge.weights[distance as usize + time as usize];
            if weight == Weight::MAX {
                continue;
            }
            let new_distance = distance + weight;
            if new_distance < dist[neighbour.node as usize] {
                dist[neighbour.node as usize] = new_distance;
                prev[neighbour.node as usize] = node;
                heap.push(DijkstraState {
                    node: neighbour.node,
                    distance: new_distance,
                });
            }
        }
    }
    /*let mut steps = vec![];
    let mut node = end;
    while node != start {
        let prev_node = prev[node as usize];
        if prev_node == Weight::MAX {
            break;
        }
        let weight = dist[node as usize] - dist[prev_node as usize];
        steps.push((prev_node, weight, node));
        node = prev_node;
    }
    steps.reverse();
    return TemporalPath {
        graph: graph,
        steps,
        start,
    };*/

    let mut steps = vec![];
    let mut node = end;
    while node != start {
        let prev_node = prev[node as usize];
        if prev_node == u16::MAX {
            break;
        }
        let weight = dist[node as usize] - dist[prev_node as usize];
        steps.push(graph.nodes[prev_node as usize].iter().find(|x| x.0.node == node).unwrap());
        node = prev_node;
    }
    steps.reverse();
    TemporalPath {
        steps,
        graph,
        start,
        start_time: time as Time,
    }
}

fn recompute_all_distance_matrix(graph: &mut TimeVaryingGraph) {
    let max_node_index = graph.nodes.len();
    let mut nb_dijkstras = 0;
    for t in 0..graph.max_time {
        for i in 0..graph.dst_mat_undel.distances.len() {
            for j in 0..graph.dst_mat_undel[i as Node].len() {
                if graph.dst_mat_del[t as usize][i as Node][j] == 0 {
                    //println!("Recomputing distance from {} to {} at time {}", i, j, t);
                    // TODO : take adventage of the fact that we already have the shortest path from a bunch of other nodes
                    // FIXME : what the fuck is happening
                    let path = temporal_dijkstra(graph, i as Node, j as Node, graph.max_time, t, max_node_index.try_into().unwrap(), &graph.dst_mat_del[t as usize]);
                    nb_dijkstras += 1;
                    let subpaths = TemporalSubPathsPyramid::from_path(graph, &path);
                    for subpath in &subpaths.subpaths {
                        let (from, to, time_start, total_length) = (subpath.start, subpath.end, t, subpath.total_weight);
                        let new_time = t + time_start;
                        if new_time < graph.max_time {
                            graph.dst_mat_del[new_time as usize][from][to as usize] = total_length;
                        }
                    }
                }
            }
        }
        //println!("Time {} nb dijkstras : {}", t, nb_dijkstras);
    }
}

fn sum_dma(dma : &Vec<DistanceMatrix>, max_time: Time) -> (f64, u64) {
    let mut sum = 0.0;
    //let mut sum_int = 0;
    let mut reachables = 0;
    let distances_len = dma[0 as usize].distances.len();
    for t in 0..dma.len() {
        for i in 0..distances_len {
            for j in 0..distances_len {
                if i == j {
                    continue;
                }
                let distance = dma[t as usize][i as Node][j as usize];
                if distance < max_time && distance != 0 {
                    //sum_int += distance as u64;
                    sum += 1.0 / (distance as f64);
                    reachables += 1;
                }
            }
        }
    }
    //println!("Sum int : {}", sum_int);
    sum -= distances_len as f64;
    sum /= dma.len() as f64 * (distances_len * (distances_len - 1)) as f64;
    return (sum, reachables);
}

struct DeletedLinksMatrix {
    links: Vec<Vec<Vec<u64>>>
}

fn create_phantom_edges(graph: &Graph, max_time: Time, deleted_edges: &Vec<DeletedLink>, dst_mat_del: &mut Vec<DistanceMatrix>, delta : u16) -> Vec<Vec<(Neighbour, TimeVaryingEdge)>> {
    let mut annex_edges = vec![];
    /*for edge in &graph.edges {
        let mut weights = vec![];
        for t in 0..max_time {
            // TODO : rewrite that to be more efficient and less ugly
            let mut wait = 0;
            for deleted_edge in deleted_edges {
                if deleted_edge.from == edge.from && deleted_edge.to == edge.to {
                    while deleted_edge.times.contains(&(t + wait)) {
                        wait += 1;
                    }
                }
            }
            let weight_t = get_weight2(edge, t) + wait as u32;
            // Remove weights that are too high
            weights.push(if weight_t + t as u32 <= max_time as u32 { weight_t } else { u32::MAX });
        }
        annex_edges.push(TimeVaryingEdge {
            from: edge.from,
            to: edge.to,
            weight: weights,
        });
    }*/
    for (node_idx, node) in graph.edges.iter().enumerate() {
        let mut edges = vec![];
        for neighbour in node {
            let weight = &neighbour.weight;
            let mut weights = vec![];
            for t in 0..max_time {
                let mut wait = 0;
                for deleted_edge in deleted_edges {
                    if deleted_edge.from == node_idx as Node && deleted_edge.to == neighbour.node {
                        // FIXME : i think delta is here ???
                        while deleted_edge.times.contains(&((t + wait) / delta)) { // maybe we shouldnt divide by delta here
                            wait += 1; // maybe use delta here instead ?
                        }
                    }
                }
                let weight_t = weight + wait;
                // Remove weights that are too high
                weights.push(if weight_t + t as Time <= max_time { weight_t } else { Weight::MAX });
            }
            edges.push((neighbour.clone(), TimeVaryingEdge {
                weights,
            }));
        }
        annex_edges.push(edges);
    }
    
    return annex_edges;
}

fn graph_to_temporal(graph: &Graph, max_time: Time, deleted_edges: &Vec<DeletedLink>, delta : u16) -> TimeVaryingGraph {
    let mut edges : Vec<TimeVaryingEdge> = vec![];
    let (mut dst_mat_undel, mut paths) = benchmark!("apsp", graph.all_pairs_shortest_paths(max_time));
    //let (paths,_) = benchmark!("johnson", johnson(graph, max_time));
    //let mut dst_mat_undel = benchmark!("from_corr_paths", from_shortest_corr_paths(&paths, &graph.nodes, graph.max_node_index, max_time, deleted_edges));
    //print_matrix(&dst_mat_undel);
    // TODO : change that to infinity once debuggin is done
    let mut dst_mat_del = vec![dst_mat_undel.clone(); max_time as usize];

    // TODO : yeah this is very space consuming, probably better to switch to a sparse matrix or hashmap
    let deleted_edges_matrix = benchmark!("deleted_edges_matrix", {
        let max_node_index = graph.nb_nodes as u64;
        let mut deleted_edges_matrix = vec![vec![vec![]; max_node_index as usize]; max_node_index as usize];
        for deleted_link in deleted_edges {
            for t in &deleted_link.times {
                deleted_edges_matrix[deleted_link.from as usize][deleted_link.to as usize].push(*t);
            }
        }
        deleted_edges_matrix
    });

    let annex_edges = benchmark!("create_phantom_edges", create_phantom_edges(&graph, max_time, &deleted_edges, &mut dst_mat_del, delta));
    benchmark!("invalidate_deleted_edges", invalidate_path_pyramid(&mut paths, deleted_edges_matrix, max_time, &mut dst_mat_del));
    TimeVaryingGraph {
        max_time : max_time.into(),
        nodes: annex_edges,
        dst_mat_undel,
        dst_mat_del,
    }
}

fn print_dma(dma: &Vec<DistanceMatrix>) {
    for t in 0..dma.len() {
        println!("Time : {}", t);
        for i in 0..dma[t as usize].distances.len() {
            for j in 0..dma[t as usize].distances[i as usize].len() {
                let distance_to_str = if dma[t as usize][i as Node][j as usize] == Weight::MAX {
                    "∞".to_string()
                } else {
                    dma[t as usize][i as Node][j as usize].to_string()
                };
                print!("{} ", distance_to_str);
            }
            println!();
        }
    }
}

impl Attack for DynamicAttack {

    fn max_time(&self) -> Time {
        return self.max_time;
    }

    fn delta(&self) -> Time {
        return self.delta;
    }

    fn blocked_links(&self) -> &Vec<DeletedLink> {
        return &self.blocked_links;
    }

    fn efficiency(&self, graph: &Graph) -> (f64, usize) {
        let mut time_graph= graph_to_temporal(&graph, self.max_time, &self.blocked_links, self.delta);
        recompute_all_distance_matrix(&mut time_graph);
        cap_times(&mut time_graph.dst_mat_del, self.max_time);
        let (a,b) = sum_dma(&time_graph.dst_mat_del, self.max_time);
        print_dma(&time_graph.dst_mat_del);
        return (a,b as usize);
    }
}

pub fn cap_times(dma: &mut Vec<DistanceMatrix>, max_time: Time) {
    for t in 0..dma.len() {
        for i in 0..dma[t as usize].distances.len() {
            for j in 0..dma[t as usize].distances[i as usize].len() {
                if i == j {
                    dma[t as usize][i as Node][j as usize] = 1;
                }
                let weight = dma[t as usize][i as Node][j as usize];
                if weight == 0 || weight == Weight::MAX {
                    dma[t as usize][i as Node][j as usize] = Weight::MAX;
                    continue;
                }
                let weight = dma[t as usize][i as Node][j as usize];
                let weight = weight as usize + t as usize;
                if weight >= max_time as usize {
                    dma[t as usize][i as Node][j as usize] = Weight::MAX;
                }
                else {
                    dma[t as usize][i as Node][j as usize] = weight as Weight - t as Weight;
                }
            }
        }
    }
}

// FIXME : it looks like it doesnt take into account phantom edges or smth????, probably something wrong with dijkstra
pub fn dumb_efficiency(graph: &Graph, attack: &dyn Attack) -> (f64, usize, Vec<DistanceMatrix>) {
    // create phantom edges
    let mut time_graph = graph_to_temporal(&graph, attack.max_time(), &attack.blocked_links(), attack.delta());
    // recompute all distance matrices from scratch
    for t in 0..time_graph.max_time {
        for i in 0..time_graph.dst_mat_undel.distances.len() {
            for j in 0..time_graph.dst_mat_undel.distances[i as usize].len() {
                // write infinity in the distance matrix
                time_graph.dst_mat_del[t as usize][i as Node][j as usize] = Weight::MAX;
                let path = temporal_dijkstra(&time_graph, i as Node, j as Node, time_graph.max_time, t, time_graph.nodes.len().try_into().unwrap(), &time_graph.dst_mat_del[t as usize]);
                if path.steps.len() == 0 {
                    continue;
                }
                if i == 8 && j == 16 {
                    print!("Path from 8 to 16 at time {} : ", t);
                    path.print();
                }
                // write the length of the path in the distance matrix
                let mut total_length = 0;
                for subpath in &path.steps {
                    if subpath.1.weights[t as usize] == Weight::MAX {
                        total_length = Weight::MAX;
                        break;
                    }
                    if (subpath.1.weights[t as usize] as usize + t as usize) >= time_graph.max_time as usize {
                        total_length = Weight::MAX;
                        break;
                    }
                    let weight = subpath.0.weight;
                    total_length += weight;
                }
                if i == 8 && j == 16 {
                    println!("Total length : {}", total_length);
                }
                /*if total_length >= time_graph.max_time as Weight {
                    total_length = Weight::MAX;
                }
                if total_length + t as Weight >= time_graph.max_time as Weight {
                    total_length = Weight::MAX;
                }
                if total_length == 0 {
                    total_length = Weight::MAX;
                }*/
                time_graph.dst_mat_del[t as usize][i as Node][j as usize] = total_length;
            }
        }
    }
    cap_times(&mut time_graph.dst_mat_del, time_graph.max_time.try_into().unwrap());
    print_dma(&time_graph.dst_mat_del);
    let (a,b) = sum_dma(&time_graph.dst_mat_del, time_graph.max_time.try_into().unwrap());
    return (a,b as usize, time_graph.dst_mat_del);
}

pub fn improved_efficiency(graph: &Graph, attack: &dyn Attack) -> (f64, usize, Vec<DistanceMatrix>) {
    // create phantom edges
    let mut time_graph = graph_to_temporal(&graph, attack.max_time(), &attack.blocked_links(), attack.delta());
    // print the phantom edges
    for (node_idx, node) in time_graph.nodes.iter().enumerate() {
        for neighbour in node {
            println!("{} -> {} : {:?}", node_idx, neighbour.0.node, neighbour.1.weights);
        }
    }
    // recompute all distance matrices from scratch
    for t in 0..time_graph.max_time {
        for i in 0..time_graph.dst_mat_undel.distances.len() {
            for j in 0..time_graph.dst_mat_undel.distances[i as usize].len() {
                // write infinity in the distance matrix
                /*time_graph.dst_mat_del[t as usize][i as Node][j as usize] = Weight::MAX;
                let path = temporal_dijkstra(&time_graph, i as Node, j as Node, time_graph.max_time, t, time_graph.nodes.len().try_into().unwrap(), &time_graph.dst_mat_del[t as usize]);
                // write the length of the path in the distance matrix
                let mut total_length = 0;
                for subpath in &path.steps {
                    total_length += subpath.1.weights[t as usize];
                }*/
                /*if total_length >= time_graph.max_time as Weight {
                    total_length = Weight::MAX;
                }
                if total_length + t as Weight >= time_graph.max_time as Weight {
                    total_length = Weight::MAX;
                }
                if total_length == 0 {
                    total_length = Weight::MAX;
                }*/
                //time_graph.dst_mat_del[t as usize][i as Node][j as usize] = total_length;
                /*
                let path = temporal_dijkstra(graph, i as Node, j as Node, graph.max_time, t, max_node_index.try_into().unwrap(), &graph.dst_mat_del[t as usize]);
                    nb_dijkstras += 1;
                    let subpaths = TemporalSubPathsPyramid::from_path(graph, &path);
                    for subpath in &subpaths.subpaths {
                        let (from, to, time_start, total_length) = (subpath.start, subpath.end, t, subpath.total_weight);
                        let new_time = t + time_start;
                        if new_time < graph.max_time {
                            graph.dst_mat_del[new_time as usize][from][to as usize] = total_length;
                        }
                } */

                let path = temporal_dijkstra(&time_graph, i as Node, j as Node, time_graph.max_time, t, time_graph.nodes.len().try_into().unwrap(), &time_graph.dst_mat_del[t as usize]);
                if path.steps.len() == 0 {
                    continue;
                }
                if i == 8 && j == 16 {
                    print!("Path from 8 to 16 at time {} : ", t);
                    path.print();
                }
                
                let subpaths = TemporalSubPathsPyramid::from_path(&time_graph, &path);
                for subpath in &subpaths.subpaths {
                    let (from, to, time_start, total_length) = (subpath.start, subpath.end, t /*+ (subpath.total_weight as u64)*/, subpath.total_weight);
                    if from == 8 && to == 16 {
                        println!("{:?}", subpath);
                        println!("Total length : {}", total_length);
                        println!("From : {}, to : {}, time_start : {}, total_length : {}", from, to, time_start, total_length);
                    }
                    let old_distance = time_graph.dst_mat_del[t as usize][from][to as usize];
                    //if total_length < old_distance || old_distance == 0 {
                    if ((time_start as usize) < (time_graph.max_time as usize)) && (total_length < old_distance || old_distance == 0) {
                        time_graph.dst_mat_del[time_start as usize][from][to as usize] = total_length;
                    }
                }
            }
        }
    }
    cap_times(&mut time_graph.dst_mat_del, time_graph.max_time.try_into().unwrap());
    print_dma(&time_graph.dst_mat_del);
    let (a,b) = sum_dma(&time_graph.dst_mat_del, time_graph.max_time.try_into().unwrap());
    return (a,b as usize, time_graph.dst_mat_del);
}