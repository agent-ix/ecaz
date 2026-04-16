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

pub fn nearest_centroid_l2(sample: &[f32], centroids: &[f32], group_size: usize) -> usize {
    assert!(group_size > 0, "grouped PQ group size must be positive");
    assert_eq!(
        sample.len(),
        group_size,
        "grouped PQ sample length {} must match group size {}",
        sample.len(),
        group_size
    );
    assert_eq!(
        centroids.len() % group_size,
        0,
        "grouped PQ centroid length {} must be divisible by group size {}",
        centroids.len(),
        group_size
    );
    assert_eq!(
        centroids.len() / group_size,
        GROUPED_PQ_CENTROIDS,
        "grouped PQ centroid count mismatch: got {}, expected {}",
        centroids.len() / group_size,
        GROUPED_PQ_CENTROIDS
    );

    let mut best_index = 0usize;
    let mut best_distance = squared_l2(sample, centroid_slice(centroids, 0, group_size));
    for centroid_index in 1..GROUPED_PQ_CENTROIDS {
        let distance = squared_l2(
            sample,
            centroid_slice(centroids, centroid_index, group_size),
        );
        if distance < best_distance {
            best_distance = distance;
            best_index = centroid_index;
        }
    }
    best_index
}

pub fn encode_grouped_pq<'a, I>(vector: &[f32], codebooks: I, group_size: usize) -> Vec<u8>
where
    I: IntoIterator<Item = &'a [f32]>,
{
    assert!(group_size > 0, "grouped PQ group size must be positive");
    let codebooks = codebooks.into_iter().collect::<Vec<_>>();
    assert_eq!(
        vector.len(),
        codebooks.len() * group_size,
        "grouped PQ vector length {} must match {} codebook groups of size {}",
        vector.len(),
        codebooks.len(),
        group_size
    );

    let mut centroid_indices = vec![0_u8; codebooks.len()];
    for (group_index, centroid_index) in centroid_indices.iter_mut().enumerate() {
        let start = group_index * group_size;
        let end = start + group_size;
        *centroid_index =
            nearest_centroid_l2(&vector[start..end], codebooks[group_index], group_size) as u8;
    }
    pack_grouped_pq_nibbles(&centroid_indices)
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
    debug_assert!(
        packed_nibbles.len() >= group_count.div_ceil(2),
        "grouped PQ packed nibble length {} is too short for group count {}",
        packed_nibbles.len(),
        group_count
    );
    // This scalar reference uses row-major `[group][centroid]` layout. Future SIMD
    // scorers can repack LUT bytes internally, but they should agree with this
    // stable logical layout and score on the same grouped-search codes.
    (0..group_count)
        .map(|group_index| {
            let centroid_index = grouped_pq_nibble(packed_nibbles, group_index);
            lut_f32[group_index * GROUPED_PQ_CENTROIDS + centroid_index]
        })
        .sum()
}

fn squared_l2(lhs: &[f32], rhs: &[f32]) -> f32 {
    lhs.iter()
        .zip(rhs.iter())
        .map(|(left, right)| {
            let delta = left - right;
            delta * delta
        })
        .sum()
}

fn centroid_slice(centroids: &[f32], centroid_index: usize, group_size: usize) -> &[f32] {
    let start = centroid_index * group_size;
    &centroids[start..start + group_size]
}

#[cfg(test)]
mod tests {
    use super::{
        build_grouped_pq_lut_f32, encode_grouped_pq, grouped_pq_nibble, grouped_pq_score_f32,
        nearest_centroid_l2, pack_grouped_pq_nibbles, GROUPED_PQ_CENTROIDS,
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

    #[test]
    fn nearest_centroid_l2_prefers_lowest_distance() {
        let centroids = vec![
            -1.0, -1.0, // 0
            1.0, 1.0, // 1
            4.0, 4.0, // 2
            9.0, 9.0, // 3
            20.0, 20.0, // 4
            21.0, 21.0, // 5
            22.0, 22.0, // 6
            23.0, 23.0, // 7
            24.0, 24.0, // 8
            25.0, 25.0, // 9
            26.0, 26.0, // 10
            27.0, 27.0, // 11
            28.0, 28.0, // 12
            29.0, 29.0, // 13
            30.0, 30.0, // 14
            31.0, 31.0, // 15
        ];

        assert_eq!(nearest_centroid_l2(&[1.5, 1.5], &centroids, 2), 1);
    }

    #[test]
    fn encode_grouped_pq_packs_nibbles_from_shared_codebooks() {
        let codebooks = [
            &[
                -1.0, 0.0, 0.0, 1.0, 10.0, 10.0, 10.0, 10.0, 20.0, 20.0, 20.0, 20.0, 30.0, 30.0,
                30.0, 30.0, 40.0, 40.0, 40.0, 40.0, 50.0, 50.0, 50.0, 50.0, 60.0, 60.0, 60.0, 60.0,
                70.0, 70.0, 70.0, 70.0,
            ][..],
            &[
                10.0, 10.0, 10.0, 10.0, -2.0, 0.0, 0.0, 2.0, 20.0, 20.0, 20.0, 20.0, 30.0, 30.0,
                30.0, 30.0, 40.0, 40.0, 40.0, 40.0, 50.0, 50.0, 50.0, 50.0, 60.0, 60.0, 60.0, 60.0,
                70.0, 70.0, 70.0, 70.0,
            ][..],
        ];

        let packed = encode_grouped_pq(&[1.0, 1.0, -2.0, -2.0], codebooks, 2);
        assert_eq!(packed, vec![0x21]);
        assert_eq!(grouped_pq_nibble(&packed, 0), 1);
        assert_eq!(grouped_pq_nibble(&packed, 1), 2);
    }
}
