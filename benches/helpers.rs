//! Shared benchmark helpers: seeded data generation.

use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Generate a random unit vector of the given dimension, seeded for reproducibility.
pub fn random_unit_vector(dim: usize, seed: u64) -> Vec<f32> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut values: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
    let norm = values.iter().map(|v| v * v).sum::<f32>().sqrt();
    for value in &mut values {
        *value /= norm.max(f32::EPSILON);
    }
    values
}

/// Generate a corpus of `n` random unit vectors.
#[allow(dead_code)]
pub fn random_corpus(n: usize, dim: usize, base_seed: u64) -> Vec<Vec<f32>> {
    (0..n)
        .map(|i| random_unit_vector(dim, base_seed + i as u64))
        .collect()
}

/// Generate a corpus of `n` clustered unit vectors (Gaussian mixture model).
///
/// Creates `n_clusters` random cluster centers, then generates `n / n_clusters`
/// vectors per cluster by adding Gaussian noise with standard deviation `spread`
/// and re-normalizing to unit length. This produces realistic angular clustering
/// similar to real embedding models.
#[allow(dead_code)]
pub fn random_clustered_corpus(
    dim: usize,
    n: usize,
    n_clusters: usize,
    spread: f32,
    seed: u64,
) -> Vec<Vec<f32>> {
    // Generate cluster centers
    let centers: Vec<Vec<f32>> = (0..n_clusters)
        .map(|i| random_unit_vector(dim, seed + 100_000 + i as u64))
        .collect();

    let mut rng = ChaCha8Rng::seed_from_u64(seed + 200_000);
    let mut corpus = Vec::with_capacity(n);

    for i in 0..n {
        let center = &centers[i % n_clusters];
        // Add Gaussian noise via Box-Muller transform
        let mut vec: Vec<f32> = center
            .iter()
            .map(|&c| {
                let u1: f32 = rng.gen_range(0.0001f32..1.0);
                let u2: f32 = rng.gen_range(0.0f32..std::f32::consts::TAU);
                let noise = (-2.0 * u1.ln()).sqrt() * u2.cos() * spread;
                c + noise
            })
            .collect();
        // Re-normalize to unit length
        let norm = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
        for v in &mut vec {
            *v /= norm.max(f32::EPSILON);
        }
        corpus.push(vec);
    }
    corpus
}

/// Generate pairs of near-duplicate unit vectors at a controlled angular distance.
///
/// Returns `(base_vectors, perturbed_vectors)` where each perturbed vector is
/// approximately `angle_radians` away from its corresponding base vector.
/// Useful for stress-testing quantization precision at small angular differences.
#[allow(dead_code)]
pub fn near_duplicate_pairs(
    dim: usize,
    n: usize,
    angle_radians: f32,
    seed: u64,
) -> (Vec<Vec<f32>>, Vec<Vec<f32>>) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed + 300_000);
    let mut bases = Vec::with_capacity(n);
    let mut perturbed = Vec::with_capacity(n);

    for i in 0..n {
        let base = random_unit_vector(dim, seed + i as u64);

        // Generate a random perturbation orthogonal to base, then mix
        let mut noise: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0f32..1.0)).collect();
        // Gram-Schmidt: remove projection onto base
        let dot: f32 = noise.iter().zip(base.iter()).map(|(n, b)| n * b).sum();
        for (n, b) in noise.iter_mut().zip(base.iter()) {
            *n -= dot * b;
        }
        let noise_norm = noise.iter().map(|v| v * v).sum::<f32>().sqrt();
        if noise_norm < f32::EPSILON {
            // Degenerate case: just use the base
            perturbed.push(base.clone());
            bases.push(base);
            continue;
        }
        for v in &mut noise {
            *v /= noise_norm;
        }

        // Rotate base toward noise by angle_radians
        let cos_a = angle_radians.cos();
        let sin_a = angle_radians.sin();
        let pert: Vec<f32> = base
            .iter()
            .zip(noise.iter())
            .map(|(&b, &n)| cos_a * b + sin_a * n)
            .collect();
        // Already unit-length by construction, but renormalize for safety
        let norm = pert.iter().map(|v| v * v).sum::<f32>().sqrt();
        let pert: Vec<f32> = pert.iter().map(|v| v / norm.max(f32::EPSILON)).collect();

        bases.push(base);
        perturbed.push(pert);
    }
    (bases, perturbed)
}
