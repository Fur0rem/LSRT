use std::{fs::File, io::{BufRead, BufReader}};

use sscanf::scanf;

use crate::{graph::{Graph, Node, Time}, paths::TemporalSubPathsPyramid};
use crate::time_varying_graph::{temporal_dijkstra, graph_to_temporal};
use crate::distance_matrix::sum_dma;

pub trait Attack : std::fmt::Debug {
    // efficiency + reachables
    fn efficiency(&self, graph: &Graph) -> (f64, usize);
    fn max_time(&self) -> Time;
    fn blocked_links(&self) -> &Vec<DeletedLink>;
    fn delta(&self) -> Time;
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
            return Box::new(DynamicAttack::from_file(&mut reader));
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

        let distance_matrix = new_graph.all_pairs_shortest_paths(self.max_time);
        println!("{:?}", distance_matrix);
        
        let (sum, reachables) = sum_dma(&vec![distance_matrix], self.max_time);

        let efficiency = sum / ((self.max_time as f64) * (graph.nb_nodes * (graph.nb_nodes - 1)) as f64);
        return (efficiency, reachables as usize);
    }
}


// TODO : ew it's ugly af
impl DynamicAttack {
    pub fn from_file(reader: &mut BufReader<File>) -> Self {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        println!("Line : {}", line);
        let (nb_edges, delta, max_time) = scanf!(line.trim(), "{} {} {}", usize, u16, u16).unwrap();

        // Then for each line, nb_blocked (node1,node2) <the times at which it is blocked>
        // For example : 4 (0,2) 5 9 17 18
        let mut blocked_links = vec![];
        for _ in 0..nb_edges {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
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
        // create the time varying graph
        let mut time_graph = graph_to_temporal(&graph, self.max_time(), &self.blocked_links(), self.delta());
        let max_node_index = time_graph.nodes.len().try_into().unwrap();
    
        // recompute all distance matrices from scratch
        for t in 0..time_graph.max_time {
            for i in 0..time_graph.nodes.len() {
                for j in 0..time_graph.nodes.len() {
                    if time_graph.dst_mat_del[t as usize][i as Node][j as usize] != 0 {
                        continue;
                    }
                    let path = temporal_dijkstra(&time_graph, i as Node, j as Node, time_graph.max_time, t, max_node_index);
                    let subpaths = TemporalSubPathsPyramid::from_path(&time_graph, &path);
    
                    for subpath in &subpaths.subpaths {
                        let (from, to, time_start, total_length) = (subpath.start, subpath.end, subpath.start_time, subpath.total_weight);

                        if time_start >= time_graph.max_time as u16 {
                            continue;
                        }
                        time_graph.dst_mat_del[time_start as usize][from][to as usize] = total_length;
                    }
                }
            }
        }
    
        let (a,b) = sum_dma(&time_graph.dst_mat_del, time_graph.max_time.try_into().unwrap());
    
        return (a,b as usize);
    }
}
