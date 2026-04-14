pub fn pack_grouped_pq_nibbles(indices: &[u8]) -> Vec<u8> {
    let mut packed_nibbles = vec![0_u8; indices.len().div_ceil(2)];
    for (group_index, &centroid_index) in indices.iter().enumerate() {
        assert!(
            centroid_index < 16,
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

pub fn grouped_pq_score_f32(lut_f32: &[f32], group_count: usize, packed_nibbles: &[u8]) -> f32 {
    (0..group_count)
        .map(|group_index| {
            let centroid_index = grouped_pq_nibble(packed_nibbles, group_index);
            lut_f32[group_index * 16 + centroid_index]
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::{grouped_pq_nibble, grouped_pq_score_f32, pack_grouped_pq_nibbles};

    #[test]
    fn pack_grouped_pq_nibbles_packs_even_count() {
        assert_eq!(pack_grouped_pq_nibbles(&[0x1, 0x2, 0x3, 0x4]), vec![0x21, 0x43]);
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
        let mut lut = vec![0.0_f32; 3 * 16];
        lut[1] = 1.5;
        lut[16 + 3] = -0.25;
        lut[32 + 2] = 2.0;

        assert_eq!(grouped_pq_score_f32(&lut, 3, &packed), 3.25);
    }
}
