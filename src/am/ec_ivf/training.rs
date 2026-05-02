//! IVF centroid training helpers.

use crate::am::common::training as common_training;

pub(super) type SphericalKMeansModel = common_training::SphericalKMeansModel;

pub(super) fn resolve_auto_nlists(requested_nlists: u32, row_count: usize) -> usize {
    common_training::resolve_auto_nlists(requested_nlists, row_count)
}

pub(super) fn deterministic_sample_indices(
    row_count: usize,
    sample_limit: usize,
    seed: u64,
) -> Vec<usize> {
    common_training::deterministic_sample_indices(row_count, sample_limit, seed)
}

pub(super) fn normalize_vector(source: &[f32], dimensions: usize) -> Result<Vec<f32>, String> {
    common_training::normalize_vector("ec_ivf", source, dimensions)
}

pub(super) fn train_spherical_kmeans(
    source_vectors: &[&[f32]],
    dimensions: usize,
    nlists: usize,
    seed: u64,
    max_iterations: usize,
) -> Result<SphericalKMeansModel, String> {
    common_training::train_spherical_kmeans(
        "ec_ivf",
        source_vectors,
        dimensions,
        nlists,
        seed,
        max_iterations,
    )
}

pub(super) fn assign_vector_to_centroid(
    source: &[f32],
    model: &SphericalKMeansModel,
) -> Result<usize, String> {
    common_training::assign_vector_to_centroid("ec_ivf", source, model)
}

#[cfg(test)]
mod tests {
    use super::{
        assign_vector_to_centroid, deterministic_sample_indices, normalize_vector,
        resolve_auto_nlists, train_spherical_kmeans,
    };

    fn approx_eq(left: f32, right: f32) -> bool {
        (left - right).abs() < 1e-5
    }

    fn norm(vector: &[f32]) -> f32 {
        vector.iter().map(|value| value * value).sum::<f32>().sqrt()
    }

    #[test]
    fn resolve_auto_nlists_handles_empty_and_small_tables() {
        assert_eq!(resolve_auto_nlists(0, 0), 0);
        assert_eq!(resolve_auto_nlists(0, 1), 1);
        assert_eq!(resolve_auto_nlists(0, 10), 4);
        assert_eq!(resolve_auto_nlists(7, 10), 7);
    }

    #[test]
    fn deterministic_sample_indices_are_stable_and_unique() {
        let first = deterministic_sample_indices(100, 10, 42);
        let second = deterministic_sample_indices(100, 10, 42);
        let mut sorted = first.clone();
        sorted.sort_unstable();
        sorted.dedup();

        assert_eq!(first, second);
        assert_eq!(sorted.len(), first.len());
        assert_eq!(deterministic_sample_indices(4, 10, 1), vec![0, 1, 2, 3]);
        assert!(deterministic_sample_indices(4, 0, 1).is_empty());
    }

    #[test]
    fn normalize_vector_rejects_bad_inputs() {
        assert!(normalize_vector(&[1.0, 2.0], 3)
            .unwrap_err()
            .contains("dimensions mismatch"));
        assert!(normalize_vector(&[f32::NAN], 1)
            .unwrap_err()
            .contains("non-finite"));
        assert!(normalize_vector(&[0.0, 0.0], 2)
            .unwrap_err()
            .contains("non-zero"));
    }

    #[test]
    fn normalize_vector_returns_unit_length() {
        let normalized = normalize_vector(&[3.0, 4.0], 2).unwrap();
        assert!(approx_eq(norm(&normalized), 1.0));
        assert!(approx_eq(normalized[0], 0.6));
        assert!(approx_eq(normalized[1], 0.8));
    }

    #[test]
    fn spherical_kmeans_rejects_empty_training_input() {
        let err = train_spherical_kmeans(&[], 2, 2, 1, 5).unwrap_err();
        assert!(err.contains("at least one source vector"));
    }

    #[test]
    fn spherical_kmeans_is_deterministic_for_seed() {
        let vectors = [
            vec![1.0, 0.0],
            vec![0.9, 0.1],
            vec![-1.0, 0.0],
            vec![-0.9, -0.1],
        ];
        let refs = vectors.iter().map(Vec::as_slice).collect::<Vec<_>>();

        let first = train_spherical_kmeans(&refs, 2, 2, 99, 8).unwrap();
        let second = train_spherical_kmeans(&refs, 2, 2, 99, 8).unwrap();

        assert_eq!(first, second);
        assert_eq!(first.centroid_count(), 2);
        assert!(first
            .centroids
            .iter()
            .all(|centroid| approx_eq(norm(centroid), 1.0)));
    }

    #[test]
    fn spherical_kmeans_supports_more_lists_than_rows() {
        let vectors = [vec![1.0, 0.0], vec![0.0, 1.0]];
        let refs = vectors.iter().map(Vec::as_slice).collect::<Vec<_>>();

        let model = train_spherical_kmeans(&refs, 2, 5, 7, 3).unwrap();

        assert_eq!(model.centroid_count(), 5);
        assert!(model
            .centroids
            .iter()
            .all(|centroid| approx_eq(norm(centroid), 1.0)));
    }

    #[test]
    fn assign_vector_to_centroid_uses_inner_product_router() {
        let vectors = [
            vec![1.0, 0.0],
            vec![0.95, 0.05],
            vec![-1.0, 0.0],
            vec![-0.95, -0.05],
        ];
        let refs = vectors.iter().map(Vec::as_slice).collect::<Vec<_>>();
        let model = train_spherical_kmeans(&refs, 2, 2, 0, 10).unwrap();

        let positive = assign_vector_to_centroid(&[1.0, 0.02], &model).unwrap();
        let negative = assign_vector_to_centroid(&[-1.0, -0.02], &model).unwrap();

        assert_ne!(positive, negative);
    }
}
