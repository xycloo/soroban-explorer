use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response {
    #[serde(rename = "_links")]
    pub links: Links,
    #[serde(rename = "_embedded")]
    pub embedded: Embedded,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Links {
    pub next: Next,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Next {
    pub href: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Embedded {
    pub records: Vec<Record>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Record {
    pub id: String,
    pub paging_token: String,
    pub transaction_successful: bool,
    pub source_account: String,
    pub r#type: String,
    pub type_i: i64,
    pub created_at: String,
    pub transaction_hash: String,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
    pub function: Option<String>,
    pub footprint: Option<String>,
    pub funder: Option<String>,
    pub account: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub value: String,
    #[serde(rename = "type")]
    pub type_field: String,
}
