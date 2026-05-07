use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::quant::{
    grouped_pq::{encode_grouped_pq, nearest_centroid_l2},
    prod::ProdQuantizer,
    rotation,
};

const DEFAULT_AUTO_NLISTS_MAX: usize = 4096;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SphericalKMeansModel {
    pub(crate) dimensions: usize,
    pub(crate) centroids: Vec<Vec<f32>>,
}

impl SphericalKMeansModel {
    pub(crate) fn centroid_count(&self) -> usize {
        self.centroids.len()
    }
}

pub(crate) fn resolve_auto_nlists(requested_nlists: u32, row_count: usize) -> usize {
    if row_count == 0 {
        return 0;
    }
    if requested_nlists > 0 {
        return requested_nlists as usize;
    }

    let sqrt_rows = (row_count as f64).sqrt().ceil() as usize;
    sqrt_rows.clamp(1, DEFAULT_AUTO_NLISTS_MAX.min(row_count))
}

pub(crate) fn deterministic_sample_indices(
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

pub(crate) fn normalize_vector(
    error_label: &str,
    source: &[f32],
    dimensions: usize,
) -> Result<Vec<f32>, String> {
    validate_assignable_vector(error_label, source, dimensions)?;

    let norm_sq = vector_norm_sq(source);
    let inv_norm = (norm_sq.sqrt() as f32).recip();
    Ok(source.iter().map(|value| *value * inv_norm).collect())
}

pub(crate) fn train_spherical_kmeans(
    error_label: &str,
    source_vectors: &[&[f32]],
    dimensions: usize,
    nlists: usize,
    seed: u64,
    max_iterations: usize,
) -> Result<SphericalKMeansModel, String> {
    if dimensions == 0 {
        return Err(format!(
            "{error_label} spherical k-means requires dimensions > 0"
        ));
    }
    if source_vectors.is_empty() {
        return Err(format!(
            "{error_label} spherical k-means requires at least one source vector"
        ));
    }
    if nlists == 0 {
        return Err(format!(
            "{error_label} spherical k-means requires at least one list"
        ));
    }

    let samples = source_vectors
        .iter()
        .map(|source| normalize_vector(error_label, source, dimensions))
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
            let centroid_index = nearest_centroid(error_label, sample, &centroids)?;
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

            let normalized = normalize_vector(error_label, &sums[centroid_index], dimensions)?;
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

pub(crate) fn assign_vector_to_centroid(
    error_label: &str,
    source: &[f32],
    model: &SphericalKMeansModel,
) -> Result<usize, String> {
    validate_assignable_vector(error_label, source, model.dimensions)?;
    nearest_centroid(error_label, source, &model.centroids)
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

fn nearest_centroid(
    error_label: &str,
    vector: &[f32],
    centroids: &[Vec<f32>],
) -> Result<usize, String> {
    if centroids.is_empty() {
        return Err(format!(
            "{error_label} centroid assignment requires at least one centroid"
        ));
    }

    let mut best_index = 0;
    let mut best_score = f32::NEG_INFINITY;
    for (centroid_index, centroid) in centroids.iter().enumerate() {
        if centroid.len() != vector.len() {
            return Err(format!(
                "{error_label} centroid dimensions mismatch: got {}, expected {}",
                centroid.len(),
                vector.len()
            ));
        }
        let score = inner_product(vector, centroid);
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

fn validate_assignable_vector(
    error_label: &str,
    source: &[f32],
    dimensions: usize,
) -> Result<(), String> {
    if source.len() != dimensions {
        return Err(format!(
            "{error_label} vector dimensions mismatch: got {}, expected {dimensions}",
            source.len()
        ));
    }
    if source.iter().any(|value| !value.is_finite()) {
        return Err(format!("{error_label} vector contains a non-finite value"));
    }

    let norm_sq = vector_norm_sq(source);
    if norm_sq <= f64::EPSILON {
        return Err(format!(
            "{error_label} spherical k-means requires non-zero vectors"
        ));
    }
    Ok(())
}

fn vector_norm_sq(source: &[f32]) -> f64 {
    source
        .iter()
        .map(|value| (*value as f64) * (*value as f64))
        .sum::<f64>()
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SrhtForwardTransform {
    pub(crate) transform_dim: usize,
    pub(crate) signs: Vec<f32>,
}

impl SrhtForwardTransform {
    pub(crate) fn for_dimensions(dimensions: usize, seed: u64) -> Self {
        let transform_dim = rotation::effective_transform_dim(dimensions);
        let signs = rotation::sign_vector(transform_dim, seed);
        Self {
            transform_dim,
            signs,
        }
    }

    pub(crate) fn apply(&self, source: &[f32]) -> Vec<f32> {
        rotation::srht_padded(source, &self.signs)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GroupedPq4Model {
    pub codebooks: Vec<Vec<f32>>,
    pub group_count: usize,
    pub group_size: usize,
    pub transform_dim: usize,
    pub signs: Vec<f32>,
}

pub fn train_grouped_pq4_model(
    source_vectors: &[&[f32]],
    dimensions: usize,
    seed: u64,
    group_size: usize,
    train_size: usize,
    kmeans_iters: usize,
) -> Result<GroupedPq4Model, String> {
    if source_vectors.is_empty() {
        return Err("grouped codebook training requires at least one source vector".to_owned());
    }

    let transform = SrhtForwardTransform::for_dimensions(dimensions, seed);
    if transform.transform_dim % group_size != 0 {
        return Err(format!(
            "transform dim {} is not divisible by group_size {group_size}",
            transform.transform_dim
        ));
    }

    let transformed = source_vectors
        .iter()
        .map(|vector| transform.apply(vector))
        .collect::<Vec<_>>();
    let group_count = transform.transform_dim / group_size;
    let sample_count = train_size.min(transformed.len());
    let sample_indices = sample_indices(
        transformed.len(),
        sample_count,
        seed ^ 0xA5A5_5A5A_DEAD_BEEF,
    );
    let mut codebooks = Vec::with_capacity(group_count);

    for group_index in 0..group_count {
        let mut samples = Vec::with_capacity(sample_count * group_size);
        for &sample_index in &sample_indices {
            let start = group_index * group_size;
            let end = start + group_size;
            samples.extend_from_slice(&transformed[sample_index][start..end]);
        }
        codebooks.push(train_group_codebook(
            &samples,
            group_size,
            kmeans_iters,
            seed ^ (group_index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15),
        )?);
    }

    Ok(GroupedPq4Model {
        codebooks,
        group_count,
        group_size,
        transform_dim: transform.transform_dim,
        signs: transform.signs,
    })
}

pub fn derive_grouped_pq4_code(source: &[f32], model: &GroupedPq4Model) -> Vec<u8> {
    let rotated = rotation::srht_padded(source, &model.signs);
    encode_grouped_pq(
        &rotated,
        model.codebooks.iter().map(Vec::as_slice),
        model.group_size,
    )
}

pub(crate) fn persisted_binary_sidecar_word_count(dimensions: u16, bits: u8, seed: u64) -> usize {
    crate::quant::rabitq::persisted_sidecar_word_count(dimensions, bits, seed)
}

pub(crate) fn derive_persisted_binary_words(quantizer: &ProdQuantizer, code: &[u8]) -> Vec<u64> {
    crate::quant::rabitq::derive_persisted_sidecar_words(quantizer, code)
}

fn sample_indices(len: usize, sample_count: usize, seed: u64) -> Vec<usize> {
    if sample_count >= len {
        return (0..len).collect();
    }

    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut indices = (0..len).collect::<Vec<_>>();
    for i in 0..sample_count {
        let swap_index = rng.gen_range(i..len);
        indices.swap(i, swap_index);
    }
    indices.truncate(sample_count);
    indices
}

fn train_group_codebook(
    samples: &[f32],
    group_size: usize,
    kmeans_iters: usize,
    seed: u64,
) -> Result<Vec<f32>, String> {
    const CENTROIDS: usize = 16;

    let sample_count = samples.len() / group_size;
    if sample_count == 0 {
        return Err("grouped codebook training requires at least one sample".to_owned());
    }
    if sample_count < CENTROIDS {
        return Ok(seed_group_codebook_from_small_samples(
            samples,
            group_size,
            sample_count,
            seed,
        ));
    }

    let init_indices = sample_indices(sample_count, CENTROIDS, seed);
    let mut centroids = vec![0.0_f32; CENTROIDS * group_size];
    for (centroid_index, sample_index) in init_indices.into_iter().enumerate() {
        let sample = sample_slice(samples, sample_index, group_size);
        centroid_slice_mut(&mut centroids, centroid_index, group_size).copy_from_slice(sample);
    }

    let mut assignments = vec![0usize; sample_count];
    let mut sums = vec![0.0_f32; CENTROIDS * group_size];
    let mut counts = [0usize; CENTROIDS];

    for _ in 0..kmeans_iters {
        sums.fill(0.0);
        counts.fill(0);

        for (sample_index, assignment) in assignments.iter_mut().enumerate() {
            let sample = sample_slice(samples, sample_index, group_size);
            let centroid_index = nearest_centroid_l2(sample, &centroids, group_size);
            *assignment = centroid_index;
            counts[centroid_index] += 1;
            let centroid_sum = centroid_slice_mut(&mut sums, centroid_index, group_size);
            for (dst, value) in centroid_sum.iter_mut().zip(sample.iter()) {
                *dst += *value;
            }
        }

        for (centroid_index, &count) in counts.iter().enumerate() {
            if count == 0 {
                let fallback_sample = sample_slice(
                    samples,
                    (seed as usize + centroid_index) % sample_count,
                    group_size,
                );
                centroid_slice_mut(&mut centroids, centroid_index, group_size)
                    .copy_from_slice(fallback_sample);
                continue;
            }

            let inv_count = (count as f32).recip();
            let centroid_sum = centroid_slice(&sums, centroid_index, group_size);
            let centroid = centroid_slice_mut(&mut centroids, centroid_index, group_size);
            for (dst, value) in centroid.iter_mut().zip(centroid_sum.iter()) {
                *dst = *value * inv_count;
            }
        }
    }

    Ok(centroids)
}

fn seed_group_codebook_from_small_samples(
    samples: &[f32],
    group_size: usize,
    sample_count: usize,
    seed: u64,
) -> Vec<f32> {
    const CENTROIDS: usize = 16;

    let mut centroids = vec![0.0_f32; CENTROIDS * group_size];
    for centroid_index in 0..CENTROIDS {
        let sample_index = (seed as usize + centroid_index) % sample_count;
        let sample = sample_slice(samples, sample_index, group_size);
        centroid_slice_mut(&mut centroids, centroid_index, group_size).copy_from_slice(sample);
    }
    centroids
}

fn sample_slice(samples: &[f32], sample_index: usize, group_size: usize) -> &[f32] {
    let start = sample_index * group_size;
    &samples[start..start + group_size]
}

fn centroid_slice(centroids: &[f32], centroid_index: usize, group_size: usize) -> &[f32] {
    let start = centroid_index * group_size;
    &centroids[start..start + group_size]
}

fn centroid_slice_mut(
    centroids: &mut [f32],
    centroid_index: usize,
    group_size: usize,
) -> &mut [f32] {
    let start = centroid_index * group_size;
    &mut centroids[start..start + group_size]
}

#[cfg(test)]
mod tests {
    use super::{derive_grouped_pq4_code, train_grouped_pq4_model, SrhtForwardTransform};
    use crate::quant::grouped_pq::GROUPED_PQ_CENTROIDS;

    #[test]
    fn srht_forward_transform_pads_to_effective_dimension() {
        let transform = SrhtForwardTransform::for_dimensions(13, 42);
        let rotated = transform.apply(&[1.0_f32; 13]);
        assert_eq!(rotated.len(), transform.transform_dim);
    }

    #[test]
    fn grouped_pq4_model_trains_and_derives_codes() {
        let source_vectors = (0..16)
            .map(|i| {
                (0..16)
                    .map(|dim| ((i * 17 + dim) as f32 * 0.07).sin())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let source_refs = source_vectors.iter().map(Vec::as_slice).collect::<Vec<_>>();

        let model = train_grouped_pq4_model(&source_refs, 16, 42, 4, 16, 3).unwrap();
        assert_eq!(model.group_size, 4);
        assert_eq!(model.group_count, 4);
        assert!(model
            .codebooks
            .iter()
            .all(|codebook| codebook.len() == 4 * GROUPED_PQ_CENTROIDS));

        let code = derive_grouped_pq4_code(source_refs[0], &model);
        assert_eq!(code.len(), model.group_count.div_ceil(2));
    }

    #[test]
    fn grouped_pq4_training_rejects_empty_input() {
        let source_refs: [&[f32]; 0] = [];
        let error = train_grouped_pq4_model(&source_refs, 16, 42, 4, 16, 3).unwrap_err();
        assert!(error.contains("source vector"));
    }
}
