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
