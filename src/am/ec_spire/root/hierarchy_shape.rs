#[derive(Debug, Clone, PartialEq, Eq)]
struct SpireHierarchyObjectSummary {
    pid: u64,
    kind: storage::SpirePartitionObjectKind,
    level: u16,
    parent_pid: u64,
    child_pids: Vec<u64>,
}

fn hierarchy_object_summary(
    header: &storage::SpirePartitionObjectHeader,
    child_pids: Vec<u64>,
) -> SpireHierarchyObjectSummary {
    SpireHierarchyObjectSummary {
        pid: header.pid,
        kind: header.kind,
        level: header.level,
        parent_pid: header.parent_pid,
        child_pids,
    }
}

fn validate_recursive_hierarchy_shape(
    objects: &[SpireHierarchyObjectSummary],
) -> Result<bool, String> {
    if objects.is_empty() {
        return Ok(false);
    }

    let mut by_pid = HashMap::with_capacity(objects.len());
    for object in objects {
        if object.pid == 0 {
            return Err("ec_spire hierarchy object pid 0 is invalid".to_owned());
        }
        if by_pid.insert(object.pid, object).is_some() {
            return Err(format!(
                "ec_spire hierarchy contains duplicate active pid {}",
                object.pid
            ));
        }
    }

    let roots = objects
        .iter()
        .filter(|object| object.kind == storage::SpirePartitionObjectKind::Root)
        .collect::<Vec<_>>();
    if roots.len() != 1 {
        return Err(format!(
            "ec_spire hierarchy needs exactly one root object, found {}",
            roots.len()
        ));
    }
    let root = roots[0];
    if root.parent_pid != 0 {
        return Err(format!(
            "ec_spire root pid {} must use parent_pid 0, got {}",
            root.pid, root.parent_pid
        ));
    }
    if root.level == 0 {
        return Err(format!("ec_spire root pid {} must use level > 0", root.pid));
    }

    let has_internal = objects
        .iter()
        .any(|object| object.kind == storage::SpirePartitionObjectKind::Internal);
    for object in objects {
        match object.kind {
            storage::SpirePartitionObjectKind::Root
            | storage::SpirePartitionObjectKind::Internal => {
                if object.kind == storage::SpirePartitionObjectKind::Internal
                    && object.parent_pid == 0
                {
                    return Err(format!(
                        "ec_spire internal routing pid {} must have nonzero parent_pid",
                        object.pid
                    ));
                }
                if object.level == 0 {
                    return Err(format!(
                        "ec_spire routing pid {} must use level > 0",
                        object.pid
                    ));
                }
                let mut seen_children = HashSet::with_capacity(object.child_pids.len());
                for child_pid in &object.child_pids {
                    if !seen_children.insert(*child_pid) {
                        return Err(format!(
                            "ec_spire routing pid {} references duplicate child pid {}",
                            object.pid, child_pid
                        ));
                    }
                    let child = by_pid.get(child_pid).ok_or_else(|| {
                        format!(
                            "ec_spire routing pid {} references missing child pid {}",
                            object.pid, child_pid
                        )
                    })?;
                    if child.parent_pid != object.pid {
                        return Err(format!(
                            "ec_spire child pid {} parent_pid {} does not match routing pid {}",
                            child.pid, child.parent_pid, object.pid
                        ));
                    }
                    if object.level == 1 {
                        if child.kind != storage::SpirePartitionObjectKind::Leaf || child.level != 0
                        {
                            return Err(format!(
                                "ec_spire level-1 routing pid {} child pid {} must be a level-0 leaf",
                                object.pid, child.pid
                            ));
                        }
                    } else if child.kind != storage::SpirePartitionObjectKind::Internal
                        || child.level.checked_add(1) != Some(object.level)
                    {
                        return Err(format!(
                            "ec_spire routing pid {} level {} child pid {} has kind {:?} level {}",
                            object.pid, object.level, child.pid, child.kind, child.level
                        ));
                    }
                }
            }
            storage::SpirePartitionObjectKind::Leaf => {
                if object.level != 0 {
                    return Err(format!(
                        "ec_spire leaf pid {} must use level 0, got {}",
                        object.pid, object.level
                    ));
                }
                let parent = by_pid.get(&object.parent_pid).ok_or_else(|| {
                    format!(
                        "ec_spire leaf pid {} references missing parent pid {}",
                        object.pid, object.parent_pid
                    )
                })?;
                if parent.kind != storage::SpirePartitionObjectKind::Root
                    && parent.kind != storage::SpirePartitionObjectKind::Internal
                {
                    return Err(format!(
                        "ec_spire leaf pid {} parent pid {} is not a routing object",
                        object.pid, object.parent_pid
                    ));
                }
                if !parent.child_pids.contains(&object.pid) {
                    return Err(format!(
                        "ec_spire leaf pid {} is not referenced by parent pid {}",
                        object.pid, object.parent_pid
                    ));
                }
            }
            storage::SpirePartitionObjectKind::Delta => {
                if object.level != 0 {
                    return Err(format!(
                        "ec_spire delta pid {} must use level 0, got {}",
                        object.pid, object.level
                    ));
                }
                let parent = by_pid.get(&object.parent_pid).ok_or_else(|| {
                    format!(
                        "ec_spire delta pid {} references missing base leaf pid {}",
                        object.pid, object.parent_pid
                    )
                })?;
                if parent.kind != storage::SpirePartitionObjectKind::Leaf {
                    return Err(format!(
                        "ec_spire delta pid {} parent pid {} is not a leaf",
                        object.pid, object.parent_pid
                    ));
                }
            }
        }
    }

    Ok(has_internal)
}

fn hierarchy_snapshot_status(
    root_routing_object_count: u64,
    internal_routing_object_count: u64,
    leaf_object_count: u64,
    hierarchy_shape_valid: bool,
    per_level_nprobe_supported: bool,
) -> (&'static str, &'static str) {
    if root_routing_object_count == 0 && leaf_object_count == 0 {
        return ("empty", "none");
    }
    if root_routing_object_count == 0 {
        return (
            "no_root_object",
            "inspect active epoch metadata before enabling recursive routing",
        );
    }
    if root_routing_object_count > 1 {
        return (
            "multiple_root_objects",
            "inspect active epoch metadata before enabling recursive routing",
        );
    }
    if !hierarchy_shape_valid {
        return (
            "invalid_hierarchy_shape",
            "inspect active root/internal/leaf parent-child metadata before scanning recursively",
        );
    }
    if internal_routing_object_count == 0 {
        return (
            "single_level_foundation",
            "set recursive_fanout >= 2 during build to publish recursive routing metadata",
        );
    }
    if per_level_nprobe_supported {
        return ("hierarchy_metadata_present", "none");
    }
    (
        "hierarchy_metadata_present",
        "recursive routing is available; per-level nprobe metadata remains deferred",
    )
}
