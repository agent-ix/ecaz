#[path = "../../../src/am/ec_spire/coordinator/remote_candidates/candidate_merge_model.rs"]
mod candidate_merge_model;
#[path = "../../../src/am/ec_spire/update/epoch_publish_model.rs"]
mod epoch_publish_model;

#[cfg(test)]
mod tests {
    use super::candidate_merge_model::{
        merge_candidate_model_inputs, SpireCandidateMergeModelInput,
    };
    use super::epoch_publish_model::{SpireEpochPublishModel, SpireEpochPublishVisibility};
    use shuttle::sync::{Arc, Mutex, RwLock};
    use shuttle::thread;

    fn candidate(
        input_index: usize,
        node_id: u32,
        score: f32,
        dedupe_key: &[u8],
    ) -> SpireCandidateMergeModelInput {
        SpireCandidateMergeModelInput {
            input_index,
            dedupe_key: dedupe_key.to_vec(),
            served_epoch: 7,
            node_id,
            pid: u64::from(node_id),
            object_version: 1,
            row_index: 0,
            assignment_role_rank: 0,
            row_locator: vec![node_id as u8],
            score,
        }
    }

    #[test]
    fn candidate_merge_is_order_invariant_under_concurrent_receive() {
        shuttle::check_random(
            || {
                let received = Arc::new(Mutex::new(Vec::new()));

                let first = {
                    let received = Arc::clone(&received);
                    thread::spawn(move || {
                        received
                            .lock()
                            .unwrap()
                            .push(candidate(0, 2, 0.4, b"shared-global-vec"));
                    })
                };
                let second = {
                    let received = Arc::clone(&received);
                    thread::spawn(move || {
                        received
                            .lock()
                            .unwrap()
                            .push(candidate(1, 3, 0.2, b"shared-global-vec"));
                    })
                };

                first.join().unwrap();
                second.join().unwrap();

                let merged =
                    merge_candidate_model_inputs(received.lock().unwrap().clone(), Some(1))
                        .expect("candidate merge should succeed");
                assert_eq!(merged.input_count, 2);
                assert_eq!(merged.duplicate_vec_id_count, 1);
                assert_eq!(merged.selected_input_indices, vec![1]);
            },
            128,
        );
    }

    #[test]
    fn epoch_publish_visibility_never_exposes_partial_replacement() {
        shuttle::check_random(
            || {
                let model = Arc::new(RwLock::new(SpireEpochPublishModel::new(7)));
                let observed = Arc::new(Mutex::new(Vec::new()));

                let writer = {
                    let model = Arc::clone(&model);
                    thread::spawn(move || {
                        let mut model = model.write().unwrap();
                        model.begin_publish(8).expect("publish should begin");
                        assert_eq!(
                            model.scanner_visibility(),
                            SpireEpochPublishVisibility::Old { epoch: 7 }
                        );
                        thread::yield_now();
                        model.commit_publish().expect("publish should commit");
                    })
                };
                let scanner = {
                    let model = Arc::clone(&model);
                    let observed = Arc::clone(&observed);
                    thread::spawn(move || {
                        for _ in 0..2 {
                            let visibility = model.read().unwrap().scanner_visibility();
                            observed.lock().unwrap().push(visibility);
                            thread::yield_now();
                        }
                    })
                };

                writer.join().unwrap();
                scanner.join().unwrap();

                let final_visibility = model.read().unwrap().active_visibility();
                assert_eq!(
                    final_visibility,
                    SpireEpochPublishVisibility::New { epoch: 8 }
                );
                for visibility in observed.lock().unwrap().iter().copied() {
                    assert!(
                        matches!(
                            visibility,
                            SpireEpochPublishVisibility::Old { epoch: 7 }
                                | SpireEpochPublishVisibility::New { epoch: 7 }
                                | SpireEpochPublishVisibility::New { epoch: 8 }
                        ),
                        "scanner observed partial epoch state: {visibility:?}"
                    );
                }
            },
            128,
        );
    }
}
