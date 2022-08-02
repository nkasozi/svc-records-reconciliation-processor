use crate::internal::{
    models::view_models::{
        requests::reconcile_file_chunk_request::ReconcileFileChunkRequest,
        responses::reconcile_file_chunk_response::ReconcileFileChunkResponse,
    },
    shared_reconciler_rust_libraries::models::entities::app_errors::AppError,
};
use async_trait::async_trait;
use mockall::automock;

#[automock]
#[async_trait]
pub trait FileChunkReconciliationServiceInterface: Send + Sync {
    async fn reconcile_file_chunk(
        &self,
        primary_file_chunk: &ReconcileFileChunkRequest,
    ) -> Result<ReconcileFileChunkResponse, AppError>;
}
