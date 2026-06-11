#[cfg(test)]
mod tests {
    use super::{
        encode_assignment_input, encode_assignment_payload, SpireAssignmentPayloadFormat,
        SpireAssignmentQuantCodec, SpirePreparedAssignmentScorer,
    };
    use crate::am::common::candidate_batch::{CandidateBatch, CandidateMeta, CandidatePayload};
    use crate::am::common::quant_codec::{QuantCodec, QuantCodecKind, QuantSearchCodecTag};
    use crate::am::ec_spire::storage::{
        SpireLeafAssignmentRow, SPIRE_ASSIGNMENT_FLAG_PRIMARY, SPIRE_PAYLOAD_FORMAT_NONE,
        SPIRE_PAYLOAD_FORMAT_RABITQ, SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
    };
    use crate::quant::prod::{ExactScoreMode, ProdQuantizer};
    use crate::quant::rabitq::RaBitQQuantizer;
    use crate::storage::page::ItemPointer;

    fn tid(block_number: u32, offset_number: u16) -> ItemPointer {
        ItemPointer {
            block_number,
            offset_number,
        }
    }

    fn assignment_row(
        payload_format: SpireAssignmentPayloadFormat,
        gamma: f32,
        encoded_payload: Vec<u8>,
    ) -> SpireLeafAssignmentRow {
        SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: crate::am::ec_spire::storage::SpireVecId::local(1),
            heap_tid: tid(10, 1),
            payload_format: payload_format.tag(),
            gamma,
            encoded_payload,
        }
    }

    #[test]
    fn turboquant_assignment_scorer_matches_direct_quantizer_score() {
        let source = vec![0.25, -0.5, 0.75, 1.0];
        let query = vec![1.0, 0.5, -0.25, 0.125];
        let (gamma, payload) =
            encode_assignment_payload(SpireAssignmentPayloadFormat::TurboQuant, &source).unwrap();
        let assignment = assignment_row(
            SpireAssignmentPayloadFormat::TurboQuant,
            gamma,
            payload.clone(),
        );
        let scorer = SpirePreparedAssignmentScorer::prepare(
            SpireAssignmentPayloadFormat::TurboQuant,
            source.len(),
            &query,
        )
        .unwrap();
        let quantizer = ProdQuantizer::cached(
            source.len(),
            crate::DEFAULT_QUANT_BITS,
            crate::DEFAULT_QUANT_SEED,
        );
        let prepared = quantizer.prepare_ip_query(&query);
        let expected = quantizer.score_ip_from_parts(&prepared, gamma, &payload);

        let observed = scorer.score_assignment_ip(&assignment).unwrap();

        assert_eq!(scorer.dimensions(), source.len());
        assert_eq!(assignment.payload_format, SPIRE_PAYLOAD_FORMAT_TURBOQUANT);
        assert!((observed - expected).abs() <= f32::EPSILON);
    }

    #[test]
    fn turboquant_assignment_scorer_uses_no_qjl_4bit_lut_path() {
        let dim = 1536;
        let source_a = (0..dim)
            .map(|index| ((index as f32) * 0.013).sin() * 0.5)
            .collect::<Vec<_>>();
        let source_b = (0..dim)
            .map(|index| ((index as f32) * 0.017).cos() * 0.25)
            .collect::<Vec<_>>();
        let query = (0..dim)
            .map(|index| ((index as f32) * 0.019).sin())
            .collect::<Vec<_>>();
        let (gamma_a, payload_a) =
            encode_assignment_payload(SpireAssignmentPayloadFormat::TurboQuant, &source_a)
                .unwrap();
        let (gamma_b, payload_b) =
            encode_assignment_payload(SpireAssignmentPayloadFormat::TurboQuant, &source_b)
                .unwrap();
        let assignment = assignment_row(
            SpireAssignmentPayloadFormat::TurboQuant,
            gamma_a,
            payload_a.clone(),
        );
        let scorer =
            SpirePreparedAssignmentScorer::prepare(SpireAssignmentPayloadFormat::TurboQuant, dim, &query)
                .unwrap();
        let quantizer =
            ProdQuantizer::cached(dim, crate::DEFAULT_QUANT_BITS, crate::DEFAULT_QUANT_SEED);
        let prepared_generic = quantizer.prepare_ip_query(&query);
        let prepared_lut = quantizer.prepare_ip_query_lut_no_qjl_4bit(&query);
        let expected_generic = quantizer.score_ip_from_parts(&prepared_generic, gamma_a, &payload_a);
        let expected_lut =
            quantizer.score_ip_from_parts_lut_no_qjl_4bit(&prepared_lut, &payload_a);

        let observed = scorer.score_assignment_ip(&assignment).unwrap();

        match &scorer {
            SpirePreparedAssignmentScorer::TurboQuant { no_qjl_4bit_lut, .. } => {
                assert!(no_qjl_4bit_lut.is_some());
            }
            SpirePreparedAssignmentScorer::RaBitQ { .. } => {
                panic!("TurboQuant prepare should return TurboQuant scorer")
            }
        }
        assert!(gamma_a > 0.0);
        assert!((expected_lut - expected_generic).abs() < 1e-6);
        assert!((observed - expected_generic).abs() < 1e-6);

        let payload_stride = payload_a.len();
        assert_eq!(payload_stride, payload_b.len());
        let mut payloads = payload_a.clone();
        payloads.extend_from_slice(&payload_b);
        let mut batch_scores = [0.0_f32; 2];
        scorer
            .score_batch_ip(
                payload_stride,
                &payloads,
                &[gamma_a, gamma_b],
                &mut batch_scores,
            )
            .unwrap();

        assert!((batch_scores[0] - observed).abs() < 1e-6);
        assert_eq!(
            scorer.score_zero_gamma_payload_chunks_max_prevalidated(payload_stride, &payloads),
            batch_scores[0].max(batch_scores[1])
        );
    }

    #[test]
    fn turboquant_qjl_assignment_batch_uses_qjl32_path() {
        let _guard = crate::am::common::candidate_batch::CANDIDATE_BATCH_COUNTER_TEST_LOCK
            .lock()
            .unwrap();
        crate::am::common::candidate_batch::reset_candidate_batch_scoring_counters();

        let dim = 1024;
        let query = (0..dim)
            .map(|index| ((index as f32) * 0.019).sin())
            .collect::<Vec<_>>();
        let scorer =
            SpirePreparedAssignmentScorer::prepare(SpireAssignmentPayloadFormat::TurboQuant, dim, &query)
                .unwrap();
        let quantizer =
            ProdQuantizer::cached(dim, crate::DEFAULT_QUANT_BITS, crate::DEFAULT_QUANT_SEED);
        assert_eq!(quantizer.exact_score_mode(), ExactScoreMode::MseLutQjl);
        let prepared = quantizer.prepare_ip_query(&query);
        match &scorer {
            SpirePreparedAssignmentScorer::TurboQuant { no_qjl_4bit_lut, .. } => {
                assert!(no_qjl_4bit_lut.is_none());
            }
            SpirePreparedAssignmentScorer::RaBitQ { .. } => {
                panic!("TurboQuant prepare should return TurboQuant scorer")
            }
        }

        let encoded = (0..39)
            .map(|row| {
                let source = (0..dim)
                    .map(|col| ((row + col * 5) % 31) as f32 / dim as f32)
                    .collect::<Vec<_>>();
                encode_assignment_payload(SpireAssignmentPayloadFormat::TurboQuant, &source)
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let payload_stride = encoded[0].1.len();
        let mut payloads = Vec::with_capacity(payload_stride * encoded.len());
        let mut gammas = Vec::with_capacity(encoded.len());
        for (gamma, payload) in &encoded {
            assert_eq!(payload.len(), payload_stride);
            gammas.push(*gamma);
            payloads.extend_from_slice(payload);
        }
        let mut raw_scores = vec![0.0_f32; encoded.len()];

        scorer
            .score_batch_ip(payload_stride, &payloads, &gammas, &mut raw_scores)
            .unwrap();

        for (index, ((gamma, payload), score)) in encoded.iter().zip(raw_scores.iter()).enumerate()
        {
            let scalar = quantizer.score_ip_from_parts(&prepared, *gamma, payload);
            let tolerance = 1e-6_f32.max(scalar.abs() * 1e-6);
            assert!(
                (*score - scalar).abs() <= tolerance,
                "index={index} batch={score} scalar={scalar}",
            );
        }
        let snapshots = crate::am::common::candidate_batch::block_kernel_scoring_snapshots();
        let qjl = snapshots
            .iter()
            .filter(|snapshot| snapshot.surface == "spire" && snapshot.quant_kind == "turboquant_qjl")
            .collect::<Vec<_>>();
        assert!(qjl
            .iter()
            .any(|snapshot| snapshot.kernel_candidates == 32));
        assert!(qjl
            .iter()
            .any(|snapshot| snapshot.scalar_candidates == 7));

        crate::am::common::candidate_batch::reset_candidate_batch_scoring_counters();

        let mut batch = CandidateBatch::with_capacity(encoded.len());
        for (index, (gamma, payload)) in encoded.iter().enumerate() {
            batch
                .push(
                    index,
                    CandidatePayload::new(
                        payload,
                        if index % 2 == 0 {
                            CandidateMeta::Gamma(*gamma)
                        } else {
                            CandidateMeta::GammaAndResidualSigns {
                                gamma: *gamma,
                                signs: &[],
                            }
                        },
                    ),
                )
                .unwrap();
        }
        let mut candidate_scores = vec![0.0_f32; batch.len()];

        scorer
            .score_candidate_batch_ip(&batch, &mut candidate_scores)
            .unwrap();

        for (index, ((gamma, payload), score)) in
            encoded.iter().zip(candidate_scores.iter()).enumerate()
        {
            let scalar = quantizer.score_ip_from_parts(&prepared, *gamma, payload);
            let tolerance = 1e-6_f32.max(scalar.abs() * 1e-6);
            assert!(
                (*score - scalar).abs() <= tolerance,
                "index={index} batch={score} scalar={scalar}",
            );
        }
        let snapshots = crate::am::common::candidate_batch::block_kernel_scoring_snapshots();
        let qjl = snapshots
            .iter()
            .filter(|snapshot| snapshot.surface == "spire" && snapshot.quant_kind == "turboquant_qjl")
            .collect::<Vec<_>>();
        assert!(qjl
            .iter()
            .any(|snapshot| snapshot.kernel_candidates == 32));
        assert!(qjl
            .iter()
            .any(|snapshot| snapshot.scalar_candidates == 7));

        crate::am::common::candidate_batch::reset_candidate_batch_scoring_counters();
    }

    #[test]
    fn rabitq_assignment_scorer_matches_direct_quantizer_score() {
        let source = vec![0.25, -0.5, 0.75, 1.0];
        let query = vec![1.0, 0.5, -0.25, 0.125];
        let (gamma, payload) =
            encode_assignment_payload(SpireAssignmentPayloadFormat::RaBitQ, &source).unwrap();
        let assignment =
            assignment_row(SpireAssignmentPayloadFormat::RaBitQ, gamma, payload.clone());
        let scorer = SpirePreparedAssignmentScorer::prepare(
            SpireAssignmentPayloadFormat::RaBitQ,
            source.len(),
            &query,
        )
        .unwrap();
        let quantizer = RaBitQQuantizer::cached_seeded_srht_bits(
            source.len(),
            crate::DEFAULT_QUANT_SEED,
            crate::DEFAULT_QUANT_BITS,
        )
        .unwrap();
        let prepared = quantizer.prepare_estimator(&query);
        let expected = prepared.estimate_ip_scalar_only(&payload);

        let observed = scorer.score_assignment_ip(&assignment).unwrap();

        assert_eq!(assignment.payload_format, SPIRE_PAYLOAD_FORMAT_RABITQ);
        assert_eq!(gamma, 0.0);
        assert!((observed - expected).abs() <= f32::EPSILON);
    }

    #[test]
    fn rabitq_assignment_scorer_can_prune_below_cutoff() {
        let source = vec![0.25, -0.5, 0.75, 1.0];
        let query = vec![1.0, 0.5, -0.25, 0.125];
        let (gamma, payload) =
            encode_assignment_payload(SpireAssignmentPayloadFormat::RaBitQ, &source).unwrap();
        let assignment =
            assignment_row(SpireAssignmentPayloadFormat::RaBitQ, gamma, payload.clone());
        let scorer = SpirePreparedAssignmentScorer::prepare(
            SpireAssignmentPayloadFormat::RaBitQ,
            source.len(),
            &query,
        )
        .unwrap();
        let full_score = scorer.score_assignment_ip(&assignment).unwrap();

        assert_eq!(
            scorer
                .try_score_assignment_ip(&assignment, f32::NEG_INFINITY)
                .unwrap(),
            Some(full_score)
        );
        assert_eq!(
            scorer
                .try_score_assignment_ip(&assignment, f32::MAX)
                .unwrap(),
            None
        );
    }

    #[test]
    fn assignment_scorer_batch_matches_scalar_scores() {
        let query = vec![1.0, 0.5, -0.25, 0.125];
        let sources = [vec![0.25, -0.5, 0.75, 1.0], vec![-0.125, 0.25, 0.5, -1.0]];

        for payload_format in [
            SpireAssignmentPayloadFormat::TurboQuant,
            SpireAssignmentPayloadFormat::RaBitQ,
        ] {
            let scorer =
                SpirePreparedAssignmentScorer::prepare(payload_format, query.len(), &query)
                    .unwrap();
            let mut payload_stride = None;
            let mut payloads = Vec::new();
            let mut gammas = Vec::new();
            let mut scalar_scores = Vec::new();

            for source in &sources {
                let (gamma, payload) = encode_assignment_payload(payload_format, source).unwrap();
                let assignment = assignment_row(payload_format, gamma, payload.clone());
                scalar_scores.push(scorer.score_assignment_ip(&assignment).unwrap());
                payload_stride = Some(payload_stride.unwrap_or(payload.len()));
                assert_eq!(payload_stride, Some(payload.len()));
                gammas.push(gamma);
                payloads.extend_from_slice(&payload);
            }

            let mut batch_scores = vec![0.0; sources.len()];
            scorer
                .score_batch_ip(
                    payload_stride.unwrap(),
                    &payloads,
                    &gammas,
                    &mut batch_scores,
                )
                .unwrap();

            assert_eq!(batch_scores.len(), scalar_scores.len());
            for (batch_score, scalar_score) in batch_scores.iter().zip(scalar_scores.iter()) {
                assert!((batch_score - scalar_score).abs() <= f32::EPSILON);
            }
        }
    }

    #[test]
    fn common_quant_codec_scores_turboquant_assignments() {
        let query = vec![1.0, 0.5, -0.25, 0.125];
        let source = vec![0.25, -0.5, 0.75, 1.0];
        let codec =
            SpireAssignmentQuantCodec::new(SpireAssignmentPayloadFormat::TurboQuant, query.len());
        let encoded = QuantCodec::encode_source(&codec, &source).unwrap();
        let prepared = QuantCodec::prepare_ip_query(&codec, &query).unwrap();
        let mut batch = CandidateBatch::with_capacity(1);
        batch
            .push(
                10_u32,
                CandidatePayload::new(&encoded.code, CandidateMeta::Gamma(encoded.gamma)),
            )
            .unwrap();
        let mut batch_scores = vec![0.0];

        QuantCodec::score_ip_batch(&codec, &prepared, &batch, &mut batch_scores).unwrap();

        assert_eq!(QuantCodec::codec_kind(&codec), QuantCodecKind::TurboQuant);
        assert_eq!(
            QuantCodec::search_codec_tag(&codec),
            QuantSearchCodecTag::TurboQuant
        );
        assert_eq!(encoded.dimensions, query.len() as u16);
        assert_eq!(encoded.code.len(), QuantCodec::payload_len(&codec));
        assert_eq!(
            batch_scores[0],
            prepared
                .score_payload_ip(
                    SpireAssignmentPayloadFormat::TurboQuant,
                    encoded.gamma,
                    &encoded.code
                )
                .unwrap()
        );
    }

    #[test]
    fn common_quant_codec_batch_delegates_to_prepared_scorer_batch() {
        let query = vec![1.0, 0.5, -0.25, 0.125];
        let sources = [vec![0.25, -0.5, 0.75, 1.0], vec![-0.125, 0.25, 0.5, -1.0]];
        let codec =
            SpireAssignmentQuantCodec::new(SpireAssignmentPayloadFormat::TurboQuant, query.len());
        let prepared = QuantCodec::prepare_ip_query(&codec, &query).unwrap();
        let encoded = sources
            .iter()
            .map(|source| QuantCodec::encode_source(&codec, source).unwrap())
            .collect::<Vec<_>>();
        let mut batch = CandidateBatch::with_capacity(encoded.len());
        for (index, encoded) in encoded.iter().enumerate() {
            batch
                .push(
                    index,
                    CandidatePayload::new(&encoded.code, CandidateMeta::Gamma(encoded.gamma)),
                )
                .unwrap();
        }
        let mut trait_scores = vec![0.0; batch.len()];
        let mut direct_scores = vec![0.0; batch.len()];

        QuantCodec::score_ip_batch(&codec, &prepared, &batch, &mut trait_scores).unwrap();
        prepared
            .score_candidate_batch_ip(&batch, &mut direct_scores)
            .unwrap();

        assert_eq!(trait_scores.len(), direct_scores.len());
        for (trait_score, direct_score) in trait_scores.iter().zip(direct_scores.iter()) {
            assert_eq!(trait_score.to_bits(), direct_score.to_bits());
        }
    }

    #[test]
    fn common_quant_codec_batch_preserves_prepared_scorer_length_error() {
        let query = vec![1.0, 0.5, -0.25, 0.125];
        let source = vec![0.25, -0.5, 0.75, 1.0];
        let codec =
            SpireAssignmentQuantCodec::new(SpireAssignmentPayloadFormat::TurboQuant, query.len());
        let encoded = QuantCodec::encode_source(&codec, &source).unwrap();
        let prepared = QuantCodec::prepare_ip_query(&codec, &query).unwrap();
        let mut batch = CandidateBatch::with_capacity(1);
        batch
            .push(
                0_u32,
                CandidatePayload::new(&encoded.code, CandidateMeta::Gamma(encoded.gamma)),
            )
            .unwrap();
        let mut trait_scores = Vec::new();
        let mut direct_scores = Vec::new();

        let trait_err =
            QuantCodec::score_ip_batch(&codec, &prepared, &batch, &mut trait_scores).unwrap_err();
        let direct_err = prepared
            .score_candidate_batch_ip(&batch, &mut direct_scores)
            .unwrap_err();

        assert_eq!(trait_err, direct_err);
        assert_eq!(
            trait_err,
            "ec_spire candidate batch scorer output count 0 does not match candidate count 1"
        );
    }

    #[test]
    fn prepared_scorer_quant_codec_matches_implicit_supported_format_state() {
        let query = vec![1.0, 0.5, -0.25, 0.125];
        for (payload_format, expected_kind, expected_tag) in [
            (
                SpireAssignmentPayloadFormat::TurboQuant,
                QuantCodecKind::TurboQuant,
                QuantSearchCodecTag::TurboQuant,
            ),
            (
                SpireAssignmentPayloadFormat::RaBitQ,
                QuantCodecKind::RaBitQ,
                QuantSearchCodecTag::RaBitQ {
                    bits: crate::DEFAULT_QUANT_BITS,
                },
            ),
        ] {
            let scorer =
                SpirePreparedAssignmentScorer::prepare(payload_format, query.len(), &query)
                    .unwrap();
            let codec = scorer.quant_codec();

            assert_eq!(codec.payload_format, scorer.payload_format());
            assert_eq!(codec.dimensions, scorer.dimensions());
            assert_eq!(QuantCodec::codec_kind(&codec), expected_kind);
            assert_eq!(QuantCodec::search_codec_tag(&codec), expected_tag);
            assert_eq!(QuantCodec::payload_len(&codec), scorer.payload_stride().unwrap());
        }
    }

    #[test]
    fn common_quant_codec_scores_rabitq_assignments() {
        let query = vec![1.0, 0.5, -0.25, 0.125];
        let sources = [vec![0.25, -0.5, 0.75, 1.0], vec![-0.125, 0.25, 0.5, -1.0]];
        let codec =
            SpireAssignmentQuantCodec::new(SpireAssignmentPayloadFormat::RaBitQ, query.len());
        let prepared = QuantCodec::prepare_ip_query(&codec, &query).unwrap();
        let encoded = sources
            .iter()
            .map(|source| QuantCodec::encode_source(&codec, source).unwrap())
            .collect::<Vec<_>>();
        let mut batch = CandidateBatch::with_capacity(encoded.len());
        for (index, encoded) in encoded.iter().enumerate() {
            batch
                .push(
                    index,
                    CandidatePayload::new(&encoded.code, CandidateMeta::RaBitQ),
                )
                .unwrap();
        }
        let mut batch_scores = vec![0.0; batch.len()];

        QuantCodec::score_ip_batch(&codec, &prepared, &batch, &mut batch_scores).unwrap();

        assert_eq!(QuantCodec::codec_kind(&codec), QuantCodecKind::RaBitQ);
        assert_eq!(
            QuantCodec::search_codec_tag(&codec),
            QuantSearchCodecTag::RaBitQ {
                bits: crate::DEFAULT_QUANT_BITS
            }
        );
        for (index, encoded) in encoded.iter().enumerate() {
            assert_eq!(encoded.gamma, 0.0);
            let scalar = prepared
                .score_payload_ip(SpireAssignmentPayloadFormat::RaBitQ, 0.0, &encoded.code)
                .unwrap();
            assert!(
                (batch_scores[index] - scalar).abs() <= f32::EPSILON,
                "index={index} batch={} scalar={scalar}",
                batch_scores[index]
            );
        }
    }

    #[test]
    fn common_quant_codec_rejects_spire_pq_fastscan_without_model() {
        let query = vec![1.0, 0.5, -0.25, 0.125];
        let source = vec![0.25, -0.5, 0.75, 1.0];
        let codec =
            SpireAssignmentQuantCodec::new(SpireAssignmentPayloadFormat::PqFastScan, query.len());

        assert_eq!(QuantCodec::codec_kind(&codec), QuantCodecKind::GroupedPq);
        assert!(QuantCodec::encode_source(&codec, &source).is_err());
        assert!(QuantCodec::prepare_ip_query(&codec, &query).is_err());
    }

    #[test]
    fn zero_gamma_payload_chunk_max_uses_exact_single_payload_score() {
        let query = vec![1.0, 0.5, -0.25, 0.125];
        let sources = [vec![0.25, -0.5, 0.75, 1.0], vec![-0.125, 0.25, 0.5, -1.0]];

        for payload_format in [
            SpireAssignmentPayloadFormat::TurboQuant,
            SpireAssignmentPayloadFormat::RaBitQ,
        ] {
            let scorer =
                SpirePreparedAssignmentScorer::prepare(payload_format, query.len(), &query)
                    .unwrap();
            let payloads = sources
                .iter()
                .map(|source| encode_assignment_payload(payload_format, source).unwrap().1)
                .collect::<Vec<_>>();
            let payload_stride = payloads[0].len();
            assert!(payloads.iter().all(|payload| payload.len() == payload_stride));

            let first_single =
                scorer.score_zero_gamma_payload_prevalidated(payloads[0].as_slice());
            let first_max = scorer.score_zero_gamma_payload_chunks_max_prevalidated(
                payload_stride,
                payloads[0].as_slice(),
            );
            assert_eq!(first_single, first_max);

            let mut multi_payload = Vec::new();
            for payload in &payloads {
                multi_payload.extend_from_slice(payload);
            }
            let expected_multi = payloads
                .iter()
                .map(|payload| scorer.score_zero_gamma_payload_prevalidated(payload))
                .fold(f32::NEG_INFINITY, f32::max);
            let observed_multi = scorer
                .score_zero_gamma_payload_chunks_max_prevalidated(payload_stride, &multi_payload);
            assert_eq!(expected_multi, observed_multi);
        }
    }

    #[test]
    fn assignment_scorer_rejects_mismatched_format_and_bad_lengths() {
        let source = vec![0.25, -0.5, 0.75, 1.0];
        let query = vec![1.0, 0.5, -0.25, 0.125];
        let (gamma, mut payload) =
            encode_assignment_payload(SpireAssignmentPayloadFormat::TurboQuant, &source).unwrap();
        let scorer = SpirePreparedAssignmentScorer::prepare(
            SpireAssignmentPayloadFormat::TurboQuant,
            source.len(),
            &query,
        )
        .unwrap();
        let mut assignment = assignment_row(
            SpireAssignmentPayloadFormat::TurboQuant,
            gamma,
            payload.clone(),
        );

        assignment.payload_format = SPIRE_PAYLOAD_FORMAT_RABITQ;
        assert!(scorer.score_assignment_ip(&assignment).is_err());

        assignment.payload_format = SPIRE_PAYLOAD_FORMAT_TURBOQUANT;
        payload.pop();
        assignment.encoded_payload = payload;
        assert!(scorer.score_assignment_ip(&assignment).is_err());
    }

    #[test]
    fn assignment_scorer_batch_rejects_bad_shapes() {
        let source = vec![0.25, -0.5, 0.75, 1.0];
        let query = vec![1.0, 0.5, -0.25, 0.125];
        let (gamma, payload) =
            encode_assignment_payload(SpireAssignmentPayloadFormat::TurboQuant, &source).unwrap();
        let scorer = SpirePreparedAssignmentScorer::prepare(
            SpireAssignmentPayloadFormat::TurboQuant,
            source.len(),
            &query,
        )
        .unwrap();
        let mut out = [0.0];

        assert!(scorer
            .score_batch_ip(payload.len() + 1, &payload, &[gamma], &mut out)
            .is_err());
        assert!(scorer
            .score_batch_ip(payload.len(), &payload, &[], &mut out)
            .is_err());

        let (_, rabitq_payload) =
            encode_assignment_payload(SpireAssignmentPayloadFormat::RaBitQ, &source).unwrap();
        let rabitq_scorer = SpirePreparedAssignmentScorer::prepare(
            SpireAssignmentPayloadFormat::RaBitQ,
            source.len(),
            &query,
        )
        .unwrap();
        assert!(rabitq_scorer
            .score_batch_ip(rabitq_payload.len(), &rabitq_payload, &[1.0], &mut out)
            .is_err());
    }

    #[test]
    fn assignment_scorer_rejects_unscoreable_and_deferred_formats() {
        assert!(SpireAssignmentPayloadFormat::from_tag(SPIRE_PAYLOAD_FORMAT_NONE).is_err());
        assert!(SpirePreparedAssignmentScorer::prepare(
            SpireAssignmentPayloadFormat::PqFastScan,
            4,
            &[1.0, 0.5, -0.25, 0.125],
        )
        .is_err());
        assert!(encode_assignment_payload(
            SpireAssignmentPayloadFormat::PqFastScan,
            &[0.25, -0.5, 0.75, 1.0],
        )
        .is_err());
    }

    #[test]
    fn assignment_scorer_validates_query_and_source_shape() {
        assert!(encode_assignment_payload(SpireAssignmentPayloadFormat::TurboQuant, &[]).is_err());
        assert!(encode_assignment_payload(
            SpireAssignmentPayloadFormat::TurboQuant,
            &[1.0, f32::NAN]
        )
        .is_err());
        assert!(SpirePreparedAssignmentScorer::prepare(
            SpireAssignmentPayloadFormat::TurboQuant,
            2,
            &[1.0],
        )
        .is_err());
        assert!(SpirePreparedAssignmentScorer::prepare(
            SpireAssignmentPayloadFormat::TurboQuant,
            2,
            &[1.0, f32::INFINITY],
        )
        .is_err());
    }

    #[test]
    fn encode_assignment_input_builds_leaf_assignment_input() {
        let source = vec![0.25, -0.5, 0.75, 1.0];
        let (gamma, payload) =
            encode_assignment_payload(SpireAssignmentPayloadFormat::TurboQuant, &source).unwrap();

        let input = encode_assignment_input(
            SpireAssignmentPayloadFormat::TurboQuant,
            tid(10, 2),
            &source,
        )
        .unwrap();

        assert_eq!(input.heap_tid, tid(10, 2));
        assert_eq!(input.payload_format, SPIRE_PAYLOAD_FORMAT_TURBOQUANT);
        assert_eq!(input.gamma, gamma);
        assert_eq!(input.encoded_payload, payload);
    }

    #[test]
    fn encode_assignment_input_rejects_invalid_locator() {
        assert!(encode_assignment_input(
            SpireAssignmentPayloadFormat::TurboQuant,
            ItemPointer::INVALID,
            &[0.25, -0.5, 0.75, 1.0],
        )
        .is_err());
    }
}
