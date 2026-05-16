unsafe fn custom_scan_store_remote_tuple_payload(
    state: *mut SpireCustomScanExecState,
    scan_state: *mut pg_sys::ScanState,
    output: &super::SpireRemoteProductionScanOutputRow,
) -> *mut pg_sys::TupleTableSlot {
    unsafe {
        if output.tuple_payload_missing {
            pgrx::error!(
                "EcSpireDistributedScan remote tuple payload is missing for node_id {} output",
                output.node_id
            );
        }
        if let Some(payload) = output.typed_tuple_payload.as_ref() {
            return custom_scan_store_tuple_payload_typed(
                (*scan_state).ss_ScanTupleSlot,
                payload,
                &mut (*state).tuple_payload_inputs,
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
            (*scan_state).ss_ScanTupleSlot,
            payload_json,
            &mut (*state).tuple_payload_inputs,
        )
    }
}

unsafe fn custom_scan_store_tuple_payload_json(
    slot: *mut pg_sys::TupleTableSlot,
    payload_json: &str,
    attr_inputs: &mut [Option<SpireCustomScanPayloadAttrIo>],
) -> *mut pg_sys::TupleTableSlot {
    unsafe {
        if slot.is_null() {
            pgrx::error!("EcSpireDistributedScan tuple payload slot is null");
        }
        let tuple_desc = (*slot).tts_tupleDescriptor;
        if tuple_desc.is_null() {
            pgrx::error!("EcSpireDistributedScan tuple payload slot has no tuple descriptor");
        }
        let payload = serde_json::from_str::<serde_json::Value>(payload_json).unwrap_or_else(|e| {
            pgrx::error!("EcSpireDistributedScan remote tuple payload JSON decode failed: {e}")
        });
        let payload_object = payload.as_object().unwrap_or_else(|| {
            pgrx::error!("EcSpireDistributedScan remote tuple payload must be a JSON object")
        });

        if attr_inputs.len() != usize::try_from((*tuple_desc).natts).unwrap_or(usize::MAX) {
            pgrx::error!("EcSpireDistributedScan tuple payload input cache width mismatch");
        }

        pg_sys::ExecClearTuple(slot);
        let natts = (*tuple_desc).natts;
        for attr_index in 0..natts {
            let attr = pg_sys::TupleDescAttr(tuple_desc, attr_index);
            if attr.is_null() || (*attr).attisdropped {
                *(*slot).tts_isnull.add(attr_index as usize) = true;
                *(*slot).tts_values.add(attr_index as usize) = pg_sys::Datum::from(0);
                continue;
            }
            let attr_name = std::ffi::CStr::from_ptr((*attr).attname.data.as_ptr())
                .to_str()
                .unwrap_or_else(|_| {
                    pgrx::error!("EcSpireDistributedScan relation attribute name is not UTF-8")
                });
            match payload_object.get(attr_name) {
                None | Some(serde_json::Value::Null) => {
                    *(*slot).tts_isnull.add(attr_index as usize) = true;
                    *(*slot).tts_values.add(attr_index as usize) = pg_sys::Datum::from(0);
                }
                Some(value) => {
                    *(*slot).tts_isnull.add(attr_index as usize) = false;
                    let Some(attr_input) = attr_inputs
                        .get_mut(attr_index as usize)
                        .and_then(Option::as_mut)
                    else {
                        pgrx::error!(
                            "EcSpireDistributedScan tuple payload input cache missing attribute {}",
                            attr_index + 1
                        );
                    };
                    *(*slot).tts_values.add(attr_index as usize) =
                        custom_scan_json_value_to_datum(value, attr_name, attr_input);
                }
            }
        }
        (*slot).tts_nvalid = i16::try_from(natts)
            .unwrap_or_else(|_| pgrx::error!("EcSpireDistributedScan tuple descriptor too wide"));
        pg_sys::ExecStoreVirtualTuple(slot)
    }
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn custom_scan_store_tuple_payload_json_for_test(
    slot: *mut pg_sys::TupleTableSlot,
    payload_json: &str,
) -> *mut pg_sys::TupleTableSlot {
    unsafe {
        if slot.is_null() {
            pgrx::error!("EcSpireDistributedScan tuple payload slot is null");
        }
        let mut attr_inputs = custom_scan_payload_attr_io((*slot).tts_tupleDescriptor);
        custom_scan_store_tuple_payload_json(slot, payload_json, &mut attr_inputs)
    }
}

unsafe fn custom_scan_json_value_to_datum(
    value: &serde_json::Value,
    attr_name: &str,
    attr_input: &mut SpireCustomScanPayloadAttrIo,
) -> pg_sys::Datum {
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

unsafe fn custom_scan_store_tuple_payload_typed(
    slot: *mut pg_sys::TupleTableSlot,
    payload: &super::SpireRemoteTypedTuplePayload,
    attr_inputs: &mut [Option<SpireCustomScanPayloadAttrIo>],
) -> *mut pg_sys::TupleTableSlot {
    unsafe {
        if slot.is_null() {
            pgrx::error!("EcSpireDistributedScan tuple payload slot is null");
        }
        let tuple_desc = (*slot).tts_tupleDescriptor;
        if tuple_desc.is_null() {
            pgrx::error!("EcSpireDistributedScan tuple payload slot has no tuple descriptor");
        }
        if attr_inputs.len() != usize::try_from((*tuple_desc).natts).unwrap_or(usize::MAX) {
            pgrx::error!("EcSpireDistributedScan tuple payload input cache width mismatch");
        }
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

        pg_sys::ExecClearTuple(slot);
        let natts = (*tuple_desc).natts;
        for attr_index in 0..natts {
            let attr = pg_sys::TupleDescAttr(tuple_desc, attr_index);
            if attr.is_null() || (*attr).attisdropped {
                *(*slot).tts_isnull.add(attr_index as usize) = true;
                *(*slot).tts_values.add(attr_index as usize) = pg_sys::Datum::from(0);
                continue;
            }
            let attr_name = std::ffi::CStr::from_ptr((*attr).attname.data.as_ptr())
                .to_str()
                .unwrap_or_else(|_| {
                    pgrx::error!("EcSpireDistributedScan relation attribute name is not UTF-8")
                });
            let attr_attnum = (*attr).attnum;
            let payload_position = payload
                .payload_attnums
                .iter()
                .position(|attnum| *attnum == attr_attnum);
            let Some(payload_position) = payload_position else {
                *(*slot).tts_isnull.add(attr_index as usize) = true;
                *(*slot).tts_values.add(attr_index as usize) = pg_sys::Datum::from(0);
                continue;
            };
            custom_scan_validate_typed_payload_attr(payload, payload_position, attr_name);
            if payload.payload_nulls[payload_position] {
                *(*slot).tts_isnull.add(attr_index as usize) = true;
                *(*slot).tts_values.add(attr_index as usize) = pg_sys::Datum::from(0);
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
            *(*slot).tts_isnull.add(attr_index as usize) = false;
            *(*slot).tts_values.add(attr_index as usize) = custom_scan_binary_value_to_datum(
                &payload.payload_values[payload_position],
                attr_name,
                attr_input,
            );
        }
        (*slot).tts_nvalid = i16::try_from(natts)
            .unwrap_or_else(|_| pgrx::error!("EcSpireDistributedScan tuple descriptor too wide"));
        pg_sys::ExecStoreVirtualTuple(slot)
    }
}

unsafe fn custom_scan_validate_typed_payload_attr(
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

unsafe fn custom_scan_binary_value_to_datum(
    value: &[u8],
    attr_name: &str,
    attr_input: &mut SpireCustomScanPayloadAttrIo,
) -> pg_sys::Datum {
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

