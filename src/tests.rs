#[cfg(test)]
mod tests {

	use crate::distance_matrix::{DistanceMatrixArray, SquareMatrix};
	use crate::graph::{dijkstra, Graph, Neighbour, Node, Tick};
	use crate::paths::{SubPathsPyramid, TemporalSubPathsPyramid};
	use crate::time_varying_graph::{
		create_phantom_edges, create_phantom_edges_aggressive, temporal_dijkstra, temporal_dijkstra_write, TimeVaryingGraph,
	};
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

		println!("Phantom edges: {:?}", phantom_edges);

		// FIXME : index timevaryingedge
		assert_eq!(phantom_edges[0][0].neighbour, 1);
		assert_eq!(phantom_edges[0][0][0], 5);
		assert_eq!(phantom_edges[0][0][1], 4);
		assert_eq!(phantom_edges[0][0][2], 3);
		assert_eq!(phantom_edges[0][0][3], 2);
		assert_eq!(phantom_edges[0][0][4], 1);
		assert_eq!(phantom_edges[0][0][5], 1);
		assert_eq!(phantom_edges[0][0][6], 1);
		assert_eq!(phantom_edges[0][1].neighbour, 2);
		assert_eq!(phantom_edges[0][1][0], 2);
		assert_eq!(phantom_edges[0][1][1], 2);
		assert_eq!(phantom_edges[3][0][max_time as usize - 1], Tick::MAX);
	}

	#[test]
	fn test_phantom_edges_aggressive() {
		let graph = make_test_graph();
		let deleted_edges = make_test_deleted_links();

		let max_time = 10;
		let delta = 2;

		let phantom_edges = create_phantom_edges_aggressive(&graph, max_time, &deleted_edges, delta);

		println!("Phantom edges: {:?}", phantom_edges);

		let graph = Graph {
			edges: vec![vec![Neighbour { node: 1, weight: 3 }], vec![Neighbour { node: 2, weight: 2 }]],
		};
		let links = vec![
			DeletedLink {
				from: 0,
				to: 1,
				times: vec![0, 2, 4, 6, 8],
			},
			DeletedLink {
				from: 1,
				to: 2,
				times: vec![0, 2],
			},
		];

		let phantom_edges = create_phantom_edges_aggressive(&graph, max_time, &links, delta);

		println!("Phantom edges: {:?}", phantom_edges);
	}

	#[test]
	fn test_pyramid() {
		// make a graph line with 4 nodes (0 -> 1 -> 2 -> 3)
		let neighbours_of_0 = vec![Neighbour { node: 1, weight: 1 }];
		let neighbours_of_1 = vec![Neighbour { node: 2, weight: 1 }];
		let neighbours_of_2 = vec![Neighbour { node: 3, weight: 1 }];
		let neighbours_of_3 = vec![];

		let graph = Graph::from_neighbours(vec![neighbours_of_0, neighbours_of_1, neighbours_of_2, neighbours_of_3]);

		let path_from_0_to_3 = dijkstra(&graph, 0, 3);
		let pyr = SubPathsPyramid::from_path(&graph, &path_from_0_to_3);
		pyr.print()
	}

	#[test]
	fn test_temporal_dijkstra() {
		let graph = make_test_graph();
		let deleted_edges = make_test_deleted_links();

		let max_time = 10;
		let delta = 2;

		let tvg = TimeVaryingGraph::from_kind(&graph, max_time, &deleted_edges, delta);

		let path_from_0_to_3_at_time_1 = temporal_dijkstra(&tvg, 0, 3, 1);
		path_from_0_to_3_at_time_1.print();
		println!("Total weight: {}", path_from_0_to_3_at_time_1.cost());
		let pyr = TemporalSubPathsPyramid::from_path(&tvg, &path_from_0_to_3_at_time_1);
		println!("Pyramid: {:?}", pyr.subpaths);

		let path_from_0_to_2_at_time_0 = temporal_dijkstra(&tvg, 0, 2, 0);
		path_from_0_to_2_at_time_0.print();
		println!("Total weight: {}", path_from_0_to_2_at_time_0.cost());
		let pyr = TemporalSubPathsPyramid::from_path(&tvg, &path_from_0_to_2_at_time_0);
		println!("Pyramid: {:?}", pyr.subpaths);

		let path_from_0_to_2_at_time_5 = temporal_dijkstra(&tvg, 0, 2, 5);
		path_from_0_to_2_at_time_5.print();
		println!("Total weight: {}", path_from_0_to_2_at_time_5.cost());
		let pyr = TemporalSubPathsPyramid::from_path(&tvg, &path_from_0_to_2_at_time_5);
		println!("Pyramid: {:?}", pyr.subpaths);

		let path_from_1_to_2_at_time_7 = temporal_dijkstra(&tvg, 1, 2, 7);
		path_from_1_to_2_at_time_7.print();
		println!("Total weight: {}", path_from_1_to_2_at_time_7.cost());
		let pyr = TemporalSubPathsPyramid::from_path(&tvg, &path_from_1_to_2_at_time_7);
		println!("Pyramid: {:?}", pyr.subpaths);
	}

	#[test]
	fn test_temportal_dijkstra_write() {
		let graph = make_test_graph();
		let deleted_edges = make_test_deleted_links();

		let max_time = 10;
		let delta = 2;

		let tvg = TimeVaryingGraph::from_kind(&graph, max_time, &deleted_edges, delta);

		let mut dma = DistanceMatrixArray::zeroed(graph.nb_nodes(), max_time);
		for time in 0..max_time {
			for node in 0..graph.nb_nodes() {
				for other_node in 0..graph.nb_nodes() {
					if node == other_node {
						continue;
					}
					let path = temporal_dijkstra(&tvg, node, other_node, time);
					dma[(time, node, other_node)] = path.cost();
				}
			}
		}

		let mut dma2 = DistanceMatrixArray::zeroed(graph.nb_nodes(), max_time);
		for time in 0..max_time {
			for node in 0..graph.nb_nodes() {
				for other_node in 0..graph.nb_nodes() {
					temporal_dijkstra_write(&tvg, node, other_node, time, &mut dma2);
				}
			}
		}

		for time in 0..max_time {
			for node in 0..graph.nb_nodes() {
				for other_node in 0..graph.nb_nodes() {
					assert_eq!(dma[(time, node, other_node)], dma2[(time, node, other_node)]);
				}
			}
		}
	}

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
		let nb_nodes = graph.edges.len();
		println!("Nb nodes: {}", nb_nodes);

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
		let (efficiency, reachables) = dma.efficiency_reachables(max_time);
		assert!(efficiency.abs() < 0.0001); // Floating point comparison
		assert_eq!(reachables, 0);
	}

	#[test]
	pub fn test_range_pyramid() {
		for i in 0..10 {
			println!("{} : {:?}", i, SubPathsPyramid::get_range(i));
		}
	}

	#[test]
	// does work yay
	pub fn test_index_pyramid() {
		for i in 0..10 {
			println!(
				"{} : prof {} left {} right {} parents {:?}",
				i,
				SubPathsPyramid::get_prof(i),
				SubPathsPyramid::get_idx_left_son(i),
				SubPathsPyramid::get_idx_right_son(i),
				SubPathsPyramid::get_idx_parents(i),
			);
			assert_eq!(SubPathsPyramid::get_prof(i), TemporalSubPathsPyramid::get_prof(i));
			assert_eq!(SubPathsPyramid::get_idx_left_son(i), TemporalSubPathsPyramid::get_idx_left_son(i));
			assert_eq!(SubPathsPyramid::get_idx_right_son(i), TemporalSubPathsPyramid::get_idx_right_son(i));
			assert_eq!(SubPathsPyramid::get_idx_parents(i), TemporalSubPathsPyramid::get_idx_parents(i));
		}
	}

	#[test]
	fn test_path_unvalidation() {
		let neighbours_of_0 = vec![Neighbour { node: 1, weight: 1 }];
		let neighbours_of_1 = vec![Neighbour { node: 0, weight: 1 }, Neighbour { node: 2, weight: 1 }];
		let neighbours_of_2 = vec![Neighbour { node: 3, weight: 1 }];
		let neighbours_of_3 = vec![Neighbour { node: 1, weight: 1 }];

		let graph = Graph::from_neighbours(vec![neighbours_of_0, neighbours_of_1, neighbours_of_2, neighbours_of_3]);

		let max_time = 10;
		let delta = 3;

		let deleted_links = vec![
			DeletedLink {
				from: 0,
				to: 1,
				times: vec![0, 1],
			},
			DeletedLink {
				from: 1,
				to: 2,
				times: vec![2],
			},
			DeletedLink {
				from: 3,
				to: 1,
				times: vec![0, 2],
			},
		];

		let attack = DynamicAttack {
			blocked_links: deleted_links,
			delta,
			max_time,
		};

		let (dst_mat, paths) = graph.all_pairs_distances_and_paths(max_time);
		for path in paths.iter() {
			println!("-----------------");
			path.print();
		}
		let mut dma = DistanceMatrixArray::propagated(&dst_mat, max_time);

		let mut already_done = SquareMatrix::new(graph.edges.len(), false);
		let mut times_matrix = SquareMatrix::new(graph.edges.len(), None);
		// push the blocked links times
		let blocked_links = attack.blocked_links();
		for link in blocked_links.iter() {
			println!("Link: {:?}", link);
			times_matrix[(link.from as usize, link.to as usize)] = Some(&link.times);
		}

		for path in paths.iter() {
			path.unvalidate_path(&mut dma, &attack, &mut already_done, &times_matrix);
		}

		dma.print(max_time);
	}

	// TODO : test index pyramid
}
