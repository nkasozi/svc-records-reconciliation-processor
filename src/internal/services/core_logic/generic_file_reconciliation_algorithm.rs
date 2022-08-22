use async_trait::async_trait;

use crate::internal::{
    interfaces::file_reconciliation_algorithm::FileReconciliationAlgorithmInterface,
    shared_reconciler_rust_libraries::models::entities::{
        app_errors::AppError,
        file_upload_chunk::{FileUploadChunk, ReconStatus},
        recon_tasks_models::{ComparisonPair, ReconciliationConfigs},
    },
};

pub struct GenericFileReconciliationAlgorithm {}

#[async_trait]
impl FileReconciliationAlgorithmInterface for GenericFileReconciliationAlgorithm {
    async fn reconcile_primary_file_chunk(
        &self,
        primary_file_chunk: &mut FileUploadChunk,
        comparison_file_chunk: &FileUploadChunk,
    ) -> Result<FileUploadChunk, AppError> {
        //for each row in the primary file chunk
        for (index, primary_chunk_row) in primary_file_chunk
            .clone()
            .chunk_rows
            .into_iter()
            .enumerate()
        {
            //if the row has already failed reconciliation, we can skip it
            if primary_chunk_row.recon_result == ReconStatus::Failed {
                continue;
            }

            //we get all the columns in the primary chunk row
            let primary_file_row_parts = primary_chunk_row.parsed_columns_from_row.clone();

            //for each row in the comparison file chnk
            for comparison_chunk_row in comparison_file_chunk.chunk_rows.clone() {
                //we get all the columns in the comparison chunk row
                let comparison_file_row_parts =
                    comparison_chunk_row.parsed_columns_from_row.clone();

                //get the comparison pairs that are used to uniquely identify the same row in both files
                let row_id_comparison_pairs = primary_file_chunk
                    .clone()
                    .get_row_identifier_comparison_pairs();

                //we then check if this is supposed to be the same row in both files
                //by checking the identity columns in the comparison comparison_pairs
                if !self.are_same_row_identifiers(
                    &primary_file_row_parts,
                    &comparison_file_row_parts,
                    &row_id_comparison_pairs,
                    &primary_file_chunk.recon_config,
                ) {
                    continue;
                }

                //if the columns to compare doesnt even match
                //we shouldnt even try reconciling the rows
                if primary_file_row_parts.len() != comparison_file_row_parts.len() {
                    //we mark the row as failed reconciliation
                    let reason = format!(
                        "Count of columns to compare does not match. RowNumber: [{}], PrimaryFile Has [{}] columns to compare, ComparisonFile has [{}] columns to compare",
                        primary_chunk_row.row_number,
                        primary_file_row_parts.len(),
                        comparison_file_row_parts.len()
                    );
                    primary_file_chunk.chunk_rows[index].recon_result = ReconStatus::Failed;
                    primary_file_chunk.chunk_rows[index]
                        .recon_result_reasons
                        .push(reason);
                    break;
                }

                //if its supposed to be the same row, we can begin checking each column in the primary file row
                //vs each column in the comparison file row using the comparison pairs
                let comparison_pairs =
                    primary_file_chunk.get_comparison_pairs_that_are_not_row_identifiers();

                for pair in comparison_pairs {
                    //ok its time to compare actual values in the row
                    //so we read the column value from the primary file
                    let primary_file_row_column_value = primary_file_row_parts
                        .get(pair.primary_file_column_index)
                        .map(|s| s.to_owned())
                        .unwrap_or(String::from(""));

                    //and we also read the column value from the comparison file
                    let comparison_file_row_column_value = comparison_file_row_parts
                        .get(pair.comparison_file_column_index)
                        .map(|s| s.to_owned())
                        .unwrap_or(String::from(""));

                    //we check if the values match
                    if !self.are_column_values_the_same(
                        &primary_file_row_column_value,
                        &comparison_file_row_column_value,
                        &primary_file_chunk.recon_config,
                    ) {
                        //if they dont match, then we
                        // mark the row as failed reconciliation

                        let primary_file_column_header = primary_file_chunk
                            .column_headers
                            .get(pair.primary_file_column_index)
                            .map(|s| s.to_owned())
                            .unwrap_or(String::from(""));

                        let comparison_file_column_header = comparison_file_chunk
                            .column_headers
                            .get(pair.comparison_file_column_index)
                            .map(|s| s.to_owned())
                            .unwrap_or(String::from(""));

                        let reason = format!(
                            "RowNumber: [{}], Column [{}]: PrimaryFile has value [{}] while ComparisonFile has value [{}] in Column [{}]",
                            primary_chunk_row.row_number,
                            primary_file_column_header,
                            primary_file_row_column_value,
                            comparison_file_row_column_value,
                            comparison_file_column_header,
                        );

                        //a record can fail reconciliation for many reasons
                        //which is why we just append
                        primary_file_chunk.chunk_rows[index].recon_result = ReconStatus::Failed;
                        primary_file_chunk.chunk_rows[index]
                            .recon_result_reasons
                            .push(reason);
                        break;
                    }
                }

                //record is still pending reconciliation but all the column values in the row are a perfect match
                if primary_file_chunk.chunk_rows[index].recon_result == ReconStatus::Pending {
                    primary_file_chunk.chunk_rows[index].recon_result = ReconStatus::Successful;
                    continue;
                }
            }
        }

        return Ok(primary_file_chunk.clone());
    }
}

impl GenericFileReconciliationAlgorithm {
    //checks to see if 2 string column values from a row in 2 different files are the same
    pub fn are_same_row_identifiers(
        &self,
        primary_file_row_parts: &Vec<String>,
        comparison_file_row_parts: &Vec<String>,
        row_id_comparison_pairs: &Vec<ComparisonPair>,
        recon_configs: &ReconciliationConfigs,
    ) -> bool {
        for pair in row_id_comparison_pairs {
            //so we read the column value from the primary file
            let primary_file_row_column_value = primary_file_row_parts
                .get(pair.primary_file_column_index)
                .map(|s| s.to_owned())
                .unwrap_or(String::from(""));

            //and we also read the column value from the comparison file
            let comparison_file_row_column_value = comparison_file_row_parts
                .get(pair.comparison_file_column_index)
                .map(|s| s.to_owned())
                .unwrap_or(String::from(""));

            //we check if the values match
            if !self.are_column_values_the_same(
                &primary_file_row_column_value,
                &comparison_file_row_column_value,
                &recon_configs,
            ) {
                return false;
            }
        }
        return true;
    }

    //checks to see if 2 string column values from a row in 2 different files are the same
    pub fn are_column_values_the_same(
        &self,
        primary_file_row_column_value: &String,
        comparison_file_row_column_value: &String,
        recon_configs: &ReconciliationConfigs,
    ) -> bool {
        let mut sanitized_primary_file_row_column_value = primary_file_row_column_value.clone();
        let mut sanitized_comparison_file_row_column_value =
            comparison_file_row_column_value.clone();

        if recon_configs.should_ignore_white_space {
            sanitized_comparison_file_row_column_value = sanitized_comparison_file_row_column_value
                .trim()
                .to_string();
            sanitized_primary_file_row_column_value =
                sanitized_primary_file_row_column_value.trim().to_string()
        }

        if recon_configs.should_reconciliation_be_case_sensitive {
            return sanitized_primary_file_row_column_value
                .eq(&sanitized_comparison_file_row_column_value);
        }

        return sanitized_primary_file_row_column_value
            .eq_ignore_ascii_case(&sanitized_comparison_file_row_column_value);
    }
}
