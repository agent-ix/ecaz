use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::quant::{
    grouped_pq::{encode_grouped_pq, nearest_centroid_l2},
    rotation,
};

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
pub(crate) struct GroupedPq4Model {
    pub(crate) codebooks: Vec<Vec<f32>>,
    pub(crate) group_count: usize,
    pub(crate) group_size: usize,
    pub(crate) transform_dim: usize,
    pub(crate) signs: Vec<f32>,
}

pub(crate) fn train_grouped_pq4_model(
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

pub(crate) fn derive_grouped_pq4_code(source: &[f32], model: &GroupedPq4Model) -> Vec<u8> {
    let rotated = rotation::srht_padded(source, &model.signs);
    encode_grouped_pq(
        &rotated,
        model.codebooks.iter().map(Vec::as_slice),
        model.group_size,
    )
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
