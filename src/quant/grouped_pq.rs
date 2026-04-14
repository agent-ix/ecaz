pub const GROUPED_PQ_CENTROIDS: usize = 16;

pub fn pack_grouped_pq_nibbles(indices: &[u8]) -> Vec<u8> {
    let mut packed_nibbles = vec![0_u8; indices.len().div_ceil(2)];
    for (group_index, &centroid_index) in indices.iter().enumerate() {
        assert!(
            centroid_index < GROUPED_PQ_CENTROIDS as u8,
            "grouped PQ centroid index must fit in 4 bits"
        );
        if group_index % 2 == 0 {
            packed_nibbles[group_index / 2] = centroid_index;
        } else {
            packed_nibbles[group_index / 2] |= centroid_index << 4;
        }
    }
    packed_nibbles
}

pub fn grouped_pq_nibble(packed_nibbles: &[u8], group_index: usize) -> usize {
    let packed = packed_nibbles[group_index / 2];
    if group_index % 2 == 0 {
        usize::from(packed & 0x0F)
    } else {
        usize::from(packed >> 4)
    }
}

pub fn build_grouped_pq_lut_f32(
    rotated_query: &[f32],
    flat_codebooks: &[f32],
    group_size: usize,
) -> Vec<f32> {
    assert!(group_size > 0, "grouped PQ group size must be positive");
    assert_eq!(
        rotated_query.len() % group_size,
        0,
        "grouped PQ query length {} must be divisible by group size {}",
        rotated_query.len(),
        group_size
    );

    let group_count = rotated_query.len() / group_size;
    assert_eq!(
        flat_codebooks.len(),
        group_count * GROUPED_PQ_CENTROIDS * group_size,
        "grouped PQ codebook length mismatch: got {}, expected {}",
        flat_codebooks.len(),
        group_count * GROUPED_PQ_CENTROIDS * group_size
    );

    let mut lut = vec![0.0_f32; group_count * GROUPED_PQ_CENTROIDS];
    for group_index in 0..group_count {
        let query_group = &rotated_query[group_index * group_size..(group_index + 1) * group_size];
        let codebook_group = &flat_codebooks[group_index * GROUPED_PQ_CENTROIDS * group_size
            ..(group_index + 1) * GROUPED_PQ_CENTROIDS * group_size];
        let row =
            &mut lut[group_index * GROUPED_PQ_CENTROIDS..(group_index + 1) * GROUPED_PQ_CENTROIDS];

        for (centroid_index, slot) in row.iter_mut().enumerate() {
            let centroid =
                &codebook_group[centroid_index * group_size..(centroid_index + 1) * group_size];
            *slot = query_group
                .iter()
                .zip(centroid.iter())
                .map(|(query, value)| query * value)
                .sum();
        }
    }

    lut
}

pub fn grouped_pq_score_f32(lut_f32: &[f32], group_count: usize, packed_nibbles: &[u8]) -> f32 {
    (0..group_count)
        .map(|group_index| {
            let centroid_index = grouped_pq_nibble(packed_nibbles, group_index);
            lut_f32[group_index * GROUPED_PQ_CENTROIDS + centroid_index]
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::{
        build_grouped_pq_lut_f32, grouped_pq_nibble, grouped_pq_score_f32, pack_grouped_pq_nibbles,
        GROUPED_PQ_CENTROIDS,
    };

    #[test]
    fn pack_grouped_pq_nibbles_packs_even_count() {
        assert_eq!(
            pack_grouped_pq_nibbles(&[0x1, 0x2, 0x3, 0x4]),
            vec![0x21, 0x43]
        );
    }

    #[test]
    fn pack_grouped_pq_nibbles_packs_odd_count() {
        assert_eq!(pack_grouped_pq_nibbles(&[0xA, 0xB, 0xC]), vec![0xBA, 0x0C]);
    }

    #[test]
    fn grouped_pq_nibble_reads_even_and_odd_groups() {
        let packed = vec![0x21, 0x43, 0x05];
        assert_eq!(grouped_pq_nibble(&packed, 0), 0x1);
        assert_eq!(grouped_pq_nibble(&packed, 1), 0x2);
        assert_eq!(grouped_pq_nibble(&packed, 2), 0x3);
        assert_eq!(grouped_pq_nibble(&packed, 3), 0x4);
        assert_eq!(grouped_pq_nibble(&packed, 4), 0x5);
    }

    #[test]
    fn grouped_pq_score_f32_sums_lut_rows_by_nibble() {
        let packed = pack_grouped_pq_nibbles(&[1, 3, 2]);
        let mut lut = vec![0.0_f32; 3 * GROUPED_PQ_CENTROIDS];
        lut[1] = 1.5;
        lut[16 + 3] = -0.25;
        lut[32 + 2] = 2.0;

        assert_eq!(grouped_pq_score_f32(&lut, 3, &packed), 3.25);
    }

    #[test]
    fn build_grouped_pq_lut_f32_uses_flat_codebooks_by_group() {
        let query = vec![1.0_f32, 2.0, 3.0, 4.0];
        let mut flat_codebooks = Vec::with_capacity(2 * GROUPED_PQ_CENTROIDS * 2);
        for centroid in 1..=GROUPED_PQ_CENTROIDS {
            flat_codebooks.push(centroid as f32);
            flat_codebooks.push(0.0);
        }
        for centroid in 1..=GROUPED_PQ_CENTROIDS {
            flat_codebooks.push(0.0);
            flat_codebooks.push(centroid as f32);
        }

        let lut = build_grouped_pq_lut_f32(&query, &flat_codebooks, 2);

        assert_eq!(lut.len(), 2 * GROUPED_PQ_CENTROIDS);
        assert_eq!(lut[0], 1.0);
        assert_eq!(lut[1], 2.0);
        assert_eq!(lut[15], 16.0);
        assert_eq!(lut[16], 4.0);
        assert_eq!(lut[17], 8.0);
        assert_eq!(lut[31], 64.0);
    }
}
