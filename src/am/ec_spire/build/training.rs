pub(super) fn train_single_level_centroid_plan(
    dimensions: u16,
    source_vectors: &[Vec<f32>],
    requested_nlists: u32,
    seed: u64,
) -> Result<SpireSingleLevelCentroidPlan, String> {
    if dimensions == 0 {
        return Err("ec_spire centroid plan requires dimensions > 0".to_owned());
    }
    let nlists = common_training::resolve_auto_nlists(requested_nlists, source_vectors.len());
    let source_refs = source_vectors.iter().map(Vec::as_slice).collect::<Vec<_>>();
    let model = common_training::train_spherical_kmeans(
        "ec_spire",
        &source_refs,
        usize::from(dimensions),
        nlists,
        seed,
        SPIRE_DEFAULT_KMEANS_ITERATIONS,
    )?;
    let mut assignment_indexes = Vec::with_capacity(source_vectors.len());
    for source in source_vectors {
        let assignment_index =
            common_training::assign_vector_to_centroid("ec_spire", source, &model)?;
        assignment_indexes.push(
            u32::try_from(assignment_index)
                .map_err(|_| "ec_spire centroid assignment index exceeds u32".to_owned())?,
        );
    }

    Ok(SpireSingleLevelCentroidPlan {
        dimensions,
        centroids: model.centroids,
        assignment_indexes,
    })
}

impl SpireBuildState {
    fn new(options: options::EcSpireOptions, tuple_layout: SpireIndexedTupleLayout) -> Self {
        Self {
            options,
            tuple_layout,
            scanned_tuples: 0,
            tuples: Vec::new(),
            dimensions: None,
        }
    }

    fn push(&mut self, tuple: SpireBuildTuple) {
        self.try_push(tuple)
            .unwrap_or_else(|e| pgrx::error!("ec_spire ambuild found invalid indexed tuple: {e}"));
    }

    fn try_push(&mut self, tuple: SpireBuildTuple) -> Result<(), String> {
        if tuple.heap_tid == ItemPointer::INVALID {
            return Err("heap tid must be valid".to_owned());
        }
        if tuple.assignment.heap_tid != tuple.heap_tid {
            return Err("assignment heap tid must match build tuple heap tid".to_owned());
        }
        if SpireAssignmentPayloadFormat::from_tag(tuple.assignment.payload_format)?
            != self.options.assignment_payload_format()
        {
            return Err("assignment payload format does not match build options".to_owned());
        }
        if tuple.source_vector.len() != usize::from(tuple.dimensions) {
            return Err(format!(
                "source dimensions mismatch: source dim {} vs indexed dim {}",
                tuple.source_vector.len(),
                tuple.dimensions
            ));
        }
        common_training::normalize_vector(
            "ec_spire",
            &tuple.source_vector,
            usize::from(tuple.dimensions),
        )?;

        match self.dimensions {
            None => self.dimensions = Some(tuple.dimensions),
            Some(dimensions) if dimensions == tuple.dimensions => {}
            Some(dimensions) => {
                return Err(format!(
                    "dimension mismatch: saw {} after {}",
                    tuple.dimensions, dimensions
                ));
            }
        }

        self.scanned_tuples += 1;
        self.tuples.push(tuple);
        Ok(())
    }

    fn training_sample_count(&self) -> usize {
        resolve_training_sample_count(self.options.training_sample_rows, self.tuples.len())
    }

    fn training_sample_vectors(&self) -> Vec<&[f32]> {
        let indices = common_training::deterministic_sample_indices(
            self.tuples.len(),
            self.training_sample_count(),
            self.options.seed as u64,
        );
        indices
            .into_iter()
            .map(|index| self.tuples[index].source_vector.as_slice())
            .collect()
    }

    fn assignment_inputs(&self) -> Vec<SpireLeafAssignmentInput> {
        self.tuples
            .iter()
            .map(|tuple| tuple.assignment.clone())
            .collect()
    }

    fn assignment_identity_inputs(&self) -> Vec<SpireLeafAssignmentIdentityInput> {
        self.tuples
            .iter()
            .map(|tuple| SpireLeafAssignmentIdentityInput {
                assignment: tuple.assignment.clone(),
                vec_id_source_identity: tuple.vec_id_source_identity.clone(),
            })
            .collect()
    }

    fn source_vectors(&self) -> Vec<Vec<f32>> {
        self.tuples
            .iter()
            .map(|tuple| tuple.source_vector.clone())
            .collect()
    }

    fn train_centroid_plan(&self) -> Result<SpireSingleLevelCentroidPlan, String> {
        let dimensions = self
            .dimensions
            .ok_or_else(|| "ec_spire centroid training requires at least one tuple".to_owned())?;
        let requested_nlists = u32::try_from(self.options.nlists)
            .map_err(|_| "ec_spire nlists reloption must be non-negative".to_owned())?;
        let nlists = common_training::resolve_auto_nlists(requested_nlists, self.tuples.len());
        let sample_vectors = self.training_sample_vectors();
        let model = common_training::train_spherical_kmeans(
            "ec_spire",
            &sample_vectors,
            usize::from(dimensions),
            nlists,
            self.options.seed as u64,
            SPIRE_DEFAULT_KMEANS_ITERATIONS,
        )?;
        let mut assignment_indexes = Vec::with_capacity(self.tuples.len());
        for tuple in &self.tuples {
            let centroid_index = common_training::assign_vector_to_centroid(
                "ec_spire",
                &tuple.source_vector,
                &model,
            )?;
            assignment_indexes.push(
                u32::try_from(centroid_index)
                    .map_err(|_| "ec_spire centroid assignment index exceeds u32".to_owned())?,
            );
        }

        Ok(SpireSingleLevelCentroidPlan {
            dimensions,
            centroids: model.centroids,
            assignment_indexes,
        })
    }
}

fn resolve_training_sample_count(requested_sample_rows: i32, row_count: usize) -> usize {
    if row_count == 0 {
        return 0;
    }
    if requested_sample_rows > 0 {
        return (requested_sample_rows as usize).min(row_count);
    }
    row_count.min(SPIRE_DEFAULT_AUTO_TRAINING_SAMPLE_ROWS)
}
