#[cfg(test)]
mod tests {
    use loom::sync::atomic::{AtomicUsize, Ordering};
    use loom::sync::Arc;
    use loom::thread;

    #[test]
    fn worker_slot_claim_release_is_exclusive() {
        loom::model(|| {
            let slot = Arc::new(AtomicUsize::new(0));
            let wins = Arc::new(AtomicUsize::new(0));

            let mut handles = Vec::new();
            for worker_id in 1..=2 {
                let slot = Arc::clone(&slot);
                let wins = Arc::clone(&wins);
                handles.push(thread::spawn(move || {
                    if slot
                        .compare_exchange(0, worker_id, Ordering::AcqRel, Ordering::Acquire)
                        .is_ok()
                    {
                        wins.fetch_add(1, Ordering::AcqRel);
                        assert_eq!(slot.load(Ordering::Acquire), worker_id);
                        slot.store(0, Ordering::Release);
                    }
                }));
            }

            for handle in handles {
                handle.join().unwrap();
            }
            assert!(wins.load(Ordering::Acquire) <= 2);
            assert_eq!(slot.load(Ordering::Acquire), 0);
        });
    }
}
