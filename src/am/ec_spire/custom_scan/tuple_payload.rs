fn custom_scan_store_remote_tuple_payload(
    state: &mut SpireCustomScanExecState,
    access_state: CustomScanAccessState<'_>,
    output: &super::SpireRemoteProductionScanOutputRow,
) -> *mut pg_sys::TupleTableSlot {
    if output.tuple_payload_missing {
        pgrx::error!(
            "EcSpireDistributedScan remote tuple payload is missing for node_id {} output",
            output.node_id
        );
    }
    if let Some(payload) = output.typed_tuple_payload.as_ref() {
        return custom_scan_store_tuple_payload_typed(
            access_state.tuple_slot(),
            payload,
            &mut state.tuple_payload_inputs,
        );
    }
    let Some(payload_json) = output.tuple_payload_json.as_deref() else {
        pgrx::error!(
            "EcSpireDistributedScan tuple payload delivery requires remote payload for node_id {} output; heap_lookup_owner {}",
            output.node_id,
            output.heap_lookup_owner
        );
    };
    custom_scan_store_tuple_payload_json(
        access_state.tuple_slot(),
        payload_json,
        &mut state.tuple_payload_inputs,
    )
}

fn custom_scan_store_tuple_payload_json(
    slot: *mut pg_sys::TupleTableSlot,
    payload_json: &str,
    attr_inputs: &mut [Option<SpireCustomScanPayloadAttrIo>],
) -> *mut pg_sys::TupleTableSlot {
    let writer =
        tuple_payload_writer(slot).unwrap_or_else(|error| pgrx::error!("{error}"));
    custom_scan_store_tuple_payload_json_with_writer(writer, payload_json, attr_inputs)
}

fn custom_scan_store_tuple_payload_json_with_writer(
    mut writer: TupleSlotWriter<'_>,
    payload_json: &str,
    attr_inputs: &mut [Option<SpireCustomScanPayloadAttrIo>],
) -> *mut pg_sys::TupleTableSlot {
    let payload = serde_json::from_str::<serde_json::Value>(payload_json).unwrap_or_else(|e| {
        pgrx::error!("EcSpireDistributedScan remote tuple payload JSON decode failed: {e}")
    });
    let payload_object = payload.as_object().unwrap_or_else(|| {
        pgrx::error!("EcSpireDistributedScan remote tuple payload must be a JSON object")
    });

    writer
        .validate_input_width(attr_inputs.len())
        .unwrap_or_else(|error| pgrx::error!("{error}"));

    writer.clear();
    let natts = writer.natts();
    for attr_index in 0..natts {
        let Some(attr) = writer
            .attribute(attr_index)
            .unwrap_or_else(|error| pgrx::error!("{error}"))
        else {
            writer.set_null(attr_index);
            continue;
        };
        match payload_object.get(attr.name.as_str()) {
            None | Some(serde_json::Value::Null) => {
                writer.set_null(attr_index);
            }
            Some(value) => {
                let Some(attr_input) = attr_inputs
                    .get_mut(attr_index as usize)
                    .and_then(Option::as_mut)
                else {
                    pgrx::error!(
                        "EcSpireDistributedScan tuple payload input cache missing attribute {}",
                        attr_index + 1
                    );
                };
                let datum = custom_scan_json_value_to_datum(value, attr.name.as_str(), attr_input);
                writer.set_datum(attr_index, datum);
            }
        }
    }
    writer
        .store_virtual_tuple()
        .unwrap_or_else(|error| pgrx::error!("{error}"))
}

fn tuple_payload_writer<'slot>(
    slot: *mut pg_sys::TupleTableSlot,
) -> Result<TupleSlotWriter<'slot>, String> {
    // SAFETY: SPIRE custom scan callers pass the scan output TupleTableSlot for
    // the active callback; `TupleSlotWriter` centralizes descriptor validation
    // and slot value/null writes for that callback scope.
    unsafe { TupleSlotWriter::from_raw_slot(slot, "EcSpireDistributedScan") }
}

fn tuple_payload_attr_io_for_slot(
    slot: *mut pg_sys::TupleTableSlot,
) -> Vec<Option<SpireCustomScanPayloadAttrIo>> {
    let writer =
        tuple_payload_writer(slot).unwrap_or_else(|error| pgrx::error!("{error}"));
    // SAFETY: writer construction validated the live slot and tuple descriptor.
    unsafe { custom_scan_payload_attr_io(writer.tuple_desc()) }
}

fn custom_scan_json_value_to_datum(
    value: &serde_json::Value,
    attr_name: &str,
    attr_input: &mut SpireCustomScanPayloadAttrIo,
) -> pg_sys::Datum {
    // SAFETY: attr_input contains PostgreSQL input function metadata for the
    // target attribute, and the CString input lives through InputFunctionCall.
    unsafe {
        let input_text = match value {
            serde_json::Value::String(value) => value.clone(),
            serde_json::Value::Bool(value) => value.to_string(),
            serde_json::Value::Number(value) => value.to_string(),
            serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                pgrx::error!(
                    "EcSpireDistributedScan tuple payload column \"{attr_name}\" has unsupported non-scalar JSON value"
                )
            }
            serde_json::Value::Null => {
                pgrx::error!("EcSpireDistributedScan cannot convert JSON null to non-null datum")
            }
        };
        let input = CString::new(input_text)
            .unwrap_or_else(|_| pgrx::error!("EcSpireDistributedScan tuple payload contains NUL"));
        pg_sys::InputFunctionCall(
            &mut attr_input.input_flinfo,
            input.as_ptr().cast_mut(),
            attr_input.input_typioparam,
            attr_input.typmod,
        )
    }
}

fn custom_scan_store_tuple_payload_typed(
    slot: *mut pg_sys::TupleTableSlot,
    payload: &super::SpireRemoteTypedTuplePayload,
    attr_inputs: &mut [Option<SpireCustomScanPayloadAttrIo>],
) -> *mut pg_sys::TupleTableSlot {
    let writer =
        tuple_payload_writer(slot).unwrap_or_else(|error| pgrx::error!("{error}"));
    custom_scan_store_tuple_payload_typed_with_writer(writer, payload, attr_inputs)
}

fn custom_scan_store_tuple_payload_typed_with_writer(
    mut writer: TupleSlotWriter<'_>,
    payload: &super::SpireRemoteTypedTuplePayload,
    attr_inputs: &mut [Option<SpireCustomScanPayloadAttrIo>],
) -> *mut pg_sys::TupleTableSlot {
    writer
        .validate_input_width(attr_inputs.len())
        .unwrap_or_else(|error| pgrx::error!("{error}"));
    let payload_width = payload.payload_attnums.len();
    for (label, width) in [
        ("payload_names", payload.payload_names.len()),
        ("payload_type_oids", payload.payload_type_oids.len()),
        ("payload_typmods", payload.payload_typmods.len()),
        ("payload_collations", payload.payload_collations.len()),
        ("payload_nulls", payload.payload_nulls.len()),
        ("payload_values", payload.payload_values.len()),
        ("payload_formats", payload.payload_formats.len()),
    ] {
        if width != payload_width {
            pgrx::error!(
                "EcSpireDistributedScan typed tuple payload {label} width {width} does not match attnum width {payload_width}"
            );
        }
    }
    if payload.tuple_transport != super::SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1
        || payload.tuple_transport_status != super::SPIRE_REMOTE_STATUS_READY
    {
        pgrx::error!(
            "EcSpireDistributedScan unsupported typed tuple transport {} status {}",
            payload.tuple_transport,
            payload.tuple_transport_status
        );
    }

    writer.clear();
    let natts = writer.natts();
    for attr_index in 0..natts {
        let Some(attr) = writer
            .attribute(attr_index)
            .unwrap_or_else(|error| pgrx::error!("{error}"))
        else {
            writer.set_null(attr_index);
            continue;
        };
        let payload_position = payload
            .payload_attnums
            .iter()
            .position(|attnum| *attnum == attr.attnum);
        let Some(payload_position) = payload_position else {
            writer.set_null(attr_index);
            continue;
        };
        custom_scan_validate_typed_payload_attr(payload, payload_position, attr.name.as_str());
        if payload.payload_nulls[payload_position] {
            writer.set_null(attr_index);
            continue;
        }
        let Some(attr_input) = attr_inputs
            .get_mut(attr_index as usize)
            .and_then(Option::as_mut)
        else {
            pgrx::error!(
                "EcSpireDistributedScan tuple payload input cache missing attribute {}",
                attr_index + 1
            );
        };
        let datum = custom_scan_binary_value_to_datum(
            &payload.payload_values[payload_position],
            attr.name.as_str(),
            attr_input,
        );
        writer.set_datum(attr_index, datum);
    }
    writer
        .store_virtual_tuple()
        .unwrap_or_else(|error| pgrx::error!("{error}"))
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn custom_scan_store_tuple_payload_json_for_test(
    slot: &crate::storage::slot_guard::TupleTableSlotGuard,
    payload_json: &str,
) -> *mut pg_sys::TupleTableSlot {
    let slot = slot.as_ptr();
    let mut attr_inputs = tuple_payload_attr_io_for_slot(slot);
    custom_scan_store_tuple_payload_json(slot, payload_json, &mut attr_inputs)
}

fn custom_scan_validate_typed_payload_attr(
    payload: &super::SpireRemoteTypedTuplePayload,
    payload_position: usize,
    attr_name: &str,
) {
    if payload.payload_names[payload_position] != attr_name {
        pgrx::error!(
            "EcSpireDistributedScan typed tuple payload attnum {} name mismatch: remote {}, local {}",
            payload.payload_attnums[payload_position],
            payload.payload_names[payload_position],
            attr_name
        );
    }
    if payload.payload_formats[payload_position]
        != super::SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1
    {
        pgrx::error!(
            "EcSpireDistributedScan typed tuple payload column \"{attr_name}\" has unsupported format {}",
            payload.payload_formats[payload_position]
        );
    }
}

fn custom_scan_binary_value_to_datum(
    value: &[u8],
    attr_name: &str,
    attr_input: &mut SpireCustomScanPayloadAttrIo,
) -> pg_sys::Datum {
    // SAFETY: attr_input contains PostgreSQL binary receive metadata for the
    // target attribute, and StringInfoData points at bytes kept alive for the
    // duration of ReceiveFunctionCall.
    unsafe {
        let len = core::ffi::c_int::try_from(value.len()).unwrap_or_else(|_| {
            pgrx::error!(
                "EcSpireDistributedScan typed tuple payload column \"{attr_name}\" is too large"
            )
        });
        let mut bytes = value.to_vec();
        let mut input = pg_sys::StringInfoData {
            data: bytes.as_mut_ptr().cast(),
            len,
            maxlen: len,
            cursor: 0,
        };
        let datum = pg_sys::ReceiveFunctionCall(
            &mut attr_input.receive_flinfo,
            &mut input,
            attr_input.receive_typioparam,
            attr_input.typmod,
        );
        if input.cursor != input.len {
            pgrx::error!(
                "EcSpireDistributedScan typed tuple payload column \"{attr_name}\" binary receive left unread bytes"
            );
        }
        datum
    }
}
