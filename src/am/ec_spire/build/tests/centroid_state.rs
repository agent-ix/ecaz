    #[test]
    fn single_level_centroid_plan_routes_vectors_with_common_training() {
        let source_vectors = vec![vec![1.0, 0.0], vec![-1.0, 0.0]];

        let plan = train_single_level_centroid_plan(2, &source_vectors, 2, 42).unwrap();

        assert_eq!(plan.dimensions, 2);
        assert_eq!(plan.centroid_count(), 2);
        assert_eq!(plan.assignment_indexes.len(), source_vectors.len());
        assert_ne!(plan.assignment_indexes[0], plan.assignment_indexes[1]);
    }

    #[test]
    fn single_level_centroid_plan_resolves_auto_nlists_and_rejects_bad_vectors() {
        let source_vectors = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let plan = train_single_level_centroid_plan(2, &source_vectors, 0, 42).unwrap();

        assert_eq!(plan.centroid_count(), 2);

        assert!(train_single_level_centroid_plan(2, &[vec![1.0]], 1, 42)
            .unwrap_err()
            .contains("dimensions mismatch"));
        assert!(
            train_single_level_centroid_plan(2, &[vec![0.0, 0.0]], 1, 42)
                .unwrap_err()
                .contains("non-zero")
        );
    }

    #[test]
    fn single_level_route_map_routes_query_to_centroid_pid() {
        let centroid_plan = SpireSingleLevelCentroidPlan {
            dimensions: 2,
            centroids: vec![vec![1.0, 0.0], vec![-1.0, 0.0]],
            assignment_indexes: Vec::new(),
        };
        let route_map =
            SpireSingleLevelRouteMap::from_centroid_plan(&centroid_plan, &[11, 12]).unwrap();

        assert_eq!(route_map.get(0).unwrap().pid, 11);
        assert_eq!(route_map.get(1).unwrap().pid, 12);
        assert_eq!(route_map.route_pid_for_vector(&[1.0, 0.0]).unwrap(), 11);
        assert_eq!(route_map.route_pid_for_vector(&[-1.0, 0.0]).unwrap(), 12);
        assert!(route_map.route_pid_for_vector(&[1.0]).is_err());
    }

    #[test]
    fn build_state_collects_assignments_and_training_sample() {
        let mut state = SpireBuildState::new(options(1), SpireIndexedVectorKind::Ecvector);

        state.try_push(build_tuple(1, vec![1.0, 0.0])).unwrap();
        state.try_push(build_tuple(2, vec![0.0, 1.0])).unwrap();

        assert_eq!(state.scanned_tuples, 2);
        assert_eq!(state.dimensions, Some(2));
        assert_eq!(state.tuples.len(), 2);
        assert_eq!(state.training_sample_count(), 1);
        assert_eq!(state.training_sample_vectors().len(), 1);
        assert_eq!(resolve_training_sample_count(0, 12_000), 10_000);
    }

    #[test]
    fn build_state_trains_centroid_plan_for_all_collected_rows() {
        let mut state = SpireBuildState::new(options(1), SpireIndexedVectorKind::Ecvector);
        state.try_push(build_tuple(1, vec![1.0, 0.0])).unwrap();
        state.try_push(build_tuple(2, vec![0.0, 1.0])).unwrap();
        state.try_push(build_tuple(3, vec![-1.0, 0.0])).unwrap();

        let plan = state.train_centroid_plan().unwrap();

        assert_eq!(plan.dimensions, 2);
        assert_eq!(plan.centroid_count(), 2);
        assert_eq!(plan.assignment_indexes.len(), 3);
    }

    #[test]
    fn build_state_rejects_invalid_tuple_without_advancing() {
        let mut state = SpireBuildState::new(options(0), SpireIndexedVectorKind::Ecvector);
        state.try_push(build_tuple(1, vec![1.0, 0.0])).unwrap();
        let mut bad = build_tuple(2, vec![0.0, 1.0]);
        bad.dimensions = 3;

        let error = state.try_push(bad).unwrap_err();

        assert!(error.contains("source dimensions mismatch"));
        assert_eq!(state.scanned_tuples, 1);
        assert_eq!(state.tuples.len(), 1);
    }

    #[test]
    fn build_state_rejects_payload_format_mismatch() {
        let mut state = SpireBuildState::new(options(0), SpireIndexedVectorKind::Ecvector);
        let mut bad = build_tuple(1, vec![1.0, 0.0]);
        bad.assignment.payload_format = SpireAssignmentPayloadFormat::RaBitQ.tag();

        let error = state.try_push(bad).unwrap_err();

        assert!(error.contains("payload format"));
        assert_eq!(state.scanned_tuples, 0);
    }

    #[test]
    fn build_state_rejects_zero_vectors() {
        let mut state = SpireBuildState::new(options(0), SpireIndexedVectorKind::Ecvector);
        let mut bad = build_tuple(1, vec![1.0, 0.0]);
        bad.source_vector = vec![0.0, 0.0];

        let error = state.try_push(bad).unwrap_err();

        assert!(error.contains("non-zero"));
        assert_eq!(state.scanned_tuples, 0);
    }

    #[test]
    fn single_level_route_map_rejects_pid_count_mismatch() {
        let centroid_plan = SpireSingleLevelCentroidPlan {
            dimensions: 2,
            centroids: vec![vec![1.0, 0.0], vec![-1.0, 0.0]],
            assignment_indexes: Vec::new(),
        };

        assert!(SpireSingleLevelRouteMap::from_centroid_plan(&centroid_plan, &[11]).is_err());
    }

