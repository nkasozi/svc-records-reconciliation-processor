use crate::internal::entities::{app_error::AppError, recon_task::ReconFileDetails};
use async_trait::async_trait;
use mockall::automock;

#[automock]
#[async_trait]
pub trait ReconTasksDetailsRetrieverInterface: Send + Sync {
    async fn get_primary_recon_task_details(
        &self,
        task_id: String,
    ) -> Result<ReconFileDetails, AppError>;
}
