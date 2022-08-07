use crate::internal::{
    interfaces::{
        file_chunk_reconciliation_service::FileChunkReconciliationServiceInterface,
        file_reconciliation_algorithm::FileReconciliationAlgorithmInterface,
        pubsub_repo::PubSubRepositoryInterface,
    },
    models::view_models::{
        requests::reconcile_file_chunk_request::ReconcileFileChunkRequest,
        responses::reconcile_file_chunk_response::ReconcileFileChunkResponse,
    },
    shared_reconciler_rust_libraries::models::entities::app_errors::{AppError, AppErrorKind},
};
use async_trait::async_trait;
use validator::Validate;

pub struct FileChunkReconciliationService {
    pub pubsub_repo: Box<dyn PubSubRepositoryInterface>,
    pub file_reconciliation_algorithm: Box<dyn FileReconciliationAlgorithmInterface>,
}

#[async_trait]
impl FileChunkReconciliationServiceInterface for FileChunkReconciliationService {
    /**
    uploads a file chunk to the repository

    # Errors

    This function will return an error if the request fails validation or fails to be uploaded.
    */
    async fn reconcile_file_chunk(
        &self,
        reconcile_primary_file_chunk_request: &ReconcileFileChunkRequest,
    ) -> Result<ReconcileFileChunkResponse, AppError> {
        //validate request
        match reconcile_primary_file_chunk_request.validate() {
            Ok(_) => (),
            Err(e) => {
                return Err(AppError::new(
                    AppErrorKind::BadClientRequest,
                    e.to_string().replace("\n", " , "),
                ));
            }
        }

        let mut primary_file_chunk = reconcile_primary_file_chunk_request
            .primary_file_chunk
            .clone();

        //go get the next chunk from the comparison file
        let comparison_file_chunk = self
            .pubsub_repo
            .get_next_comparison_file_upload_chunk()
            .await?;

        //we reconcile the primary file chunk
        let reconciled_primary_file_chunk = self
            .file_reconciliation_algorithm
            .reconcile_primary_file_chunk(&mut primary_file_chunk, &comparison_file_chunk)
            .await?;

        //after reconciliation of the chunk,
        //we acknowledge reciept of the comparison file chunk by marking it as processed
        let is_processed = self
            .pubsub_repo
            .mark_comparison_file_chunk_as_processed(&comparison_file_chunk)
            .await?;

        if !is_processed {
            return Err(AppError::new(
                AppErrorKind::InternalError,
                String::from("failed to mark comparison file chunk as processed"),
            ));
        }

        //we insert this primary file chunk back into the buttom of the primary file queue
        let is_inserted = self
            .pubsub_repo
            .insert_file_chunk_in_primary_file_queue(&reconciled_primary_file_chunk)
            .await?;

        if !is_inserted {
            return Err(AppError::new(
                AppErrorKind::InternalError,
                String::from("failed to insert primary file chunk to bottom of PrimaryFileQueue"),
            ));
        }

        //we then return success such that its removed from the top of the primary file queue
        return Ok(ReconcileFileChunkResponse {
            file_chunk_id: reconciled_primary_file_chunk.id,
        });
    }
}
