use stellar_xdr::{ReadXdr, ScObject, ScVal};

use self::utils::{build_processed_from_filter, exp_backoff_request};

use super::types::{self, common::Processed};

mod utils {
    use super::types::{
        self,
        common::{Deployed, Event, Invocation, Processed},
    };
    use crate::transactions::get_transaction;
    use stellar_xdr::{
        InvokeHostFunctionResult, LedgerFootprint, OperationResult, OperationResultTr, ReadXdr,
        ScObject, ScVal, TransactionMeta, TransactionMetaV3, TransactionResult,
        TransactionResultResult,
    };

    pub async fn exp_backoff_request(url: &str) -> types::operation::Response {
        let backoff = backoff::ExponentialBackoff::default();
        backoff::future::retry(backoff, || async {
            let result = reqwest::get(url).await;
            match result {
                Ok(resp) => {
                    if resp.status().is_success() {
                        match resp.json::<types::operation::Response>().await {
                            Ok(resp) => Ok(resp),
                            Err(_) => Err(backoff::Error::transient(())),
                        }
                    } else {
                        Err(backoff::Error::transient(()))
                    }
                }
                Err(_) => Err(backoff::Error::transient(())),
            }
        })
        .await
        .unwrap()
    }

    pub async fn build_processed_from_filter(
        base_url: &str,
        records: impl Iterator<Item = &types::operation::Record>,
        events: &mut Vec<Processed>,
    ) {
        for r in records {
            let source_account = &r.source_account;
            match r.function.as_deref() {
                Some("HostFunctionHostFnInvokeContract") => {
                    let id = if let Some(id) = r.parameters.get(0) {
                        if let Ok(ScVal::Object(Some(ScObject::Bytes(id)))) =
                            ScVal::from_xdr_base64(&id.value)
                        {
                            Some(hex::encode(id))
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let function = if let Some(function) = r.parameters.get(1) {
                        if let Ok(ScVal::Symbol(function)) = ScVal::from_xdr_base64(&function.value)
                        {
                            Some(function.to_string_lossy())
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    let args = r
                        .parameters
                        .iter()
                        .skip(2)
                        .map(|a| ScVal::from_xdr_base64(&a.value).ok())
                        .collect::<Vec<_>>();
                    let tx = get_transaction(base_url, &r.transaction_hash)
                        .await
                        .unwrap();
                    let result = if let Ok(TransactionResult {
                        result: TransactionResultResult::TxSuccess(op_results),
                        ..
                    }) = TransactionResult::from_xdr_base64(tx.result_xdr)
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
                        if let Ok(TransactionMeta::V3(TransactionMetaV3 { events, .. })) =
                            TransactionMeta::from_xdr_base64(tx.result_meta_xdr)
                        {
                            Some(events.into())
                        } else {
                            None
                        };
                    let footprint = if let Some(footprint) = &r.footprint {
                        if let Ok(footprint) = LedgerFootprint::from_xdr_base64(footprint) {
                            Some(footprint)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    if let (Some(id), Some(function)) = (id, function) {
                        events.push(Processed {
                            source_account: source_account.to_string(),
                            tx: r.transaction_hash.clone(),
                            at: r.created_at.clone(),
                            body: Event::Invocation(Invocation {
                                id,
                                function,
                                args,
                                result,
                                footprint,
                                events: contract_events,
                            }),
                        });
                    }
                }
                Some("HostFunctionHostFnCreateContractWithSourceAccount") => {
                    let tx = get_transaction(base_url, &r.transaction_hash)
                        .await
                        .unwrap();
                    let id = if let Ok(TransactionResult {
                        result: TransactionResultResult::TxSuccess(op_results),
                        ..
                    }) = TransactionResult::from_xdr_base64(tx.result_xdr)
                    {
                        if let Some(OperationResult::OpInner(
                            OperationResultTr::InvokeHostFunction(
                                InvokeHostFunctionResult::Success(ScVal::Object(Some(
                                    ScObject::Bytes(id),
                                ))),
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
                    let bytes = if let Some(code) = r.parameters.get(0) {
                        if let Ok(ScVal::Object(Some(ScObject::Bytes(bytes)))) =
                            ScVal::from_xdr_base64(&code.value)
                        {
                            Some(bytes.into())
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    if let (Some(id), Some(bytes)) = (id, bytes) {
                        events.push(Processed {
                            source_account: source_account.to_string(),
                            tx: r.transaction_hash.clone(),
                            at: r.created_at.clone(),
                            body: Event::Deployment(Deployed { id, bytes }),
                        });
                    }
                }
                _ => {}
            }
        }
    }
}

pub async fn get_operations(base_url: &str, url: &str) -> (Vec<Processed>, Option<String>, String) {
    let resp = exp_backoff_request(url).await;
    let records = resp
        .embedded
        .records
        .iter()
        .filter(|r| r.r#type == "invoke_host_function");

    let mut events: Vec<Processed> = vec![];
    build_processed_from_filter(base_url, records, &mut events).await;
    (
        events,
        resp.embedded
            .records
            .first()
            .map(|r| r.paging_token.clone()),
        resp.links.next.href,
    )
}

pub async fn get_contract_operations(
    base_url: &str,
    url: &str,
    contract_id: &str,
) -> (Vec<Processed>, Option<String>, String) {
    let resp = exp_backoff_request(url).await;
    let records = resp.embedded.records.iter().filter(|r| {
        let id = if let Some(id) = r.parameters.get(0) {
            if let Ok(ScVal::Object(Some(ScObject::Bytes(id)))) = ScVal::from_xdr_base64(&id.value)
            {
                Some(hex::encode(id))
            } else {
                None
            }
        } else {
            None
        };
        if r.r#type == "invoke_host_function" && id == Some(contract_id.to_string()) {
            true
        } else {
            false
        }
    });

    let mut events: Vec<Processed> = vec![];
    build_processed_from_filter(base_url, records, &mut events).await;
    (
        events,
        resp.embedded
            .records
            .first()
            .map(|r| r.paging_token.clone()),
        resp.links.next.href,
    )
}
