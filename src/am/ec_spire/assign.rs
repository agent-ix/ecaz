//! Leaf PID and vector-identity assignment helpers.

use super::storage::SpireVecId;

pub(super) const SPIRE_FIRST_LOCAL_VEC_SEQ: u64 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireLocalVecIdAllocator {
    next_local_vec_seq: u64,
}

impl Default for SpireLocalVecIdAllocator {
    fn default() -> Self {
        Self {
            next_local_vec_seq: SPIRE_FIRST_LOCAL_VEC_SEQ,
        }
    }
}

impl SpireLocalVecIdAllocator {
    pub(super) fn new(next_local_vec_seq: u64) -> Result<Self, String> {
        if next_local_vec_seq == 0 {
            return Err("ec_spire local vec_id sequence 0 is invalid".to_owned());
        }
        Ok(Self { next_local_vec_seq })
    }

    pub(super) fn next_local_vec_seq(&self) -> u64 {
        self.next_local_vec_seq
    }

    pub(super) fn allocate(&mut self) -> Result<SpireVecId, String> {
        let local_vec_seq = self.next_local_vec_seq;
        let next = local_vec_seq
            .checked_add(1)
            .ok_or_else(|| "ec_spire local vec_id sequence exhausted".to_owned())?;
        self.next_local_vec_seq = next;
        Ok(SpireVecId::local(local_vec_seq))
    }

    pub(super) fn observe(&mut self, vec_id: &SpireVecId) -> Result<(), String> {
        let Some(local_vec_seq) = vec_id.local_sequence() else {
            return Ok(());
        };
        let next = local_vec_seq
            .checked_add(1)
            .ok_or_else(|| "ec_spire observed local vec_id sequence is exhausted".to_owned())?;
        if next > self.next_local_vec_seq {
            self.next_local_vec_seq = next;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{SpireLocalVecIdAllocator, SPIRE_FIRST_LOCAL_VEC_SEQ};
    use crate::am::ec_spire::storage::SpireVecId;

    #[test]
    fn allocator_starts_at_first_local_sequence() {
        let mut allocator = SpireLocalVecIdAllocator::default();

        let first = allocator.allocate().unwrap();
        let second = allocator.allocate().unwrap();

        assert_eq!(first.local_sequence(), Some(SPIRE_FIRST_LOCAL_VEC_SEQ));
        assert_eq!(second.local_sequence(), Some(SPIRE_FIRST_LOCAL_VEC_SEQ + 1));
        assert_eq!(
            allocator.next_local_vec_seq(),
            SPIRE_FIRST_LOCAL_VEC_SEQ + 2
        );
    }

    #[test]
    fn allocator_rejects_zero_next_sequence() {
        assert!(SpireLocalVecIdAllocator::new(0).is_err());
    }

    #[test]
    fn allocator_observes_local_ids_without_rewinding() {
        let mut allocator = SpireLocalVecIdAllocator::new(10).unwrap();

        allocator.observe(&SpireVecId::local(20)).unwrap();
        assert_eq!(allocator.next_local_vec_seq(), 21);

        allocator.observe(&SpireVecId::local(5)).unwrap();
        assert_eq!(allocator.next_local_vec_seq(), 21);
    }

    #[test]
    fn allocator_ignores_global_ids() {
        let mut allocator = SpireLocalVecIdAllocator::new(10).unwrap();

        allocator
            .observe(&SpireVecId::global(&[1, 2, 3]).unwrap())
            .unwrap();

        assert_eq!(allocator.next_local_vec_seq(), 10);
    }

    #[test]
    fn allocator_reports_sequence_exhaustion_without_advancing() {
        let mut allocator = SpireLocalVecIdAllocator::new(u64::MAX).unwrap();

        assert!(allocator.allocate().is_err());
        assert_eq!(allocator.next_local_vec_seq(), u64::MAX);
        assert!(allocator.observe(&SpireVecId::local(u64::MAX)).is_err());
        assert_eq!(allocator.next_local_vec_seq(), u64::MAX);
    }
}
