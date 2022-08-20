use crate::internal::shared_reconciler_rust_libraries::models::entities::file_upload_chunk::FileUploadChunk;
use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedisStreamMessage {
    pub tracestate: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub source: String,
    pub data: FileUploadChunk,
    pub pubsubname: String,
    pub specversion: String,
    pub traceid: String,
    pub traceparent: String,
    pub topic: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub description: String,
    pub price: i64,
    pub widget_field: String,
}
