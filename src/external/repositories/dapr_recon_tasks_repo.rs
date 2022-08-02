use async_trait::async_trait;

use crate::internal::{
    entities::{app_error::AppError, recon_task::ReconFileDetails},
    interfaces::recon_tasks_repo::ReconTasksDetailsRetrieverInterface,
};

pub struct ReconTasksDetailsRetriever {
    pub dapr_grpc_server_address: String,
    pub dapr_service_name: String,
}

#[async_trait]
impl ReconTasksDetailsRetrieverInterface for ReconTasksDetailsRetriever {
    async fn get_primary_recon_task_details(
        &self,
        task_id: String,
    ) -> Result<ReconFileDetails, AppError> {
        todo!()
    }
}
