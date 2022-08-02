use crate::internal::{
    interfaces::{
        file_chunk_reconciliation_service::FileChunkReconciliationServiceInterface,
        pubsub_repo::PubSubRepositoryInterface,
        recon_tasks_repo::ReconTasksDetailsRetrieverInterface,
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
use uuid::Uuid;
use validator::Validate;

const FILE_CHUNK_PREFIX: &'static str = "FILE-CHUNK";

pub struct FileChunkReconciliationService {
    pub pubsub_repo: Box<dyn PubSubRepositoryInterface>,
    pub recon_tasks_repo: Box<dyn ReconTasksDetailsRetrieverInterface>,
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

        //get the reconciliation task details
        let primary_file_recon_task_details = self
            .recon_tasks_repo
            .get_recon_task_details(
                reconcile_primary_file_chunk_request
                    .primary_file_chunk
                    .upload_request_id
                    .clone(),
            )
            .await?;

        //go get the next chunk from the comparison file
        let comparison_file_chunk = self
            .pubsub_repo
            .get_next_comparison_file_upload_chunk()
            .await?;

        //for each row in the primary file chunk
        for primary_chunk_row in reconcile_primary_file_chunk_request
            .primary_file_chunk
            .chunk_rows
        {
            //we get all the columns in the primary chunk row
            let primary_file_row_parts = primary_chunk_row.parsed_columns_from_row.clone();
            //if primary_file_row_parts.len() != primary_file_recon_task_details.

            //for each row in the comparison file chnk
            for comparison_chunk_row in comparison_file_chunk.chunk_rows {
                //we get all the columns in the comparison chunk row
                let comparison_file_row_parts =
                    comparison_chunk_row.parsed_columns_from_row.clone();

                //we then check if this is supposed to be the same row in both files
                //by checking the identity columns in the comparison comparison_pairs

                //if its supposed to be the same row, we can begin checking each column in the primary file row
                //vs each column in the comparison file row
                for primary_file_column_in_row in primary_file_row_parts.clone() {
                    //if
                }
            }
        }

        // match file_save_result {
        //     Ok(file_chunk_id) => Ok(ReconcileFileChunkResponse { file_chunk_id }),
        //     Err(e) => Err(e),
        // }
        todo!()
    }
}

impl FileChunkReconciliationService {
    fn transform_into_file_upload_chunk(
        &self,
        upload_file_chunk_request: &ReconcileFileChunkRequest,
    ) -> FileUploadChunk {
        FileUploadChunk {
            id: self.generate_uuid(FILE_CHUNK_PREFIX),
            upload_request_id: upload_file_chunk_request
                .primary_file_chunk
                .upload_request_id
                .clone(),
            chunk_sequence_number: upload_file_chunk_request
                .primary_file_chunk
                .chunk_sequence_number
                .clone(),
            chunk_source: upload_file_chunk_request
                .primary_file_chunk
                .chunk_source
                .clone(),
            chunk_rows: upload_file_chunk_request
                .primary_file_chunk
                .chunk_rows
                .clone(),
            date_created: chrono::Utc::now().timestamp(),
            date_modified: chrono::Utc::now().timestamp(),
            comparison_pairs: upload_file_chunk_request
                .primary_file_chunk
                .comparison_pairs
                .clone(),
            recon_config: upload_file_chunk_request
                .primary_file_chunk
                .recon_config
                .clone(),
        }
    }

    fn generate_uuid(&self, prefix: &str) -> String {
        let id = Uuid::new_v4().to_string();
        let full_id = String::from(format!("{}-{}", prefix, id));
        return full_id;
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::internal::{
//         entities::app_error::{AppError, AppErrorKind},
//         interfaces::{
//             file_chunk_reconciliation_service::FileChunkReconciliationServiceInterface,
//             pubsub_repo::{MockPubSubRepositoryInterface, PubSubRepositoryInterface},
//         },
//         view_models::reconcile_file_chunk_request::ReconcileFileChunkRequest,
//     };

//     use crate::internal::entities::file_upload_chunk::FileUploadChunkSource;

//     use super::FileChunkReconciliationService;

//     #[actix_rt::test]
//     async fn given_valid_request_calls_correct_dependencie_and_returns_success() {
//         let mut mock_file_upload_repo = Box::new(MockPubSubRepositoryInterface::new());

//         mock_file_upload_repo
//             .expect_save_file_upload_chunk()
//             .returning(|_y| Ok(String::from("FILE_CHUNK_1234")));

//         let sut = FileChunkReconciliationService {
//             pubsub_repo: mock_file_upload_repo,
//         };

//         let test_request = ReconcileFileChunkRequest {
//             upload_request_id: String::from("1234"),
//             chunk_sequence_number: 2,
//             chunk_source: FileUploadChunkSource::PrimaryFileChunk,
//             chunk_rows: vec![String::from("testing, 1234")],
//         };

//         let actual = sut.reconcile_file_chunk(&test_request).await;

//         assert!(actual.is_ok());
//     }

//     #[actix_rt::test]
//     async fn given_invalid_request_returns_error() {
//         let mut mock_file_upload_repo = Box::new(MockPubSubRepositoryInterface::new());

//         mock_file_upload_repo
//             .expect_save_file_upload_chunk()
//             .returning(|_y| Ok(String::from("FILE_CHUNK_1234")));

//         let sut = FileChunkReconciliationService {
//             pubsub_repo: mock_file_upload_repo,
//         };

//         let test_request = ReconcileFileChunkRequest {
//             upload_request_id: String::from("1234"),
//             chunk_sequence_number: 0,
//             chunk_source: FileUploadChunkSource::ComparisonFileChunk,
//             chunk_rows: vec![String::from("testing, 1234")],
//         };

//         let actual = sut.reconcile_file_chunk(&test_request).await;

//         assert!(actual.is_err());
//     }

//     #[actix_rt::test]
//     async fn given_valid_request_but_repo_returns_error_returns_error() {
//         let mut mock_file_upload_repo = Box::new(MockPubSubRepositoryInterface::new());

//         mock_file_upload_repo
//             .expect_save_file_upload_chunk()
//             .returning(|_y| {
//                 Err(AppError::new(
//                     AppErrorKind::ConnectionError,
//                     "unable to connect".to_string(),
//                 ))
//             });

//         let sut = FileChunkReconciliationService {
//             pubsub_repo: mock_file_upload_repo,
//         };

//         let test_request = ReconcileFileChunkRequest {
//             upload_request_id: String::from("1234"),
//             chunk_sequence_number: 2,
//             chunk_source: FileUploadChunkSource::ComparisonFileChunk,
//             chunk_rows: vec![String::from("testing, 1234")],
//         };

//         let actual = sut.reconcile_file_chunk(&test_request).await;

//         assert!(actual.is_err());
//     }
// }
