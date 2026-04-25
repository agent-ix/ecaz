//! IVF centroid training helpers.

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

const DEFAULT_AUTO_NLISTS_MAX: usize = 4096;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SphericalKMeansModel {
    pub(super) dimensions: usize,
    pub(super) centroids: Vec<Vec<f32>>,
}

impl SphericalKMeansModel {
    pub(super) fn centroid_count(&self) -> usize {
        self.centroids.len()
    }
}

pub(super) fn resolve_auto_nlists(requested_nlists: u32, row_count: usize) -> usize {
    if row_count == 0 {
        return 0;
    }
    if requested_nlists > 0 {
        return requested_nlists as usize;
    }

    let sqrt_rows = (row_count as f64).sqrt().ceil() as usize;
    sqrt_rows.clamp(1, DEFAULT_AUTO_NLISTS_MAX.min(row_count))
}

pub(super) fn deterministic_sample_indices(
    row_count: usize,
    sample_limit: usize,
    seed: u64,
) -> Vec<usize> {
    if sample_limit >= row_count {
        return (0..row_count).collect();
    }
    if sample_limit == 0 {
        return Vec::new();
    }

    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut indices = (0..row_count).collect::<Vec<_>>();
    for i in 0..sample_limit {
        let swap_index = rng.gen_range(i..row_count);
        indices.swap(i, swap_index);
    }
    indices.truncate(sample_limit);
    indices
}

pub(super) fn normalize_vector(source: &[f32], dimensions: usize) -> Result<Vec<f32>, String> {
    if source.len() != dimensions {
        return Err(format!(
            "ec_ivf vector dimensions mismatch: got {}, expected {dimensions}",
            source.len()
        ));
    }
    if source.iter().any(|value| !value.is_finite()) {
        return Err("ec_ivf vector contains a non-finite value".into());
    }

    let norm_sq = source
        .iter()
        .map(|value| (*value as f64) * (*value as f64))
        .sum::<f64>();
    if norm_sq <= f64::EPSILON {
        return Err("ec_ivf spherical k-means requires non-zero vectors".into());
    }

    let inv_norm = (norm_sq.sqrt() as f32).recip();
    Ok(source.iter().map(|value| *value * inv_norm).collect())
}

pub(super) fn train_spherical_kmeans(
    source_vectors: &[&[f32]],
    dimensions: usize,
    nlists: usize,
    seed: u64,
    max_iterations: usize,
) -> Result<SphericalKMeansModel, String> {
    if dimensions == 0 {
        return Err("ec_ivf spherical k-means requires dimensions > 0".into());
    }
    if source_vectors.is_empty() {
        return Err("ec_ivf spherical k-means requires at least one source vector".into());
    }
    if nlists == 0 {
        return Err("ec_ivf spherical k-means requires at least one list".into());
    }

    let samples = source_vectors
        .iter()
        .map(|source| normalize_vector(source, dimensions))
        .collect::<Result<Vec<_>, _>>()?;
    let mut centroids = initial_centroids(&samples, nlists, seed);
    let mut assignments = vec![usize::MAX; samples.len()];
    let mut sums = vec![vec![0.0_f32; dimensions]; nlists];
    let mut counts = vec![0_usize; nlists];

    for iteration in 0..max_iterations {
        sums.iter_mut().for_each(|sum| sum.fill(0.0));
        counts.fill(0);

        let mut changed = false;
        for (sample_index, sample) in samples.iter().enumerate() {
            let centroid_index = nearest_centroid_for_normalized(sample, &centroids)?;
            if assignments[sample_index] != centroid_index {
                assignments[sample_index] = centroid_index;
                changed = true;
            }
            counts[centroid_index] += 1;
            for (dst, value) in sums[centroid_index].iter_mut().zip(sample.iter()) {
                *dst += *value;
            }
        }

        for centroid_index in 0..nlists {
            if counts[centroid_index] == 0 {
                let fallback_index =
                    fallback_sample_index(seed, iteration, centroid_index, samples.len());
                centroids[centroid_index].copy_from_slice(&samples[fallback_index]);
                continue;
            }

            let normalized = normalize_vector(&sums[centroid_index], dimensions)?;
            centroids[centroid_index].copy_from_slice(&normalized);
        }

        if !changed {
            break;
        }
    }

    Ok(SphericalKMeansModel {
        dimensions,
        centroids,
    })
}

pub(super) fn assign_vector_to_centroid(
    source: &[f32],
    model: &SphericalKMeansModel,
) -> Result<usize, String> {
    let normalized = normalize_vector(source, model.dimensions)?;
    nearest_centroid_for_normalized(&normalized, &model.centroids)
}

fn initial_centroids(samples: &[Vec<f32>], centroid_count: usize, seed: u64) -> Vec<Vec<f32>> {
    let mut centroids = Vec::with_capacity(centroid_count);
    let seeded_count = centroid_count.min(samples.len());
    let initial_indices = deterministic_sample_indices(samples.len(), seeded_count, seed);

    for sample_index in initial_indices {
        centroids.push(samples[sample_index].clone());
    }
    for centroid_index in centroids.len()..centroid_count {
        let sample_index = fallback_sample_index(seed, 0, centroid_index, samples.len());
        centroids.push(samples[sample_index].clone());
    }
    centroids
}

fn fallback_sample_index(
    seed: u64,
    iteration: usize,
    centroid_index: usize,
    sample_count: usize,
) -> usize {
    debug_assert!(sample_count > 0);
    seed.wrapping_add(iteration as u64)
        .wrapping_add((centroid_index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)) as usize
        % sample_count
}

fn nearest_centroid_for_normalized(
    normalized: &[f32],
    centroids: &[Vec<f32>],
) -> Result<usize, String> {
    if centroids.is_empty() {
        return Err("ec_ivf centroid assignment requires at least one centroid".into());
    }

    let mut best_index = 0;
    let mut best_score = f32::NEG_INFINITY;
    for (centroid_index, centroid) in centroids.iter().enumerate() {
        if centroid.len() != normalized.len() {
            return Err(format!(
                "ec_ivf centroid dimensions mismatch: got {}, expected {}",
                centroid.len(),
                normalized.len()
            ));
        }
        let score = inner_product(normalized, centroid);
        if score > best_score {
            best_score = score;
            best_index = centroid_index;
        }
    }
    Ok(best_index)
}

fn inner_product(left: &[f32], right: &[f32]) -> f32 {
    left.iter()
        .zip(right.iter())
        .map(|(left, right)| left * right)
        .sum()
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
