pub use crate::graplinc::grapl::api::plugin_work_queue::v1beta1::{
    plugin_work_queue_service_client,
    plugin_work_queue_service_server,
    AcknowledgeRequest as AcknowledgeRequestProto,
    AcknowledgeResponse as AcknowledgeResponseProto,
    ExecutionJob as ExecutionJobProto,
    GetExecuteAnalyzerRequest as GetExecuteAnalyzerRequestProto,
    GetExecuteAnalyzerResponse as GetExecuteAnalyzerResponseProto,
    GetExecuteGeneratorRequest as GetExecuteGeneratorRequestProto,
    GetExecuteGeneratorResponse as GetExecuteGeneratorResponseProto,
    PutExecuteAnalyzerRequest as PutExecuteAnalyzerRequestProto,
    PutExecuteAnalyzerResponse as PutExecuteAnalyzerResponseProto,
    PutExecuteGeneratorRequest as PutExecuteGeneratorRequestProto,
    PutExecuteGeneratorResponse as PutExecuteGeneratorResponseProto,
};

#[derive(Debug, thiserror::Error)]
pub enum PluginWorkQueueDeserializationError {
    #[error("Missing a required field")]
    MissingRequiredField(&'static str),
    #[error("Empty field")]
    EmptyField(&'static str),
}

pub struct ExecutionJob {
    tenant_id: uuid::Uuid,
    plugin_id: uuid::Uuid,
    data: Vec<u8>,
}

impl TryFrom<ExecutionJobProto> for ExecutionJob {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(value: ExecutionJobProto) -> Result<Self, Self::Error> {
        let tenant_id = value
            .tenant_id
            .ok_or(Self::Error::MissingRequiredField("ExecutionJob.tenant_id"))?
            .into();
        let plugin_id = value
            .plugin_id
            .ok_or(Self::Error::MissingRequiredField("ExecutionJob.plugin_id"))?
            .into();

        let data = value.data;
        if data.is_empty() {
            return Err(Self::Error::EmptyField("ExecutionJob.data"));
        }
        Ok(Self {
            tenant_id,
            plugin_id,
            data,
        })
    }
}

impl From<ExecutionJob> for ExecutionJobProto {
    fn from(value: ExecutionJob) -> Self {
        assert!(!value.data.is_empty());

        Self {
            tenant_id: Some(value.tenant_id.into()),
            plugin_id: Some(value.plugin_id.into()),
            data: value.data,
        }
    }
}

pub struct AcknowledgeRequest {
    request_id: uuid::Uuid,
}

impl TryFrom<AcknowledgeRequestProto> for AcknowledgeRequest {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(value: AcknowledgeRequestProto) -> Result<Self, Self::Error> {
        let request_id = value
            .request_id
            .ok_or(Self::Error::MissingRequiredField(
                "AcknowledgeRequest.request_id",
            ))?
            .into();
        Ok(Self { request_id })
    }
}

impl From<AcknowledgeRequest> for AcknowledgeRequestProto {
    fn from(value: AcknowledgeRequest) -> Self {
        Self {
            request_id: Some(value.request_id.into()),
        }
    }
}

pub struct AcknowledgeResponse {}

impl TryFrom<AcknowledgeResponseProto> for AcknowledgeResponse {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(_value: AcknowledgeResponseProto) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<AcknowledgeResponse> for AcknowledgeResponseProto {
    fn from(_value: AcknowledgeResponse) -> Self {
        Self {}
    }
}

pub struct GetExecuteAnalyzerRequest {}

impl TryFrom<GetExecuteAnalyzerRequestProto> for GetExecuteAnalyzerRequest {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(_value: GetExecuteAnalyzerRequestProto) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<GetExecuteAnalyzerRequest> for GetExecuteAnalyzerRequestProto {
    fn from(_value: GetExecuteAnalyzerRequest) -> Self {
        Self {}
    }
}

pub struct GetExecuteAnalyzerResponse {
    execution_job: ExecutionJob,
    request_id: uuid::Uuid,
}

impl TryFrom<GetExecuteAnalyzerResponseProto> for GetExecuteAnalyzerResponse {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(value: GetExecuteAnalyzerResponseProto) -> Result<Self, Self::Error> {
        let request_id = value
            .request_id
            .ok_or(Self::Error::MissingRequiredField(
                "GetExecuteAnalyzerResponse.request_id",
            ))?
            .into();
        let execution_job = value
            .execution_job
            .ok_or(Self::Error::MissingRequiredField(
                "GetExecuteAnalyzerResponse.execution_job",
            ))?
            .try_into()?;
        Ok(Self {
            request_id,
            execution_job,
        })
    }
}

impl From<GetExecuteAnalyzerResponse> for GetExecuteAnalyzerResponseProto {
    fn from(value: GetExecuteAnalyzerResponse) -> Self {
        Self {
            execution_job: Some(value.execution_job.into()),
            request_id: Some(value.request_id.into()),
        }
    }
}

pub struct GetExecuteGeneratorRequest {}

impl TryFrom<GetExecuteGeneratorRequestProto> for GetExecuteGeneratorRequest {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(_value: GetExecuteGeneratorRequestProto) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<GetExecuteGeneratorRequest> for GetExecuteGeneratorRequestProto {
    fn from(_value: GetExecuteGeneratorRequest) -> Self {
        Self {}
    }
}

pub struct GetExecuteGeneratorResponse {
    execution_job: ExecutionJob,
    request_id: uuid::Uuid,
}

impl TryFrom<GetExecuteGeneratorResponseProto> for GetExecuteGeneratorResponse {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(value: GetExecuteGeneratorResponseProto) -> Result<Self, Self::Error> {
        let request_id = value
            .request_id
            .ok_or(Self::Error::MissingRequiredField(
                "GetExecuteGeneratorResponse.request_id",
            ))?
            .into();
        let execution_job = value
            .execution_job
            .ok_or(Self::Error::MissingRequiredField(
                "GetExecuteGeneratorResponse.execution_job",
            ))?
            .try_into()?;
        Ok(Self {
            request_id,
            execution_job,
        })
    }
}

impl From<GetExecuteGeneratorResponse> for GetExecuteGeneratorResponseProto {
    fn from(value: GetExecuteGeneratorResponse) -> Self {
        Self {
            execution_job: Some(value.execution_job.into()),
            request_id: Some(value.request_id.into()),
        }
    }
}

pub struct PutExecuteAnalyzerRequest {
    execution_job: ExecutionJob,
}

impl TryFrom<PutExecuteAnalyzerRequestProto> for PutExecuteAnalyzerRequest {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(value: PutExecuteAnalyzerRequestProto) -> Result<Self, Self::Error> {
        let execution_job = value
            .execution_job
            .ok_or(Self::Error::MissingRequiredField(
                "PutExecuteAnalyzerRequest.execution_job",
            ))?
            .try_into()?;
        Ok(Self { execution_job })
    }
}

impl From<PutExecuteAnalyzerRequest> for PutExecuteAnalyzerRequestProto {
    fn from(value: PutExecuteAnalyzerRequest) -> Self {
        Self {
            execution_job: Some(value.execution_job.into()),
        }
    }
}

pub struct PutExecuteAnalyzerResponse {}

impl TryFrom<PutExecuteAnalyzerResponseProto> for PutExecuteAnalyzerResponse {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(_value: PutExecuteAnalyzerResponseProto) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<PutExecuteAnalyzerResponse> for PutExecuteAnalyzerResponseProto {
    fn from(_value: PutExecuteAnalyzerResponse) -> Self {
        Self {}
    }
}

pub struct PutExecuteGeneratorRequest {
    execution_job: ExecutionJob,
}

impl TryFrom<PutExecuteGeneratorRequestProto> for PutExecuteGeneratorRequest {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(value: PutExecuteGeneratorRequestProto) -> Result<Self, Self::Error> {
        let execution_job = value
            .execution_job
            .ok_or(Self::Error::MissingRequiredField(
                "PutExecuteGeneratorRequest.execution_job",
            ))?
            .try_into()?;
        Ok(Self { execution_job })
    }
}

impl From<PutExecuteGeneratorRequest> for PutExecuteGeneratorRequestProto {
    fn from(value: PutExecuteGeneratorRequest) -> Self {
        Self {
            execution_job: Some(value.execution_job.into()),
        }
    }
}

pub struct PutExecuteGeneratorResponse {}

impl TryFrom<PutExecuteGeneratorResponseProto> for PutExecuteGeneratorResponse {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(_value: PutExecuteGeneratorResponseProto) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<PutExecuteGeneratorResponse> for PutExecuteGeneratorResponseProto {
    fn from(_value: PutExecuteGeneratorResponse) -> Self {
        Self {}
    }
}
