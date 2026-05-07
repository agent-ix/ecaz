    fn top_graph_node(
        child_pid: u64,
        centroid_ordinal: u32,
        neighbors: Vec<u32>,
    ) -> SpireTopGraphNodeRecord {
        SpireTopGraphNodeRecord {
            child_pid,
            centroid_ordinal,
            neighbors,
        }
    }

    #[test]
    fn top_graph_partition_object_round_trips_nodes() {
        let object = SpireTopGraphPartitionObject::new(
            90,
            3,
            11,
            2,
            128,
            3,
            16,
            1.2,
            1,
            vec![
                top_graph_node(21, 0, vec![1, 2]),
                top_graph_node(22, 1, vec![0, 2]),
                top_graph_node(23, 2, vec![0, 1]),
            ],
        )
        .expect("top graph object should build");

        let decoded = SpireTopGraphPartitionObject::decode(&object.encode().unwrap()).unwrap();

        assert_eq!(decoded, object);
        assert_eq!(decoded.header.kind, SpirePartitionObjectKind::TopGraph);
        assert_eq!(decoded.header.parent_pid, 11);
        assert_eq!(decoded.header.child_count, 3);
        assert_eq!(decoded.root_pid, 11);
        assert_eq!(decoded.dimensions, 128);
        assert_eq!(decoded.graph_degree, 3);
        assert_eq!(decoded.build_list_size, 16);
        assert_eq!(decoded.entry_node, 1);
        assert_eq!(decoded.nodes[1].neighbors, vec![0, 2]);
    }

    #[test]
    fn top_graph_partition_object_rejects_neighbor_outside_node_count() {
        let error = SpireTopGraphPartitionObject::new(
            90,
            3,
            11,
            2,
            128,
            3,
            16,
            1.2,
            0,
            vec![top_graph_node(21, 0, vec![1])],
        )
        .unwrap_err();

        assert!(error.contains("outside node count"));
    }

    #[test]
    fn top_graph_partition_object_rejects_duplicate_centroid_ordinal() {
        let error = SpireTopGraphPartitionObject::new(
            90,
            3,
            11,
            2,
            128,
            3,
            16,
            1.2,
            0,
            vec![
                top_graph_node(21, 0, vec![]),
                top_graph_node(22, 0, vec![]),
            ],
        )
        .unwrap_err();

        assert!(error.contains("duplicate centroid ordinal 0"));
    }
