#![no_main]
//! Task 46 §Approach 2.a: structure-aware top-k merge property test.
//!
//! Draws two `Arbitrary`-derived `Vec<i64>` of bounded length, sorts
//! each, runs a single-pass O(k) merge-truncate, and asserts the
//! result equals the slower reference (`concat -> sort -> truncate`).
//! This is a property test of the merge primitive itself; future
//! ECAZ code can pick up the same primitive and the target will then
//! validate it directly.

use libfuzzer_sys::fuzz_target;

#[derive(Debug, arbitrary::Arbitrary)]
struct StructuredInput {
    /// Maps to `k` in `1..=4096`.
    k_minus_one: u16,
    /// Both lists drawn independently. Bounded length by libFuzzer
    /// `max_len`; sorted in-place before merge.
    a: Vec<i64>,
    b: Vec<i64>,
}

/// Single-pass O(k) merge-truncate over two ascending-sorted inputs.
/// This is the "system under test" — the property assertion checks
/// it against the slower but obviously-correct reference of
/// `concat -> sort -> truncate`.
fn merge_truncate_ascending(a: &[i64], b: &[i64], k: usize) -> Vec<i64> {
    let mut result = Vec::with_capacity(k.min(a.len() + b.len()));
    let (mut i, mut j) = (0, 0);
    while result.len() < k {
        match (a.get(i), b.get(j)) {
            (Some(x), Some(y)) => {
                if x <= y {
                    result.push(*x);
                    i += 1;
                } else {
                    result.push(*y);
                    j += 1;
                }
            }
            (Some(x), None) => {
                result.push(*x);
                i += 1;
            }
            (None, Some(y)) => {
                result.push(*y);
                j += 1;
            }
            (None, None) => break,
        }
    }
    result
}

fuzz_target!(|input: StructuredInput| {
    let k = 1 + (input.k_minus_one as usize % 4096);
    let mut a = input.a.clone();
    let mut b = input.b.clone();
    a.sort();
    b.sort();

    let merged = merge_truncate_ascending(&a, &b, k);

    let mut reference: Vec<i64> = a.iter().chain(b.iter()).copied().collect();
    reference.sort();
    reference.truncate(k);

    assert_eq!(
        merged, reference,
        "merge-truncate must equal sort-truncate for k={k}, |a|={}, |b|={}",
        a.len(),
        b.len(),
    );
});
