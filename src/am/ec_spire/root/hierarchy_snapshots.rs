pub(crate) unsafe fn index_insert_debt_snapshot(
    index_relation: pg_sys::Relation,
) -> SpireIndexInsertDebtSnapshot {
    let root_control = unsafe { page::read_root_control_page(index_relation) };
    let leaf_rows = unsafe { index_leaf_snapshot(index_relation) };
    let active_leaf_count = u64::try_from(leaf_rows.len())
        .unwrap_or_else(|_| pgrx::error!("ec_spire leaf row count exceeds u64"));
    let leaf_count_with_deltas = leaf_rows
        .iter()
        .filter(|row| row.delta_object_count > 0)
        .count()
        .try_into()
        .unwrap_or_else(|_| pgrx::error!("ec_spire leaf delta row count exceeds u64"));
    let delta_object_count = leaf_rows
        .iter()
        .map(|row| row.delta_object_count)
        .sum::<u64>();
    let delta_insert_assignment_count = leaf_rows
        .iter()
        .map(|row| row.delta_insert_assignment_count)
        .sum::<u64>();
    let max_delta_objects_per_leaf = leaf_rows
        .iter()
        .map(|row| row.delta_object_count)
        .max()
        .unwrap_or(0);
    let batching_recommended =
        max_delta_objects_per_leaf > 1 || delta_object_count > active_leaf_count;
    let recommendation = if batching_recommended {
        "batch post-build inserts by routed base leaf before publishing replacement epochs"
    } else {
        "none"
    };

    SpireIndexInsertDebtSnapshot {
        active_epoch: root_control.active_epoch,
        active_leaf_count,
        leaf_count_with_deltas,
        delta_object_count,
        delta_insert_assignment_count,
        max_delta_objects_per_leaf,
        insert_batching_supported: false,
        batching_recommended,
        recommendation,
    }
}

fn empty_top_graph_snapshot(
    active_epoch: u64,
    top_graph_plan: options::SpireTopGraphOptionPlan,
    status: &'static str,
    recommendation: &'static str,
) -> SpireIndexTopGraphSnapshot {
    SpireIndexTopGraphSnapshot {
        active_epoch,
        top_graph_enabled: top_graph_plan.enabled,
        top_graph_count: 0,
        top_graph_pid: 0,
        root_pid: 0,
        object_version: 0,
        published_epoch_backref: 0,
        level: 0,
        node_count: 0,
        dimensions: 0,
        graph_degree: top_graph_plan.graph_degree,
        build_list_size: top_graph_plan.build_list_size,
        alpha: top_graph_plan.alpha,
        entry_node: 0,
        edge_count: 0,
        max_node_degree: 0,
        effective_route_count: 0,
        effective_search_list_size: top_graph_plan.search_list_size.unwrap_or(0),
        configured_search_list_size: top_graph_plan.search_list_size,
        object_bytes: 0,
        status,
        recommendation,
    }
}

fn parse_remote_search_consistency_mode(input: &str) -> Result<meta::SpireConsistencyMode, String> {
    match input {
        "strict" => Ok(meta::SpireConsistencyMode::Strict),
        "degraded" => Ok(meta::SpireConsistencyMode::Degraded),
        other => Err(format!(
            "ec_spire remote search consistency_mode must be 'strict' or 'degraded', got '{other}'"
        )),
    }
}

fn remote_search_row_locator(heap_tid: crate::storage::page::ItemPointer) -> Vec<u8> {
    let mut row_locator = Vec::with_capacity(crate::storage::page::ITEM_POINTER_BYTES);
    heap_tid.encode_into(&mut row_locator);
    row_locator
}

pub(crate) unsafe fn remote_search_candidates(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchCandidateRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchCandidateRow>, String> {
        if requested_epoch == 0 {
            return Err("ec_spire remote search requested_epoch must be greater than 0".to_owned());
        }
        if top_k == 0 {
            return Ok(Vec::new());
        }

        let requested_consistency_mode = parse_remote_search_consistency_mode(consistency_mode)?;
        let query = scan::SpireScanQuery::new(query)?;
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch != requested_epoch {
            return Err(format!(
                "ec_spire remote search requested epoch {requested_epoch} does not match active epoch {}",
                root_control.active_epoch
            ));
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        if epoch_manifest.consistency_mode != requested_consistency_mode {
            return Err(format!(
                "ec_spire remote search requested consistency_mode '{consistency_mode}' does not match active epoch consistency mode '{}'",
                consistency_mode_name(epoch_manifest.consistency_mode)
            ));
        }
        let snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store = unsafe {
            storage::SpireRelationObjectStoreSet::for_index_relation_and_placements(
                index_relation,
                &placement_directory,
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            )?
        };
        let relation_options = unsafe { options::relation_options(index_relation) };
        let candidates = scan::collect_quantized_selected_leaf_candidates(
            &snapshot,
            &object_store,
            query.values(),
            &selected_pids,
            relation_options.assignment_payload_format(),
            if relation_options.boundary_replica_count > 0 {
                options::SpireCandidateDedupeMode::VecIdDedupeEnabled
            } else {
                options::SpireCandidateDedupeMode::NoReplicaDedupeDisabled
            },
            Some(top_k),
        )?;

        Ok(candidates
            .into_iter()
            .map(|candidate| SpireRemoteSearchCandidateRow {
                served_epoch: candidate.epoch,
                node_id: meta::SPIRE_LOCAL_NODE_ID,
                pid: candidate.pid,
                object_version: candidate.object_version,
                row_index: candidate.row_index,
                assignment_flags: candidate.assignment_flags,
                vec_id: candidate.vec_id.as_bytes().to_vec(),
                row_locator: remote_search_row_locator(candidate.heap_tid),
                score: candidate.score,
            })
            .collect())
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_top_graph_snapshot(
    index_relation: pg_sys::Relation,
) -> SpireIndexTopGraphSnapshot {
    let result = (|| -> Result<SpireIndexTopGraphSnapshot, String> {
        let relation_options = unsafe { options::relation_options(index_relation) };
        let top_graph_plan = relation_options.top_graph_plan()?;
        let root_control = unsafe { page::read_root_control_page(index_relation) };

        if root_control.active_epoch == 0 {
            return Ok(empty_top_graph_snapshot(
                root_control.active_epoch,
                top_graph_plan,
                "empty",
                "populate the index before expecting a published SPIRE top graph",
            ));
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store = unsafe {
            storage::SpireRelationObjectStoreSet::for_index_relation_and_placements(
                index_relation,
                &placement_directory,
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            )?
        };
        let active_leaf_count = count_snapshot_options_leaf_pids(
            &meta::SpirePublishedEpochSnapshot::new(
                &epoch_manifest,
                &object_manifest,
                &placement_directory,
            )?,
            &object_store,
            relation_options.recursive_fanout().is_some(),
        )?;
        let relation_nprobe = u32::try_from(relation_options.nprobe)
            .map_err(|_| "ec_spire nprobe reloption must be non-negative".to_owned())?;
        let nprobe = options::resolve_scan_nprobe(active_leaf_count, relation_nprobe);
        let effective_search_list_size = top_graph_plan
            .search_list_size
            .unwrap_or(nprobe.effective_nprobe);

        let mut top_graphs = Vec::new();
        for manifest_entry in &snapshot.object_manifest().entries {
            let lookup = snapshot.require_lookup(manifest_entry.pid, "top graph snapshot")?;
            if lookup.placement.state != meta::SpirePlacementState::Available {
                continue;
            }
            let header = storage::SpireObjectReader::read_object_header(
                &object_store,
                lookup.placement,
            )?;
            if header.kind == storage::SpirePartitionObjectKind::TopGraph {
                top_graphs.push((
                    lookup.placement,
                    storage::SpireObjectReader::read_top_graph_object(
                        &object_store,
                        lookup.placement,
                    )?,
                ));
            }
        }

        if top_graphs.is_empty() {
            let (status, recommendation) = if top_graph_plan.enabled {
                (
                    "missing_top_graph",
                    "rebuild the recursive index or disable top_graph_enabled; enabled scans fail closed without a graph object",
                )
            } else {
                ("disabled", "none")
            };
            let mut snapshot = empty_top_graph_snapshot(
                root_control.active_epoch,
                top_graph_plan,
                status,
                recommendation,
            );
            snapshot.effective_route_count = nprobe.effective_nprobe;
            snapshot.effective_search_list_size = effective_search_list_size;
            return Ok(snapshot);
        }

        top_graphs.sort_by_key(|(_, graph)| graph.header.pid);
        let top_graph_count = u64::try_from(top_graphs.len())
            .map_err(|_| "ec_spire top graph snapshot count exceeds u64".to_owned())?;
        let (placement, top_graph) = &top_graphs[0];
        let mut edge_count = 0_u64;
        let mut max_node_degree = 0_u64;
        for node in &top_graph.nodes {
            let node_degree = u64::try_from(node.neighbors.len())
                .map_err(|_| "ec_spire top graph snapshot node degree exceeds u64".to_owned())?;
            edge_count = edge_count
                .checked_add(node_degree)
                .ok_or_else(|| "ec_spire top graph snapshot edge count overflow".to_owned())?;
            max_node_degree = max_node_degree.max(node_degree);
        }
        let (status, recommendation) = if top_graph_count == 1 && top_graph_plan.enabled {
            ("ready", "none")
        } else if top_graph_count == 1 {
            ("available_disabled", "none")
        } else {
            (
                "multiple_top_graphs",
                "repair or rebuild the index; enabled scans fail closed when multiple top graph objects are visible",
            )
        };

        Ok(SpireIndexTopGraphSnapshot {
            active_epoch: root_control.active_epoch,
            top_graph_enabled: top_graph_plan.enabled,
            top_graph_count,
            top_graph_pid: top_graph.header.pid,
            root_pid: top_graph.root_pid,
            object_version: top_graph.header.object_version,
            published_epoch_backref: top_graph.header.published_epoch_backref,
            level: top_graph.header.level,
            node_count: u64::try_from(top_graph.node_count())
                .map_err(|_| "ec_spire top graph snapshot node count exceeds u64".to_owned())?,
            dimensions: top_graph.dimensions,
            graph_degree: top_graph.graph_degree,
            build_list_size: top_graph.build_list_size,
            alpha: top_graph.alpha,
            entry_node: u64::from(top_graph.entry_node),
            edge_count,
            max_node_degree,
            effective_route_count: nprobe.effective_nprobe,
            effective_search_list_size,
            configured_search_list_size: top_graph_plan.search_list_size,
            object_bytes: u64::from(placement.object_bytes),
            status,
            recommendation,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_hierarchy_snapshot(
    index_relation: pg_sys::Relation,
) -> SpireIndexHierarchySnapshot {
    let result = (|| -> Result<SpireIndexHierarchySnapshot, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            let (status, recommendation) = hierarchy_snapshot_status(0, 0, 0, true, false);
            return Ok(SpireIndexHierarchySnapshot {
                active_epoch: 0,
                root_pid: 0,
                root_level: 0,
                max_observed_level: 0,
                hierarchy_depth: 0,
                routing_object_count: 0,
                root_routing_object_count: 0,
                internal_routing_object_count: 0,
                leaf_object_count: 0,
                delta_object_count: 0,
                centroid_dimensions: 0,
                root_child_count: 0,
                distinct_leaf_parent_count: 0,
                recursive_routing_supported: false,
                per_level_nprobe_supported: false,
                status,
                recommendation,
            });
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store = unsafe {
            storage::SpireRelationObjectStoreSet::for_index_relation_and_placements(
                index_relation,
                &placement_directory,
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            )?
        };

        let mut root_pid = 0_u64;
        let mut root_level = 0_u16;
        let mut max_observed_level = 0_u16;
        let mut routing_object_count = 0_u64;
        let mut root_routing_object_count = 0_u64;
        let mut internal_routing_object_count = 0_u64;
        let mut leaf_object_count = 0_u64;
        let mut delta_object_count = 0_u64;
        let mut centroid_dimensions = 0_u16;
        let mut root_child_count = 0_u64;
        let mut leaf_parent_pids = HashSet::new();
        let mut hierarchy_objects = Vec::new();

        for manifest_entry in &snapshot.object_manifest().entries {
            let lookup = snapshot.require_lookup(manifest_entry.pid, "hierarchy snapshot")?;
            let placement = lookup.placement;
            if placement.state != meta::SpirePlacementState::Available {
                continue;
            }
            let header = storage::SpireObjectReader::read_object_header(&object_store, placement)?;
            max_observed_level = max_observed_level.max(header.level);
            match header.kind {
                storage::SpirePartitionObjectKind::Root => {
                    let routing_object =
                        storage::SpireObjectReader::read_routing_object(&object_store, placement)?;
                    routing_object_count =
                        routing_object_count.checked_add(1).ok_or_else(|| {
                            "ec_spire hierarchy snapshot routing object count overflow".to_owned()
                        })?;
                    root_routing_object_count =
                        root_routing_object_count.checked_add(1).ok_or_else(|| {
                            "ec_spire hierarchy snapshot root object count overflow".to_owned()
                        })?;
                    root_pid = header.pid;
                    root_level = header.level;
                    centroid_dimensions = routing_object.dimensions;
                    hierarchy_objects.push(hierarchy_object_summary(
                        &routing_object.header,
                        routing_object.child_pids.clone(),
                    ));
                    root_child_count =
                        u64::try_from(routing_object.child_count()).map_err(|_| {
                            "ec_spire hierarchy snapshot root child count exceeds u64".to_owned()
                        })?;
                }
                storage::SpirePartitionObjectKind::Internal => {
                    routing_object_count =
                        routing_object_count.checked_add(1).ok_or_else(|| {
                            "ec_spire hierarchy snapshot routing object count overflow".to_owned()
                        })?;
                    internal_routing_object_count = internal_routing_object_count
                        .checked_add(1)
                        .ok_or_else(|| {
                            "ec_spire hierarchy snapshot internal object count overflow".to_owned()
                        })?;
                    let routing_object =
                        storage::SpireObjectReader::read_routing_object(&object_store, placement)?;
                    hierarchy_objects.push(hierarchy_object_summary(
                        &routing_object.header,
                        routing_object.child_pids.clone(),
                    ));
                }
                storage::SpirePartitionObjectKind::Leaf => {
                    leaf_object_count = leaf_object_count.checked_add(1).ok_or_else(|| {
                        "ec_spire hierarchy snapshot leaf object count overflow".to_owned()
                    })?;
                    leaf_parent_pids.insert(header.parent_pid);
                    hierarchy_objects.push(hierarchy_object_summary(&header, Vec::new()));
                }
                storage::SpirePartitionObjectKind::Delta => {
                    delta_object_count = delta_object_count.checked_add(1).ok_or_else(|| {
                        "ec_spire hierarchy snapshot delta object count overflow".to_owned()
                    })?;
                    hierarchy_objects.push(hierarchy_object_summary(&header, Vec::new()));
                }
                storage::SpirePartitionObjectKind::TopGraph => {}
            }
        }

        let hierarchy_depth = if root_routing_object_count == 0 {
            0
        } else {
            max_observed_level.max(root_level)
        };
        let hierarchy_shape_valid = validate_recursive_hierarchy_shape(&hierarchy_objects).is_ok();
        let per_level_nprobe_supported = hierarchy_shape_valid && internal_routing_object_count > 0;
        let (status, recommendation) = hierarchy_snapshot_status(
            root_routing_object_count,
            internal_routing_object_count,
            leaf_object_count,
            hierarchy_shape_valid,
            per_level_nprobe_supported,
        );

        Ok(SpireIndexHierarchySnapshot {
            active_epoch: root_control.active_epoch,
            root_pid,
            root_level,
            max_observed_level,
            hierarchy_depth,
            routing_object_count,
            root_routing_object_count,
            internal_routing_object_count,
            leaf_object_count,
            delta_object_count,
            centroid_dimensions,
            root_child_count,
            distinct_leaf_parent_count: u64::try_from(leaf_parent_pids.len()).map_err(|_| {
                "ec_spire hierarchy snapshot leaf parent count exceeds u64".to_owned()
            })?,
            recursive_routing_supported: hierarchy_shape_valid && internal_routing_object_count > 0,
            per_level_nprobe_supported,
            status,
            recommendation,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_object_snapshot(
    index_relation: pg_sys::Relation,
) -> Vec<SpireIndexObjectSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexObjectSnapshotRow>, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store = unsafe {
            storage::SpireRelationObjectStoreSet::for_index_relation_and_placements(
                index_relation,
                &placement_directory,
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            )?
        };
        let mut rows = Vec::with_capacity(snapshot.object_manifest().entries.len());

        for manifest_entry in &snapshot.object_manifest().entries {
            let lookup = snapshot.require_lookup(manifest_entry.pid, "object snapshot")?;
            let placement = lookup.placement;
            let mut row = SpireIndexObjectSnapshotRow {
                active_epoch: root_control.active_epoch,
                pid: manifest_entry.pid,
                object_kind: "unreadable",
                object_version: manifest_entry.object_version,
                published_epoch_backref: 0,
                level: 0,
                parent_pid: 0,
                child_count: 0,
                assignment_count: 0,
                node_id: placement.node_id,
                local_store_id: placement.local_store_id,
                store_relid: placement.store_relid,
                placement_state: placement_state_name(placement.state),
                object_bytes: u64::from(placement.object_bytes),
                object_readable: false,
            };
            if placement.state == meta::SpirePlacementState::Available {
                let header =
                    storage::SpireObjectReader::read_object_header(&object_store, placement)?;
                row.object_kind = partition_object_kind_name(header.kind);
                row.object_version = header.object_version;
                row.published_epoch_backref = header.published_epoch_backref;
                row.level = header.level;
                row.parent_pid = header.parent_pid;
                row.child_count = u64::from(header.child_count);
                row.assignment_count = u64::from(header.assignment_count);
                row.object_readable = true;
            }
            rows.push(row);
        }

        rows.sort_by_key(|row| row.pid);
        Ok(rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_delta_snapshot(
    index_relation: pg_sys::Relation,
) -> Vec<SpireIndexDeltaSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexDeltaSnapshotRow>, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store = unsafe {
            storage::SpireRelationObjectStoreSet::for_index_relation_and_placements(
                index_relation,
                &placement_directory,
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            )?
        };
        let mut rows = Vec::new();

        for manifest_entry in &snapshot.object_manifest().entries {
            let lookup = snapshot.require_lookup(manifest_entry.pid, "delta snapshot")?;
            let placement = lookup.placement;
            if placement.state != meta::SpirePlacementState::Available {
                continue;
            }
            let header = storage::SpireObjectReader::read_object_header(&object_store, placement)?;
            if header.kind != storage::SpirePartitionObjectKind::Delta {
                continue;
            }
            let delta_object =
                storage::SpireObjectReader::read_delta_object(&object_store, placement)?;
            let mut insert_assignment_count = 0_u64;
            let mut delete_assignment_count = 0_u64;
            for assignment in &delta_object.assignments {
                if storage::is_delete_delta_assignment(assignment) {
                    delete_assignment_count =
                        delete_assignment_count.checked_add(1).ok_or_else(|| {
                            "ec_spire delta snapshot delete assignment count overflow".to_owned()
                        })?;
                } else if assignment.flags & storage::SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT != 0 {
                    insert_assignment_count =
                        insert_assignment_count.checked_add(1).ok_or_else(|| {
                            "ec_spire delta snapshot insert assignment count overflow".to_owned()
                        })?;
                }
            }
            rows.push(SpireIndexDeltaSnapshotRow {
                active_epoch: root_control.active_epoch,
                delta_pid: header.pid,
                parent_leaf_pid: header.parent_pid,
                object_version: header.object_version,
                published_epoch_backref: header.published_epoch_backref,
                node_id: placement.node_id,
                local_store_id: placement.local_store_id,
                store_relid: placement.store_relid,
                placement_state: placement_state_name(placement.state),
                assignment_count: u64::from(header.assignment_count),
                insert_assignment_count,
                delete_assignment_count,
                object_bytes: u64::from(placement.object_bytes),
            });
        }

        rows.sort_by_key(|row| row.delta_pid);
        Ok(rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_scan_placement_snapshot(
    index_relation: pg_sys::Relation,
    query_values: Vec<f32>,
) -> Vec<SpireIndexScanPlacementSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexScanPlacementSnapshotRow>, String> {
        let query = scan::SpireScanQuery::new(query_values)?;
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store = unsafe {
            storage::SpireRelationObjectStoreSet::for_index_relation_and_placements(
                index_relation,
                &placement_directory,
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            )?
        };
        let diagnostics = scan::collect_single_level_scan_placement_diagnostics(
            &snapshot,
            &object_store,
            &query,
            options::relation_options(index_relation),
        )?;
        let rows = diagnostics
            .stores
            .into_iter()
            .map(|store| SpireIndexScanPlacementSnapshotRow {
                active_epoch: store.epoch,
                effective_nprobe: diagnostics.scan_plan.nprobe,
                effective_nprobe_source: diagnostics.scan_plan.nprobe_source,
                effective_rerank_width: diagnostics.scan_plan.rerank_width as u64,
                effective_rerank_width_source: diagnostics.scan_plan.rerank_width_source,
                node_id: store.node_id,
                local_store_id: store.local_store_id,
                scanned_pid_count: store.scanned_pid_count as u64,
                leaf_pid_count: store.leaf_pid_count as u64,
                delta_pid_count: store.delta_pid_count as u64,
                candidate_row_count: store.candidate_row_count as u64,
                leaf_candidate_row_count: store.leaf_candidate_row_count as u64,
                delta_candidate_row_count: store.delta_candidate_row_count as u64,
                delete_delta_row_count: store.delete_delta_row_count as u64,
                dropped_unselected_delta_route_count: store
                    .dropped_unselected_delta_route_count as u64,
            })
            .collect();
        Ok(rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_root_routing_snapshot(
    index_relation: pg_sys::Relation,
) -> Vec<SpireIndexRootRoutingSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexRootRoutingSnapshotRow>, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        collect_root_routing_snapshot_rows(&snapshot, &object_store)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn index_routing_centroid_snapshot(
    index_relation: pg_sys::Relation,
) -> Vec<SpireIndexRoutingCentroidSnapshotRow> {
    let result = (|| -> Result<Vec<SpireIndexRoutingCentroidSnapshotRow>, String> {
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch == 0 {
            return Ok(Vec::new());
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let object_store =
            unsafe { storage::SpireRelationObjectStore::for_index_relation(index_relation)? };
        collect_routing_centroid_snapshot_rows(&snapshot, &object_store)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn collect_root_routing_snapshot_rows(
    snapshot: &meta::SpireValidatedEpochSnapshot<'_>,
    object_store: &impl storage::SpireObjectReader,
) -> Result<Vec<SpireIndexRootRoutingSnapshotRow>, String> {
    let mut root = None;
    // Walk the full manifest so malformed epochs with multiple roots are reported.
    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup = snapshot.require_lookup(manifest_entry.pid, "root routing snapshot")?;
        let header = object_store.read_object_header(lookup.placement)?;
        if header.kind != storage::SpirePartitionObjectKind::Root {
            continue;
        }
        if root.is_some() {
            return Err("ec_spire root routing snapshot found multiple root objects".to_owned());
        }
        root = Some((
            manifest_entry.pid,
            manifest_entry.object_version,
            object_store.read_routing_object(lookup.placement)?,
        ));
    }

    let Some((root_pid, root_object_version, root_object)) = root else {
        return Err("ec_spire root routing snapshot found no active root object".to_owned());
    };
    let root_child_count = u64::try_from(root_object.child_count())
        .map_err(|_| "ec_spire root routing child count exceeds u64".to_owned())?;
    root_object
        .children()
        .map(|child| {
            let child_lookup = snapshot.require_lookup(child.child_pid, "root routing child")?;
            let child_header = object_store.read_object_header(child_lookup.placement)?;
            Ok(SpireIndexRootRoutingSnapshotRow {
                active_epoch: snapshot.epoch_manifest().epoch,
                root_pid,
                root_object_version,
                root_level: root_object.header.level,
                root_child_count,
                centroid_dimensions: root_object.dimensions,
                centroid_index: child.centroid_index,
                child_pid: child.child_pid,
                child_kind: partition_object_kind_name(child_header.kind),
                child_object_version: child_header.object_version,
                child_level: child_header.level,
                child_parent_pid: child_header.parent_pid,
                child_assignment_count: u64::from(child_header.assignment_count),
                child_node_id: child_lookup.placement.node_id,
                child_local_store_id: child_lookup.placement.local_store_id,
                child_store_relid: child_lookup.placement.store_relid,
                child_placement_state: placement_state_name(child_lookup.placement.state),
                child_object_bytes: u64::from(child_lookup.placement.object_bytes),
            })
        })
        .collect::<Result<Vec<_>, String>>()
}

fn collect_routing_centroid_snapshot_rows(
    snapshot: &meta::SpireValidatedEpochSnapshot<'_>,
    object_store: &impl storage::SpireObjectReader,
) -> Result<Vec<SpireIndexRoutingCentroidSnapshotRow>, String> {
    let mut rows = Vec::new();
    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup =
            snapshot.require_lookup(manifest_entry.pid, "routing centroid snapshot parent")?;
        let parent_header = object_store.read_object_header(lookup.placement)?;
        if parent_header.kind != storage::SpirePartitionObjectKind::Root
            && parent_header.kind != storage::SpirePartitionObjectKind::Internal
        {
            continue;
        }
        let parent = object_store.read_routing_object(lookup.placement)?;
        let parent_child_count = u64::try_from(parent.child_count())
            .map_err(|_| "ec_spire routing centroid child count exceeds u64".to_owned())?;
        for child in parent.children() {
            let child_lookup =
                snapshot.require_lookup(child.child_pid, "routing centroid snapshot child")?;
            let child_header = object_store.read_object_header(child_lookup.placement)?;
            rows.push(SpireIndexRoutingCentroidSnapshotRow {
                active_epoch: snapshot.epoch_manifest().epoch,
                parent_pid: parent.header.pid,
                parent_kind: partition_object_kind_name(parent.header.kind),
                parent_object_version: parent.header.object_version,
                parent_level: parent.header.level,
                parent_child_count,
                centroid_dimensions: parent.dimensions,
                centroid_index: child.centroid_index,
                child_pid: child.child_pid,
                child_kind: partition_object_kind_name(child_header.kind),
                child_object_version: child_header.object_version,
                child_level: child_header.level,
                child_parent_pid: child_header.parent_pid,
                child_assignment_count: u64::from(child_header.assignment_count),
                child_node_id: child_lookup.placement.node_id,
                child_local_store_id: child_lookup.placement.local_store_id,
                child_store_relid: child_lookup.placement.store_relid,
                child_placement_state: placement_state_name(child_lookup.placement.state),
                child_object_bytes: u64::from(child_lookup.placement.object_bytes),
                centroid: child.centroid.to_vec(),
            });
        }
    }
    Ok(rows)
}
