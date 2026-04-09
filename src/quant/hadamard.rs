//! Fast Walsh-Hadamard Transform primitives.

/// In-place unnormalized FWHT.
///
/// The input length must be a power of two.
pub fn fwht_in_place(values: &mut [f32]) {
    assert!(
        values.len().is_power_of_two(),
        "fwht input length must be a power of two, got {}",
        values.len()
    );

    let mut width = 1;
    while width < values.len() {
        let step = width * 2;
        for chunk in values.chunks_exact_mut(step) {
            let (left, right) = chunk.split_at_mut(width);
            for i in 0..width {
                let a = left[i];
                let b = right[i];
                left[i] = a + b;
                right[i] = a - b;
            }
        }
        width = step;
    }
}

/// In-place tiled FWHT over equal-sized power-of-two chunks.
pub fn fwht_tiled_in_place(values: &mut [f32], tile_size: usize) {
    assert!(
        tile_size.is_power_of_two(),
        "fwht tile size must be a power of two, got {}",
        tile_size
    );
    assert!(
        values.len() % tile_size == 0,
        "fwht tiled input length {} must be divisible by tile size {}",
        values.len(),
        tile_size
    );
    for chunk in values.chunks_exact_mut(tile_size) {
        fwht_in_place(chunk);
    }
}

/// Apply the orthonormalized FWHT in place.
pub fn orthonormal_fwht_in_place(values: &mut [f32]) {
    fwht_in_place(values);
    let scale = (values.len() as f32).sqrt().recip();
    for value in values {
        *value *= scale;
    }
}

/// Apply an orthonormal tiled FWHT in place.
pub fn orthonormal_fwht_tiled_in_place(values: &mut [f32], tile_size: usize) {
    fwht_tiled_in_place(values, tile_size);
    let scale = (tile_size as f32).sqrt().recip();
    for value in values {
        *value *= scale;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fwht_preserves_norm_after_normalization() {
        let input = vec![1.0_f32, -2.0, 0.5, 3.25, -1.5, 4.0, 2.0, -0.25];
        let mut rotated = input.clone();
        orthonormal_fwht_in_place(&mut rotated);

        let input_norm = input.iter().map(|v| v * v).sum::<f32>().sqrt();
        let output_norm = rotated.iter().map(|v| v * v).sum::<f32>().sqrt();
        let rel_err = ((input_norm - output_norm) / input_norm.max(1.0)).abs();
        assert!(rel_err < 1e-5, "relative error = {rel_err}");
    }

    #[test]
    fn miri_fwht_small() {
        let mut data = vec![1.0f32, -2.0, 0.5, 3.0];
        fwht_in_place(&mut data);
    }

    #[test]
    fn miri_orthonormal_fwht_small() {
        let mut data = vec![1.0f32, -2.0, 0.5, 3.0, -1.5, 4.0, 2.0, -0.25];
        orthonormal_fwht_in_place(&mut data);
    }

    #[test]
    fn tiled_fwht_matches_chunkwise_full_fwht() {
        let mut tiled = (0..16).map(|i| i as f32 * 0.25 - 1.0).collect::<Vec<_>>();
        let mut chunkwise = tiled.clone();

        fwht_tiled_in_place(&mut tiled, 8);
        for chunk in chunkwise.chunks_exact_mut(8) {
            fwht_in_place(chunk);
        }

        assert_eq!(tiled, chunkwise);
    }

    #[test]
    fn tiled_orthonormal_fwht_preserves_norm_per_tile() {
        let input = (0..1536)
            .map(|i| (i as f32 * 0.001).sin())
            .collect::<Vec<_>>();
        let mut rotated = input.clone();
        orthonormal_fwht_tiled_in_place(&mut rotated, 512);

        let input_norm = input.iter().map(|v| v * v).sum::<f32>().sqrt();
        let output_norm = rotated.iter().map(|v| v * v).sum::<f32>().sqrt();
        let rel_err = ((input_norm - output_norm) / input_norm.max(1.0)).abs();
        assert!(rel_err < 1e-5, "relative error = {rel_err}");
    }
}
