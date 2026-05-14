#[cfg(test)]
mod tests {
    use super::*;

    fn inputs(
        effective_nprobe: u32,
        hierarchy_depth: u16,
        local_store_count: u32,
    ) -> SpireCostInputs {
        SpireCostInputs {
            dimensions: 1536,
            nlists: 128,
            relation_nprobe: effective_nprobe,
            session_nprobe: None,
            effective_nprobe,
            effective_nprobe_source: "test",
            local_store_count,
            recursive_fanout: Some(16),
            relation_rerank_width: 100,
            session_rerank_width: None,
            effective_rerank_width: 100,
            effective_rerank_width_source: "test",
            storage_format: options::SpireStorageFormat::TurboQuant,
            hierarchy_depth,
            routing_object_count: if hierarchy_depth > 1 { 8 } else { 1 },
            routing_child_count: 4_096,
            root_child_count: 128,
            leaf_object_count: 128,
            leaf_assignment_count: 12_800,
            routing_object_bytes: 8 * pg_sys::BLCKSZ as u64,
            leaf_object_bytes: 512 * pg_sys::BLCKSZ as u64,
            index_pages: 600.0,
            reltuples: 12_800.0,
        }
    }

    fn default_constants() -> PlannerCostConstants {
        PlannerCostConstants {
            random_page_cost: 4.0,
            seq_page_cost: 1.0,
            cpu_operator_cost: 0.0025,
        }
    }

    fn assert_linear_total_cost_scaling(
        label: &str,
        baseline: PlannerCostEstimate,
        doubled: PlannerCostEstimate,
        tripled: PlannerCostEstimate,
    ) {
        let double_delta = doubled.total_cost - baseline.total_cost;
        let triple_delta = tripled.total_cost - baseline.total_cost;

        assert!(double_delta > 0.0, "{label} should increase total cost");
        assert!(
            (triple_delta - (2.0 * double_delta)).abs() < 1e-8,
            "{label} should scale linearly: baseline={baseline:?} doubled={doubled:?} tripled={tripled:?}"
        );
    }

    #[test]
    fn cost_increases_with_effective_nprobe() {
        let low = estimate_spire_cost(&inputs(4, 2, 1), default_constants());
        let high = estimate_spire_cost(&inputs(32, 2, 1), default_constants());

        assert!(low.total_cost.is_finite());
        assert!(high.total_cost > low.total_cost);
    }

    #[test]
    fn startup_cost_increases_with_recursive_depth() {
        let shallow = estimate_spire_cost(&inputs(8, 1, 1), default_constants());
        let deep = estimate_spire_cost(&inputs(8, 4, 1), default_constants());

        assert!(deep.startup_cost > shallow.startup_cost);
    }

    #[test]
    fn local_store_count_affects_page_and_coordination_cost() {
        let single_store = estimate_spire_cost(&inputs(8, 2, 1), default_constants());
        let multi_store = estimate_spire_cost(&inputs(8, 2, 4), default_constants());

        assert!(multi_store.total_cost > single_store.total_cost);
    }

    #[test]
    fn packet_30976_default_tuning_preserves_legacy_modeled_costs() {
        let estimate = estimate_spire_cost_with_tuning(
            &inputs(8, 2, 4),
            default_constants(),
            SpireCostModelTuning::packet_30976_defaults(),
        );

        assert!((estimate.startup_cost - 23.0404).abs() < f64::EPSILON);
        assert!((estimate.total_cost - 90.5604).abs() < f64::EPSILON);
    }

    #[test]
    fn non_default_tuning_changes_modeled_costs() {
        let baseline = estimate_spire_cost_with_tuning(
            &inputs(8, 2, 1),
            default_constants(),
            SpireCostModelTuning::packet_30976_defaults(),
        );
        let tuned = estimate_spire_cost_with_tuning(
            &inputs(8, 2, 1),
            default_constants(),
            SpireCostModelTuning {
                routing_dimension_scale: 0.02,
                leaf_dimension_scale: 0.02,
                index_page_scale: 2.0,
                local_store_page_fanout_scale: 0.10,
                storage_scoring_multiplier: 2.0,
                rerank_multiplier: 2.70,
            },
        );

        assert!(tuned.startup_cost > baseline.startup_cost);
        assert!(tuned.total_cost > baseline.total_cost);
    }

    #[test]
    fn individual_cost_tuning_knobs_scale_modeled_costs_linearly() {
        let constants = default_constants();
        let defaults = SpireCostModelTuning::packet_30976_defaults();
        let multistore_inputs = inputs(8, 2, 4);
        let mut rerank_inputs = inputs(8, 2, 1);
        rerank_inputs.relation_rerank_width = 0;
        rerank_inputs.effective_rerank_width = 0;

        let baseline = estimate_spire_cost_with_tuning(&multistore_inputs, constants, defaults);
        assert_linear_total_cost_scaling(
            "routing dimension scale",
            baseline,
            estimate_spire_cost_with_tuning(
                &multistore_inputs,
                constants,
                SpireCostModelTuning {
                    routing_dimension_scale: defaults.routing_dimension_scale * 2.0,
                    ..defaults
                },
            ),
            estimate_spire_cost_with_tuning(
                &multistore_inputs,
                constants,
                SpireCostModelTuning {
                    routing_dimension_scale: defaults.routing_dimension_scale * 3.0,
                    ..defaults
                },
            ),
        );
        assert_linear_total_cost_scaling(
            "leaf dimension scale",
            baseline,
            estimate_spire_cost_with_tuning(
                &multistore_inputs,
                constants,
                SpireCostModelTuning {
                    leaf_dimension_scale: defaults.leaf_dimension_scale * 2.0,
                    ..defaults
                },
            ),
            estimate_spire_cost_with_tuning(
                &multistore_inputs,
                constants,
                SpireCostModelTuning {
                    leaf_dimension_scale: defaults.leaf_dimension_scale * 3.0,
                    ..defaults
                },
            ),
        );
        assert_linear_total_cost_scaling(
            "index page scale",
            baseline,
            estimate_spire_cost_with_tuning(
                &multistore_inputs,
                constants,
                SpireCostModelTuning {
                    index_page_scale: defaults.index_page_scale * 2.0,
                    ..defaults
                },
            ),
            estimate_spire_cost_with_tuning(
                &multistore_inputs,
                constants,
                SpireCostModelTuning {
                    index_page_scale: defaults.index_page_scale * 3.0,
                    ..defaults
                },
            ),
        );
        assert_linear_total_cost_scaling(
            "local store page fanout scale",
            baseline,
            estimate_spire_cost_with_tuning(
                &multistore_inputs,
                constants,
                SpireCostModelTuning {
                    local_store_page_fanout_scale: defaults.local_store_page_fanout_scale * 2.0,
                    ..defaults
                },
            ),
            estimate_spire_cost_with_tuning(
                &multistore_inputs,
                constants,
                SpireCostModelTuning {
                    local_store_page_fanout_scale: defaults.local_store_page_fanout_scale * 3.0,
                    ..defaults
                },
            ),
        );
        assert_linear_total_cost_scaling(
            "storage scoring multiplier",
            baseline,
            estimate_spire_cost_with_tuning(
                &multistore_inputs,
                constants,
                SpireCostModelTuning {
                    storage_scoring_multiplier: defaults.storage_scoring_multiplier * 2.0,
                    ..defaults
                },
            ),
            estimate_spire_cost_with_tuning(
                &multistore_inputs,
                constants,
                SpireCostModelTuning {
                    storage_scoring_multiplier: defaults.storage_scoring_multiplier * 3.0,
                    ..defaults
                },
            ),
        );

        let rerank_baseline = estimate_spire_cost_with_tuning(&rerank_inputs, constants, defaults);
        assert_linear_total_cost_scaling(
            "rerank multiplier",
            rerank_baseline,
            estimate_spire_cost_with_tuning(
                &rerank_inputs,
                constants,
                SpireCostModelTuning {
                    rerank_multiplier: defaults.rerank_multiplier * 2.0,
                    ..defaults
                },
            ),
            estimate_spire_cost_with_tuning(
                &rerank_inputs,
                constants,
                SpireCostModelTuning {
                    rerank_multiplier: defaults.rerank_multiplier * 3.0,
                    ..defaults
                },
            ),
        );
    }

    #[test]
    fn storage_scoring_guc_scales_format_baseline() {
        let tuning = SpireCostModelTuning {
            storage_scoring_multiplier: 2.0,
            ..SpireCostModelTuning::packet_30976_defaults()
        };

        assert_eq!(
            effective_storage_scoring_multiplier(options::SpireStorageFormat::RaBitQ, tuning),
            0.90
        );
        assert_eq!(
            effective_storage_scoring_multiplier(options::SpireStorageFormat::TurboQuant, tuning),
            2.0
        );
    }
}
