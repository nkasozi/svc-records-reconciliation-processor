use crate::internal::shared_reconciler_rust_libraries::models::entities::{
    app_errors::AppError, file_upload_chunk::FileUploadChunk,
};
use async_trait::async_trait;
use mockall::automock;

#[automock]
#[async_trait]
pub trait PubSubRepositoryInterface: Send + Sync {
    async fn get_next_comparison_file_upload_chunk(&self) -> Result<FileUploadChunk, AppError>;
    async fn mark_comparison_file_chunk_as_processed(
        &self,
        file_chunk: &FileUploadChunk,
    ) -> Result<bool, AppError>;
    async fn insert_file_chunk_in_primary_file_queue(
        &self,
        file_chunk: &FileUploadChunk,
    ) -> Result<bool, AppError>;
}
