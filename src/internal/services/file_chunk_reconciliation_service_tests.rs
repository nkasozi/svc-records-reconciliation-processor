use crate::internal::{
    interfaces::{
        file_chunk_reconciliation_service::FileChunkReconciliationServiceInterface,
        file_reconciliation_algorithm::MockFileReconciliationAlgorithmInterface,
        pubsub_repo::MockPubSubRepositoryInterface,
    },
    models::view_models::requests::reconcile_file_chunk_request::ReconcileFileChunkRequest,
    shared_reconciler_rust_libraries::models::entities::{
        app_errors::{AppError, AppErrorKind},
        file_chunk_queue::FileChunkQueue,
        file_upload_chunk::{FileUploadChunk, FileUploadChunkSource},
        recon_tasks_models::{ComparisonPair, ReconciliationConfigs},
    },
};

use super::file_chunk_reconciliation_service::FileChunkReconciliationService;

#[actix_web::test]
async fn given_valid_request_calls_correct_dependencies() {
    //setup
    let (mut mock_pubsub_repo, mut mock_file_recon_algo) = setup_dependencies();

    mock_pubsub_repo
        .expect_get_next_comparison_file_upload_chunk()
        .times(1)
        .returning(|_y| Ok(dummy_comparison_file()));

    mock_file_recon_algo
        .expect_reconcile_primary_file_chunk()
        .times(1)
        .returning(|_y, _x| Ok(dummy_reconciled_file_upload_chunk()));

    mock_pubsub_repo
        .expect_mark_comparison_file_chunk_as_processed()
        .times(1)
        .returning(|_y| Ok(true));

    mock_pubsub_repo
        .expect_insert_file_chunk_in_primary_file_queue()
        .times(1)
        .returning(|_y| Ok(true));

    mock_pubsub_repo
        .expect_insert_file_chunk_into_recon_results_queue()
        .times(0)
        .returning(|_y| Ok(true));

    let sut = setup(mock_pubsub_repo, mock_file_recon_algo);

    let request = get_dummy_valid_request();

    //act
    let actual = sut.reconcile_file_chunk(&request).await;

    //assert
    assert_eq!(actual.is_ok(), true);
}

#[actix_web::test]
async fn given_valid_request_and_last_comparison_file_chunk_calls_correct_dependencies() {
    //setup
    let (mut mock_pubsub_repo, mut mock_file_recon_algo) = setup_dependencies();

    mock_pubsub_repo
        .expect_get_next_comparison_file_upload_chunk()
        .times(1)
        .returning(|_y| {
            let mut comparison_file_chunk = dummy_comparison_file();
            comparison_file_chunk.is_last_chunk = true;
            Ok(comparison_file_chunk)
        });

    mock_file_recon_algo
        .expect_reconcile_primary_file_chunk()
        .times(1)
        .returning(|_y, _x| Ok(dummy_reconciled_file_upload_chunk()));

    mock_pubsub_repo
        .expect_mark_comparison_file_chunk_as_processed()
        .times(1)
        .returning(|_y| Ok(true));

    mock_pubsub_repo
        .expect_insert_file_chunk_in_primary_file_queue()
        .times(0)
        .returning(|_y| Ok(true));

    mock_pubsub_repo
        .expect_insert_file_chunk_into_recon_results_queue()
        .times(1)
        .returning(|_y| Ok(true));

    let sut = setup(mock_pubsub_repo, mock_file_recon_algo);

    let request = get_dummy_valid_request();

    //act
    let actual = sut.reconcile_file_chunk(&request).await;

    //assert
    assert_eq!(actual.is_ok(), true);
}

#[actix_web::test]
async fn given_invalid_request_returns_error() {}

#[actix_web::test]
async fn given_valid_request_but_call_to_dependency_fails_returns_error() {
    //setup
    let (mut mock_pubsub_repo, mock_file_recon_algo) = setup_dependencies();

    mock_pubsub_repo
        .expect_get_next_comparison_file_upload_chunk()
        .times(1)
        .returning(|_y| {
            Err(AppError::new(
                AppErrorKind::ConnectionError,
                "unable to connect".to_string(),
            ))
        });

    let sut = setup(mock_pubsub_repo, mock_file_recon_algo);

    let request = get_dummy_valid_request();

    //act
    let actual = sut.reconcile_file_chunk(&request).await;

    //assert
    assert_eq!(actual.is_err(), true);
}

fn setup_dependencies() -> (
    Box<MockPubSubRepositoryInterface>,
    Box<MockFileReconciliationAlgorithmInterface>,
) {
    let mock_pubsub_repo = Box::new(MockPubSubRepositoryInterface::new());
    let mock_file_recon_algo = Box::new(MockFileReconciliationAlgorithmInterface::new());

    return (mock_pubsub_repo, mock_file_recon_algo);
}

fn setup(
    mock_pubsub_repo: Box<MockPubSubRepositoryInterface>,
    mock_file_recon_algo: Box<MockFileReconciliationAlgorithmInterface>,
) -> FileChunkReconciliationService {
    let sut = FileChunkReconciliationService {
        pubsub_repo: mock_pubsub_repo,
        file_reconciliation_algorithm: mock_file_recon_algo,
    };
    return sut;
}

fn get_dummy_valid_request() -> ReconcileFileChunkRequest {
    ReconcileFileChunkRequest {
        primary_file_chunk: FileUploadChunk {
            id: String::from("src-file-1234"),
            upload_request_id: String::from("file-1234"),
            chunk_sequence_number: 1,
            chunk_source: FileUploadChunkSource::ComparisonFileChunk,
            chunk_rows: vec![],
            date_created: chrono::Utc::now().timestamp(),
            date_modified: chrono::Utc::now().timestamp(),
            comparison_pairs: vec![new_same_column_index_comparison_pair(0)],
            column_headers: vec![],
            recon_config: default_recon_configs(),
            primary_file_chunks_queue: FileChunkQueue {
                topic_id: String::from("src-file-chunks-queue-1"),
                last_acknowledged_id: Option::None,
            },
            comparison_file_chunks_queue: FileChunkQueue {
                topic_id: String::from("cmp-file-chunks-queue-1"),
                last_acknowledged_id: Option::None,
            },
            result_chunks_queue: FileChunkQueue {
                topic_id: String::from("results-file-chunks-queue-1"),
                last_acknowledged_id: Option::None,
            },
            is_last_chunk: false,
        },
    }
}

fn default_recon_configs() -> ReconciliationConfigs {
    ReconciliationConfigs {
        should_check_for_duplicate_records_in_comparison_file: true,
        should_reconciliation_be_case_sensitive: true,
        should_ignore_white_space: true,
        should_do_reverse_reconciliation: true,
    }
}

fn new_same_column_index_comparison_pair(column_index: usize) -> ComparisonPair {
    ComparisonPair {
        primary_file_column_index: column_index,
        comparison_file_column_index: column_index,
        is_row_identifier: true,
    }
}

fn dummy_reconciled_file_upload_chunk() -> FileUploadChunk {
    FileUploadChunk {
        id: String::from("src-file-1234"),
        upload_request_id: String::from("file-1234"),
        chunk_sequence_number: 1,
        chunk_source: FileUploadChunkSource::ComparisonFileChunk,
        chunk_rows: vec![],
        date_created: chrono::Utc::now().timestamp(),
        date_modified: chrono::Utc::now().timestamp(),
        comparison_pairs: vec![new_same_column_index_comparison_pair(0)],
        column_headers: vec![],
        recon_config: default_recon_configs(),
        primary_file_chunks_queue: FileChunkQueue {
            topic_id: String::from("src-file-chunks-queue-1"),
            last_acknowledged_id: Option::None,
        },
        comparison_file_chunks_queue: FileChunkQueue {
            topic_id: String::from("cmp-file-chunks-queue-1"),
            last_acknowledged_id: Option::None,
        },
        result_chunks_queue: FileChunkQueue {
            topic_id: String::from("results-file-chunks-queue-1"),
            last_acknowledged_id: Option::None,
        },
        is_last_chunk: false,
    }
}

fn dummy_comparison_file() -> FileUploadChunk {
    FileUploadChunk {
        id: String::from("cmp-file-1234"),
        upload_request_id: String::from("file-1234"),
        chunk_sequence_number: 1,
        chunk_source: FileUploadChunkSource::ComparisonFileChunk,
        chunk_rows: vec![],
        date_created: chrono::Utc::now().timestamp(),
        date_modified: chrono::Utc::now().timestamp(),
        comparison_pairs: vec![new_same_column_index_comparison_pair(0)],
        column_headers: vec![],
        recon_config: default_recon_configs(),
        primary_file_chunks_queue: FileChunkQueue {
            topic_id: String::from("src-file-chunks-queue-1"),
            last_acknowledged_id: Option::None,
        },
        comparison_file_chunks_queue: FileChunkQueue {
            topic_id: String::from("cmp-file-chunks-queue-1"),
            last_acknowledged_id: Option::None,
        },
        result_chunks_queue: FileChunkQueue {
            topic_id: String::from("results-file-chunks-queue-1"),
            last_acknowledged_id: Option::None,
        },
        is_last_chunk: false,
    }
}
