#[path = "../../../src/am/ec_spire/coordinator/remote_candidates/candidate_merge_model.rs"]
mod candidate_merge_model;
mod remote_transport_sim_model {
    use crate::candidate_merge_model::{
        merge_candidate_model_inputs, SpireCandidateMergeModelInput,
    };

    include!(
        "../../../src/am/ec_spire/coordinator/remote_candidates/remote_transport_sim_model.rs"
    );
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::net::Ipv4Addr;
    use std::time::Duration;

    use super::remote_transport_sim_model::{
        decode_remote_transport_sim_request, decode_remote_transport_sim_response,
        encode_remote_transport_sim_request, encode_remote_transport_sim_response,
        SpireRemoteTransportSimCandidate, SpireRemoteTransportSimConsistency,
        SpireRemoteTransportSimRequest, SpireRemoteTransportSimResponse,
        SpireRemoteTransportSimState, SPIRE_REMOTE_SIM_STATUS_DEGRADED_SKIPPED,
        SPIRE_REMOTE_SIM_STATUS_READY, SPIRE_REMOTE_SIM_STATUS_RECEIVE_FAILED,
    };
    use tokio::time::{sleep, timeout};
    use turmoil::net::UdpSocket;

    const MERGE_PORT: u16 = 46_040;
    const PARTITION_PORT: u16 = 46_041;
    const STALE_PORT: u16 = 46_042;

    fn sim_spire_seed_count() -> u64 {
        std::env::var("SIM_SPIRE_SEEDS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(1)
    }

    fn sim_builder(seed: u64) -> turmoil::Builder {
        let mut builder = turmoil::Builder::new();
        builder.rng_seed(seed).enable_random_order();
        builder
    }

    #[derive(Clone)]
    struct NodeSpec {
        host: &'static str,
        node_id: u32,
        served_epoch: u64,
        score: f32,
        dedupe_key: Vec<u8>,
    }

    fn request(
        node_id: u32,
        consistency: SpireRemoteTransportSimConsistency,
    ) -> SpireRemoteTransportSimRequest {
        SpireRemoteTransportSimRequest {
            requested_epoch: 7,
            node_id,
            selected_pids: vec![u64::from(node_id)],
            top_k: 1,
            consistency,
        }
    }

    fn candidate_from_request(
        request: &SpireRemoteTransportSimRequest,
        served_epoch: u64,
        score: f32,
        dedupe_key: Vec<u8>,
    ) -> SpireRemoteTransportSimCandidate {
        SpireRemoteTransportSimCandidate {
            served_epoch,
            node_id: request.node_id,
            pid: request.selected_pids[0],
            object_version: 1,
            row_index: request.node_id,
            assignment_role_rank: 0,
            row_locator: vec![request.node_id as u8],
            dedupe_key,
            score,
        }
    }

    async fn serve_node(spec: NodeSpec, port: u16) -> turmoil::Result {
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, port)).await?;
        let mut buf = [0_u8; 2048];
        let Ok(Ok((len, peer))) =
            timeout(Duration::from_millis(200), socket.recv_from(&mut buf)).await
        else {
            return Ok(());
        };
        let request = decode_remote_transport_sim_request(&buf[..len])
            .expect("sim server should decode request");
        let response = SpireRemoteTransportSimResponse::ready(
            spec.node_id,
            spec.served_epoch,
            vec![candidate_from_request(
                &request,
                spec.served_epoch,
                spec.score,
                spec.dedupe_key,
            )],
        );
        let encoded = encode_remote_transport_sim_response(&response)
            .expect("sim server should encode response");
        socket.send_to(&encoded, peer).await?;
        Ok(())
    }

    async fn run_client(
        consistency: SpireRemoteTransportSimConsistency,
        requests: Vec<SpireRemoteTransportSimRequest>,
        partitioned_host: Option<&'static str>,
        port: u16,
    ) -> SpireRemoteTransportSimState {
        let mut state =
            SpireRemoteTransportSimState::new(consistency, requests.clone()).expect("valid state");
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))
            .await
            .expect("client socket should bind");
        sleep(Duration::from_millis(1)).await;
        if let Some(host) = partitioned_host {
            turmoil::partition("client", host);
        }

        let mut waiting_for = HashSet::new();
        for request in &requests {
            let host = format!("node-{}", request.node_id);
            let encoded =
                encode_remote_transport_sim_request(request).expect("request should encode");
            match socket.send_to(&encoded, (host.as_str(), port)).await {
                Ok(_) => {
                    waiting_for.insert(request.node_id);
                }
                Err(_) => {
                    state
                        .apply_failure(request.node_id, "network_unreachable")
                        .expect("send failure should apply");
                }
            }
        }

        let mut buf = [0_u8; 4096];
        while !waiting_for.is_empty() {
            match timeout(Duration::from_millis(100), socket.recv_from(&mut buf)).await {
                Ok(Ok((len, _peer))) => {
                    let response = decode_remote_transport_sim_response(&buf[..len])
                        .expect("response should decode");
                    waiting_for.remove(&response.node_id);
                    state
                        .apply_response(response)
                        .expect("response should apply");
                }
                Ok(Err(_)) | Err(_) => break,
            }
        }

        for node_id in waiting_for {
            state
                .apply_failure(node_id, "network_unreachable")
                .expect("timeout failure should apply");
        }
        state
    }

    fn add_node(sim: &mut turmoil::Sim, spec: NodeSpec, port: u16) {
        let host = spec.host;
        sim.host(host, move || serve_node(spec.clone(), port));
    }

    struct ScriptedAdapter {
        responses: Vec<SpireRemoteTransportSimResponse>,
    }

    impl super::remote_transport_sim_model::SpireRemoteTransportSimAdapter for ScriptedAdapter {
        fn receive_candidates(
            &mut self,
            request: &SpireRemoteTransportSimRequest,
        ) -> Result<SpireRemoteTransportSimResponse, &'static str> {
            let position = self
                .responses
                .iter()
                .position(|response| response.node_id == request.node_id)
                .ok_or("network_unreachable")?;
            Ok(self.responses.remove(position))
        }
    }

    #[test]
    fn scripted_adapter_uses_extracted_transport_state() {
        let mut state = SpireRemoteTransportSimState::new(
            SpireRemoteTransportSimConsistency::Strict,
            vec![request(2, SpireRemoteTransportSimConsistency::Strict)],
        )
        .expect("state should build");
        let request = request(2, SpireRemoteTransportSimConsistency::Strict);
        let mut adapter = ScriptedAdapter {
            responses: vec![SpireRemoteTransportSimResponse::ready(
                2,
                7,
                vec![candidate_from_request(
                    &request,
                    7,
                    0.10,
                    b"adapter-vec".to_vec(),
                )],
            )],
        };
        let summary = state
            .receive_with_adapter(&mut adapter)
            .expect("adapter receive should succeed");
        assert_eq!(summary.ready_dispatch_count, 1);
        assert_eq!(summary.selected_input_nodes, vec![2]);
        assert_eq!(summary.status, SPIRE_REMOTE_SIM_STATUS_READY);
    }

    #[test]
    fn turmoil_candidate_receive_merges_async_remote_responses() -> turmoil::Result {
        for seed in 0..sim_spire_seed_count() {
            let mut sim = sim_builder(seed).build();
            add_node(
                &mut sim,
                NodeSpec {
                    host: "node-2",
                    node_id: 2,
                    served_epoch: 7,
                    score: 0.40,
                    dedupe_key: b"global-vec".to_vec(),
                },
                MERGE_PORT,
            );
            add_node(
                &mut sim,
                NodeSpec {
                    host: "node-3",
                    node_id: 3,
                    served_epoch: 7,
                    score: 0.20,
                    dedupe_key: b"global-vec".to_vec(),
                },
                MERGE_PORT,
            );
            sim.client("client", async move {
                let state = run_client(
                    SpireRemoteTransportSimConsistency::Strict,
                    vec![
                        request(2, SpireRemoteTransportSimConsistency::Strict),
                        request(3, SpireRemoteTransportSimConsistency::Strict),
                    ],
                    None,
                    MERGE_PORT,
                )
                .await;
                let summary = state.summary(Some(1)).expect("summary should build");
                assert_eq!(summary.dispatch_count, 2, "seed {seed}");
                assert_eq!(summary.ready_dispatch_count, 2, "seed {seed}");
                assert_eq!(summary.failed_dispatch_count, 0, "seed {seed}");
                assert_eq!(summary.selected_candidate_count, 1, "seed {seed}");
                assert_eq!(summary.selected_input_nodes, vec![3], "seed {seed}");
                assert_eq!(summary.status, SPIRE_REMOTE_SIM_STATUS_READY, "seed {seed}");
                Ok(())
            });
            sim.run()?;
        }
        Ok(())
    }

    #[test]
    fn turmoil_partition_degraded_skips_unreachable_remote() -> turmoil::Result {
        for seed in 0..sim_spire_seed_count() {
            let mut sim = sim_builder(seed).build();
            add_node(
                &mut sim,
                NodeSpec {
                    host: "node-2",
                    node_id: 2,
                    served_epoch: 7,
                    score: 0.30,
                    dedupe_key: b"node-2-vec".to_vec(),
                },
                PARTITION_PORT,
            );
            add_node(
                &mut sim,
                NodeSpec {
                    host: "node-3",
                    node_id: 3,
                    served_epoch: 7,
                    score: 0.10,
                    dedupe_key: b"node-3-vec".to_vec(),
                },
                PARTITION_PORT,
            );
            sim.client("client", async move {
                let state = run_client(
                    SpireRemoteTransportSimConsistency::Degraded,
                    vec![
                        request(2, SpireRemoteTransportSimConsistency::Degraded),
                        request(3, SpireRemoteTransportSimConsistency::Degraded),
                    ],
                    Some("node-3"),
                    PARTITION_PORT,
                )
                .await;
                let summary = state.summary(Some(1)).expect("summary should build");
                assert_eq!(summary.ready_dispatch_count, 1, "seed {seed}");
                assert_eq!(summary.degraded_skipped_dispatch_count, 1, "seed {seed}");
                assert_eq!(summary.failed_dispatch_count, 0, "seed {seed}");
                assert_eq!(summary.selected_input_nodes, vec![2], "seed {seed}");
                assert_eq!(summary.status, SPIRE_REMOTE_SIM_STATUS_READY, "seed {seed}");
                Ok(())
            });
            sim.run()?;
        }
        Ok(())
    }

    #[test]
    fn turmoil_strict_rejects_stale_served_epoch_response() -> turmoil::Result {
        for seed in 0..sim_spire_seed_count() {
            let mut sim = sim_builder(seed).build();
            add_node(
                &mut sim,
                NodeSpec {
                    host: "node-2",
                    node_id: 2,
                    served_epoch: 6,
                    score: 0.10,
                    dedupe_key: b"stale-vec".to_vec(),
                },
                STALE_PORT,
            );
            sim.client("client", async move {
                let state = run_client(
                    SpireRemoteTransportSimConsistency::Strict,
                    vec![request(2, SpireRemoteTransportSimConsistency::Strict)],
                    None,
                    STALE_PORT,
                )
                .await;
                let summary = state.summary(Some(1)).expect("summary should build");
                assert_eq!(summary.ready_dispatch_count, 0, "seed {seed}");
                assert_eq!(summary.failed_dispatch_count, 1, "seed {seed}");
                assert_eq!(summary.selected_candidate_count, 0, "seed {seed}");
                assert_eq!(
                    summary.status, SPIRE_REMOTE_SIM_STATUS_RECEIVE_FAILED,
                    "seed {seed}"
                );
                Ok(())
            });
            sim.run()?;
        }
        Ok(())
    }

    #[test]
    fn transport_consistency_parser_keeps_governance_names_stable() {
        assert_eq!(
            SpireRemoteTransportSimConsistency::parse("strict").unwrap(),
            SpireRemoteTransportSimConsistency::Strict
        );
        assert_eq!(
            SpireRemoteTransportSimConsistency::parse("degraded").unwrap(),
            SpireRemoteTransportSimConsistency::Degraded
        );
        assert!(SpireRemoteTransportSimConsistency::parse("eventual").is_err());
        assert_eq!(SPIRE_REMOTE_SIM_STATUS_DEGRADED_SKIPPED, "degraded_skipped");
    }
}
