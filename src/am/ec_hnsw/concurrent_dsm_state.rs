pub(crate) const EC_HNSW_CONCURRENT_DSM_INSERT_STATE_UNINSERTED: u32 = 0;
pub(crate) const EC_HNSW_CONCURRENT_DSM_INSERT_STATE_INSERTING: u32 = 1;
pub(crate) const EC_HNSW_CONCURRENT_DSM_INSERT_STATE_READY: u32 = 2;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum EcHnswConcurrentDsmInsertBegin {
    AlreadyReady,
    Started { level: u8 },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum EcHnswConcurrentDsmInsertError {
    DuplicateInProgress,
    UnknownState(u32),
    CompleteWithoutInsert,
}

pub(crate) trait EcHnswConcurrentDsmInsertStateCell {
    fn load_acquire(&self) -> u32;
    fn store_release(&self, value: u32);
    fn compare_exchange_acqrel_acquire(&self, current: u32, new: u32) -> bool;
    fn spin_wait(&self) {
        std::hint::spin_loop();
    }
}

pub(crate) fn begin_concurrent_dsm_node_insert_state<C>(
    cell: &C,
    level: u8,
) -> Result<EcHnswConcurrentDsmInsertBegin, EcHnswConcurrentDsmInsertError>
where
    C: EcHnswConcurrentDsmInsertStateCell,
{
    match cell.load_acquire() {
        EC_HNSW_CONCURRENT_DSM_INSERT_STATE_READY => {
            Ok(EcHnswConcurrentDsmInsertBegin::AlreadyReady)
        }
        EC_HNSW_CONCURRENT_DSM_INSERT_STATE_UNINSERTED => {
            if cell.compare_exchange_acqrel_acquire(
                EC_HNSW_CONCURRENT_DSM_INSERT_STATE_UNINSERTED,
                EC_HNSW_CONCURRENT_DSM_INSERT_STATE_INSERTING,
            ) {
                Ok(EcHnswConcurrentDsmInsertBegin::Started { level })
            } else if cell.load_acquire() == EC_HNSW_CONCURRENT_DSM_INSERT_STATE_INSERTING {
                Err(EcHnswConcurrentDsmInsertError::DuplicateInProgress)
            } else {
                begin_concurrent_dsm_node_insert_state(cell, level)
            }
        }
        EC_HNSW_CONCURRENT_DSM_INSERT_STATE_INSERTING => {
            Err(EcHnswConcurrentDsmInsertError::DuplicateInProgress)
        }
        state => Err(EcHnswConcurrentDsmInsertError::UnknownState(state)),
    }
}

pub(crate) fn complete_concurrent_dsm_node_insert_state<C>(
    cell: &C,
) -> Result<(), EcHnswConcurrentDsmInsertError>
where
    C: EcHnswConcurrentDsmInsertStateCell,
{
    if cell.compare_exchange_acqrel_acquire(
        EC_HNSW_CONCURRENT_DSM_INSERT_STATE_INSERTING,
        EC_HNSW_CONCURRENT_DSM_INSERT_STATE_READY,
    ) {
        return Ok(());
    }

    match cell.load_acquire() {
        EC_HNSW_CONCURRENT_DSM_INSERT_STATE_READY => Ok(()),
        EC_HNSW_CONCURRENT_DSM_INSERT_STATE_UNINSERTED => {
            Err(EcHnswConcurrentDsmInsertError::CompleteWithoutInsert)
        }
        EC_HNSW_CONCURRENT_DSM_INSERT_STATE_INSERTING => {
            Err(EcHnswConcurrentDsmInsertError::DuplicateInProgress)
        }
        state => Err(EcHnswConcurrentDsmInsertError::UnknownState(state)),
    }
}

pub(crate) fn wait_until_concurrent_dsm_node_ready<C>(cell: &C) -> bool
where
    C: EcHnswConcurrentDsmInsertStateCell,
{
    loop {
        match cell.load_acquire() {
            EC_HNSW_CONCURRENT_DSM_INSERT_STATE_READY => return true,
            EC_HNSW_CONCURRENT_DSM_INSERT_STATE_INSERTING => cell.spin_wait(),
            EC_HNSW_CONCURRENT_DSM_INSERT_STATE_UNINSERTED => return false,
            _ => return false,
        }
    }
}
