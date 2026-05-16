#![flux::opts(scrape_quals = true)]

#[flux::sig(fn(dim: usize{0 < dim && dim <= 4096}) -> usize{v: v == dim + 1})]
pub fn payload_word_capacity(dim: usize) -> usize {
    dim + 1
}

#[flux::sig(fn(dim: usize{0 < dim && dim <= 4096}, idx: usize{idx < dim}) -> usize{v: 0 < v && v < dim + 1})]
pub fn payload_word_offset(dim: usize, idx: usize) -> usize {
    let _ = dim;
    idx + 1
}

#[flux::sig(fn(dim: usize{0 < dim && dim <= 4096}, idx: usize{idx < dim}) -> bool[true])]
pub fn payload_index_fits(dim: usize, idx: usize) -> bool {
    payload_word_offset(dim, idx) < payload_word_capacity(dim)
}
