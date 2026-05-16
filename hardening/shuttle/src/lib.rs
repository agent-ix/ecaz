#[cfg(test)]
mod tests {
    use shuttle::sync::{Arc, Mutex};
    use shuttle::thread;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct Candidate {
        score_micros: i32,
        node_id: u32,
        row_id: u32,
    }

    fn merge_top_k(mut rows: Vec<Candidate>, k: usize) -> Vec<Candidate> {
        rows.sort();
        rows.truncate(k);
        rows
    }

    #[test]
    fn coordinator_candidate_merge_is_order_independent() {
        shuttle::check_random(
            || {
                let rows = Arc::new(Mutex::new(Vec::new()));
                let mut handles = Vec::new();

                for candidate in [
                    Candidate {
                        score_micros: 20,
                        node_id: 2,
                        row_id: 1,
                    },
                    Candidate {
                        score_micros: 10,
                        node_id: 1,
                        row_id: 1,
                    },
                    Candidate {
                        score_micros: 10,
                        node_id: 1,
                        row_id: 0,
                    },
                ] {
                    let rows = Arc::clone(&rows);
                    handles.push(thread::spawn(move || {
                        rows.lock().unwrap().push(candidate);
                    }));
                }

                for handle in handles {
                    handle.join().unwrap();
                }

                let merged = merge_top_k(rows.lock().unwrap().clone(), 2);
                assert_eq!(
                    merged,
                    vec![
                        Candidate {
                            score_micros: 10,
                            node_id: 1,
                            row_id: 0,
                        },
                        Candidate {
                            score_micros: 10,
                            node_id: 1,
                            row_id: 1,
                        },
                    ]
                );
            },
            128,
        );
    }
}
