use async_trait::async_trait;
use mockall::automock;

use crate::internal::{
    models::view_models::responses::svc_recon_tasks_details_response::ReconTaskResponseDetails,
    shared_reconciler_rust_libraries::models::entities::app_errors::AppError,
};

#[automock]
#[async_trait]
pub trait ReconTasksDetailsRetrieverInterface: Send + Sync {
    async fn get_recon_task_details<'a>(
        &self,
        task_id: String,
    ) -> Result<ReconTaskResponseDetails, AppError>;
}
