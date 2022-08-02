use async_trait::async_trait;

use crate::internal::{
    interfaces::recon_tasks_repo::ReconTasksDetailsRetrieverInterface,
    models::view_models::responses::svc_recon_tasks_details_response::ReconTaskResponseDetails,
    shared_reconciler_rust_libraries::models::entities::app_errors::AppError,
};

pub struct ReconTasksDetailsRetriever {
    pub dapr_grpc_server_address: String,
    pub dapr_service_name: String,
}

#[async_trait]
impl ReconTasksDetailsRetrieverInterface for ReconTasksDetailsRetriever {
    async fn get_recon_task_details(
        &self,
        task_id: String,
    ) -> Result<ReconTaskResponseDetails, AppError> {
        todo!()
    }
}
