use std::net::SocketAddr;

use tonic::{
    transport::Server,
    Code,
    Request,
    Response,
    Status,
};

pub use crate::model_plugin_deployer::{
    model_plugin_deployer_rpc_service_server::{
        ModelPluginDeployerRpcService,
        ModelPluginDeployerRpcServiceServer,
    },
    DeployModelResponse,
};
use crate::model_plugin_deployer::{
    DeployModelRequest,
    SchemaType,
};

pub struct ModelPluginDeployer {}

impl ModelPluginDeployer {
    fn new() -> ModelPluginDeployer {
        ModelPluginDeployer {}
    }

    /// The actual business logic for `deploy_model`
    fn handle_deploy_model(
        &self,
        inner_request: DeployModelRequest,
    ) -> Result<DeployModelResponse, Status> {
        match SchemaType::from_i32(inner_request.schema_type) {
            Some(SchemaType::Graphql) => {
                // TODO: Read the schema as graphql
                Ok(DeployModelResponse {})
            }
            _ => Err(Status::new(Code::InvalidArgument, "Unhandled schema type")),
        }
    }
}

#[tonic::async_trait]
impl ModelPluginDeployerRpcService for ModelPluginDeployer {
    /// Bind `handle_deploy_model` to the grpc service
    #[tracing::instrument(
        source_addr = request.remote_addr(),
        client_id = request.get_ref().request_meta.client_id,
        skip(self, request),
    )]
    async fn deploy_model(
        &self,
        request: Request<DeployModelRequest>,
    ) -> Result<Response<DeployModelResponse>, Status> {
        let start = quanta::Instant::now();
        let inner_request = request.into_inner();
        let reply = self.handle_deploy_model(inner_request)?;

        let delta = quanta::Instant::now().duration_since(start);
        metrics::histogram!("request_ns", delta);

        Ok(Response::new(reply))
    }
}

pub async fn exec_service(socket_addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<ModelPluginDeployerRpcServiceServer<ModelPluginDeployer>>()
        .await;

    let model_plugin_deployer_instance = ModelPluginDeployer::new();

    metrics::register_counter!("request_count", "count of requests made to endpoint");
    metrics::register_histogram!("request_ns", "nanoseconds for request execution");

    tracing::info!(
        message="About to start ModelPluginDeployer + HealthServer",
        addr=?socket_addr,
    );

    Server::builder()
        .add_service(health_service)
        .add_service(ModelPluginDeployerRpcServiceServer::new(
            model_plugin_deployer_instance,
        ))
        .serve(socket_addr)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_error_if_schema_type_not_defined() -> Result<(), String> {
        let service_instance = ModelPluginDeployer::new();
        let inner_request = DeployModelRequest::default();
        let response = service_instance.handle_deploy_model(inner_request);
        match response {
            Ok(_) => Err("Unexpected OK".into()),
            Err(status) => {
                assert_eq!(status.code(), Code::InvalidArgument);
                assert_eq!(status.message(), "Unhandled schema type");
                Ok(())
            }
        }
    }
}
