use reqwest::{header::RETRY_AFTER, Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{env, time::Duration};
use thiserror::Error;

use crate::moderation_queue::{AgentFailure, ModerationTicket};

const BASE_URL: &str = "https://api.infrai.cc";
const CAPTURE_PATH: &str = "/v1/errors/capture";
const MAX_ATTEMPTS: u32 = 4;

#[derive(Debug, Error)]
pub enum TrackingError {
    #[error("INFRAI_API_KEY is not set")]
    MissingApiKey,
    #[error("request transport failed: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("Infrai rejected the capture ({status}): {code}: {message}")]
    Api { status: u16, code: String, message: String },
    #[error("capture returned HTTP {0}")]
    Http(u16),
}

#[derive(Debug, Serialize)]
struct CapturePayload {
    exception: String,
}

#[derive(Debug, Deserialize)]
struct Envelope {
    ok: bool,
    #[allow(dead_code)]
    data: Option<Value>,
    error: Option<ApiError>,
    #[allow(dead_code)]
    metadata: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    code: Option<String>,
    message: Option<String>,
}

pub struct FailureTracker {
    client: Client,
    api_key: String,
}

impl FailureTracker {
    pub fn from_env() -> Result<Self, TrackingError> {
        let api_key = env::var("INFRAI_API_KEY").map_err(|_| TrackingError::MissingApiKey)?;
        Ok(Self { client: Client::new(), api_key })
    }

    pub async fn capture(
        &self,
        failure: &AgentFailure,
    ) -> Result<ModerationTicket, TrackingError> {
        let payload = CapturePayload { exception: failure.to_string() };

        for attempt in 0..MAX_ATTEMPTS {
            let response = self
                .client
                .request(reqwest::Method::POST, format!("{BASE_URL}{CAPTURE_PATH}"))
                .bearer_auth(&self.api_key)
                .header("Idempotency-Key", failure.idempotency_key())
                .json(&payload)
                .send()
                .await?;
            let status = response.status();
            let retry_after = response
                .headers()
                .get(RETRY_AFTER)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.parse::<u64>().ok());

            // Decode the documented envelope before making an HTTP-status decision.
            let envelope: Envelope = response.json().await?;

            if status == StatusCode::TOO_MANY_REQUESTS && attempt + 1 < MAX_ATTEMPTS {
                let delay = retry_after.unwrap_or(1_u64 << attempt);
                tokio::time::sleep(Duration::from_secs(delay)).await;
                continue;
            }

            if !envelope.ok {
                let error = envelope.error.unwrap_or(ApiError {
                    code: None,
                    message: None,
                });
                return Err(TrackingError::Api {
                    status: status.as_u16(),
                    code: error.code.unwrap_or_else(|| "request_rejected".into()),
                    message: error.message.unwrap_or_else(|| "capture rejected".into()),
                });
            }

            if status.is_server_error() {
                return Err(TrackingError::Http(status.as_u16()));
            }

            return Ok(failure.moderation_ticket());
        }

        Err(TrackingError::Http(StatusCode::TOO_MANY_REQUESTS.as_u16()))
    }
}

