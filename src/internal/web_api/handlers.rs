use crate::internal::interfaces::file_chunk_reconciliation_service::FileChunkReconciliationServiceInterface;
use crate::internal::models::view_models::requests::reconcile_file_chunk_request::ReconcileFileChunkRequest;
use crate::internal::shared_reconciler_rust_libraries::models::entities::app_errors::AppErrorKind;
use actix_web::{
    post,
    web::{self, Data},
    HttpResponse,
};

#[post("/reconcile-file-chunk")]
async fn reconcile_file_chunk(
    task_details: web::Json<ReconcileFileChunkRequest>,
    service: Data<Box<dyn FileChunkReconciliationServiceInterface>>,
) -> HttpResponse {
    let recon_task_details = service.reconcile_file_chunk(&task_details.0).await;

    return match recon_task_details {
        Ok(details) => HttpResponse::Ok().json(details),

        Err(err) => match err.kind {
            AppErrorKind::BadClientRequest => HttpResponse::BadRequest().json(format!("{}", err)),
            _ => HttpResponse::InternalServerError().json(format!("{}", err)),
        },
    };
}
