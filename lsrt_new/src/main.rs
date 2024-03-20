mod graph;
use graph::{Graph, SubPathsPyramid};

use crate::graph::dijkstra;

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
    let graph = Graph::from_sparse_row_format("../Saints_graph").unwrap();
    //graph.print();
    let (distance_matrix, shortest_paths) = benchmark!("all_pairs_shortest_paths", graph.all_pairs_shortest_paths(255));
    println!("{:?}", distance_matrix);

    // let path = dijkstra(&graph, 0, 80);
    // path.print();

    // let pyr_path = SubPathsPyramid::from_path(&graph, &path);
    
    // println!("{:?}", pyr_path.subpaths);

}