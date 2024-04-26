#[cfg(test)]
mod tests {

	use crate::attack::{Attack, AttackResult, DynamicAttack, StaticAttack};
	use crate::distance_matrix::{DistanceMatrix, DistanceMatrixArray, SquareMatrix};
	use crate::graph::{dijkstra, dijkstra_write, Graph, Neighbour};
	use crate::paths::{SubPathsPyramid, TemporalSubPathsPyramid};
	use crate::time_varying_graph::{
		create_phantom_edges, create_phantom_edges_aggressive, temporal_dijkstra, temporal_dijkstra_write, PhantomEdgesMap,
		TimeVaryingGraph,
	};
	use crate::DeletedLink;
	use crate::{Node, Tick};

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

		let max_tick = 10;
		let delta = 2;

		let phantom_edges = create_phantom_edges(&graph, max_tick, &deleted_edges, delta);

		println!("Phantom edges: {:?}", phantom_edges);

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
		assert_eq!(phantom_edges[3][0][max_tick - 1], Tick::MAX);
	}

	#[test]
	fn test_phantom_edges_aggressive() {
		let graph = make_test_graph();
		let deleted_edges = make_test_deleted_links();

		let max_tick = 10;
		let delta = 2;

		let phantom_edges = create_phantom_edges_aggressive(&graph, max_tick, &deleted_edges, delta);

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

		let phantom_edges = create_phantom_edges_aggressive(&graph, max_tick, &links, delta);

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

		let max_tick = 10;
		let delta = 2;

		let tvg = TimeVaryingGraph::from_kind(&graph, max_tick, &deleted_edges, delta);
		let edges_map = PhantomEdgesMap::from_graph(&tvg);

		let path_from_0_to_3_at_time_1 = temporal_dijkstra(&tvg, 0, 3, 1, &edges_map);
		path_from_0_to_3_at_time_1.print();
		println!("Total weight: {}", path_from_0_to_3_at_time_1.cost());
		let pyr = TemporalSubPathsPyramid::from_path(&tvg, &path_from_0_to_3_at_time_1);
		println!("Pyramid: {:?}", pyr.subpaths);

		let path_from_0_to_2_at_time_0 = temporal_dijkstra(&tvg, 0, 2, 0, &edges_map);
		path_from_0_to_2_at_time_0.print();
		println!("Total weight: {}", path_from_0_to_2_at_time_0.cost());
		let pyr = TemporalSubPathsPyramid::from_path(&tvg, &path_from_0_to_2_at_time_0);
		println!("Pyramid: {:?}", pyr.subpaths);

		let path_from_0_to_2_at_time_5 = temporal_dijkstra(&tvg, 0, 2, 5, &edges_map);
		path_from_0_to_2_at_time_5.print();
		println!("Total weight: {}", path_from_0_to_2_at_time_5.cost());
		let pyr = TemporalSubPathsPyramid::from_path(&tvg, &path_from_0_to_2_at_time_5);
		println!("Pyramid: {:?}", pyr.subpaths);

		let path_from_1_to_2_at_time_7 = temporal_dijkstra(&tvg, 1, 2, 7, &edges_map);
		path_from_1_to_2_at_time_7.print();
		println!("Total weight: {}", path_from_1_to_2_at_time_7.cost());
		let pyr = TemporalSubPathsPyramid::from_path(&tvg, &path_from_1_to_2_at_time_7);
		println!("Pyramid: {:?}", pyr.subpaths);
	}

	#[test]
	fn test_temportal_dijkstra_write() {
		let graph = make_test_graph();
		let deleted_edges = make_test_deleted_links();

		let max_tick = 10;
		let delta = 2;

		let tvg = TimeVaryingGraph::from_kind(&graph, max_tick, &deleted_edges, delta);
		let edges_map = PhantomEdgesMap::from_graph(&tvg);

		let mut dma = DistanceMatrixArray::zeroed(graph.nb_nodes(), max_tick);
		for time in 0..max_tick {
			for node in 0..graph.nb_nodes() {
				for other_node in 0..graph.nb_nodes() {
					if node == other_node {
						continue;
					}
					let path = temporal_dijkstra(&tvg, node, other_node, time, &edges_map);
					dma[(time, node, other_node)] = path.cost();
				}
			}
		}

		let mut dma2 = DistanceMatrixArray::zeroed(graph.nb_nodes(), max_tick);
		for time in 0..max_tick {
			for node in 0..graph.nb_nodes() {
				for other_node in 0..graph.nb_nodes() {
					temporal_dijkstra_write(&tvg, node, other_node, time, &mut dma2, &edges_map);
				}
			}
		}

		for time in 0..max_tick {
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
	fn test_dijkstra_write() {
		let graph = make_test_graph();

		let mut dm1 = DistanceMatrix::zeroed(&graph);
		let mut dm2 = DistanceMatrix::zeroed(&graph);
		let mut dm3 = DistanceMatrix::zeroed(&graph);
		let mut dm4 = DistanceMatrix::zeroed(&graph);

		for i in 0..graph.nb_nodes() {
			for j in 0..graph.nb_nodes() {
				if dm1[(i, j)] == 0 {
					let path = dijkstra(&graph, i, j);
					dm1[(i, j)] = path.cost();
				}
				if dm2[(i, j)] == 0 {
					dijkstra_write(&graph, i, j, &mut dm2);
				}
				let path = dijkstra(&graph, i, j);
				dm3[(i, j)] = path.cost();
				dijkstra_write(&graph, i, j, &mut dm4);
			}
		}

		for i in 0..graph.nb_nodes() {
			for j in 0..graph.nb_nodes() {
				assert_eq!(dm1[(i, j)], dm2[(i, j)]);
				assert_eq!(dm1[(i, j)], dm3[(i, j)]);
				assert_eq!(dm1[(i, j)], dm4[(i, j)]);
			}
		}
	}

	#[test]
	fn coherence_static_dynamic() {
		let graph = make_test_graph();
		let max_tick = 10;
		let deleted_links = vec![(0, 1), (1, 2), (3, 1)];
		let delta = 1;

		// Build up a dynamic attack with the same deleted links
		let deleted_links_dynamic = deleted_links
			.iter()
			.map(|(from, to)| DeletedLink {
				from: *from,
				to: *to,
				times: (0..max_tick).collect(),
			})
			.collect();
		let attack_dynamic = DynamicAttack {
			blocked_links: deleted_links_dynamic,
			delta,
			max_tick,
		};

		// Build up a static attack with the same deleted links
		let mut blocked_links = vec![vec![]; graph.edges.len()];
		for (from, to) in deleted_links {
			blocked_links[from as usize].push(to);
		}
		let attack_static = StaticAttack {
			blocked_links,
			delta,
			max_tick,
		};

		assert_eq!(attack_dynamic.blocked_links(), attack_static.blocked_links());

		let (efficiency_dynamic, reach_dynamic) = attack_dynamic.efficiency_reachables(&graph);
		let (efficiency_static, reach_static) = attack_static.efficiency_reachables(&graph);

		let dm_static = match attack_static.all_pairs_distances(&graph) {
			AttackResult::Static(dms) => dms,
			_ => panic!("Expected a static matrix"),
		};
		let dm_dynamic = match attack_dynamic.all_pairs_distances(&graph) {
			AttackResult::Dynamic(dma) => dma,
			_ => panic!("Expected a dynamic matrix"),
		};

		dm_static.print();
		dm_dynamic.print();

		println!("Efficiency dynamic: {}", efficiency_dynamic);
		println!("Efficiency static: {}", efficiency_static);
		assert!((efficiency_dynamic - efficiency_static).abs() < 0.0001); // Floating point comparison
		assert_eq!(reach_dynamic, reach_static);
	}

	#[test]
	fn delete_all_links() {
		let graph = make_test_graph();
		let max_tick = 10;
		let delta = 1;
		let nb_nodes = graph.edges.len();
		println!("Nb nodes: {}", nb_nodes);

		let deleted_links = graph.edges.iter().enumerate().flat_map(|(from, neighbours)| {
			neighbours.iter().map(move |neighbour| DeletedLink {
				from: from as Node,
				to: neighbour.node,
				times: (0..max_tick).collect(),
			})
		});

		let attack = DynamicAttack {
			blocked_links: deleted_links.collect(),
			delta,
			max_tick,
		};

		let attack_result = attack.all_pairs_distances(&graph);
		let dma = match attack_result {
			AttackResult::Dynamic(dma) => dma,
			_ => panic!("Expected a dynamic matrix"),
		};
		let (efficiency, reachables) = dma.efficiency_reachables(max_tick);
		assert!(efficiency.abs() < 0.0001); // Floating point comparison
		assert_eq!(reachables, 0);
	}

	#[test]
	fn efficiency_of_one() {
		let neighbours_of_0 = vec![
			Neighbour { node: 1, weight: 1 },
			Neighbour { node: 2, weight: 1 },
			Neighbour { node: 3, weight: 1 },
		];
		let neighbours_of_1 = vec![
			Neighbour { node: 0, weight: 1 },
			Neighbour { node: 2, weight: 1 },
			Neighbour { node: 3, weight: 1 },
		];
		let neighbours_of_2 = vec![
			Neighbour { node: 0, weight: 1 },
			Neighbour { node: 1, weight: 1 },
			Neighbour { node: 3, weight: 1 },
		];
		let neighbours_of_3 = vec![
			Neighbour { node: 0, weight: 1 },
			Neighbour { node: 1, weight: 1 },
			Neighbour { node: 2, weight: 1 },
		];

		let graph = Graph::from_neighbours(vec![neighbours_of_0, neighbours_of_1, neighbours_of_2, neighbours_of_3]);
		let max_tick = 10;
		let delta = 1;

		let none_attack = DynamicAttack {
			blocked_links: vec![],
			delta,
			max_tick,
		};

		let none_attack_static = StaticAttack {
			blocked_links: vec![vec![]; graph.edges.len()],
			delta,
			max_tick,
		};

		let dm_static = match none_attack_static.all_pairs_distances(&graph) {
			AttackResult::Static(dms) => dms,
			_ => panic!("Expected a static matrix"),
		};
		let dm_dynamic = match none_attack.all_pairs_distances(&graph) {
			AttackResult::Dynamic(dma) => dma,
			_ => panic!("Expected a dynamic matrix"),
		};

		let (efficiency_static, reach_static) = dm_static.efficiency_reachables(max_tick);
		let (efficiency_dynamic, reach_dynamic) = dm_dynamic.efficiency_reachables(max_tick);

		assert_eq!(efficiency_static, 1.0);
		assert_eq!(efficiency_dynamic, 1.0);
		assert_eq!(
			reach_static,
			((graph.edges.len() - 1) * graph.edges.len() * usize::from(max_tick)) as u64
		);
		assert_eq!(reach_dynamic, reach_static);
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
	fn test_path_invalidation() {
		let neighbours_of_0 = vec![Neighbour { node: 1, weight: 1 }];
		let neighbours_of_1 = vec![Neighbour { node: 2, weight: 1 }];
		let neighbours_of_2 = vec![Neighbour { node: 3, weight: 1 }];
		let neighbours_of_3 = vec![Neighbour { node: 4, weight: 1 }];
		let neighbours_of_4 = vec![Neighbour { node: 5, weight: 1 }];
		let neighbours_of_5 = vec![];

		let graph = Graph::from_neighbours(vec![
			neighbours_of_0,
			neighbours_of_1,
			neighbours_of_2,
			neighbours_of_3,
			neighbours_of_4,
			neighbours_of_5,
		]);

		let path = dijkstra(&graph, 0, 5);
		let path = SubPathsPyramid::from_path(&graph, &path);
		let deleted_links = vec![
			DeletedLink {
				from: 1,
				to: 2,
				times: vec![0, 2],
			},
			DeletedLink {
				from: 3,
				to: 4,
				times: vec![0, 2],
			},
		];

		let max_tick = 10;
		let delta = 1;

		let tvg = TimeVaryingGraph::from_aggressive(&graph, max_tick, &deleted_links, delta);
		let edges_map = PhantomEdgesMap::from_graph(&tvg);
		let dst_mat = graph.all_pairs_distances();
		//dst_mat.print();
		let mut dma = DistanceMatrixArray::propagated(&dst_mat, max_tick);
		//dma.print(tvg.max_tick());
		let mut already_done = SquareMatrix::new(false, graph.nb_nodes().into());
		path.invalidate_path(&tvg, &mut already_done, &mut dma, &dst_mat, &edges_map);
		//dma.print(tvg.max_tick());

		path.print();
		for t in 0..max_tick {
			for i in 0..graph.nb_nodes() {
				for j in 0..graph.nb_nodes() {
					if dma[(t, i, j)] == 0 {
						println!("Invalidation from {} to {} at time {}", i, j, t);
					}
				}
			}
		}
	}
}
