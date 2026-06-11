use crate::quant::isa::Isa;
use std::time::Instant;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct BatchScoringTiming {
    pub(super) kernel_isa: Option<Isa>,
    pub(super) kernel_candidates: usize,
    pub(super) kernel_elapsed_nanos: u64,
    pub(super) scalar_candidates: usize,
    pub(super) scalar_elapsed_nanos: u64,
}

impl BatchScoringTiming {
    pub(super) fn record_run(
        &mut self,
        isa: Isa,
        candidate_count: usize,
        elapsed_nanos: u64,
        scalar_isa_counts_as_kernel: bool,
    ) {
        if candidate_count == 0 {
            return;
        }
        if isa == Isa::Scalar && !scalar_isa_counts_as_kernel {
            self.scalar_candidates += candidate_count;
            self.scalar_elapsed_nanos = self.scalar_elapsed_nanos.saturating_add(elapsed_nanos);
        } else {
            self.kernel_isa = Some(isa);
            self.kernel_candidates += candidate_count;
            self.kernel_elapsed_nanos = self.kernel_elapsed_nanos.saturating_add(elapsed_nanos);
        }
    }
}

pub(super) fn score_width_cascade<T: Copy, O>(
    items: &[T],
    out_scores: &mut [O],
    block_width: usize,
    scalar_block_counts_as_kernel: bool,
    mut score_block: impl FnMut(&[T], &mut [O]) -> Isa,
    mut score_remainder: impl FnMut(&[T], &mut [O], &mut BatchScoringTiming),
) -> BatchScoringTiming {
    debug_assert!(block_width > 0);
    debug_assert_eq!(items.len(), out_scores.len());

    let mut timing = BatchScoringTiming::default();
    let mut block_start = 0usize;
    while block_start + block_width <= items.len() {
        let block_end = block_start + block_width;
        let block_started = Instant::now();
        let isa = score_block(
            &items[block_start..block_end],
            &mut out_scores[block_start..block_end],
        );
        timing.record_run(
            isa,
            block_width,
            u64::try_from(block_started.elapsed().as_nanos()).unwrap_or(u64::MAX),
            scalar_block_counts_as_kernel,
        );
        block_start = block_end;
    }

    score_remainder(
        &items[block_start..],
        &mut out_scores[block_start..],
        &mut timing,
    );
    timing
}
