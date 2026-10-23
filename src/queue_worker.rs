use std::env;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerError {
    MissingApiKey,
    Api { code: String, message: String },
    InvalidJob,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatorJob {
    pub kind: String,
    pub asset_id: String,
    pub subscriber_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobAction {
    DeliverAsset { asset_id: String, subscriber_id: String },
    PublishUpdate { subscriber_id: String },
    ProcessContent { asset_id: String },
}

pub fn decide(job: &CreatorJob) -> Result<JobAction, WorkerError> {
    if job.asset_id.is_empty() || job.subscriber_id.is_empty() {
        return Err(WorkerError::InvalidJob);
    }
    match job.kind.as_str() {
        "digital_asset" => Ok(JobAction::DeliverAsset {
            asset_id: job.asset_id.clone(),
            subscriber_id: job.subscriber_id.clone(),
        }),
        "subscriber_update" => Ok(JobAction::PublishUpdate {
            subscriber_id: job.subscriber_id.clone(),
        }),
        "content" => Ok(JobAction::ProcessContent {
            asset_id: job.asset_id.clone(),
        }),
        _ => Err(WorkerError::InvalidJob),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub method: &'static str,
    pub path: &'static str,
    pub authorization: String,
    pub body: String,
}

pub struct InfraiQueue {
    api_key: String,
}

impl InfraiQueue {
    pub fn from_env() -> Result<Self, WorkerError> {
        let api_key = env::var("INFRAI_API_KEY").map_err(|_| WorkerError::MissingApiKey)?;
        Ok(Self { api_key })
    }

    pub fn consume_request(&self, queue: &str, max_messages: u32) -> Request {
        Request {
            method: "POST",
            path: "/v1/queue/consume",
            authorization: format!("Bearer {}", self.api_key),
            body: format!("{{\"queue\":\"{}\",\"max_messages\":{}}}", queue, max_messages),
        }
    }

    pub fn ack_request(&self, queue: &str, message_id: &str) -> Request {
        Request {
            method: "POST",
            path: "/v1/queue/ack",
            authorization: format!("Bearer {}", self.api_key),
            body: format!("{{\"queue\":\"{}\",\"message_id\":\"{}\"}}", queue, message_id),
        }
    }

    pub fn nack_request(&self, queue: &str, message_id: &str) -> Request {
        Request {
            method: "POST",
            path: "/v1/queue/nack",
            authorization: format!("Bearer {}", self.api_key),
            body: format!("{{\"queue\":\"{}\",\"message_id\":\"{}\"}}", queue, message_id),
        }
    }

    pub fn check_envelope(&self, envelope: Envelope) -> Result<String, WorkerError> {
        if envelope.ok {
            Ok(envelope.data)
        } else {
            Err(WorkerError::Api { code: envelope.code, message: envelope.message })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope {
    pub ok: bool,
    pub data: String,
    pub code: String,
    pub message: String,
}

pub async fn run_one(job: CreatorJob) -> Result<JobAction, WorkerError> {
    decide(&job)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_job_is_sent_to_processing_instead_of_delivery() {
        let job = CreatorJob {
            kind: "content".into(),
            asset_id: "post-42".into(),
            subscriber_id: "member-7".into(),
        };
        assert_eq!(decide(&job), Ok(JobAction::ProcessContent { asset_id: "post-42".into() }));
    }
}
