mod graph;
use graph::Graph;

mod attack;
use attack::*;

mod paths;

use crate::graph::dijkstra;

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

fn main() {
    let old_efficiency;
    let new_efficiency;
    let imp_efficiency;
    let old_distance_matrix;
    let imp_distance_matrix;

    benchmark!("everything", {
        let graph = Graph::from_sparse_row_format("../Saints_graph").unwrap();
        let attack = <dyn Attack>::from_file("../attack.tQL7hx", &graph);
        let (efficiency, reachables) = attack.efficiency(&graph);
        println!("efficiency: {}, reachables: {}", efficiency, reachables);
        new_efficiency = efficiency;
    });

    println!("NEW");

    benchmark!("everything old", {
        let graph = Graph::from_sparse_row_format("../Saints_graph").unwrap();
        let attack = <dyn Attack>::from_file("../attack.tQL7hx", &graph);
        let (efficiency, reachables, distance_matrix) = dumb_efficiency(&graph, &*attack);
        println!("new efficiency: {}, new reachables: {}", efficiency, reachables);
        old_distance_matrix = distance_matrix;
        old_efficiency = efficiency;
    });
    println!("IMPROVED");
    benchmark!("everything improved", {
        let graph = Graph::from_sparse_row_format("../Saints_graph").unwrap();
        let attack = <dyn Attack>::from_file("../attack.tQL7hx", &graph);
        let (efficiency, reachables, distance_matrix) = improved_efficiency(&graph, &*attack);
        println!("new efficiency: {}, new reachables: {}", efficiency, reachables);
        imp_distance_matrix = distance_matrix;
        imp_efficiency = efficiency;
    });

    /*
    benchmark!("everything", {
        let graph = Graph::from_sparse_row_format("../Maza_graph").unwrap();
        let attack = <dyn Attack>::from_file("../attack.7N80g5", &graph);
        let (efficiency, reachables) = attack.efficiency(&graph);
        println!("efficiency: {}, reachables: {}", efficiency, reachables);
        new_efficiency = efficiency;
    });
    println!("NEW");
    benchmark!("everything old", {
        let graph = Graph::from_sparse_row_format("../Maza_graph").unwrap();
        let attack = <dyn Attack>::from_file("../attack.7N80g5", &graph);
        let (efficiency, reachables, distance_matrix) = dumb_efficiency(&graph, &*attack);
        println!("new efficiency: {}, new reachables: {}", efficiency, reachables);
        old_efficiency = efficiency;
        old_distance_matrix = distance_matrix;
    });
    println!("IMPROVED");
    benchmark!("everything improved", {
        let graph = Graph::from_sparse_row_format("../Maza_graph").unwrap();
        let attack = <dyn Attack>::from_file("../attack.7N80g5", &graph);
        let (efficiency, reachables, distance_matrix) = improved_efficiency(&graph, &*attack);
        println!("new efficiency: {}, new reachables: {}", efficiency, reachables);
        imp_efficiency = efficiency;
        imp_distance_matrix = distance_matrix;
    });*/

    println!("new: {}, old: {}, improved: {}", new_efficiency, old_efficiency, imp_efficiency);

    for t in 0..old_distance_matrix.len() {
        for i in 0..old_distance_matrix[t].distances.len() {
            for j in 0..old_distance_matrix[t][i as u16].len() {
                if old_distance_matrix[t][i as u16][j] != imp_distance_matrix[t][i as u16][j] {
                    println!("t : {}  i : {}  j : {}  old : {}  imp : {}", t, i, j, old_distance_matrix[t][i as u16][j], imp_distance_matrix[t][i as u16][j]);
                }
            }
        }
    }
}