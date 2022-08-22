use crate::internal::{
    interfaces::file_reconciliation_algorithm::FileReconciliationAlgorithmInterface,
    shared_reconciler_rust_libraries::models::entities::{
        file_chunk_queue::FileChunkQueue,
        file_upload_chunk::{
            FileUploadChunk, FileUploadChunkRow, FileUploadChunkSource, ReconStatus,
        },
        recon_tasks_models::{ComparisonPair, ReconciliationConfigs},
    },
};

use super::generic_file_reconciliation_algorithm::GenericFileReconciliationAlgorithm;

#[actix_web::test]
async fn test_is_same_column_values_given_same_exact_column_values_returns_true() {
    //setup
    let primary_file_colum_value = String::from("test");
    let comparison_file_colum_value = String::from("test");
    let recon_configs = ReconciliationConfigs {
        should_check_for_duplicate_records_in_comparison_file: false,
        should_reconciliation_be_case_sensitive: false,
        should_ignore_white_space: true,
        should_do_reverse_reconciliation: false,
    };
    let sut = setup();

    //act
    let actual = sut.are_column_values_the_same(
        &primary_file_colum_value,
        &comparison_file_colum_value,
        &recon_configs,
    );

    //assert
    let expected = true;
    assert_eq!(actual, expected);
}

#[actix_web::test]
async fn test_is_same_column_values_given_column_values_with_space_and_no_trim_config_returns_false(
) {
    //setup
    let primary_file_colum_value = String::from("test ");
    let comparison_file_colum_value = String::from("test");
    let recon_configs = ReconciliationConfigs {
        should_check_for_duplicate_records_in_comparison_file: false,
        should_reconciliation_be_case_sensitive: false,
        should_ignore_white_space: false,
        should_do_reverse_reconciliation: false,
    };
    let sut = setup();

    //act
    let actual = sut.are_column_values_the_same(
        &primary_file_colum_value,
        &comparison_file_colum_value,
        &recon_configs,
    );

    //assert
    let expected = false;
    assert_eq!(actual, expected);
}

#[actix_web::test]
async fn test_is_same_column_values_given_column_values_with_mixed_case_and_consider_case_config_returns_false(
) {
    //setup
    let primary_file_colum_value = String::from("testMixCase");
    let comparison_file_colum_value = String::from("testmixCase");
    let recon_configs = ReconciliationConfigs {
        should_check_for_duplicate_records_in_comparison_file: false,
        should_reconciliation_be_case_sensitive: true,
        should_ignore_white_space: true,
        should_do_reverse_reconciliation: false,
    };
    let sut = setup();

    //act
    let actual = sut.are_column_values_the_same(
        &primary_file_colum_value,
        &comparison_file_colum_value,
        &recon_configs,
    );

    //assert
    let expected = false;
    assert_eq!(actual, expected);
}

#[actix_web::test]
async fn test_is_same_column_values_given_column_values_with_mixed_case_and_ignore_case_config_returns_true(
) {
    //setup
    let primary_file_colum_value = String::from("testMixCase");
    let comparison_file_colum_value = String::from("testmixCase");
    let recon_configs = ReconciliationConfigs {
        should_check_for_duplicate_records_in_comparison_file: false,
        should_reconciliation_be_case_sensitive: false,
        should_ignore_white_space: false,
        should_do_reverse_reconciliation: false,
    };
    let sut = setup();

    //act
    let actual = sut.are_column_values_the_same(
        &primary_file_colum_value,
        &comparison_file_colum_value,
        &recon_configs,
    );

    //assert
    let expected = true;
    assert_eq!(actual, expected);
}

#[actix_web::test]
async fn test_is_same_column_values_given_different_column_values_returns_false() {
    //setup
    let primary_file_colum_value = String::from("test something else");
    let comparison_file_colum_value = String::from("test");
    let recon_configs = ReconciliationConfigs {
        should_check_for_duplicate_records_in_comparison_file: false,
        should_reconciliation_be_case_sensitive: false,
        should_ignore_white_space: true,
        should_do_reverse_reconciliation: false,
    };
    let sut = setup();

    //act
    let actual = sut.are_column_values_the_same(
        &primary_file_colum_value,
        &comparison_file_colum_value,
        &recon_configs,
    );

    //assert
    let expected = false;
    assert_eq!(actual, expected);
}

#[actix_web::test]
async fn test_are_same_row_identifiers_given_same_row_values_returns_true() {
    //setup
    let primary_file_row_values = vec![String::from("column_1"), String::from("column_2")];
    let comparison_file_row_values = vec![String::from("column_1"), String::from("column_2")];
    let comparison_pairs = vec![default_comparison_pair(0), default_comparison_pair(1)];
    let recon_configs = default_recon_configs();
    let sut = setup();

    //act
    let actual = sut.are_same_row_identifiers(
        &primary_file_row_values,
        &comparison_file_row_values,
        &comparison_pairs,
        &recon_configs,
    );

    //assert
    let expected = true;
    assert_eq!(actual, expected);
}

#[actix_web::test]
async fn test_are_same_row_identifiers_given_different_row_values_returns_false() {
    //setup
    let primary_file_row_values = vec![String::from("column 1"), String::from("column_2")];
    let comparison_file_row_values = vec![String::from("column_1"), String::from("column_2")];
    let comparison_pairs = vec![default_comparison_pair(0), default_comparison_pair(1)];
    let recon_configs = default_recon_configs();
    let sut = setup();

    //act
    let actual = sut.are_same_row_identifiers(
        &primary_file_row_values,
        &comparison_file_row_values,
        &comparison_pairs,
        &recon_configs,
    );

    //assert
    let expected = false;
    assert_eq!(actual, expected);
}

#[actix_web::test]
async fn test_reconcile_primary_file_chunk_given_same_chunks_does_correct_reconciliation() {
    //setup
    let mut primary_file_chunk = build_valid_file_chunk("primary");

    let comparison_file_chunk = build_valid_file_chunk("comparison");

    let sut = setup();

    //act
    let result = sut
        .reconcile_primary_file_chunk(&mut primary_file_chunk, &comparison_file_chunk)
        .await;

    //assert
    assert_eq!(result.is_ok(), true);

    let reconciled_chunk = result.unwrap();

    assert_eq!(
        reconciled_chunk.chunk_rows[0].recon_result,
        ReconStatus::Successful
    );
    assert_eq!(
        reconciled_chunk.chunk_rows[1].recon_result,
        ReconStatus::Successful
    );
}

#[actix_web::test]
async fn test_reconcile_primary_file_chunk_given_partially_simalar_chunks_does_correct_reconciliation(
) {
    //setup
    let mut primary_file_chunk = build_valid_file_chunk("primary");
    primary_file_chunk.chunk_rows[0].parsed_columns_from_row[1] = String::from("");

    let comparison_file_chunk = build_valid_file_chunk("comparison");

    let sut = setup();

    //act
    let result = sut
        .reconcile_primary_file_chunk(&mut primary_file_chunk, &comparison_file_chunk)
        .await;

    //assert
    assert_eq!(result.is_ok(), true);

    let reconciled_chunk = result.unwrap();

    assert_eq!(
        reconciled_chunk.chunk_rows[0].recon_result,
        ReconStatus::Failed
    );
    assert_eq!(
        reconciled_chunk.chunk_rows[1].recon_result,
        ReconStatus::Successful
    );
}

#[actix_web::test]
async fn test_reconcile_primary_file_chunk_given_different_chunks_returns_unreconciled() {
    //setup
    let mut primary_file_chunk = build_valid_file_chunk("primary");
    primary_file_chunk.chunk_rows[0].parsed_columns_from_row[1] = String::from("");
    primary_file_chunk.chunk_rows[1].parsed_columns_from_row[1] = String::from("");

    let comparison_file_chunk = build_valid_file_chunk("comparison");

    let sut = setup();

    //act
    let result = sut
        .reconcile_primary_file_chunk(&mut primary_file_chunk, &comparison_file_chunk)
        .await;

    //assert
    assert_eq!(result.is_ok(), true);

    let reconciled_chunk = result.unwrap();

    assert_eq!(
        reconciled_chunk.chunk_rows[0].recon_result,
        ReconStatus::Failed
    );
    assert_eq!(
        reconciled_chunk.chunk_rows[1].recon_result,
        ReconStatus::Failed
    );
}

fn setup() -> GenericFileReconciliationAlgorithm {
    GenericFileReconciliationAlgorithm {}
}

fn default_comparison_pair(column_index: usize) -> ComparisonPair {
    ComparisonPair {
        primary_file_column_index: column_index,
        comparison_file_column_index: column_index,
        is_row_identifier: false,
    }
}

fn build_valid_file_chunk(prefix: &str) -> FileUploadChunk {
    FileUploadChunk {
        id: format!("{}-{}", prefix, String::from("file-1234")),
        upload_request_id: format!("{}-{}", prefix, String::from("file-1234")),
        chunk_sequence_number: 1,
        chunk_source: FileUploadChunkSource::ComparisonFileChunk,
        chunk_rows: vec![
            build_chunk_row(1, "142425, test, user"),
            build_chunk_row(2, "142426, test2, user2"),
        ],
        date_created: chrono::Utc::now().timestamp(),
        date_modified: chrono::Utc::now().timestamp(),
        comparison_pairs: vec![
            build_comparison_pair(0, true),
            build_comparison_pair(1, false),
        ],
        column_headers: build_column_headers("ID, userRole, userName"),
        recon_config: default_recon_configs(),
        primary_file_chunks_queue: build_file_chunks_queue("primary"),
        comparison_file_chunks_queue: build_file_chunks_queue("comparison"),
        result_chunks_queue: build_file_chunks_queue("result"),
    }
}

fn build_file_chunks_queue(prefix: &str) -> FileChunkQueue {
    FileChunkQueue {
        topic_id: format!("{}-{}", prefix, String::from("file-chunks-queue-1")),
        last_acknowledged_id: Option::None,
    }
}

fn default_recon_configs() -> ReconciliationConfigs {
    ReconciliationConfigs {
        should_check_for_duplicate_records_in_comparison_file: false,
        should_reconciliation_be_case_sensitive: true,
        should_ignore_white_space: true,
        should_do_reverse_reconciliation: false,
    }
}

fn build_chunk_row(row_number: u64, raw_line_data: &str) -> FileUploadChunkRow {
    let parsed_columns: Vec<String> = raw_line_data.split(',').map(|s| s.to_string()).collect();
    FileUploadChunkRow {
        row_number: row_number,
        raw_data: raw_line_data.to_string(),
        parsed_columns_from_row: parsed_columns,
        recon_result: ReconStatus::Pending,
        recon_result_reasons: vec![],
    }
}

fn build_column_headers(header: &str) -> Vec<String> {
    let parsed_columns: Vec<String> = header.split(',').map(|s| s.to_string()).collect();
    return parsed_columns;
}

fn build_comparison_pair(column_index: usize, is_row_identifier: bool) -> ComparisonPair {
    ComparisonPair {
        primary_file_column_index: column_index,
        comparison_file_column_index: column_index,
        is_row_identifier: is_row_identifier,
    }
}
