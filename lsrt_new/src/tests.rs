use crate::graph::Graph;

#[cfg(test)]
mod tests {
    use crate::{create_phantom_edges, graph::{DistanceMatrix, Node, Weight}, paths::TemporalSubPathsPyramid, temporal_dijkstra, write_correlated_paths, write_path, Attack, DeletedLink, DynamicAttack, TimeVaryingGraph};

    use super::*;

    fn make_test_graph() -> Graph {
        let neighbours_of_0 = vec![(1, 2)];
        let neighbours_of_1 = vec![(0, 2), (2, 3)];
        let neighbours_of_2 = vec![(0, 1), (1, 3)];
        let neighbours_of_3 = vec![(1, 2)];
        Graph::from_neighbours(vec![
            neighbours_of_0,
            neighbours_of_1,
            neighbours_of_2,
            neighbours_of_3,
        ])
    }

    fn make_test_deleted_links() -> Vec<DeletedLink> {
        vec![
            DeletedLink {
                from: 0,
                to: 1,
                times: vec![0, 1],
            },
            DeletedLink {
                from: 1,
                to: 2,
                times: vec![1, 2],
            },
            DeletedLink {
                from: 3,
                to: 1,
                times: vec![3, 4],
            },
        ]
    }

    /*#[test]
    fn test_phantom_edges() {
        let graph = make_test_graph();
        let deleted_edges = make_test_deleted_links();

        let dst_mat = DistanceMatrix::new(&graph);
        let max_time = 10;
        let delta = 2;
        let mut dst_mat_del = vec![];

        let phantom_edges = create_phantom_edges(&graph, max_time, &deleted_edges, &mut dst_mat_del, delta);
        for (i, phantom_edge) in phantom_edges.iter().enumerate() {
            for edge in phantom_edge {
                //println!("Phantom edge from {} to {} at time {:?}: ", i, edge.0.node, edge.1.weights);
            }
        }
    }

    #[test]
    fn test_temporal_dijkstra() {
        let graph = make_test_graph();
        let deleted_edges = make_test_deleted_links();

        let dst_mat = DistanceMatrix::new(&graph);
        let max_time = 10;
        let delta = 2;
        let max_node_index = graph.edges.len();
        let mut dst_mat_del = vec![];
        let dst_mat_undel = dst_mat.clone();

        let phantom_edges = create_phantom_edges(&graph, max_time, &deleted_edges, &mut dst_mat_del, delta);
        for (i, phantom_edge) in phantom_edges.iter().enumerate() {
            for edge in phantom_edge {
                println!("Phantom edge from {} to {} at time {:?}: ", i, edge.0.node, edge.1.weights);
            }
        }
        
        let tvg = TimeVaryingGraph {
            max_time : max_time.into(),
            nodes: phantom_edges,
            dst_mat_del,
            dst_mat_undel: dst_mat,
        };

        let path_from_0_to_3_at_time_1 = temporal_dijkstra(&tvg, 0, 3, max_time.try_into().unwrap(), 1, max_node_index.try_into().unwrap(), &dst_mat_undel);
        path_from_0_to_3_at_time_1.print();

        let path_from_0_to_2_at_time_0 = temporal_dijkstra(&tvg, 0, 2, max_time.try_into().unwrap(), 0, max_node_index.try_into().unwrap(), &dst_mat_undel);
        path_from_0_to_2_at_time_0.print();
        println!("Total weight: {}", path_from_0_to_2_at_time_0.cost());
        let pyr = TemporalSubPathsPyramid::from_path(&tvg, &path_from_0_to_2_at_time_0);
        println!("Pyramid: {:?}", pyr.subpaths);

        let path_from_0_to_2_at_time_5 = temporal_dijkstra(&tvg, 0, 2, max_time.try_into().unwrap(), 5, max_node_index.try_into().unwrap(), &dst_mat_undel);
        path_from_0_to_2_at_time_5.print();
        println!("Total weight: {}", path_from_0_to_2_at_time_5.cost());
        let pyr = TemporalSubPathsPyramid::from_path(&tvg, &path_from_0_to_2_at_time_5);
        println!("Pyramid: {:?}", pyr.subpaths);

        let path_from_1_to_2_at_time_7 = temporal_dijkstra(&tvg, 1, 2, max_time.try_into().unwrap(), 7, max_node_index.try_into().unwrap(), &dst_mat_undel);
        path_from_1_to_2_at_time_7.print();
        println!("Total weight: {}", path_from_1_to_2_at_time_7.cost());
        let pyr = TemporalSubPathsPyramid::from_path(&tvg, &path_from_1_to_2_at_time_7);
        println!("Pyramid: {:?}", pyr.subpaths);

    }*/

    #[test]
    fn test_write_correlated_paths() {
        let steps: &Vec<(Node, Weight, Node)> = &vec![(0, 1, 1), (1, 1, 2), (2, 1, 3), (3, 1, 4)];
        let time_start = 0;
        let mut dma = vec![DistanceMatrix::size(5); 5];

        write_correlated_paths(steps, time_start, &mut dma);
        for t in 0..dma.len() {
            dma[t].print();
            println!();
        }
        println!("----------------------");
        let time_start = 1;
        let mut dma = vec![DistanceMatrix::size(5); 5];
        write_correlated_paths(steps, time_start, &mut dma);
        for t in 0..dma.len() {
            dma[t].print();
            println!();
        }

    }
}