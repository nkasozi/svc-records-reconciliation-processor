use crate::internal::shared_reconciler_rust_libraries::models::entities::{
    app_errors::AppError, file_chunk_queue::FileChunkQueue, file_upload_chunk::FileUploadChunk,
};
use async_trait::async_trait;
use mockall::automock;

#[automock]
#[async_trait]
pub trait PubSubRepositoryInterface: Send + Sync {
    async fn get_next_comparison_file_upload_chunk(
        &self,
        queue: &FileChunkQueue,
    ) -> Result<FileUploadChunk, AppError>;

    async fn mark_comparison_file_chunk_as_processed(
        &self,
        file_chunk: &FileUploadChunk,
    ) -> Result<bool, AppError>;

    async fn insert_file_chunk_in_primary_file_queue(
        &self,
        file_chunk: &FileUploadChunk,
    ) -> Result<bool, AppError>;

    async fn insert_file_chunk_into_recon_results_queue(
        &self,
        file_chunk: &FileUploadChunk,
    ) -> Result<bool, AppError>;
}
