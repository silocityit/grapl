#![allow(unused_must_use)]

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use failure::{bail, Error};
use log::{debug, error, info, warn};
use prost::Message;
use rusoto_s3::{ListObjectsRequest, S3Client, S3};
use rusoto_sqs::SqsClient;

use grapl_config::env_helpers::{s3_event_emitters_from_env, FromEnv};
use grapl_graph_descriptions::graph_description::*;
use grapl_observe::metric_reporter::MetricReporter;
use grapl_service::decoder::ZstdProtoDecoder;
use sqs_executor::cache::NopCache;
use sqs_executor::errors::{CheckedError, Recoverable};
use sqs_executor::event_handler::{CompletedEvents, EventHandler};
use sqs_executor::event_retriever::S3PayloadRetriever;
use sqs_executor::s3_event_emitter::S3ToSqsEventNotifier;
use sqs_executor::{make_ten, time_based_key_fn};

use crate::dispatch_event::{AnalyzerDispatchEvent, AnalyzerDispatchSerializer};

pub mod dispatch_event;

#[derive(Debug)]
pub struct AnalyzerDispatcher<S>
where
    S: S3 + Send + Sync + 'static,
{
    s3_client: Arc<S>,
}

impl<S> Clone for AnalyzerDispatcher<S>
where
    S: S3 + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            s3_client: self.s3_client.clone(),
        }
    }
}

async fn get_s3_keys(
    s3_client: &impl S3,
    bucket: impl Into<String>,
) -> Result<impl IntoIterator<Item = Result<String, Error>>, Error> {
    let bucket = bucket.into();

    let list_res = tokio::time::timeout(
        Duration::from_secs(2),
        s3_client.list_objects(ListObjectsRequest {
            bucket,
            ..Default::default()
        }),
    )
    .await??;

    let contents = match list_res.contents {
        Some(contents) => contents,
        None => {
            warn!("List response returned nothing");
            Vec::new()
        }
    };

    Ok(contents.into_iter().map(|object| match object.key {
        Some(key) => Ok(key),
        None => bail!("S3Object is missing key"),
    }))
}

#[derive(thiserror::Error, Debug)]
pub enum AnalyzerDispatcherError {
    #[error("Unexpected")]
    Unexpected(String),
}

impl CheckedError for AnalyzerDispatcherError {
    fn error_type(&self) -> Recoverable {
        Recoverable::Transient
    }
}

#[async_trait]
impl<S> EventHandler for AnalyzerDispatcher<S>
where
    S: S3 + Send + Sync + 'static,
{
    type InputEvent = GeneratedSubgraphs;
    type OutputEvent = Vec<AnalyzerDispatchEvent>;
    type Error = AnalyzerDispatcherError;

    async fn handle_event(
        &mut self,
        subgraphs: Self::InputEvent,
        completed: &mut CompletedEvents,
    ) -> Result<Self::OutputEvent, Result<(Self::OutputEvent, Self::Error), Self::Error>> {
        let bucket = std::env::var("ANALYZER_BUCKET").expect("ANALYZER_BUCKET");

        let mut subgraph = Graph::new(0);

        subgraphs
            .subgraphs
            .iter()
            .for_each(|generated_subgraph| subgraph.merge(generated_subgraph));

        if subgraph.is_empty() {
            warn!("Attempted to handle empty subgraph");
            return Ok(vec![]);
        }

        info!("Retrieving S3 keys");
        let keys = match get_s3_keys(self.s3_client.as_ref(), &bucket).await {
            Ok(keys) => keys,
            Err(e) => {
                return Err(Err(AnalyzerDispatcherError::Unexpected(format!(
                    "Failed to list bucket: {} with {:?}",
                    bucket, e
                ))));
            }
        };

        let mut dispatch_events = Vec::new();

        let mut failed = None;
        for key in keys {
            let key = match key {
                Ok(key) => key,
                Err(e) => {
                    warn!("Failed to retrieve key with {:?}", e);
                    failed = Some(e);
                    continue;
                }
            };

            dispatch_events.push(AnalyzerDispatchEvent::new(key, subgraph.clone()));
        }

        if let Some(e) = failed {
            Err(Ok((
                dispatch_events,
                AnalyzerDispatcherError::Unexpected(e.to_string()),
            )))
        } else {
            Ok(dispatch_events)
        }

        // identities.into_iter().for_each(|identity| completed.add_identity(identity));
    }
}

async fn handler() -> Result<(), Box<dyn std::error::Error>> {
    let (env, _guard) = grapl_config::init_grapl_env!();

    info!("Handling event");

    let sqs_client = SqsClient::from_env();
    let s3_client = S3Client::from_env();
    let source_queue_url = grapl_config::source_queue_url();
    debug!("Queue Url: {}", source_queue_url);
    let destination_bucket = grapl_config::dest_bucket();
    info!("Output events to: {}", destination_bucket);

    let cache = &mut make_ten(async {
        NopCache {}
        // RedisCache::new(cache_address.to_owned(), MetricReporter::<Stdout>::new(&env.service_name)).await
        //     .expect("Could not create redis client")
    })
    .await;

    let serializer = &mut make_ten(async { AnalyzerDispatchSerializer::default() }).await;

    let s3_emitter =
        &mut s3_event_emitters_from_env(&env, time_based_key_fn, S3ToSqsEventNotifier::from(&env))
            .await;

    let s3_payload_retriever = &mut make_ten(async {
        S3PayloadRetriever::new(
            |region_str| grapl_config::env_helpers::init_s3_client(&region_str),
            ZstdProtoDecoder::default(),
            MetricReporter::new(&env.service_name),
        )
    })
    .await;
    let analyzer_dispatcher = &mut make_ten(async {
        AnalyzerDispatcher {
            s3_client: Arc::new(S3Client::from_env()),
        }
    })
    .await;

    info!("Starting process_loop");
    sqs_executor::process_loop(
        source_queue_url,
        std::env::var("DEAD_LETTER_QUEUE_URL").expect("DEAD_LETTER_QUEUE_URL"),
        cache,
        sqs_client.clone(),
        analyzer_dispatcher,
        s3_payload_retriever,
        s3_emitter,
        serializer,
        MetricReporter::new(&env.service_name),
    )
    .await;

    info!("Exiting");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    handler().await?;
    Ok(())
}
