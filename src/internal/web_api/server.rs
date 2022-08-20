use crate::external::pubsub::dapr_pubsub::DaprPubSubRepositoryManager;
use crate::internal::services::generic_file_reconciliation_algorithm::GenericFileReconciliationAlgorithm;

use crate::internal::web_api::handlers;
use crate::internal::{
    interfaces::file_chunk_reconciliation_service::FileChunkReconciliationServiceInterface,
    services::file_chunk_reconciliation_service::FileChunkReconciliationService,
};
use actix_web::{web::Data, App, HttpServer};

// constants
const DEFAULT_DAPR_CONNECTION_URL: &'static str = "http://localhost:5005";
const DEFAULT_REDIS_CONNECTION_URL: &'static str = "http://localhost:5005";
const DEFAULT_DAPR_PUBSUB_NAME: &'static str = "FileChunksQueue";
const DEFAULT_DAPR_PUBSUB_TOPIC: &'static str = "FileChunks";
const DEFAULT_APP_LISTEN_IP: &'static str = "0.0.0.0";
const DEFAULT_APP_LISTEN_PORT: u16 = 8080;

#[derive(Clone, Debug)]
struct AppSettings {
    pub app_port: String,

    pub app_ip: String,

    pub dapr_pubsub_name: String,

    pub dapr_pubsub_topic: String,

    pub dapr_grpc_server_ip_address: String,

    pub redis_url: String,
}

pub async fn run_async() -> Result<(), std::io::Error> {
    //retrieve app settings from the env variables
    let app_settings = read_app_settings();

    let app_listen_url = format!("{}:{}", app_settings.app_ip, app_settings.app_port);

    //just for logging purposes
    println!("App is listening on: {:?}", app_listen_url);

    HttpServer::new(move || {
        // Create some global state prior to running the handler threads
        let service = setup_service(app_settings.clone());

        // add shared state and routing
        App::new()
            .app_data(Data::new(service))
            .service(handlers::reconcile_file_chunk)
    })
    .bind(app_listen_url)?
    .run()
    .await
}

fn read_app_settings() -> AppSettings {
    AppSettings {
        app_port: std::env::var("APP_PORT").unwrap_or(DEFAULT_APP_LISTEN_PORT.to_string()),

        app_ip: std::env::var("APP_IP").unwrap_or(DEFAULT_APP_LISTEN_IP.to_string()),

        dapr_pubsub_name: std::env::var("DAPR_PUBSUB_NAME")
            .unwrap_or(DEFAULT_DAPR_PUBSUB_NAME.to_string()),

        dapr_pubsub_topic: std::env::var("DAPR_PUBSUB_TOPIC")
            .unwrap_or(DEFAULT_DAPR_PUBSUB_TOPIC.to_string()),

        dapr_grpc_server_ip_address: std::env::var("DAPR_IP")
            .unwrap_or(DEFAULT_DAPR_CONNECTION_URL.to_string()),

        redis_url: std::env::var("REDIS_URL").unwrap_or(DEFAULT_REDIS_CONNECTION_URL.to_string()),
    }
}

fn setup_service(app_settings: AppSettings) -> Box<dyn FileChunkReconciliationServiceInterface> {
    let service: Box<dyn FileChunkReconciliationServiceInterface> =
        Box::new(FileChunkReconciliationService {
            pubsub_repo: Box::new(DaprPubSubRepositoryManager {
                dapr_grpc_server_address: app_settings.dapr_grpc_server_ip_address.clone(),
                dapr_pubsub_name: app_settings.dapr_pubsub_name.clone(),
                dapr_pubsub_topic: app_settings.dapr_pubsub_topic.clone(),
                redis_url: app_settings.redis_url.clone(),
            }),
            file_reconciliation_algorithm: Box::new(GenericFileReconciliationAlgorithm {}),
        });
    service
}
