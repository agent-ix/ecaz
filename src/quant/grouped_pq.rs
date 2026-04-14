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

#[cfg(test)]
mod tests {
    use super::pack_grouped_pq_nibbles;

    #[test]
    fn pack_grouped_pq_nibbles_packs_even_count() {
        assert_eq!(pack_grouped_pq_nibbles(&[0x1, 0x2, 0x3, 0x4]), vec![0x21, 0x43]);
    }

    #[test]
    fn pack_grouped_pq_nibbles_packs_odd_count() {
        assert_eq!(pack_grouped_pq_nibbles(&[0xA, 0xB, 0xC]), vec![0xBA, 0x0C]);
    }
}
