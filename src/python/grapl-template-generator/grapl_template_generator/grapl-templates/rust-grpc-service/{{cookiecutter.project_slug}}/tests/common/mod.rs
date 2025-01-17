use test_context::AsyncTestContext;
use {{cookiecutter.snake_project_name}}::client::{Channel, Timeout};
use tonic::transport::NamedService;
use {{cookiecutter.snake_project_name}}::server::{{cookiecutter.crate_name}}RpcServer;
use {{cookiecutter.snake_project_name}}::server::{{cookiecutter.crate_name}};
use {{cookiecutter.snake_project_name}}::{{cookiecutter.snake_project_name}}::get_socket_addr;
use std::time::Duration;
use tonic_health::proto::health_client::HealthClient;
use tonic_health::proto::HealthCheckRequest;
use tonic_health::proto::health_check_response::ServingStatus;

pub struct ServiceContext {}

#[async_trait::async_trait]
impl AsyncTestContext for ServiceContext {
    async fn setup() -> Self {
        let _subscriber = ::tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(::tracing_subscriber::EnvFilter::from_default_env())
            .init();

        tokio::task::spawn(async move {
            {{cookiecutter.snake_project_name}}::server::exec_service().await
                .expect("Failed to execute service");
        });
        wait_for_healthcheck().await
            .expect("Service was never healthy");
        Self {}
    }

    async fn teardown(self) {}
}


async fn wait_for_healthcheck() -> Result<(), Box<dyn std::error::Error>> {
    // Wait until the health check succeeds. Try up to 5 times with
    // increasing backoff.

    for i in 0.. {
        match _healthcheck().await {
            Ok(()) => return Ok(()),
            Err(e) => {
                if i == 5 {
                    tracing::error!(error=?e, times=i, message="Health Check failed");
                    return Err(e);
                }
                tracing::debug!(error=?e, times=i, message="Health Check failed");
            }
        }
        tokio::time::sleep(Duration::from_millis(i * 10)).await;
    }
    unreachable!()
}

async fn _healthcheck() -> Result<(), Box<dyn std::error::Error>> {
    // Check if the healthcheck request succeeds.

    let endpoint = format!("http://{}", get_socket_addr());
    let channel = Channel::from_shared(endpoint)?.connect().await?;

    let timeout_channel = Timeout::new(channel, Duration::from_millis(1000));

    let mut client = HealthClient::new(timeout_channel);

    let request = HealthCheckRequest {
        service: {{cookiecutter.crate_name}}RpcServer::<{{cookiecutter.crate_name}}>::NAME.to_string(),
    };
    let response = client.check(request).await?;
    let response = response.into_inner();
    match response.status() {
        ServingStatus::Serving => {
            Ok(())
        }
        other => {
            Err(format!("Not serving: {:?}", other).into())
        }
    }
}
