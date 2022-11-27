use serde_derive::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response {
    pub id: String,
    pub paging_token: String,
    pub successful: bool,
    pub hash: String,
    pub ledger: i64,
    pub created_at: String,
    pub source_account: String,
    pub source_account_sequence: String,
    pub fee_account: String,
    pub fee_charged: String,
    pub max_fee: String,
    pub operation_count: i64,
    pub envelope_xdr: String,
    pub result_xdr: String,
    pub result_meta_xdr: String,
    pub fee_meta_xdr: String,
    pub memo_type: String,
    pub signatures: Vec<String>,
}
