#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireSingleLevelCentroidPlan {
    pub(super) dimensions: u16,
    pub(super) centroids: Vec<Vec<f32>>,
    pub(super) assignment_indexes: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireSingleLevelRouteEntry {
    pub(super) centroid_index: u32,
    pub(super) pid: u64,
    pub(super) centroid: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireSingleLevelRouteMap {
    pub(super) dimensions: u16,
    pub(super) entries: Vec<SpireSingleLevelRouteEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpireBoundaryAssignmentPlan {
    pub(super) primary_pid: u64,
    pub(super) replica_pids: Vec<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SpireCentroidRouteInput<'a> {
    pub(super) centroid_index: u32,
    pub(super) pid: u64,
    pub(super) centroid: &'a [f32],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SpireRankedCentroidRoute {
    pub(super) centroid_index: u32,
    pub(super) pid: u64,
    pub(super) score: f32,
}

impl SpireSingleLevelCentroidPlan {
    pub(super) fn centroid_count(&self) -> usize {
        self.centroids.len()
    }

    fn validate(&self) -> Result<(), String> {
        if self.dimensions == 0 {
            return Err("ec_spire centroid plan requires dimensions > 0".to_owned());
        }
        if self.centroids.is_empty() {
            return Err("ec_spire centroid plan requires at least one centroid".to_owned());
        }
        let dimensions = usize::from(self.dimensions);
        for (index, centroid) in self.centroids.iter().enumerate() {
            if centroid.len() != dimensions {
                return Err(format!(
                    "ec_spire centroid {index} dimensions mismatch: got {}, expected {dimensions}",
                    centroid.len()
                ));
            }
            if centroid.iter().any(|component| !component.is_finite()) {
                return Err(format!("ec_spire centroid {index} must be finite"));
            }
        }
        for &assignment_index in &self.assignment_indexes {
            let centroid_index = usize::try_from(assignment_index)
                .map_err(|_| "ec_spire centroid assignment index exceeds usize".to_owned())?;
            if centroid_index >= self.centroids.len() {
                return Err(format!(
                    "ec_spire centroid assignment index {centroid_index} exceeds centroid count {}",
                    self.centroids.len()
                ));
            }
        }
        Ok(())
    }
}

impl SpireSingleLevelRouteMap {
    pub(super) fn from_centroid_plan(
        plan: &SpireSingleLevelCentroidPlan,
        centroid_pids: &[u64],
    ) -> Result<Self, String> {
        plan.validate()?;
        if centroid_pids.len() != plan.centroid_count() {
            return Err(format!(
                "ec_spire route map pid count {} does not match centroid count {}",
                centroid_pids.len(),
                plan.centroid_count()
            ));
        }

        let mut entries = Vec::with_capacity(plan.centroid_count());
        for (centroid_index, (centroid, &pid)) in
            plan.centroids.iter().zip(centroid_pids.iter()).enumerate()
        {
            if pid == 0 {
                return Err("ec_spire route map pid 0 is invalid".to_owned());
            }
            entries.push(SpireSingleLevelRouteEntry {
                centroid_index: u32::try_from(centroid_index)
                    .map_err(|_| "ec_spire route map centroid index exceeds u32".to_owned())?,
                pid,
                centroid: centroid.clone(),
            });
        }

        let route_map = Self {
            dimensions: plan.dimensions,
            entries,
        };
        route_map.validate()?;
        Ok(route_map)
    }

    pub(super) fn route_pid_for_vector(&self, vector: &[f32]) -> Result<u64, String> {
        Ok(self.route_boundary_assignment_for_vector(vector, 0)?.primary_pid)
    }

    pub(super) fn route_boundary_assignment_for_vector(
        &self,
        vector: &[f32],
        boundary_replica_count: u32,
    ) -> Result<SpireBoundaryAssignmentPlan, String> {
        self.validate()?;

        let scored_entries = rank_centroid_routes_by_ip(
            "ec_spire route map",
            vector,
            usize::from(self.dimensions),
            self.entries.iter().map(|entry| SpireCentroidRouteInput {
                centroid_index: entry.centroid_index,
                pid: entry.pid,
                centroid: &entry.centroid,
            }),
        )?;

        let mut selected_pids = Vec::with_capacity(
            usize::try_from(boundary_replica_count)
                .unwrap_or(usize::MAX)
                .saturating_add(1)
                .min(scored_entries.len()),
        );
        for entry in scored_entries {
            if selected_pids.contains(&entry.pid) {
                continue;
            }
            selected_pids.push(entry.pid);
            if selected_pids.len()
                == usize::try_from(boundary_replica_count)
                    .unwrap_or(usize::MAX)
                    .saturating_add(1)
            {
                break;
            }
        }

        let primary_pid = selected_pids
            .first()
            .copied()
            .ok_or_else(|| "ec_spire boundary assignment needs at least one route".to_owned())?;
        Ok(SpireBoundaryAssignmentPlan {
            primary_pid,
            replica_pids: selected_pids.into_iter().skip(1).collect(),
        })
    }

    pub(super) fn get(&self, centroid_index: u32) -> Option<&SpireSingleLevelRouteEntry> {
        self.entries
            .get(usize::try_from(centroid_index).ok()?)
            .filter(|entry| entry.centroid_index == centroid_index)
    }

    fn validate(&self) -> Result<(), String> {
        if self.dimensions == 0 {
            return Err("ec_spire route map requires dimensions > 0".to_owned());
        }
        if self.entries.is_empty() {
            return Err("ec_spire route map requires at least one entry".to_owned());
        }
        let dimensions = usize::from(self.dimensions);
        for (expected_index, entry) in self.entries.iter().enumerate() {
            let expected_index = u32::try_from(expected_index)
                .map_err(|_| "ec_spire route map centroid index exceeds u32".to_owned())?;
            if entry.centroid_index != expected_index {
                return Err(format!(
                    "ec_spire route map centroid index mismatch: got {}, expected {expected_index}",
                    entry.centroid_index
                ));
            }
            if entry.pid == 0 {
                return Err("ec_spire route map pid 0 is invalid".to_owned());
            }
            if entry.centroid.len() != dimensions {
                return Err(format!(
                    "ec_spire route map centroid {} dimensions mismatch: got {}, expected {dimensions}",
                    entry.centroid_index,
                    entry.centroid.len()
                ));
            }
            if entry
                .centroid
                .iter()
                .any(|component| !component.is_finite())
            {
                return Err(format!(
                    "ec_spire route map centroid {} must be finite",
                    entry.centroid_index
                ));
            }
        }
        Ok(())
    }
}

pub(super) fn rank_centroid_routes_by_ip<'a>(
    label: &str,
    vector: &[f32],
    dimensions: usize,
    routes: impl IntoIterator<Item = SpireCentroidRouteInput<'a>>,
) -> Result<Vec<SpireRankedCentroidRoute>, String> {
    validate_route_vector(vector, dimensions)?;
    let mut ranked = routes
        .into_iter()
        .map(|route| {
            if route.centroid.len() != dimensions {
                return Err(format!(
                    "{label} centroid {} dimensions mismatch: got {}, expected {dimensions}",
                    route.centroid_index,
                    route.centroid.len()
                ));
            }
            if route.centroid.iter().any(|component| !component.is_finite()) {
                return Err(format!(
                    "{label} centroid {} must be finite",
                    route.centroid_index
                ));
            }
            Ok(SpireRankedCentroidRoute {
                centroid_index: route.centroid_index,
                pid: route.pid,
                score: inner_product(vector, route.centroid),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    ranked.sort_by(ranked_centroid_route_cmp);
    Ok(ranked)
}

fn ranked_centroid_route_cmp(
    left: &SpireRankedCentroidRoute,
    right: &SpireRankedCentroidRoute,
) -> std::cmp::Ordering {
    right
        .score
        .total_cmp(&left.score)
        .then_with(|| left.centroid_index.cmp(&right.centroid_index))
        .then_with(|| left.pid.cmp(&right.pid))
}

fn validate_route_vector(vector: &[f32], dimensions: usize) -> Result<(), String> {
    if vector.len() != dimensions {
        return Err(format!(
            "ec_spire vector dimensions mismatch: got {}, expected {dimensions}",
            vector.len()
        ));
    }
    if vector.iter().any(|value| !value.is_finite()) {
        return Err("ec_spire vector contains a non-finite value".to_owned());
    }
    let norm_sq = vector
        .iter()
        .map(|value| f64::from(*value) * f64::from(*value))
        .sum::<f64>();
    if norm_sq <= f64::EPSILON {
        return Err("ec_spire route assignment requires non-zero vectors".to_owned());
    }
    Ok(())
}

fn inner_product(left: &[f32], right: &[f32]) -> f32 {
    left.iter().zip(right.iter()).map(|(left, right)| left * right).sum()
}
