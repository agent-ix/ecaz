#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpireCandidateMergeModelInput {
    pub(crate) input_index: usize,
    pub(crate) dedupe_key: Vec<u8>,
    pub(crate) served_epoch: u64,
    pub(crate) node_id: u32,
    pub(crate) pid: u64,
    pub(crate) object_version: u64,
    pub(crate) row_index: u32,
    pub(crate) assignment_role_rank: u8,
    pub(crate) row_locator: Vec<u8>,
    pub(crate) score: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireCandidateMergeModelOutput {
    pub(crate) selected_input_indices: Vec<usize>,
    pub(crate) input_count: u64,
    pub(crate) duplicate_vec_id_count: u64,
}

fn candidate_model_cmp(
    left: &SpireCandidateMergeModelInput,
    right: &SpireCandidateMergeModelInput,
) -> std::cmp::Ordering {
    left.score
        .total_cmp(&right.score)
        .then_with(|| left.assignment_role_rank.cmp(&right.assignment_role_rank))
        .then_with(|| right.served_epoch.cmp(&left.served_epoch))
        .then_with(|| left.node_id.cmp(&right.node_id))
        .then_with(|| left.pid.cmp(&right.pid))
        .then_with(|| right.object_version.cmp(&left.object_version))
        .then_with(|| left.row_index.cmp(&right.row_index))
        .then_with(|| left.row_locator.cmp(&right.row_locator))
        .then_with(|| left.input_index.cmp(&right.input_index))
}

pub(crate) fn merge_candidate_model_inputs<I>(
    candidates: I,
    limit: Option<usize>,
) -> Result<SpireCandidateMergeModelOutput, String>
where
    I: IntoIterator<Item = SpireCandidateMergeModelInput>,
{
    let mut input_count = 0_u64;
    let mut duplicate_vec_id_count = 0_u64;
    let mut best_by_vec_id: std::collections::HashMap<Vec<u8>, SpireCandidateMergeModelInput> =
        std::collections::HashMap::new();

    for candidate in candidates {
        input_count = input_count
            .checked_add(1)
            .ok_or_else(|| "ec_spire remote candidate merge input count overflow".to_owned())?;
        if !candidate.score.is_finite() {
            return Err("ec_spire remote candidate merge received non-finite score".to_owned());
        }

        match best_by_vec_id.entry(candidate.dedupe_key.clone()) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                duplicate_vec_id_count =
                    duplicate_vec_id_count.checked_add(1).ok_or_else(|| {
                        "ec_spire remote candidate merge duplicate count overflow".to_owned()
                    })?;
                if candidate_model_cmp(&candidate, entry.get()).is_lt() {
                    *entry.get_mut() = candidate;
                }
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(candidate);
            }
        }
    }

    let mut candidates = best_by_vec_id.into_values().collect::<Vec<_>>();
    if let Some(limit) = limit {
        if candidates.len() > limit {
            candidates.select_nth_unstable_by(limit, candidate_model_cmp);
            candidates.truncate(limit);
        }
    }
    candidates.sort_by(candidate_model_cmp);
    if let Some(limit) = limit {
        candidates.truncate(limit);
    }

    Ok(SpireCandidateMergeModelOutput {
        selected_input_indices: candidates
            .into_iter()
            .map(|candidate| candidate.input_index)
            .collect(),
        input_count,
        duplicate_vec_id_count,
    })
}
