use serde::{Deserialize, Serialize};
use stellar_xdr::{ContractEvent, LedgerFootprint, ScVal};

#[derive(Clone, Serialize, Deserialize)]
pub struct Invocation {
    pub id: String,
    pub function: String,
    pub args: Vec<Option<ScVal>>,
    pub result: Option<ScVal>,
    pub footprint: Option<LedgerFootprint>,
    pub events: Option<Vec<ContractEvent>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Deployed {
    pub id: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Event {
    Deployment(Deployed),
    Invocation(Invocation),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Processed {
    pub source_account: String,
    pub tx: String,
    pub at: String,
    pub body: Event,
}
