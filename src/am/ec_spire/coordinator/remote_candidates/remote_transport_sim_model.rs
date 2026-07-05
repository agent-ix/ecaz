pub(crate) const SPIRE_REMOTE_SIM_STATUS_READY: &str = "ready";
pub(crate) const SPIRE_REMOTE_SIM_STATUS_REQUIRES_RECEIVE: &str = "requires_candidate_receive";
pub(crate) const SPIRE_REMOTE_SIM_STATUS_RECEIVE_FAILED: &str = "remote_candidate_receive_failed";
pub(crate) const SPIRE_REMOTE_SIM_STATUS_DEGRADED_SKIPPED: &str = "degraded_skipped";

const SPIRE_REMOTE_SIM_WIRE_VERSION: u8 = 1;
const SPIRE_REMOTE_SIM_WIRE_REQUEST: u8 = 1;
const SPIRE_REMOTE_SIM_WIRE_RESPONSE: u8 = 2;
const SPIRE_REMOTE_SIM_WIRE_STATUS_READY: u8 = 1;
const SPIRE_REMOTE_SIM_WIRE_STATUS_FAILED: u8 = 2;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum SpireRemoteTransportSimConsistency {
    Strict,
    Degraded,
}

impl SpireRemoteTransportSimConsistency {
    pub(crate) fn parse(value: &str) -> Result<Self, &'static str> {
        match value {
            "strict" => Ok(Self::Strict),
            "degraded" => Ok(Self::Degraded),
            _ => Err("unsupported consistency mode"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteTransportSimRequest {
    pub(crate) requested_epoch: u64,
    pub(crate) node_id: u32,
    pub(crate) selected_pids: Vec<u64>,
    pub(crate) top_k: usize,
    pub(crate) consistency: SpireRemoteTransportSimConsistency,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpireRemoteTransportSimCandidate {
    pub(crate) served_epoch: u64,
    pub(crate) node_id: u32,
    pub(crate) pid: u64,
    pub(crate) object_version: u64,
    pub(crate) row_index: u32,
    pub(crate) assignment_role_rank: u8,
    pub(crate) row_locator: Vec<u8>,
    pub(crate) dedupe_key: Vec<u8>,
    pub(crate) score: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpireRemoteTransportSimResponse {
    pub(crate) node_id: u32,
    pub(crate) served_epoch: u64,
    pub(crate) candidates: Vec<SpireRemoteTransportSimCandidate>,
    pub(crate) failure_category: Option<&'static str>,
}

impl SpireRemoteTransportSimResponse {
    pub(crate) fn ready(
        node_id: u32,
        served_epoch: u64,
        candidates: Vec<SpireRemoteTransportSimCandidate>,
    ) -> Self {
        Self {
            node_id,
            served_epoch,
            candidates,
            failure_category: None,
        }
    }

    pub(crate) fn failed(node_id: u32, served_epoch: u64, failure_category: &'static str) -> Self {
        Self {
            node_id,
            served_epoch,
            candidates: Vec::new(),
            failure_category: Some(failure_category),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum SpireRemoteTransportSimDispatchState {
    Planned,
    CandidateReceiveReady,
    CandidateReceiveFailed,
    DegradedSkipped,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpireRemoteTransportSimDispatch {
    pub(crate) request: SpireRemoteTransportSimRequest,
    pub(crate) state: SpireRemoteTransportSimDispatchState,
    pub(crate) status: &'static str,
    pub(crate) failure_category: &'static str,
    pub(crate) candidates: Vec<SpireRemoteTransportSimCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRemoteTransportSimSummary {
    pub(crate) dispatch_count: u64,
    pub(crate) ready_dispatch_count: u64,
    pub(crate) failed_dispatch_count: u64,
    pub(crate) degraded_skipped_dispatch_count: u64,
    pub(crate) selected_candidate_count: u64,
    pub(crate) selected_input_nodes: Vec<u32>,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpireRemoteTransportSimState {
    consistency: SpireRemoteTransportSimConsistency,
    dispatches: Vec<SpireRemoteTransportSimDispatch>,
}

pub(crate) trait SpireRemoteTransportSimAdapter {
    fn receive_candidates(
        &mut self,
        request: &SpireRemoteTransportSimRequest,
    ) -> Result<SpireRemoteTransportSimResponse, &'static str>;
}

impl SpireRemoteTransportSimState {
    pub(crate) fn new(
        consistency: SpireRemoteTransportSimConsistency,
        requests: Vec<SpireRemoteTransportSimRequest>,
    ) -> Result<Self, String> {
        for request in &requests {
            if request.consistency != consistency {
                return Err("ec_spire remote transport sim request consistency mismatch".to_owned());
            }
            if request.selected_pids.is_empty() {
                return Err("ec_spire remote transport sim request has no selected PIDs".to_owned());
            }
        }

        Ok(Self {
            consistency,
            dispatches: requests
                .into_iter()
                .map(|request| SpireRemoteTransportSimDispatch {
                    request,
                    state: SpireRemoteTransportSimDispatchState::Planned,
                    status: SPIRE_REMOTE_SIM_STATUS_REQUIRES_RECEIVE,
                    failure_category: "none",
                    candidates: Vec::new(),
                })
                .collect(),
        })
    }

    pub(crate) fn receive_with_adapter<A>(
        &mut self,
        adapter: &mut A,
    ) -> Result<SpireRemoteTransportSimSummary, String>
    where
        A: SpireRemoteTransportSimAdapter,
    {
        let requests = self
            .dispatches
            .iter()
            .map(|dispatch| dispatch.request.clone())
            .collect::<Vec<_>>();
        for request in &requests {
            match adapter.receive_candidates(request) {
                Ok(response) => self.apply_response(response)?,
                Err(failure_category) => self.apply_failure(request.node_id, failure_category)?,
            }
        }
        self.summary(None)
    }

    pub(crate) fn apply_response(
        &mut self,
        response: SpireRemoteTransportSimResponse,
    ) -> Result<(), String> {
        let consistency = self.consistency;
        let dispatch = self.dispatch_for_node_mut(response.node_id)?;
        if dispatch.state != SpireRemoteTransportSimDispatchState::Planned {
            return Err(format!(
                "ec_spire remote transport sim duplicate receive for node_id {}",
                response.node_id
            ));
        }

        if let Some(failure_category) = response.failure_category {
            dispatch.apply_failure(consistency, failure_category);
            return Ok(());
        }

        if response.served_epoch != dispatch.request.requested_epoch {
            dispatch.apply_failure(consistency, "served_epoch_mismatch");
            return Ok(());
        }

        for candidate in &response.candidates {
            if candidate.served_epoch != dispatch.request.requested_epoch {
                dispatch.apply_failure(consistency, "served_epoch_mismatch");
                return Ok(());
            }
            if candidate.node_id != dispatch.request.node_id {
                dispatch.apply_failure(consistency, "node_id_mismatch");
                return Ok(());
            }
            if !dispatch.request.selected_pids.contains(&candidate.pid) {
                dispatch.apply_failure(consistency, "pid_not_selected");
                return Ok(());
            }
            if !candidate.score.is_finite() {
                dispatch.apply_failure(consistency, "non_finite_score");
                return Ok(());
            }
        }

        dispatch.candidates = response.candidates;
        dispatch.state = SpireRemoteTransportSimDispatchState::CandidateReceiveReady;
        dispatch.status = SPIRE_REMOTE_SIM_STATUS_READY;
        dispatch.failure_category = "none";
        Ok(())
    }

    pub(crate) fn apply_failure(
        &mut self,
        node_id: u32,
        failure_category: &'static str,
    ) -> Result<(), String> {
        let consistency = self.consistency;
        let dispatch = self.dispatch_for_node_mut(node_id)?;
        if dispatch.state != SpireRemoteTransportSimDispatchState::Planned {
            return Err(format!(
                "ec_spire remote transport sim duplicate failure for node_id {node_id}"
            ));
        }
        dispatch.apply_failure(consistency, failure_category);
        Ok(())
    }

    pub(crate) fn summary(
        &self,
        limit: Option<usize>,
    ) -> Result<SpireRemoteTransportSimSummary, String> {
        let mut ready_dispatch_count = 0_u64;
        let mut failed_dispatch_count = 0_u64;
        let mut degraded_skipped_dispatch_count = 0_u64;
        let mut merge_inputs = Vec::new();
        let mut input_nodes = Vec::new();

        for dispatch in &self.dispatches {
            match dispatch.state {
                SpireRemoteTransportSimDispatchState::CandidateReceiveReady => {
                    ready_dispatch_count =
                        ready_dispatch_count.checked_add(1).ok_or_else(|| {
                            "ec_spire remote transport sim ready count overflow".to_owned()
                        })?;
                    for candidate in &dispatch.candidates {
                        let input_index = merge_inputs.len();
                        input_nodes.push(candidate.node_id);
                        merge_inputs.push(SpireCandidateMergeModelInput {
                            input_index,
                            dedupe_key: candidate.dedupe_key.clone(),
                            served_epoch: candidate.served_epoch,
                            node_id: candidate.node_id,
                            pid: candidate.pid,
                            object_version: candidate.object_version,
                            row_index: candidate.row_index,
                            assignment_role_rank: candidate.assignment_role_rank,
                            row_locator: candidate.row_locator.clone(),
                            score: candidate.score,
                        });
                    }
                }
                SpireRemoteTransportSimDispatchState::CandidateReceiveFailed => {
                    failed_dispatch_count =
                        failed_dispatch_count.checked_add(1).ok_or_else(|| {
                            "ec_spire remote transport sim failed count overflow".to_owned()
                        })?;
                }
                SpireRemoteTransportSimDispatchState::DegradedSkipped => {
                    degraded_skipped_dispatch_count = degraded_skipped_dispatch_count
                        .checked_add(1)
                        .ok_or_else(|| {
                            "ec_spire remote transport sim degraded skip count overflow".to_owned()
                        })?;
                }
                SpireRemoteTransportSimDispatchState::Planned => {
                    failed_dispatch_count =
                        failed_dispatch_count.checked_add(1).ok_or_else(|| {
                            "ec_spire remote transport sim pending count overflow".to_owned()
                        })?;
                }
            }
        }

        let merged = merge_candidate_model_inputs(merge_inputs, limit)?;
        let selected_input_nodes = merged
            .selected_input_indices
            .iter()
            .map(|input_index| input_nodes[*input_index])
            .collect::<Vec<_>>();
        let selected_candidate_count = u64::try_from(selected_input_nodes.len()).map_err(|_| {
            "ec_spire remote transport sim selected candidate count exceeds u64".to_owned()
        })?;
        let dispatch_count = u64::try_from(self.dispatches.len())
            .map_err(|_| "ec_spire remote transport sim dispatch count exceeds u64".to_owned())?;
        let status = if failed_dispatch_count > 0
            && self.consistency == SpireRemoteTransportSimConsistency::Strict
        {
            SPIRE_REMOTE_SIM_STATUS_RECEIVE_FAILED
        } else if ready_dispatch_count > 0 {
            SPIRE_REMOTE_SIM_STATUS_READY
        } else if degraded_skipped_dispatch_count > 0 {
            SPIRE_REMOTE_SIM_STATUS_DEGRADED_SKIPPED
        } else {
            SPIRE_REMOTE_SIM_STATUS_RECEIVE_FAILED
        };

        Ok(SpireRemoteTransportSimSummary {
            dispatch_count,
            ready_dispatch_count,
            failed_dispatch_count,
            degraded_skipped_dispatch_count,
            selected_candidate_count,
            selected_input_nodes,
            status,
        })
    }

    fn dispatch_for_node_mut(
        &mut self,
        node_id: u32,
    ) -> Result<&mut SpireRemoteTransportSimDispatch, String> {
        self.dispatches
            .iter_mut()
            .find(|dispatch| dispatch.request.node_id == node_id)
            .ok_or_else(|| {
                format!("ec_spire remote transport sim received unknown node_id {node_id}")
            })
    }
}

impl SpireRemoteTransportSimDispatch {
    fn apply_failure(
        &mut self,
        consistency: SpireRemoteTransportSimConsistency,
        failure_category: &'static str,
    ) {
        self.candidates.clear();
        self.failure_category = failure_category;
        if consistency == SpireRemoteTransportSimConsistency::Degraded {
            self.state = SpireRemoteTransportSimDispatchState::DegradedSkipped;
            self.status = SPIRE_REMOTE_SIM_STATUS_DEGRADED_SKIPPED;
        } else {
            self.state = SpireRemoteTransportSimDispatchState::CandidateReceiveFailed;
            self.status = SPIRE_REMOTE_SIM_STATUS_RECEIVE_FAILED;
        }
    }
}

pub(crate) fn encode_remote_transport_sim_request(
    request: &SpireRemoteTransportSimRequest,
) -> Result<Vec<u8>, String> {
    let mut encoded = Vec::new();
    encoded.push(SPIRE_REMOTE_SIM_WIRE_VERSION);
    encoded.push(SPIRE_REMOTE_SIM_WIRE_REQUEST);
    encoded.extend_from_slice(&request.requested_epoch.to_be_bytes());
    encoded.extend_from_slice(&request.node_id.to_be_bytes());
    encoded.extend_from_slice(&u32_len(request.selected_pids.len(), "selected PID")?.to_be_bytes());
    for pid in &request.selected_pids {
        encoded.extend_from_slice(&pid.to_be_bytes());
    }
    encoded.extend_from_slice(&u64_len(request.top_k, "top_k")?.to_be_bytes());
    encoded.push(match request.consistency {
        SpireRemoteTransportSimConsistency::Strict => 1,
        SpireRemoteTransportSimConsistency::Degraded => 2,
    });
    Ok(encoded)
}

pub(crate) fn decode_remote_transport_sim_request(
    encoded: &[u8],
) -> Result<SpireRemoteTransportSimRequest, String> {
    let mut cursor = SpireRemoteTransportSimWireCursor::new(encoded);
    cursor.expect_header(SPIRE_REMOTE_SIM_WIRE_REQUEST)?;
    let requested_epoch = cursor.read_u64()?;
    let node_id = cursor.read_u32()?;
    let pid_count = cursor.read_u32()? as usize;
    let mut selected_pids = Vec::with_capacity(pid_count);
    for _ in 0..pid_count {
        selected_pids.push(cursor.read_u64()?);
    }
    let top_k = usize::try_from(cursor.read_u64()?)
        .map_err(|_| "ec_spire remote transport sim wire top_k exceeds usize".to_owned())?;
    let consistency = match cursor.read_u8()? {
        1 => SpireRemoteTransportSimConsistency::Strict,
        2 => SpireRemoteTransportSimConsistency::Degraded,
        _ => return Err("ec_spire remote transport sim wire unknown consistency".to_owned()),
    };
    cursor.expect_finished()?;
    Ok(SpireRemoteTransportSimRequest {
        requested_epoch,
        node_id,
        selected_pids,
        top_k,
        consistency,
    })
}

pub(crate) fn encode_remote_transport_sim_response(
    response: &SpireRemoteTransportSimResponse,
) -> Result<Vec<u8>, String> {
    let mut encoded = Vec::new();
    encoded.push(SPIRE_REMOTE_SIM_WIRE_VERSION);
    encoded.push(SPIRE_REMOTE_SIM_WIRE_RESPONSE);
    encoded.extend_from_slice(&response.node_id.to_be_bytes());
    encoded.extend_from_slice(&response.served_epoch.to_be_bytes());
    if let Some(failure_category) = response.failure_category {
        encoded.push(SPIRE_REMOTE_SIM_WIRE_STATUS_FAILED);
        write_bytes(
            &mut encoded,
            failure_category.as_bytes(),
            "failure category",
        )?;
        return Ok(encoded);
    }

    encoded.push(SPIRE_REMOTE_SIM_WIRE_STATUS_READY);
    encoded.extend_from_slice(&u32_len(response.candidates.len(), "candidate")?.to_be_bytes());
    for candidate in &response.candidates {
        encoded.extend_from_slice(&candidate.served_epoch.to_be_bytes());
        encoded.extend_from_slice(&candidate.node_id.to_be_bytes());
        encoded.extend_from_slice(&candidate.pid.to_be_bytes());
        encoded.extend_from_slice(&candidate.object_version.to_be_bytes());
        encoded.extend_from_slice(&candidate.row_index.to_be_bytes());
        encoded.push(candidate.assignment_role_rank);
        write_bytes(&mut encoded, &candidate.row_locator, "row locator")?;
        write_bytes(&mut encoded, &candidate.dedupe_key, "dedupe key")?;
        encoded.extend_from_slice(&candidate.score.to_bits().to_be_bytes());
    }
    Ok(encoded)
}

pub(crate) fn decode_remote_transport_sim_response(
    encoded: &[u8],
) -> Result<SpireRemoteTransportSimResponse, String> {
    let mut cursor = SpireRemoteTransportSimWireCursor::new(encoded);
    cursor.expect_header(SPIRE_REMOTE_SIM_WIRE_RESPONSE)?;
    let node_id = cursor.read_u32()?;
    let served_epoch = cursor.read_u64()?;
    match cursor.read_u8()? {
        SPIRE_REMOTE_SIM_WIRE_STATUS_FAILED => {
            let failure_category = decode_static_failure_category(&cursor.read_bytes()?)?;
            cursor.expect_finished()?;
            Ok(SpireRemoteTransportSimResponse::failed(
                node_id,
                served_epoch,
                failure_category,
            ))
        }
        SPIRE_REMOTE_SIM_WIRE_STATUS_READY => {
            let candidate_count = cursor.read_u32()? as usize;
            let mut candidates = Vec::with_capacity(candidate_count);
            for _ in 0..candidate_count {
                candidates.push(SpireRemoteTransportSimCandidate {
                    served_epoch: cursor.read_u64()?,
                    node_id: cursor.read_u32()?,
                    pid: cursor.read_u64()?,
                    object_version: cursor.read_u64()?,
                    row_index: cursor.read_u32()?,
                    assignment_role_rank: cursor.read_u8()?,
                    row_locator: cursor.read_bytes()?,
                    dedupe_key: cursor.read_bytes()?,
                    score: f32::from_bits(cursor.read_u32()?),
                });
            }
            cursor.expect_finished()?;
            Ok(SpireRemoteTransportSimResponse::ready(
                node_id,
                served_epoch,
                candidates,
            ))
        }
        _ => Err("ec_spire remote transport sim wire unknown response status".to_owned()),
    }
}

fn decode_static_failure_category(encoded: &[u8]) -> Result<&'static str, String> {
    match encoded {
        b"network_unreachable" => Ok("network_unreachable"),
        b"served_epoch_mismatch" => Ok("served_epoch_mismatch"),
        b"remote_candidate_receive_failed" => Ok("remote_candidate_receive_failed"),
        _ => Err("ec_spire remote transport sim wire unknown failure category".to_owned()),
    }
}

fn u32_len(len: usize, context: &str) -> Result<u32, String> {
    u32::try_from(len)
        .map_err(|_| format!("ec_spire remote transport sim {context} count exceeds u32"))
}

fn u64_len(len: usize, context: &str) -> Result<u64, String> {
    u64::try_from(len).map_err(|_| format!("ec_spire remote transport sim {context} exceeds u64"))
}

fn write_bytes(encoded: &mut Vec<u8>, bytes: &[u8], context: &str) -> Result<(), String> {
    encoded.extend_from_slice(&u32_len(bytes.len(), context)?.to_be_bytes());
    encoded.extend_from_slice(bytes);
    Ok(())
}

struct SpireRemoteTransportSimWireCursor<'a> {
    encoded: &'a [u8],
    offset: usize,
}

impl<'a> SpireRemoteTransportSimWireCursor<'a> {
    fn new(encoded: &'a [u8]) -> Self {
        Self { encoded, offset: 0 }
    }

    fn expect_header(&mut self, expected_kind: u8) -> Result<(), String> {
        let version = self.read_u8()?;
        if version != SPIRE_REMOTE_SIM_WIRE_VERSION {
            return Err("ec_spire remote transport sim wire version mismatch".to_owned());
        }
        let kind = self.read_u8()?;
        if kind != expected_kind {
            return Err("ec_spire remote transport sim wire message kind mismatch".to_owned());
        }
        Ok(())
    }

    fn expect_finished(&self) -> Result<(), String> {
        if self.offset == self.encoded.len() {
            Ok(())
        } else {
            Err("ec_spire remote transport sim wire trailing bytes".to_owned())
        }
    }

    fn read_u8(&mut self) -> Result<u8, String> {
        let bytes = self.read_exact(1)?;
        Ok(bytes[0])
    }

    fn read_u32(&mut self) -> Result<u32, String> {
        let bytes = self.read_exact(4)?;
        Ok(u32::from_be_bytes(
            bytes.try_into().expect("slice length is checked"),
        ))
    }

    fn read_u64(&mut self) -> Result<u64, String> {
        let bytes = self.read_exact(8)?;
        Ok(u64::from_be_bytes(
            bytes.try_into().expect("slice length is checked"),
        ))
    }

    fn read_bytes(&mut self) -> Result<Vec<u8>, String> {
        let len = self.read_u32()? as usize;
        Ok(self.read_exact(len)?.to_vec())
    }

    fn read_exact(&mut self, len: usize) -> Result<&'a [u8], String> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or_else(|| "ec_spire remote transport sim wire offset overflow".to_owned())?;
        let bytes = self
            .encoded
            .get(self.offset..end)
            .ok_or_else(|| "ec_spire remote transport sim wire truncated".to_owned())?;
        self.offset = end;
        Ok(bytes)
    }
}
