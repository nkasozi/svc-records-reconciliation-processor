use async_trait::async_trait;
use mockall::automock;

use crate::internal::shared_reconciler_rust_libraries::models::entities::{
    app_errors::AppError, file_upload_chunk::FileUploadChunk,
};

#[automock]
#[async_trait]
pub trait FileReconciliationAlgorithmInterface: Send + Sync {
    async fn reconcile_primary_file_chunk(
        &self,
        primary_file_chunk: &mut FileUploadChunk,
        comparison_file_chunk: &FileUploadChunk,
    ) -> Result<FileUploadChunk, AppError>;
}
