#[cfg(test)]
mod tests {
    use super::{
        normalize_local_store_tablespaces_reloption, parse_nprobe_per_level_reloption,
        plan_local_store_tablespaces_with_resolver, resolve_recursive_route_budget,
        resolve_scan_max_candidate_rows_values, resolve_scan_nprobe_values,
        resolve_scan_rerank_width_values, resolve_single_level_scan_plan_values,
        resolve_single_level_scan_plan_values_with_candidate_budget,
        validate_boundary_replica_count_value, validate_local_store_count_value,
        validate_max_candidate_rows_value, validate_recursive_fanout_value, EcSpireOptions,
        SpireCandidateDedupeMode, SpireRecursiveRouteBudget, SpireSourceIdentityProvider,
        SpireStorageFormat, SpireTopGraphOptionPlan, EC_SPIRE_MAX_MAX_CANDIDATE_ROWS,
    };
    use crate::am::ec_spire::quantizer::SpireAssignmentPayloadFormat;

    #[test]
    fn storage_format_reloption_parses_and_maps_to_assignment_payload_format() {
        assert_eq!(
            SpireStorageFormat::parse_reloption("auto").unwrap(),
            SpireStorageFormat::Auto
        );
        assert_eq!(
            SpireStorageFormat::parse_reloption("turboquant").unwrap(),
            SpireStorageFormat::TurboQuant
        );
        assert_eq!(
            SpireStorageFormat::parse_reloption("pq_fastscan").unwrap(),
            SpireStorageFormat::PqFastScan
        );
        assert_eq!(
            SpireStorageFormat::parse_reloption("rabitq").unwrap(),
            SpireStorageFormat::RaBitQ
        );
        assert!(SpireStorageFormat::parse_reloption("bad").is_err());

        assert_eq!(
            SpireStorageFormat::Auto.assignment_payload_format(),
            SpireAssignmentPayloadFormat::TurboQuant
        );
        assert_eq!(
            SpireStorageFormat::RaBitQ.assignment_payload_format(),
            SpireAssignmentPayloadFormat::RaBitQ
        );
    }

    #[test]
    fn source_identity_reloption_parses_provider() {
        assert_eq!(
            SpireSourceIdentityProvider::parse_reloption("include").unwrap(),
            SpireSourceIdentityProvider::Include
        );
        assert_eq!(
            SpireSourceIdentityProvider::Include.reloption_name(),
            "include"
        );
        assert!(SpireSourceIdentityProvider::parse_reloption("uuid").is_err());
    }

    #[test]
    fn recursive_fanout_validation_rejects_one() {
        assert!(validate_recursive_fanout_value(0).is_ok());
        assert!(validate_recursive_fanout_value(2).is_ok());
        assert!(validate_recursive_fanout_value(32).is_ok());
        assert!(validate_recursive_fanout_value(1).is_err());
    }

    #[test]
    fn local_store_count_validation_bounds_phase4_surface() {
        assert!(validate_local_store_count_value(1).is_ok());
        assert!(validate_local_store_count_value(16).is_ok());
        assert!(validate_local_store_count_value(0).is_err());
        assert!(validate_local_store_count_value(17).is_err());
    }

    #[test]
    fn boundary_replica_count_validation_bounds_phase5_surface() {
        assert!(validate_boundary_replica_count_value(0).is_ok());
        assert!(validate_boundary_replica_count_value(8).is_ok());
        assert!(validate_boundary_replica_count_value(-1).is_err());
        assert!(validate_boundary_replica_count_value(9).is_err());
    }

    #[test]
    fn local_store_tablespaces_normalizes_and_allows_repeated_names() {
        assert_eq!(
            normalize_local_store_tablespaces_reloption("fast_a, fast_a", 2).unwrap(),
            "fast_a,fast_a"
        );
        assert_eq!(
            normalize_local_store_tablespaces_reloption("fast_a", 1).unwrap(),
            "fast_a"
        );
        assert!(normalize_local_store_tablespaces_reloption("fast_a", 2).is_err());
        assert!(normalize_local_store_tablespaces_reloption("fast_a,", 2).is_err());
    }

    #[test]
    fn local_store_tablespace_plan_resolves_names_and_repeats() {
        let plan = plan_local_store_tablespaces_with_resolver(
            3,
            999,
            Some("fast_a,fast_a,fast_b"),
            |name| match name {
                "fast_a" => Ok(10),
                "fast_b" => Ok(11),
                other => Err(format!("unknown tablespace {other}")),
            },
        )
        .unwrap();

        assert_eq!(plan.len(), 3);
        assert_eq!(plan[0].local_store_id, 0);
        assert_eq!(plan[0].tablespace_oid, 10);
        assert_eq!(plan[1].local_store_id, 1);
        assert_eq!(plan[1].tablespace_oid, 10);
        assert_eq!(plan[2].local_store_id, 2);
        assert_eq!(plan[2].tablespace_oid, 11);
    }

    #[test]
    fn local_store_tablespace_plan_inherits_index_tablespace_by_default() {
        let plan =
            plan_local_store_tablespaces_with_resolver(2, 999, None, |_| unreachable!()).unwrap();

        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0].local_store_id, 0);
        assert_eq!(plan[0].tablespace_oid, 999);
        assert_eq!(plan[1].local_store_id, 1);
        assert_eq!(plan[1].tablespace_oid, 999);
    }

    #[test]
    fn local_store_tablespace_plan_rejects_unknown_or_mismatched_names() {
        assert!(
            plan_local_store_tablespaces_with_resolver(2, 999, Some("fast_a"), |_| Ok(10)).is_err()
        );
        assert!(
            plan_local_store_tablespaces_with_resolver(1, 999, Some("missing"), |name| Err(
                format!("unknown tablespace {name}")
            ),)
            .is_err()
        );
    }

    #[test]
    fn scan_nprobe_resolution_uses_session_relation_and_auto_sources() {
        assert_eq!(resolve_scan_nprobe_values(0, 5, -1).effective_nprobe, 0);

        let auto = resolve_scan_nprobe_values(17, 0, -1);
        assert_eq!(auto.effective_nprobe, 5);
        assert_eq!(auto.source, "auto");

        let relation = resolve_scan_nprobe_values(17, 3, -1);
        assert_eq!(relation.effective_nprobe, 3);
        assert_eq!(relation.source, "relation");

        let session = resolve_scan_nprobe_values(17, 3, 99);
        assert_eq!(session.session_nprobe, Some(99));
        assert_eq!(session.effective_nprobe, 17);
        assert_eq!(session.source, "session");
    }

    #[test]
    fn nprobe_per_level_reloption_parses_upper_level_values() {
        assert_eq!(
            parse_nprobe_per_level_reloption("2, 3").unwrap(),
            vec![2, 3]
        );
        assert!(parse_nprobe_per_level_reloption("0").is_err());
        assert!(parse_nprobe_per_level_reloption("2,").is_err());
        assert!(parse_nprobe_per_level_reloption("bad").is_err());
        assert!(parse_nprobe_per_level_reloption(&["1"; 33].join(",")).is_err());
    }

    #[test]
    fn scan_rerank_width_resolution_uses_session_or_relation() {
        let relation = resolve_scan_rerank_width_values(128, -1);
        assert_eq!(relation.effective_rerank_width, 128);
        assert_eq!(relation.source, "relation");

        let session = resolve_scan_rerank_width_values(128, 0);
        assert_eq!(session.session_rerank_width, Some(0));
        assert_eq!(session.effective_rerank_width, 0);
        assert_eq!(session.source, "session");
    }

    #[test]
    fn scan_max_candidate_rows_resolution_uses_session_relation_and_auto_sources() {
        let auto = resolve_scan_max_candidate_rows_values(0, -1);
        assert_eq!(auto.effective_max_candidate_rows, 10_000_000);
        assert_eq!(auto.source, "auto");

        let relation = resolve_scan_max_candidate_rows_values(128, -1);
        assert_eq!(relation.effective_max_candidate_rows, 128);
        assert_eq!(relation.source, "relation");

        let session = resolve_scan_max_candidate_rows_values(128, 7);
        assert_eq!(session.session_max_candidate_rows, Some(7));
        assert_eq!(session.effective_max_candidate_rows, 7);
        assert_eq!(session.source, "session");
    }

    #[test]
    fn default_options_match_phase1_config_contract() {
        let options = EcSpireOptions::DEFAULT;

        assert_eq!(options.nlists, 0);
        assert_eq!(options.recursive_fanout, 0);
        assert_eq!(options.recursive_fanout(), None);
        assert_eq!(options.local_store_count, 1);
        assert_eq!(options.boundary_replica_count, 0);
        assert_eq!(options.nprobe, 0);
        assert_eq!(options.rerank_width, 0);
        assert_eq!(options.max_candidate_rows, 0);
        assert_eq!(options.training_sample_rows, 0);
        assert_eq!(options.seed, 42);
        assert_eq!(options.requested_pq_group_size(), None);
        assert_eq!(
            options.top_graph_plan().unwrap(),
            SpireTopGraphOptionPlan {
                enabled: false,
                graph_degree: 32,
                build_list_size: 100,
                alpha: 1.2,
                search_list_size: None,
            }
        );
        assert_eq!(options.storage_format, SpireStorageFormat::Auto);
        assert_eq!(options.source_identity, SpireSourceIdentityProvider::None);
        assert_eq!(options.nprobe_per_level, None);
        assert_eq!(options.local_store_tablespaces, None);
        assert_eq!(
            options.assignment_payload_format(),
            SpireAssignmentPayloadFormat::TurboQuant
        );
    }

    #[test]
    fn single_level_scan_plan_resolves_runtime_knobs() {
        let options = EcSpireOptions {
            nlists: 17,
            recursive_fanout: 4,
            local_store_count: 1,
            boundary_replica_count: 0,
            nprobe: 3,
            rerank_width: 128,
            max_candidate_rows: 0,
            training_sample_rows: 1000,
            seed: 7,
            pq_group_size: 0,
            top_graph_enabled: 0,
            top_graph_degree: 32,
            top_graph_build_list_size: 100,
            top_graph_alpha: 1.2,
            top_graph_search_list_size: 0,
            nprobe_per_level: None,
            storage_format: SpireStorageFormat::RaBitQ,
            source_identity: SpireSourceIdentityProvider::None,
            local_store_tablespaces: Some("fast_a".to_owned()),
        };

        let plan = resolve_single_level_scan_plan_values(17, options.clone(), -1, -1).unwrap();

        assert_eq!(plan.leaf_count, 17);
        assert_eq!(plan.nprobe, 3);
        assert_eq!(plan.nprobe_source, "relation");
        assert_eq!(plan.payload_format, SpireAssignmentPayloadFormat::RaBitQ);
        assert_eq!(plan.rerank_width, 128);
        assert_eq!(plan.rerank_width_source, "relation");
        assert_eq!(plan.candidate_limit, Some(128));
        assert_eq!(
            plan.recursive_route_budget,
            SpireRecursiveRouteBudget {
                beam_width: 3,
                max_leaf_routes: 3,
                max_routing_expansions: 17,
            }
        );
        assert_eq!(
            plan.dedupe_mode,
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled
        );
        assert_eq!(options.recursive_fanout(), Some(4));
    }

    #[test]
    fn single_level_scan_plan_carries_recursive_per_level_nprobe_policy() {
        let options = EcSpireOptions {
            nprobe: 2,
            nprobe_per_level: Some(vec![3, 4]),
            ..EcSpireOptions::DEFAULT
        };

        let plan = resolve_single_level_scan_plan_values(17, options, -1, -1).unwrap();

        assert_eq!(plan.nprobe, 2);
        assert_eq!(plan.recursive_nprobe_policy.nprobe_for_parent_level(1), 2);
        assert_eq!(plan.recursive_nprobe_policy.nprobe_for_parent_level(2), 3);
        assert_eq!(plan.recursive_nprobe_policy.nprobe_for_parent_level(3), 4);
        assert_eq!(plan.recursive_nprobe_policy.nprobe_for_parent_level(4), 1);
        assert_eq!(plan.recursive_route_budget.beam_width, 2);
        assert_eq!(plan.recursive_route_budget.max_leaf_routes, 2);
        assert_eq!(plan.recursive_route_budget.max_routing_expansions, 17);
    }

    #[test]
    fn recursive_route_budget_resolves_finite_scan_guardrails() {
        assert_eq!(
            resolve_recursive_route_budget(100, 7).unwrap(),
            SpireRecursiveRouteBudget {
                beam_width: 7,
                max_leaf_routes: 7,
                max_routing_expansions: 100,
            }
        );
        assert_eq!(
            resolve_recursive_route_budget(3, 7).unwrap(),
            SpireRecursiveRouteBudget {
                beam_width: 7,
                max_leaf_routes: 3,
                max_routing_expansions: 7,
            }
        );
        assert_eq!(
            resolve_recursive_route_budget(0, 7).unwrap(),
            SpireRecursiveRouteBudget {
                beam_width: 0,
                max_leaf_routes: 0,
                max_routing_expansions: 0,
            }
        );
    }

    #[test]
    fn single_level_scan_plan_uses_session_overrides_and_full_rerank() {
        let options = EcSpireOptions {
            nlists: 17,
            recursive_fanout: 0,
            local_store_count: 1,
            boundary_replica_count: 0,
            nprobe: 0,
            rerank_width: 128,
            max_candidate_rows: 0,
            training_sample_rows: 0,
            seed: 42,
            pq_group_size: 0,
            top_graph_enabled: 0,
            top_graph_degree: 32,
            top_graph_build_list_size: 100,
            top_graph_alpha: 1.2,
            top_graph_search_list_size: 0,
            nprobe_per_level: None,
            storage_format: SpireStorageFormat::Auto,
            source_identity: SpireSourceIdentityProvider::None,
            local_store_tablespaces: None,
        };

        let plan = resolve_single_level_scan_plan_values(17, options, 99, 0).unwrap();

        assert_eq!(plan.nprobe, 17);
        assert_eq!(plan.nprobe_source, "session");
        assert_eq!(
            plan.payload_format,
            SpireAssignmentPayloadFormat::TurboQuant
        );
        assert_eq!(plan.rerank_width, 0);
        assert_eq!(plan.rerank_width_source, "session");
        assert_eq!(plan.candidate_limit, Some(10_000_000));
        assert_eq!(
            plan.dedupe_mode,
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled
        );
    }

    #[test]
    fn single_level_scan_plan_applies_hard_candidate_budget_to_full_rerank() {
        let options = EcSpireOptions {
            nlists: 17,
            nprobe: 0,
            rerank_width: 0,
            max_candidate_rows: 3,
            ..EcSpireOptions::DEFAULT
        };

        let plan = resolve_single_level_scan_plan_values(17, options, -1, -1).unwrap();

        assert_eq!(plan.rerank_width, 0);
        assert_eq!(plan.candidate_limit, Some(3));

        let options = EcSpireOptions {
            nlists: 17,
            nprobe: 0,
            rerank_width: 128,
            max_candidate_rows: 5,
            ..EcSpireOptions::DEFAULT
        };

        let plan =
            resolve_single_level_scan_plan_values_with_candidate_budget(17, options, -1, -1, 4)
                .unwrap();

        assert_eq!(plan.rerank_width, 128);
        assert_eq!(plan.candidate_limit, Some(4));
    }

    #[test]
    fn single_level_scan_plan_rejects_invalid_manual_options() {
        let invalid = EcSpireOptions {
            nlists: 0,
            recursive_fanout: 0,
            local_store_count: 1,
            boundary_replica_count: 0,
            nprobe: -1,
            rerank_width: 0,
            max_candidate_rows: 0,
            training_sample_rows: 0,
            seed: 42,
            pq_group_size: 0,
            top_graph_enabled: 0,
            top_graph_degree: 32,
            top_graph_build_list_size: 100,
            top_graph_alpha: 1.2,
            top_graph_search_list_size: 0,
            nprobe_per_level: None,
            storage_format: SpireStorageFormat::Auto,
            source_identity: SpireSourceIdentityProvider::None,
            local_store_tablespaces: None,
        };
        assert!(resolve_single_level_scan_plan_values(1, invalid.clone(), -1, -1).is_err());

        let invalid = EcSpireOptions {
            nprobe: 0,
            rerank_width: -1,
            ..invalid
        };
        assert!(resolve_single_level_scan_plan_values(1, invalid, -1, -1).is_err());

        let invalid = EcSpireOptions {
            max_candidate_rows: -1,
            ..EcSpireOptions::DEFAULT
        };
        assert!(resolve_single_level_scan_plan_values(1, invalid, -1, -1).is_err());
        assert!(validate_max_candidate_rows_value(EC_SPIRE_MAX_MAX_CANDIDATE_ROWS + 1).is_err());
    }

    #[test]
    fn single_level_scan_plan_enables_vec_id_dedupe_for_replica_capable_indexes() {
        let options = EcSpireOptions {
            nlists: 17,
            recursive_fanout: 0,
            local_store_count: 1,
            boundary_replica_count: 1,
            nprobe: 3,
            rerank_width: 128,
            max_candidate_rows: 0,
            training_sample_rows: 0,
            seed: 42,
            pq_group_size: 0,
            top_graph_enabled: 0,
            top_graph_degree: 32,
            top_graph_build_list_size: 100,
            top_graph_alpha: 1.2,
            top_graph_search_list_size: 0,
            nprobe_per_level: None,
            storage_format: SpireStorageFormat::Auto,
            source_identity: SpireSourceIdentityProvider::None,
            local_store_tablespaces: None,
        };

        let plan = resolve_single_level_scan_plan_values(17, options, -1, -1).unwrap();

        assert_eq!(
            plan.dedupe_mode,
            SpireCandidateDedupeMode::VecIdDedupeEnabled
        );
    }

    #[test]
    fn top_graph_option_plan_resolves_enabled_params_and_auto_search_list() {
        let options = EcSpireOptions {
            top_graph_enabled: 1,
            top_graph_degree: 64,
            top_graph_build_list_size: 200,
            top_graph_alpha: 1.4,
            top_graph_search_list_size: 0,
            ..EcSpireOptions::DEFAULT
        };

        assert_eq!(
            options.top_graph_plan().unwrap(),
            SpireTopGraphOptionPlan {
                enabled: true,
                graph_degree: 64,
                build_list_size: 200,
                alpha: 1.4,
                search_list_size: None,
            }
        );

        let explicit_search = EcSpireOptions {
            top_graph_search_list_size: 37,
            ..options
        };
        assert_eq!(
            explicit_search.top_graph_plan().unwrap().search_list_size,
            Some(37)
        );
    }

    #[test]
    fn top_graph_option_plan_rejects_invalid_values() {
        let invalid_enabled = EcSpireOptions {
            top_graph_enabled: 2,
            ..EcSpireOptions::DEFAULT
        };
        assert!(invalid_enabled.top_graph_plan().is_err());

        let invalid_degree = EcSpireOptions {
            top_graph_degree: 0,
            ..EcSpireOptions::DEFAULT
        };
        assert!(invalid_degree.top_graph_plan().is_err());

        let invalid_alpha = EcSpireOptions {
            top_graph_alpha: f32::NAN,
            ..EcSpireOptions::DEFAULT
        };
        assert!(invalid_alpha.top_graph_plan().is_err());
    }
}
