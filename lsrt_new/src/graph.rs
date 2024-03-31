use std::cmp::Ordering;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::paths::{Path, SubPathsPyramid};

use std::collections::BinaryHeap;

use crate::distance_matrix::DistanceMatrix;

pub type Node = u16;
pub type Weight = u16;
pub type Time = u16;

// Adjacency list representation of a graph


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Neighbour {
    pub node: Node,
    pub weight: Weight,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    pub nb_nodes: usize,
    pub edges: Vec<Vec<Neighbour>>,
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

    pub fn remove_link(&mut self, from: Node, to: Node) {
        let mut i = 0;
        while i < self.edges[from as usize].len() {
            if self.edges[from as usize][i].node == to {
                self.edges[from as usize].remove(i);
                return;
            }
            i += 1;
        }
    }

    pub fn from_neighbours(neighbours: Vec<Vec<(Node, Weight)>>) -> Graph {
        let mut edges = vec![];
        for neighbours_of_node in neighbours {
            let mut node_edges = vec![];
            for (node, weight) in neighbours_of_node {
                node_edges.push(Neighbour { node, weight });
            }
            edges.push(node_edges);
        }
        Graph {
            nb_nodes: edges.len(),
            edges,
        }
    }

}


#[derive(Copy, Clone, Eq, PartialEq)]
pub struct DijkstraState {
    pub node: Node,
    pub distance: u16,
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

// FIXME : probably wrong cause distance is unused
pub fn dijkstra(graph: &Graph, start: Node, end: Node) -> Path {
    let mut dist = vec![u16::MAX; graph.nb_nodes];
    let mut prev = vec![u16::MAX; graph.nb_nodes];
    let mut visited = vec![false; graph.nb_nodes];
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
        steps.push(Neighbour {
            node,
            weight,
        });
        node = prev_node;
    }
    steps.reverse();
    Path {
        steps,
        start : start as Node,
        graph,
    }
}

impl Graph {

    pub fn all_pairs_shortest_paths(&self, max_time: Weight) -> DistanceMatrix {
        let mut dst_mat = vec![vec![0; self.nb_nodes]; self.nb_nodes];
        for i in 0..self.nb_nodes {
            dst_mat[i][i] = 1;
        }

        let mut paths = vec![];
        for node in 0..self.nb_nodes {
            for node2 in 0..self.nb_nodes {

                if dst_mat[node as usize][node2 as usize] != 0 {
                    continue;
                }

                let path = dijkstra(self, node as Node, node2 as Node);
                if path.steps.is_empty() {
                    dst_mat[node as usize][node2] = u16::MAX;
                    continue;
                }

                let subpaths = SubPathsPyramid::from_path(self, &path);
                for subpath in &subpaths.subpaths {
                    if subpath.total_weight > max_time {
                        dst_mat[subpath.start as usize][subpath.end as usize] = u16::MAX;
                        continue;
                    }

                    dst_mat[subpath.start as usize][subpath.end as usize] = subpath.total_weight;
                }
                paths.push(subpaths);
            }
        }

        return DistanceMatrix { distances: dst_mat };
    } 
}