use std::collections::HashMap;

use crate::internal::{
    interfaces::pubsub_repo::PubSubRepositoryInterface,
    models::view_models::responses::redis_stream_message::RedisStreamMessage,
    shared_reconciler_rust_libraries::models::entities::{
        app_errors::{AppError, AppErrorKind},
        file_chunk_queue::FileChunkQueue,
        file_upload_chunk::FileUploadChunk,
    },
};
use async_trait::async_trait;
use dapr::{dapr::dapr::proto::runtime::v1::dapr_client::DaprClient, Client};
use redis::{
    streams::{StreamId, StreamKey, StreamReadReply},
    Commands, Value,
};
//use std::collections::HashMap;
use tonic::transport::Channel as TonicChannel;

pub struct DaprPubSubRepositoryManager {
    //the dapr server ip
    pub dapr_grpc_server_address: String,

    //the dapr pub sub component name
    pub dapr_pubsub_name: String,

    //the dapr pub sub topic
    pub dapr_pubsub_topic: String,

    //the redis url
    pub redis_url: String,
}

#[async_trait]
impl PubSubRepositoryInterface for DaprPubSubRepositoryManager {
    async fn get_next_comparison_file_upload_chunk(
        &self,
        queue: &FileChunkQueue,
    ) -> Result<FileUploadChunk, AppError> {
        let mut redis_connection = self.get_redis_connection().await?;

        let read_reply: StreamReadReply =
            self.read_from_stream(&mut redis_connection, &queue).await?;

        return self.deserialize_stream_reply(&read_reply).await;
    }

    async fn mark_comparison_file_chunk_as_processed(
        &self,
        _file_chunk: &FileUploadChunk,
    ) -> Result<bool, AppError> {
        //since we are using redis streams without consumer groups
        //we do not need to acknowledge messages
        //what happens is we keep track of where we ended reading in the streams
        //by updating the primary_file_chunk.comparison_file_queue.last_acknowledged id
        //that means that this call is not necessary
        return Ok(true);
    }

    async fn insert_file_chunk_in_primary_file_queue(
        &self,
        file_chunk: &FileUploadChunk,
    ) -> Result<bool, AppError> {
        //create a dapr client
        let mut client = self.get_dapr_connection().await?;

        //call the binding
        let pubsub_name = self.dapr_pubsub_name.clone();
        let pubsub_topic = file_chunk.primary_file_chunks_queue.topic_id.clone();
        let data_content_type = "json".to_string();
        let data = serde_json::to_vec(&file_chunk).unwrap();
        let metadata = None::<HashMap<String, String>>;
        let binding_response = client
            .publish_event(pubsub_name, pubsub_topic, data_content_type, data, metadata)
            .await;

        //handle the bindings response
        match binding_response {
            //success
            Ok(_) => return Ok(true),
            //failure
            Err(e) => return Err(AppError::new(AppErrorKind::NotFound, e.to_string())),
        }
    }
}

impl DaprPubSubRepositoryManager {
    async fn get_dapr_connection(&self) -> Result<Client<DaprClient<TonicChannel>>, AppError> {
        // Create the client
        let dapr_grpc_server_address = self.dapr_grpc_server_address.clone();

        //connect to dapr
        let client_connect_result =
            dapr::Client::<dapr::client::TonicClient>::connect(dapr_grpc_server_address).await;

        //handle the connection result
        match client_connect_result {
            //connection succeeded
            Ok(s) => return Ok(s),
            //connection failed
            Err(e) => return Err(AppError::new(AppErrorKind::ConnectionError, e.to_string())),
        }
    }

    async fn get_redis_connection(&self) -> Result<redis::Connection, AppError> {
        // Create the client
        let client = self.open_redis_connection().await?;

        let client_connect_result = client.get_connection();

        //handle the connection result
        match client_connect_result {
            //connection succeeded
            Ok(s) => return Ok(s),
            //connection failed
            Err(e) => return Err(AppError::new(AppErrorKind::ConnectionError, e.to_string())),
        }
    }

    async fn open_redis_connection(&self) -> Result<redis::Client, AppError> {
        // Create the client
        let open_result = redis::Client::open("redis://127.0.0.1:6379/");

        //handle the connection result
        match open_result {
            //connection succeeded
            Ok(s) => return Ok(s),
            //connection failed
            Err(e) => return Err(AppError::new(AppErrorKind::ConnectionError, e.to_string())),
        }
    }

    async fn read_from_stream(
        &self,
        redis_connection: &mut redis::Connection,
        queue: &FileChunkQueue,
    ) -> Result<StreamReadReply, AppError> {
        //read all the messages from the stream after the last_acknowledged_id
        let read_result = redis_connection.xread(
            &[queue.topic_id.clone()],
            &[queue.last_acknowledged_id.clone()],
        );

        //handle the read_result
        match read_result {
            Ok(read_reply) => return Ok(read_reply),
            Err(e) => {
                return Err(AppError::new(
                    AppErrorKind::ResponseUnmarshalError,
                    e.to_string(),
                ))
            }
        }
    }

    async fn deserialize_stream_reply(
        &self,
        read_reply: &StreamReadReply,
    ) -> Result<FileUploadChunk, AppError> {
        //read the first message from the stream
        for StreamKey { key: _, ids } in read_reply.keys.clone() {
            for StreamId { id, map } in &ids {
                for (_, s) in map {
                    if let Value::Data(bytes) = s {
                        //first get the json string from the bytes returned
                        let data_string = String::from_utf8(bytes.to_vec()).expect("utf8");

                        //try to deserialie the json string into a struct
                        let deserialize_result: Result<RedisStreamMessage, AppError> =
                            serde_json::from_str(&data_string).unwrap_or_else(|error| {
                                return Err(AppError::new(
                                    AppErrorKind::ResponseUnmarshalError,
                                    error.to_string(),
                                ));
                            });

                        //handle deserialization results
                        match deserialize_result {
                            Ok(mut deserialized_message) => {
                                //if successfull, attach the id of the deserialized message to the file_chunk
                                deserialized_message.data.id = id.clone();
                                return Ok(deserialized_message.data);
                            }
                            Err(e) => {
                                //transform error to app error
                                return Err(AppError::new(
                                    AppErrorKind::ResponseUnmarshalError,
                                    e.to_string(),
                                ));
                            }
                        }
                    } else {
                        //by the time we are here, it means no data has been returned
                        return Err(AppError::new(
                            AppErrorKind::ResponseUnmarshalError,
                            String::from("No Data returned"),
                        ));
                    }
                }
            }
        }

        return Err(AppError::new(
            AppErrorKind::ResponseUnmarshalError,
            String::from("No Data returned"),
        ));
    }
}
