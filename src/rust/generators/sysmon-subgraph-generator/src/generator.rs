use async_trait::async_trait;
use grapl_graph_descriptions::graph_description::*;
use itertools::{
    Either,
    Itertools,
};
use log::*;
use sqs_executor::{
    cache::Cache,
    errors::{
        CheckedError,
        Recoverable,
    },
    event_handler::{
        CompletedEvents,
        EventHandler,
    },
};
use sysmon::Event;

use crate::{
    metrics::SysmonSubgraphGeneratorMetrics,
    models::SysmonTryFrom,
};

#[derive(thiserror::Error, Debug)]
pub enum SysmonGeneratorError {
    #[error("NegativeEventTime")]
    NegativeEventTime(i64),
    #[error("TimeError")]
    TimeError(#[from] chrono::ParseError),
    #[error("Unsupported event type")]
    UnsupportedEventType(String),
}

impl CheckedError for SysmonGeneratorError {
    fn error_type(&self) -> Recoverable {
        match self {
            Self::NegativeEventTime(_) => Recoverable::Persistent,
            Self::TimeError(_) => Recoverable::Persistent,
            Self::UnsupportedEventType(_) => Recoverable::Persistent,
        }
    }
}

#[derive(Clone)]
pub struct SysmonSubgraphGenerator<C: Cache> {
    cache: C,
    metrics: SysmonSubgraphGeneratorMetrics,
}

impl<C: Cache> SysmonSubgraphGenerator<C> {
    pub fn new(cache: C, metrics: SysmonSubgraphGeneratorMetrics) -> Self {
        Self { cache, metrics }
    }
}

#[async_trait]
impl<C: Cache> EventHandler for SysmonSubgraphGenerator<C> {
    type InputEvent = Vec<Result<Event, crate::serialization::SysmonDecoderError>>;
    type OutputEvent = GraphDescription;
    type Error = SysmonGeneratorError;

    async fn handle_event(
        &mut self,
        events: Self::InputEvent,
        completed: &mut CompletedEvents,
    ) -> Result<Self::OutputEvent, Result<(Self::OutputEvent, Self::Error), Self::Error>> {
        info!("Processing {} incoming Sysmon events.", events.len());

        let (deserialized_events, deserialization_errors): (Vec<_>, Vec<_>) =
            events.into_iter().partition_map(|event| match event {
                Ok(value) => Either::Left(value),
                Err(error) => Either::Right(error),
            });

        for error in deserialization_errors {
            error!("Deserialization error: {}", error);
        }

        let subgraph_results: Vec<_> = deserialized_events
            .into_iter()
            .map(GraphDescription::try_from)
            .collect();

        // Report subgraph generation metrics
        for result in subgraph_results.iter() {
            self.metrics.report_subgraph_generation(result);
        }

        let (subgraphs, mut subgraph_generation_errors): (Vec<_>, Vec<_>) = subgraph_results
            .into_iter()
            .partition_map(|result| match result {
                Ok(value) => Either::Left(value),
                Err(error) => Either::Right(error),
            });

        for error in &subgraph_generation_errors {
            match error {
                SysmonGeneratorError::UnsupportedEventType(_) => continue,
                _ => error!("GraphDescription::try_from failed with: {:?}", error),
            }
        }

        let final_subgraph =
            subgraphs
                .iter()
                .fold(GraphDescription::new(), |mut current_graph, subgraph| {
                    current_graph.merge(&subgraph);
                    current_graph
                });

        info!("Completed mapping {} subgraphs", completed.len());

        let graph_generation_error = subgraph_generation_errors.pop();

        let final_result = match (graph_generation_error, subgraphs.is_empty()) {
            (None, _) => Ok(final_subgraph),
            (Some(error), false) => Err(Ok((final_subgraph, error))),
            (Some(error), true) => Err(Err(error)),
        };

        self.metrics.report_handle_event_success(&final_result);

        final_result
    }
}
