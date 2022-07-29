use crate::internal::entities::file_upload_chunk::FileUploadChunkSource;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Validate, Debug)]
pub struct ReconcileFileChunkRequest {
    #[validate(length(min = 1, message = "please supply an upload_request_id"))]
    pub upload_request_id: String,

    #[validate(range(min = 1))]
    pub chunk_sequence_number: i64,

    pub chunk_source: FileUploadChunkSource,

    #[validate(length(min = 1, message = "please supply the chunk rows"))]
    pub chunk_rows: Vec<String>,
}
