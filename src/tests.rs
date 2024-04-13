#[cfg(test)]
mod tests {

	use crate::distance_matrix::{print_dma, sum_dma};
	use crate::graph::{dijkstra, Graph, Neighbour, Node};
	use crate::paths::TemporalSubPathsPyramid;
	use crate::time_varying_graph::{create_phantom_edges, temporal_dijkstra, TimeVaryingGraph};
	use crate::{Attack, DynamicAttack, StaticAttack};
	use crate::{AttackResult, DeletedLink};

	fn make_test_graph() -> Graph {
		let neighbours_of_0 = vec![Neighbour { node: 1, weight: 1 }, Neighbour { node: 2, weight: 2 }];
		let neighbours_of_1 = vec![
			Neighbour { node: 0, weight: 1 },
			Neighbour { node: 2, weight: 1 },
			Neighbour { node: 3, weight: 1 },
		];
		let neighbours_of_2 = vec![
			Neighbour { node: 0, weight: 3 },
			Neighbour { node: 1, weight: 1 },
			Neighbour { node: 3, weight: 1 },
		];
		let neighbours_of_3 = vec![Neighbour { node: 1, weight: 1 }, Neighbour { node: 2, weight: 1 }];

		Graph::from_neighbours(vec![neighbours_of_0, neighbours_of_1, neighbours_of_2, neighbours_of_3])
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

	#[test]
	fn test_phantom_edges() {
		let graph = make_test_graph();
		let deleted_edges = make_test_deleted_links();

		let max_time = 10;
		let delta = 2;

		let phantom_edges = create_phantom_edges(&graph, max_time, &deleted_edges, delta);

		// FIXME : index timevaryingedge
		assert_eq!(phantom_edges[0][0].node, 1);
		assert_eq!(phantom_edges[0][0].weights.weights[0], 5);
		assert_eq!(phantom_edges[0][0].weights.weights[1], 4);
		assert_eq!(phantom_edges[0][0].weights.weights[2], 3);
		assert_eq!(phantom_edges[0][0].weights.weights[3], 2);
		assert_eq!(phantom_edges[0][0].weights.weights[4], 1);
		assert_eq!(phantom_edges[0][0].weights.weights[5], 1);
		assert_eq!(phantom_edges[0][0].weights.weights[6], 1);
		assert_eq!(phantom_edges[0][1].node, 2);
		assert_eq!(phantom_edges[0][1].weights.weights[0], 2);
		assert_eq!(phantom_edges[0][1].weights.weights[1], 2);
	}

	/*#[test]
	fn test_temporal_dijkstra() {
		let graph = make_test_graph();
		let deleted_edges = make_test_deleted_links();

		let max_time = 10;
		let delta = 2;

		let phantom_edges = create_phantom_edges(&graph, max_time, &deleted_edges, delta);

		let tvg = TimeVaryingGraph {
			max_time: max_time.into(),
			nodes: phantom_edges,
		};

		let path_from_0_to_3_at_time_1 = temporal_dijkstra(&tvg, 0, 3, max_time.try_into().unwrap(), 1);
		path_from_0_to_3_at_time_1.print();

		let path_from_0_to_2_at_time_0 = temporal_dijkstra(&tvg, 0, 2, max_time.try_into().unwrap(), 0);
		path_from_0_to_2_at_time_0.print();
		println!("Total weight: {}", path_from_0_to_2_at_time_0.cost());
		let pyr = TemporalSubPathsPyramid::from_path(&tvg, &path_from_0_to_2_at_time_0);
		println!("Pyramid: {:?}", pyr.subpaths);

		let path_from_0_to_2_at_time_5 = temporal_dijkstra(&tvg, 0, 2, max_time.try_into().unwrap(), 5);
		path_from_0_to_2_at_time_5.print();
		println!("Total weight: {}", path_from_0_to_2_at_time_5.cost());
		let pyr = TemporalSubPathsPyramid::from_path(&tvg, &path_from_0_to_2_at_time_5);
		println!("Pyramid: {:?}", pyr.subpaths);

		let path_from_1_to_2_at_time_7 = temporal_dijkstra(&tvg, 1, 2, max_time.try_into().unwrap(), 7);
		path_from_1_to_2_at_time_7.print();
		println!("Total weight: {}", path_from_1_to_2_at_time_7.cost());
		let pyr = TemporalSubPathsPyramid::from_path(&tvg, &path_from_1_to_2_at_time_7);
		println!("Pyramid: {:?}", pyr.subpaths);
	}*/

	#[test]
	fn test_dijkstra() {
		let graph = make_test_graph();

		let start = 0;
		let end = 3;
		let path = dijkstra(&graph, start, end);
		assert_eq!(path.cost(), 2);
		assert_eq!(path.start, start);
		assert_eq!(
			path.steps,
			vec![Neighbour { node: 1, weight: 1 }, Neighbour { node: 3, weight: 1 },]
		);

		let start = 2;
		let end = 0;
		let path = dijkstra(&graph, start, end);
		assert_eq!(
			path.steps,
			vec![Neighbour { node: 1, weight: 1 }, Neighbour { node: 0, weight: 1 },]
		);

		let start = 1;
		let end = 0;
		let path = dijkstra(&graph, start, end);
		assert_eq!(path.steps, vec![Neighbour { node: 0, weight: 1 },]);
	}

	#[test]
	fn coherence_static_dynamic() {
		let graph = make_test_graph();
		let max_time = 10;
		let deleted_links = vec![(0, 1), (1, 2), (3, 1)];
		let delta = 1;

		// Build up a dynamic attack with the same deleted links
		let deleted_links_dynamic = deleted_links
			.iter()
			.map(|(from, to)| DeletedLink {
				from: *from,
				to: *to,
				times: (0..max_time).collect(),
			})
			.collect();
		let attack_dynamic = DynamicAttack {
			blocked_links: deleted_links_dynamic,
			delta,
			max_time,
		};

		// Build up a static attack with the same deleted links
		let mut blocked_links = vec![vec![]; graph.edges.len()];
		for (from, to) in deleted_links {
			blocked_links[from as usize].push(to);
		}
		let attack_static = StaticAttack {
			blocked_links,
			delta,
			max_time,
		};

		assert_eq!(attack_dynamic.blocked_links(), attack_static.blocked_links());

		let (efficiency_dynamic, reach_dynamic) = attack_dynamic.efficiency(&graph);
		let (efficiency_static, reach_static) = attack_static.efficiency(&graph);

		assert!((efficiency_dynamic - efficiency_static).abs() < 0.0001); // Floating point comparison
		assert_eq!(reach_dynamic, reach_static);
	}

	#[test]
	fn delete_all_links() {
		let graph = make_test_graph();
		let max_time = 10;
		let delta = 1;

		let deleted_links = graph.edges.iter().enumerate().flat_map(|(from, neighbours)| {
			neighbours.iter().map(move |neighbour| DeletedLink {
				from: from as Node,
				to: neighbour.node,
				times: (0..max_time).collect(),
			})
		});

		let attack = DynamicAttack {
			blocked_links: deleted_links.collect(),
			delta,
			max_time,
		};

		let attack_result = attack.all_pairs_distances(&graph);
		let dma = match attack_result {
			AttackResult::Dynamic(dma) => dma,
			_ => panic!("Expected a dynamic matrix"),
		};
		let (efficiency, reachables) = sum_dma(&dma, attack.max_time());
		assert!(efficiency.abs() < 0.0001); // Floating point comparison
		assert_eq!(reachables, 0);
	}
}
