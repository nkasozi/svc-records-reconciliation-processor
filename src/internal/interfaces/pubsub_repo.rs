use crate::internal::shared_reconciler_rust_libraries::models::entities::{
    app_errors::AppError, file_chunk_queue::FileChunkQueue, file_upload_chunk::FileUploadChunk,
};
use async_trait::async_trait;
use mockall::automock;

#[automock]
#[async_trait]
pub trait PubSubRepositoryInterface: Send + Sync {
    async fn get_next_comparison_file_upload_chunk<'a>(
        &'a self,
        queue: &'a FileChunkQueue,
    ) -> Result<FileUploadChunk, AppError>;

    async fn mark_comparison_file_chunk_as_processed<'a>(
        &'a self,
        file_chunk: &'a FileUploadChunk,
    ) -> Result<bool, AppError>;

    async fn insert_file_chunk_in_primary_file_queue<'a>(
        &'a self,
        file_chunk: &'a FileUploadChunk,
    ) -> Result<bool, AppError>;

    async fn insert_file_chunk_into_recon_results_queue<'a>(
        &'a self,
        file_chunk: &'a FileUploadChunk,
    ) -> Result<bool, AppError>;
}
