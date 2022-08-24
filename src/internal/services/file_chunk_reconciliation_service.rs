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
    shared_reconciler_rust_libraries::models::entities::{
        app_errors::{AppError, AppErrorKind},
        file_upload_chunk::FileUploadChunk,
    },
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

        //get a mutable handle to the primary file chunk
        let mut primary_file_chunk = reconcile_primary_file_chunk_request
            .primary_file_chunk
            .clone();

        //go get the next chunk from the comparison file
        let comparison_file_chunk = self
            .pubsub_repo
            .get_next_comparison_file_upload_chunk(&primary_file_chunk.comparison_file_chunks_queue)
            .await?;

        //we reconcile the primary file chunk
        let mut reconciled_primary_file_chunk = self
            .file_reconciliation_algorithm
            .reconcile_primary_file_chunk(&mut primary_file_chunk, &comparison_file_chunk)
            .await?;

        //after reconciliation of the chunk,
        //we acknowledge reciept of the comparison file chunk by marking it as processed
        let is_processed = self
            .pubsub_repo
            .mark_comparison_file_chunk_as_processed(&comparison_file_chunk)
            .await?;

        //if we fail to mark the comparison file chunk as processed
        //we return an error
        if !is_processed {
            return Err(AppError::new(
                AppErrorKind::InternalError,
                String::from("failed to mark comparison file chunk as processed"),
            ));
        }

        //we update the primary file chunk to point to track this comparison file chunks ID
        //as the last_acknowledged_id
        reconciled_primary_file_chunk
            .comparison_file_chunks_queue
            .last_acknowledged_id = Some(comparison_file_chunk.id);

        //if the comparison file chunk we got was actually
        //the last one in the comparison file, it means reconciliation is done
        if comparison_file_chunk.is_last_chunk {
            return self
                .insert_into_recon_results_queue(&reconciled_primary_file_chunk)
                .await;
        }

        //we insert this primary file chunk back into the
        //buttom of the primary file queue
        return self
            .reinsert_into_primary_file_chunks_queue(&reconciled_primary_file_chunk)
            .await;
    }
}

impl FileChunkReconciliationService {
    //handles insertion of a file chunk into the recon results queue
    //as well as any errors from that process
    async fn insert_into_recon_results_queue(
        &self,
        reconciled_primary_file_chunk: &FileUploadChunk,
    ) -> Result<ReconcileFileChunkResponse, AppError> {
        let is_inserted = self
            .pubsub_repo
            .insert_file_chunk_into_recon_results_queue(&reconciled_primary_file_chunk)
            .await?;

        //failed to insert
        if !is_inserted {
            return Err(AppError::new(
                AppErrorKind::InternalError,
                String::from("failed to insert primary file chunk to bottom of ReconResultsQueue"),
            ));
        }

        //we then return success such that its removed
        //from the top of the primary file queue
        return Ok(ReconcileFileChunkResponse {
            file_chunk_id: reconciled_primary_file_chunk.id.clone(),
        });
    }

    //handles insertion of a file chunk into the primary file queue
    //as well as any errors from that process
    async fn reinsert_into_primary_file_chunks_queue(
        &self,
        reconciled_primary_file_chunk: &FileUploadChunk,
    ) -> Result<ReconcileFileChunkResponse, AppError> {
        let is_inserted = self
            .pubsub_repo
            .insert_file_chunk_in_primary_file_queue(&reconciled_primary_file_chunk)
            .await?;

        //failed to insert
        if !is_inserted {
            return Err(AppError::new(
                AppErrorKind::InternalError,
                String::from("failed to insert primary file chunk to bottom of PrimaryFileQueue"),
            ));
        }

        //we then return success such that its removed from
        //the top of the primary file queue
        return Ok(ReconcileFileChunkResponse {
            file_chunk_id: reconciled_primary_file_chunk.id.clone(),
        });
    }
}
