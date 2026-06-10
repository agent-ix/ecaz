use crate::quant::isa::Isa;

pub(super) fn score_run_scalar(
    lut: &[f32],
    tile_size: usize,
    original_dim: usize,
    codes: &[&[u8]],
    out_scores: &mut [f32],
) -> Isa {
    debug_assert_eq!(out_scores.len(), codes.len());
    out_scores.fill(0.0);

    // Tile-outer / candidate-inner: each LUT tile stays hot across the
    // whole run, and per candidate the accumulation order is identical to
    // the production tiled scorer (tile-major, dim-ascending) so single
    // scores are bit-equal with the per-candidate path.
    let mut tile_start = 0usize;
    while tile_start < original_dim {
        let tile_end = (tile_start + tile_size).min(original_dim);
        let tile_lut = &lut[tile_start * 16..tile_end * 16];
        for (code, out) in codes.iter().zip(out_scores.iter_mut()) {
            // Single running accumulator per candidate, matching the
            // production scorer's order exactly (no per-tile partial sum).
            for dim_index in tile_start..tile_end {
                let packed = code[dim_index / 2];
                let centroid_index = if dim_index % 2 == 0 {
                    packed & 0x0F
                } else {
                    packed >> 4
                } as usize;
                let tile_dim = dim_index - tile_start;
                *out += tile_lut[tile_dim * 16 + centroid_index];
            }
        }
        tile_start = tile_end;
    }
    Isa::Scalar
}

#[allow(dead_code)]
pub(super) fn score_candidate(
    lut: &[f32],
    tile_size: usize,
    original_dim: usize,
    mse_packed: &[u8],
) -> f32 {
    let mut score = 0.0_f32;
    score_run_scalar(
        lut,
        tile_size,
        original_dim,
        &[mse_packed],
        std::slice::from_mut(&mut score),
    );
    score
}
