use chrono::NaiveDateTime;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Domain {
    pub id: Option<uuid::Uuid>,
    pub name: String,
    pub ttl: Option<i32>,
    pub serial: Option<i32>,
    pub email: String,
    pub gslb: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Record {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_id: Option<uuid::Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<uuid::Uuid>,
    pub name: String,
    #[serde(rename = "type")]
    pub record_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<i32>,
    pub data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gslb_region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gslb_weight: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gslb_check: Option<i32>,
}

#[derive(Deserialize)]
pub struct DomainListResponse {
    pub domains: Vec<Domain>,
}

#[derive(Deserialize)]
pub struct RecordListResponse {
    pub records: Vec<Record>,
}
