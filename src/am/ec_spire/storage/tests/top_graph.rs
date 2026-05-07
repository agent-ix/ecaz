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

    fn valid_top_graph_object() -> SpireTopGraphPartitionObject {
        SpireTopGraphPartitionObject::new(
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
        .expect("top graph object should build")
    }

    #[test]
    fn top_graph_partition_object_round_trips_nodes() {
        let object = valid_top_graph_object();

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

    #[test]
    fn top_graph_partition_object_rejects_entry_node_outside_node_count() {
        let error = SpireTopGraphPartitionObject::new(
            90,
            3,
            11,
            2,
            128,
            3,
            16,
            1.2,
            2,
            vec![top_graph_node(21, 0, vec![])],
        )
        .unwrap_err();

        assert!(error.contains("entry node 2 is outside node count 1"));
    }

    #[test]
    fn top_graph_partition_object_rejects_self_neighbor() {
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
            vec![top_graph_node(21, 0, vec![0])],
        )
        .unwrap_err();

        assert!(error.contains("cannot neighbor itself"));
    }

    #[test]
    fn top_graph_partition_object_rejects_duplicate_neighbor() {
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
                top_graph_node(21, 0, vec![1, 1]),
                top_graph_node(22, 1, vec![0]),
            ],
        )
        .unwrap_err();

        assert!(error.contains("duplicate neighbor 1"));
    }

    #[test]
    fn top_graph_partition_object_rejects_invalid_alpha() {
        let below_one = SpireTopGraphPartitionObject::new(
            90,
            3,
            11,
            2,
            128,
            3,
            16,
            0.99,
            0,
            vec![top_graph_node(21, 0, vec![])],
        )
        .unwrap_err();
        let nan = SpireTopGraphPartitionObject::new(
            90,
            3,
            11,
            2,
            128,
            3,
            16,
            f32::NAN,
            0,
            vec![top_graph_node(21, 0, vec![])],
        )
        .unwrap_err();

        assert!(below_one.contains("alpha must be finite and at least 1.0"));
        assert!(nan.contains("alpha must be finite and at least 1.0"));
    }

    #[test]
    fn top_graph_partition_object_rejects_invalid_degree() {
        let zero_degree = SpireTopGraphPartitionObject::new(
            90,
            3,
            11,
            2,
            128,
            0,
            16,
            1.2,
            0,
            vec![top_graph_node(21, 0, vec![])],
        )
        .unwrap_err();
        let too_many_neighbors = SpireTopGraphPartitionObject::new(
            90,
            3,
            11,
            2,
            128,
            1,
            16,
            1.2,
            0,
            vec![
                top_graph_node(21, 0, vec![1, 2]),
                top_graph_node(22, 1, vec![0]),
                top_graph_node(23, 2, vec![0]),
            ],
        )
        .unwrap_err();

        assert!(zero_degree.contains("degree must be greater than 0"));
        assert!(too_many_neighbors.contains("neighbor count 2 exceeds graph degree 1"));
    }

    #[test]
    fn top_graph_partition_object_rejects_invalid_dimensions_level_assignments_and_flags() {
        let dimensions = SpireTopGraphPartitionObject::new(
            90,
            3,
            11,
            2,
            0,
            3,
            16,
            1.2,
            0,
            vec![top_graph_node(21, 0, vec![])],
        )
        .unwrap_err();
        let level = SpireTopGraphPartitionObject::new(
            90,
            3,
            11,
            0,
            128,
            3,
            16,
            1.2,
            0,
            vec![top_graph_node(21, 0, vec![])],
        )
        .unwrap_err();
        let mut assignment_count = valid_top_graph_object();
        assignment_count.header.assignment_count = 1;
        let mut flags = valid_top_graph_object();
        flags.header.flags = 1;

        assert!(dimensions.contains("dimensions 0 is invalid"));
        assert!(level.contains("root level 0 is invalid"));
        assert!(assignment_count.encode().unwrap_err().contains("assignment_count must be 0"));
        assert!(flags.encode().unwrap_err().contains("flags must be 0"));
    }

    #[test]
    fn top_graph_partition_object_rejects_empty_nodes_and_duplicate_child_pid() {
        let empty = SpireTopGraphPartitionObject::new(90, 3, 11, 2, 128, 3, 16, 1.2, 0, vec![])
            .unwrap_err();
        let duplicate_child_pid = SpireTopGraphPartitionObject::new(
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
                top_graph_node(21, 1, vec![]),
            ],
        )
        .unwrap_err();

        assert!(empty.contains("requires at least one node"));
        assert!(duplicate_child_pid.contains("duplicate child pid 21"));
    }

    #[test]
    fn top_graph_partition_object_decode_rejects_reserved_bytes_and_trailing_bytes() {
        let encoded = valid_top_graph_object().encode().unwrap();
        let (_header, tail) = SpirePartitionObjectHeader::decode_prefix(&encoded).unwrap();
        let body_offset = encoded.len() - tail.len();

        let mut reserved = encoded.clone();
        reserved[body_offset + 10] = 1;
        let reserved_error = SpireTopGraphPartitionObject::decode(&reserved).unwrap_err();

        let mut trailing = encoded;
        trailing.push(0);
        let trailing_error = SpireTopGraphPartitionObject::decode(&trailing).unwrap_err();

        assert!(reserved_error.contains("reserved bytes must be zero"));
        assert!(trailing_error.contains("trailing bytes"));
    }

    #[test]
    fn top_graph_partition_object_decode_rejects_truncated_body() {
        let encoded = valid_top_graph_object().encode().unwrap();
        let (_header, tail) = SpirePartitionObjectHeader::decode_prefix(&encoded).unwrap();
        let body_offset = encoded.len() - tail.len();

        let short_header = SpireTopGraphPartitionObject::decode(&encoded[..3]).unwrap_err();
        let short_prefix =
            SpireTopGraphPartitionObject::decode(&encoded[..body_offset + 10]).unwrap_err();
        let short_node =
            SpireTopGraphPartitionObject::decode(&encoded[..body_offset + 32]).unwrap_err();
        let short_neighbors =
            SpireTopGraphPartitionObject::decode(&encoded[..encoded.len() - 1]).unwrap_err();

        assert!(short_header.contains("object header"));
        assert!(short_prefix.contains("body too short"));
        assert!(short_node.contains("node 0 body too short"));
        assert!(short_neighbors.contains("neighbors extend past object body"));
    }
