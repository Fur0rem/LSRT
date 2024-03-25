use std::{fs::File, io::{BufRead, BufReader}};

use crate::{graph::{DistanceMatrix, Graph, Neighbour, Node, Time, Weight}, paths::{invalidate_temporal_path_pyramid, SubPathsPyramid, TemporalPath, TemporalSubPathsPyramid}};
use crate::paths::Path;

use std::collections::BinaryHeap;

#[macro_use]
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
    fn efficiency(&self, graph: &Graph, max_time: Time) -> (f64, usize);
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
struct DeletedLink {
    from: Node,
    to: Node,
    times: Vec<Time>,
}

#[derive(Debug)]
pub struct DynamicAttack {
    pub blocked_links: Vec<DeletedLink>,
    pub delta: Time
}

#[derive(Debug)]
pub struct StaticAttack {
    blocked_links: Vec<Vec<Node>>,
    delta: Time,
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
            delta
        };
    }
}

impl Attack for StaticAttack {
    // Create a graph without the links that got blocked
    fn efficiency(&self, graph: &Graph, max_time: Time) -> (f64, usize) {
        let mut new_graph = graph.clone();
        for i in 0..self.blocked_links.len() {
            for j in 0..self.blocked_links[i].len() {
                new_graph.remove_link(i as Node, self.blocked_links[i][j]);
            }
        }

        let (distance_matrix, _shortest_paths) = new_graph.all_pairs_shortest_paths(max_time);
        println!("{:?}", distance_matrix);
        let mut sum = 0.0;
        let mut reachables = 0;
        for t in 0..max_time {
            for i in 0..distance_matrix.distances.len() {
                for j in 0..distance_matrix.distances[i].len() {

                    let value = distance_matrix.distances[i][j];
                    if value > max_time {
                        continue;
                    }
                    let value = value + t;
                    if value > max_time {
                        continue;
                    }
                    sum += 1.0 / (value as f64);
                    reachables += 1;
                }
            }
        }

        let efficiency = sum / ((max_time as f64) * (graph.nb_nodes * (graph.nb_nodes - 1)) as f64);
        return (efficiency, reachables);
    }
}


// TODO : that
impl DynamicAttack {
    pub fn from_file(reader: &mut BufReader<File>, graph: &Graph) -> Self {
        return Self {
            blocked_links: Vec::new(),
            delta: 0
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
        /*for edge in &graph.nodes {
            if edge.start == node {
                if dist[node as usize] == Weight::MAX {
                    continue;
                }
                let alt = dist[node as usize] + edge.weights[time as usize];
                if alt < dist[edge.end as usize] {
                    dist[edge.end as usize] = alt;
                    prev[edge.end as usize] = node;
                    heap.push(DijkstraState {
                        node: edge.end,
                        distance: alt,
                    });
                }
            }
        }*/
        for neighbour in &graph.nodes[node as usize] {
            if dist[node as usize] == Weight::MAX {
                continue;
            }
            let alt = dist[node as usize] + neighbour.1.weights[time as usize + distance_matrix[node][neighbour.0.node as usize] as usize];
            if alt < dist[neighbour.0.node as usize] {
                dist[neighbour.0.node as usize] = alt;
                prev[neighbour.0.node as usize] = node;
                heap.push(DijkstraState {
                    node: neighbour.0.node,
                    distance: alt,
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
    for t in 0..graph.max_time {
        for i in 0..graph.dst_mat_undel.distances.len() {
            for j in 0..graph.dst_mat_undel[i as Node].len() {
                if graph.dst_mat_del[t as usize][i as Node][j] == 0 {
                    //println!("Recomputing distance from {} to {} at time {}", i, j, t);
                    // TODO : take adventage of the fact that we already have the shortest path from a bunch of other nodes
                    // FIXME : what the fuck is happening
                    let path = temporal_dijkstra(graph, i as Node, j as Node, graph.max_time, t, max_node_index.try_into().unwrap(), &graph.dst_mat_del[t as usize]);
                    let subpaths = TemporalSubPathsPyramid::from_path(graph, &path);
                    for subpath in &subpaths.subpaths {
                        let (from, to, time_start, total_length) = (subpath.start, subpath.end, t, subpath.total_weight);
                        let new_time = t + time_start;
                        if new_time < graph.max_time {
                            graph.dst_mat_del[new_time as usize][from][to as usize] = total_length;
                        }
                    }
                }
                // Remove the distance if it is too high
                // TODO TAG PROUT
                if graph.dst_mat_del[t as usize][i as Node][j] == Weight::MAX {
                    continue;
                }
                if graph.dst_mat_del[t as usize][i as Node][j] + t as Weight > graph.max_time as Weight {
                    graph.dst_mat_del[t as usize][i as Node][j] = Weight::MAX;
                }
            }
        }
        println!("Time {}", t);
    }
}

fn sum_dma(dma : &Vec<DistanceMatrix>, max_time: Time) -> (f64, u64) {
    let mut sum = 0.0;
    let mut reachables = 0;
    let distances_len = dma[0 as usize].distances.len();
    for t in 0..dma.len() {
        for i in 0..distances_len {
            for j in 0..distances_len {
                if i == j {
                    continue;
                }
                let distance = dma[t as usize][i as Node][j as usize];
                if distance != Time::MAX && distance != 0 {
                    sum += 1.0 / (distance as f64);
                    reachables += 1;
                }
            }
        }
    }
    sum -= distances_len as f64;
    sum /= dma.len() as f64 * (distances_len * (distances_len - 1)) as f64;
    return (sum, reachables);
}

struct DeletedLinksMatrix {
    links: Vec<Vec<Vec<u64>>>
}

fn create_phantom_edges(graph: &Graph, max_time: Time, deleted_edges: &Vec<DeletedLink>, dst_mat_del: &mut Vec<DistanceMatrix>) -> Vec<Vec<(Neighbour, TimeVaryingEdge)>> {
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
                        while deleted_edge.times.contains(&((t + wait))) {
                            wait += 1;
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

fn graph_to_temporal(graph: &Graph, max_time: Time, deleted_edges: &Vec<DeletedLink>) -> TimeVaryingGraph {
    let mut edges : Vec<TimeVaryingEdge> = vec![];
    let (mut dst_mat_undel, mut paths) = benchmark!("apsp", graph.all_pairs_shortest_paths(max_time));
    //let (paths,_) = benchmark!("johnson", johnson(graph, max_time));
    //let mut dst_mat_undel = benchmark!("from_corr_paths", from_shortest_corr_paths(&paths, &graph.nodes, graph.max_node_index, max_time, deleted_edges));
    //print_matrix(&dst_mat_undel);
    // TODO : change that to infinity once debuggin is done
    let mut dst_mat_del = vec![dst_mat_undel.clone(); max_time as usize];

    let annex_edges = benchmark!("create_phantom_edges", create_phantom_edges(&graph, max_time, &deleted_edges, &mut dst_mat_del));
    
    TimeVaryingGraph {
        max_time : max_time.into(),
        nodes: annex_edges,
        dst_mat_undel,
        dst_mat_del,
    }
}

impl Attack for DynamicAttack {
    fn efficiency(&self, graph: &Graph, max_time: Time) -> (f64, usize) {
        let mut time_graph= graph_to_temporal(&graph, max_time, &self.blocked_links);
        recompute_all_distance_matrix(&mut time_graph);
        let (a,b) = sum_dma(&time_graph.dst_mat_del, max_time);
        return (a,b as usize);
    }
}