use stellar_xdr::{
    HostFunction, InvokeHostFunctionResult, OperationBody, OperationResult, OperationResultTr,
    ReadXdr, ScObject, ScVal, TransactionEnvelope, TransactionMeta, TransactionMetaV3,
    TransactionResult, TransactionResultResult, TransactionV1Envelope,
};

use crate::types::common::{Deployed, Event, Invocation};

use super::types;

pub async fn get_transaction(base_url: &str, hash: &str) -> Option<types::transaction::Response> {
    let url = format!("{base_url}/transactions/{hash}");

    let result = reqwest::get(&url).await;
    match result {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<types::transaction::Response>().await {
                    Ok(resp) => Some(resp),
                    Err(_) => None,
                }
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

pub fn process_tx(tx_high: types::transaction::Response) -> super::types::common::Processed {
    let envelope = stellar_xdr::TransactionEnvelope::from_xdr_base64(tx_high.envelope_xdr);
    let body = match envelope {
        Ok(content) => match content {
            TransactionEnvelope::Tx(TransactionV1Envelope { tx, .. }) => {
                match tx.operations.get(0) {
                    Some(op) => match &op.body {
                        OperationBody::InvokeHostFunction(inv_h_fn_op) => {
                            match inv_h_fn_op.function {
                                HostFunction::CreateContractWithSourceAccount => {
                                    let id = if let Ok(TransactionResult {
                                        result: TransactionResultResult::TxSuccess(op_results),
                                        ..
                                    }) =
                                        TransactionResult::from_xdr_base64(tx_high.result_xdr)
                                    {
                                        if let Some(OperationResult::OpInner(
                                            OperationResultTr::InvokeHostFunction(
                                                InvokeHostFunctionResult::Success(ScVal::Object(
                                                    Some(ScObject::Bytes(id)),
                                                )),
                                            ),
                                        )) = op_results.get(0)
                                        {
                                            Some(hex::encode(id))
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    };
                                    let bytes = if let Some(code) = inv_h_fn_op.parameters.get(0) {
                                        if let ScVal::Object(Some(ScObject::Bytes(bytes))) = code {
                                            Some(bytes.into())
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    };

                                    if let (Some(id), Some(bytes)) = (id, bytes) {
                                        Some(Event::Deployment(Deployed { id, bytes }))
                                    } else {
                                        None
                                    }
                                }
                                // HostFunction::CreateContractWithEd25519 => {}
                                HostFunction::InvokeContract => {
                                    let ctr_id = if let Some(id) = inv_h_fn_op.parameters.get(0) {
                                        if let ScVal::Object(Some(ScObject::Bytes(id))) = id {
                                            Some(hex::encode(id))
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    };

                                    let function =
                                        if let Some(function) = inv_h_fn_op.parameters.get(1) {
                                            if let ScVal::Symbol(function) = function {
                                                Some(function.to_string_lossy())
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        };
                                    let args = inv_h_fn_op
                                        .parameters
                                        .iter()
                                        .skip(2)
                                        .map(|a| Some(a.clone()))
                                        .collect::<Vec<_>>();

                                    let result = if let Ok(TransactionResult {
                                        result: TransactionResultResult::TxSuccess(op_results),
                                        ..
                                    }) =
                                        TransactionResult::from_xdr_base64(tx_high.result_xdr)
                                    {
                                        if let Some(OperationResult::OpInner(
                                            OperationResultTr::InvokeHostFunction(
                                                InvokeHostFunctionResult::Success(result),
                                            ),
                                        )) = op_results.get(0)
                                        {
                                            Some(result.clone())
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    };

                                    let contract_events =
                                        if let Ok(TransactionMeta::V3(TransactionMetaV3 {
                                            events,
                                            ..
                                        })) = TransactionMeta::from_xdr_base64(
                                            tx_high.result_meta_xdr,
                                        ) {
                                            Some(events.into())
                                        } else {
                                            None
                                        };

                                    let footprint = Some(inv_h_fn_op.footprint.clone());

                                    if let (Some(id), Some(function)) = (ctr_id, function) {
                                        Some(Event::Invocation(Invocation {
                                            id,
                                            function,
                                            args,
                                            result,
                                            footprint,
                                            events: contract_events,
                                        }))
                                    } else {
                                        None
                                    }
                                }
                                _ => None, // TODO: token contracts
                            }
                        }
                        _ => None,
                    },

                    None => None,
                }
            }
            _ => None,
        },
        Err(_) => None,
    };

    super::types::common::Processed {
        source_account: tx_high.source_account,
        tx: tx_high.id,
        at: tx_high.created_at,
        body: body.unwrap(), // should be safe
    }
}
