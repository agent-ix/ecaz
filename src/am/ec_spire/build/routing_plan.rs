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
        self.validate()?;
        let model = common_training::SphericalKMeansModel {
            dimensions: usize::from(self.dimensions),
            centroids: self
                .entries
                .iter()
                .map(|entry| entry.centroid.clone())
                .collect(),
        };
        let centroid_index =
            common_training::assign_vector_to_centroid("ec_spire", vector, &model)?;
        Ok(self.entries[centroid_index].pid)
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
