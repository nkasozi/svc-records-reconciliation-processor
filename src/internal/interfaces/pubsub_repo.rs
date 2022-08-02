use crate::internal::shared_reconciler_rust_libraries::models::entities::{
    app_errors::AppError, file_upload_chunk::FileUploadChunk,
};
use async_trait::async_trait;
use mockall::automock;

#[automock]
#[async_trait]
pub trait PubSubRepositoryInterface: Send + Sync {
    async fn get_next_comparison_file_upload_chunk(&self) -> Result<FileUploadChunk, AppError>;
}
