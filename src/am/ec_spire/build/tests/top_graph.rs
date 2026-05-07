    fn top_graph_node(
        child_pid: u64,
        centroid_ordinal: u32,
        centroid: Vec<f32>,
    ) -> SpireTopGraphNodeInput {
        SpireTopGraphNodeInput {
            child_pid,
            centroid_ordinal,
            centroid,
        }
    }

    #[test]
    fn top_graph_builds_vamana_neighbors_for_routing_children() {
        let draft = super::build_spire_top_graph_draft(SpireTopGraphBuildInput {
            root_pid: 50,
            dimensions: 2,
            graph_degree: 2,
            build_list_size: 4,
            alpha: 1.2,
            seed: 42,
            nodes: vec![
                top_graph_node(11, 0, vec![1.0, 0.0]),
                top_graph_node(12, 1, vec![0.8, 0.2]),
                top_graph_node(13, 2, vec![-1.0, 0.0]),
                top_graph_node(14, 3, vec![-0.8, 0.2]),
            ],
        })
        .expect("top graph should build");

        assert_eq!(draft.root_pid, 50);
        assert_eq!(draft.dimensions, 2);
        assert_eq!(draft.node_count, 4);
        assert_eq!(draft.graph_degree, 2);
        assert_eq!(draft.build_list_size, 4);
        assert_eq!(draft.alpha, 1.2);
        assert!(draft.entry_node < draft.node_count);
        assert_eq!(
            draft
                .nodes
                .iter()
                .map(|node| (node.child_pid, node.centroid_ordinal))
                .collect::<Vec<_>>(),
            vec![(11, 0), (12, 1), (13, 2), (14, 3)]
        );
        assert_eq!(draft.stats.node_count, 4);
        assert_eq!(draft.stats.medoid, draft.entry_node);
        assert_eq!(draft.stats.max_degree, 2);
        assert_eq!(draft.stats.list_size, 4);
        assert_eq!(draft.stats.alpha_final, 1.2);
        assert!(draft.nodes.iter().any(|node| !node.neighbors.is_empty()));
        for node in &draft.nodes {
            assert!(node.neighbors.len() <= 2);
            for &neighbor in &node.neighbors {
                assert!(neighbor < draft.node_count);
            }
        }
    }

    #[test]
    fn top_graph_builds_from_root_routing_object() {
        let root = SpireRoutingPartitionObject::root(
            50,
            3,
            2,
            vec![
                SpireRoutingChildEntry {
                    centroid_index: 0,
                    child_pid: 11,
                    centroid: vec![1.0, 0.0],
                },
                SpireRoutingChildEntry {
                    centroid_index: 1,
                    child_pid: 12,
                    centroid: vec![0.8, 0.2],
                },
                SpireRoutingChildEntry {
                    centroid_index: 2,
                    child_pid: 13,
                    centroid: vec![-1.0, 0.0],
                },
            ],
        )
        .expect("root routing object should build");

        let draft = super::build_spire_top_graph_draft_from_routing_object(
            &root,
            SpireTopGraphBuildParams {
                graph_degree: 2,
                build_list_size: 3,
                alpha: 1.2,
                seed: 42,
            },
        )
        .expect("top graph should build from root routing object");

        assert_eq!(draft.root_pid, root.header.pid);
        assert_eq!(draft.dimensions, root.dimensions);
        assert_eq!(
            draft
                .nodes
                .iter()
                .map(|node| (node.child_pid, node.centroid_ordinal))
                .collect::<Vec<_>>(),
            vec![(11, 0), (12, 1), (13, 2)]
        );
    }

    #[test]
    fn top_graph_rejects_internal_routing_object_adapter_input() {
        let internal = SpireRoutingPartitionObject::internal(
            60,
            3,
            1,
            50,
            2,
            vec![SpireRoutingChildEntry {
                centroid_index: 0,
                child_pid: 11,
                centroid: vec![1.0, 0.0],
            }],
        )
        .expect("internal routing object should build");

        let error = super::build_spire_top_graph_draft_from_routing_object(
            &internal,
            SpireTopGraphBuildParams {
                graph_degree: 2,
                build_list_size: 3,
                alpha: 1.2,
                seed: 42,
            },
        )
        .unwrap_err();

        assert!(error.contains("requires root routing object"));
    }

    #[test]
    fn top_graph_build_rejects_duplicate_child_pid() {
        let error = super::build_spire_top_graph_draft(SpireTopGraphBuildInput {
            root_pid: 50,
            dimensions: 2,
            graph_degree: 2,
            build_list_size: 4,
            alpha: 1.2,
            seed: 42,
            nodes: vec![
                top_graph_node(11, 0, vec![1.0, 0.0]),
                top_graph_node(11, 1, vec![0.0, 1.0]),
            ],
        })
        .unwrap_err();

        assert!(error.contains("duplicate child pid 11"));
    }

    #[test]
    fn top_graph_build_rejects_bad_centroid_dimension() {
        let error = super::build_spire_top_graph_draft(SpireTopGraphBuildInput {
            root_pid: 50,
            dimensions: 2,
            graph_degree: 2,
            build_list_size: 4,
            alpha: 1.2,
            seed: 42,
            nodes: vec![top_graph_node(11, 0, vec![1.0, 0.0, 0.5])],
        })
        .unwrap_err();

        assert!(error.contains("centroid dimensions mismatch"));
    }
