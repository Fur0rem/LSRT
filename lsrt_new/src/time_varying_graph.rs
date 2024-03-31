use crate::graph::{Graph, Neighbour, Node, Time, Weight};
use crate::paths::TemporalPath;
use crate::distance_matrix::DistanceMatrix;
use crate::DeletedLink;
use std::collections::BinaryHeap;

use crate::graph::DijkstraState;

// TODO : put the tvg into their own files
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeVaryingEdge {
    pub weights: Vec<Weight>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeVaryingGraph {
    pub max_time: u64,
    pub nodes: Vec<Vec<(Neighbour, TimeVaryingEdge)>>,
    pub dst_mat_del: Vec<DistanceMatrix>,
}

pub fn temporal_dijkstra<'a>(graph: &'a TimeVaryingGraph, start: Node, end: Node, max_time: u64, time_start: u64, max_node_index: u64) -> TemporalPath<'a> {
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
            if (distance as u64 + time_start) >= max_time {
                continue;
            }
            let weight = edge.weights[distance as usize + time_start as usize];
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

    let mut steps = vec![];
    let mut node = end;
    while node != start {
        let prev_node = prev[node as usize];
        if prev_node == u16::MAX {
            break;
        }
        steps.push(graph.nodes[prev_node as usize].iter().find(|x| x.0.node == node).unwrap());
        node = prev_node;
    }
    steps.reverse();
    TemporalPath {
        steps,
        graph,
        start,
        start_time: time_start as Time,
    }
}

pub fn create_phantom_edges(graph: &Graph, max_time: Time, deleted_edges: &Vec<DeletedLink>, delta : u16) -> Vec<Vec<(Neighbour, TimeVaryingEdge)>> {
    let mut annex_edges = vec![];
    for (node_idx, node) in graph.edges.iter().enumerate() {
        let mut edges = vec![];
        for neighbour in node {
            let weight = &neighbour.weight;
            let mut weights = vec![];
            for t in 0..max_time {
                let mut wait = 0;
                for deleted_edge in deleted_edges {
                    if deleted_edge.from == node_idx as Node && deleted_edge.to == neighbour.node {
                        while deleted_edge.times.contains(&((t + wait) / delta)) {
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

pub fn graph_to_temporal(graph: &Graph, max_time: Time, deleted_edges: &Vec<DeletedLink>, delta : u16) -> TimeVaryingGraph {

    let dst_mat_del = vec![DistanceMatrix::zero(graph); max_time as usize];

    let annex_edges = create_phantom_edges(&graph, max_time, &deleted_edges, delta);
    TimeVaryingGraph {
        max_time : max_time.into(),
        nodes: annex_edges,
        dst_mat_del,
    }
}