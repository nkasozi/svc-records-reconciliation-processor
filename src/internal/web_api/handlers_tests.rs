use actix_web::{
    test::{self, TestRequest},
    web::Data,
    App,
};

use crate::internal::{
    interfaces::file_chunk_reconciliation_service::{
        FileChunkReconciliationServiceInterface, MockFileChunkReconciliationServiceInterface,
    },
    models::view_models::{
        requests::reconcile_file_chunk_request::ReconcileFileChunkRequest,
        responses::reconcile_file_chunk_response::ReconcileFileChunkResponse,
    },
    shared_reconciler_rust_libraries::models::entities::{
        app_errors::{AppError, AppErrorKind},
        file_chunk_queue::FileChunkQueue,
        file_upload_chunk::{FileUploadChunk, FileUploadChunkSource},
        recon_tasks_models::ReconciliationConfigs,
    },
    web_api::handlers::reconcile_file_chunk,
};

#[actix_web::test]
async fn test_reconcile_file_chunk_calls_correct_dependecies_and_returns_success() {
    let mut app = test::init_service((move || {
        // Create some global state prior to running the handler thread
        let mut mock_service = Box::new(MockFileChunkReconciliationServiceInterface::new());

        mock_service.expect_reconcile_file_chunk().returning(|_y| {
            Ok(ReconcileFileChunkResponse {
                file_chunk_id: String::from("FILE-CHUNK-1"),
            })
        });

        let service: Box<dyn FileChunkReconciliationServiceInterface> = mock_service;

        App::new()
            .app_data(Data::new(service)) // add shared state
            .service(reconcile_file_chunk)
    })())
    .await;

    let request = get_dummy_request();

    let resp = TestRequest::post()
        .uri(&format!("/reconcile-file-chunk"))
        .set_json(request)
        .send_request(&mut app)
        .await;

    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_upload_file_chunk_when_invalid_request_returns_bad_request() {
    let mut app = test::init_service((move || {
        // Create some global state prior to running the handler thread
        let mut mock_service = Box::new(MockFileChunkReconciliationServiceInterface::new());

        mock_service.expect_reconcile_file_chunk().returning(|_y| {
            Err(AppError::new(
                AppErrorKind::BadClientRequest,
                "invalid request".to_string(),
            ))
        });

        let service: Box<dyn FileChunkReconciliationServiceInterface> = mock_service;

        App::new()
            .app_data(Data::new(service)) // add shared state
            .service(reconcile_file_chunk)
    })())
    .await;

    let request = get_dummy_request();

    let resp = TestRequest::post()
        .uri(&format!("/reconcile-file-chunk"))
        .set_json(request)
        .send_request(&mut app)
        .await;

    assert!(resp.status().is_client_error());
}

#[actix_web::test]
async fn test_upload_file_chunk_when_service_returns_error_returns_internal_error() {
    let mut app = test::init_service((move || {
        // Create some global state prior to running the handler thread
        let mut mock_service = Box::new(MockFileChunkReconciliationServiceInterface::new());

        mock_service.expect_reconcile_file_chunk().returning(|_y| {
            Err(AppError::new(
                AppErrorKind::InternalError,
                "Internal server error".to_string(),
            ))
        });

        let service: Box<dyn FileChunkReconciliationServiceInterface> = mock_service;

        App::new()
            .app_data(Data::new(service)) // add shared state
            .service(reconcile_file_chunk)
    })())
    .await;

    let request = get_dummy_request();

    let resp = TestRequest::post()
        .uri(&format!("/reconcile-file-chunk"))
        .set_json(request)
        .send_request(&mut app)
        .await;

    assert!(resp.status().is_server_error());
}

fn get_dummy_request() -> ReconcileFileChunkRequest {
    ReconcileFileChunkRequest {
        primary_file_chunk: FileUploadChunk {
            id: String::from("TEST-UPLOAD-1"),
            upload_request_id: String::from("TEST-UPLOAD-1"),
            chunk_sequence_number: 1,
            chunk_source: FileUploadChunkSource::ComparisonFileChunk,
            chunk_rows: vec![],
            date_created: chrono::Utc::now().timestamp(),
            date_modified: chrono::Utc::now().timestamp(),
            comparison_pairs: vec![],
            column_headers: vec![],
            recon_config: ReconciliationConfigs {
                should_check_for_duplicate_records_in_comparison_file: true,
                should_reconciliation_be_case_sensitive: true,
                should_ignore_white_space: true,
                should_do_reverse_reconciliation: true,
            },
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
        },
    }
}
