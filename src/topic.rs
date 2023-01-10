use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{GenericGoogleRestAPISupport, RPCError};
use async_trait::async_trait;

const INFO_ENDPOINT: &str = "https://iid.googleapis.com/iid/info/"; // + IID_TOKEN
fn put_endpoint(iid_token: &str, topic_name: &str) -> String {
    format!("https://iid.googleapis.com/iid/v1/{iid_token}/rel/topics/{topic_name}")
}
const BATCH_ENDPOINT: &str = "https://iid.googleapis.com/iid/v1";

/// [TopicManagementSupport] trait support APIs in https://developers.google.com/instance-id/reference/server
#[async_trait]
pub(crate) trait TopicManagementSupport: GenericGoogleRestAPISupport {
    /// subscribe_to_topic
    /// * topic - topic to follow. You don't need to add `/topics/` prefix.
    /// * tokens - registration tokens to be associated with the topic.
    async fn subscribe_to_topic(
        &self,
        topic: String,
        tokens: Vec<String>,
    ) -> Result<TopicManagementResponse, TopicManagementError> {
        let req = Request::subscribe(format!("/topics/{topic}"), tokens);
        self.post_request(&format!("{BATCH_ENDPOINT}:batchAdd"), req)
            .await
    }
    /// unsubscribe_to_topic
    /// * topic - topic to follow. You don't need to add `/topics/` prefix.
    /// * tokens - registration tokens to be unregistered from the topic.
    async fn unsubscribe_to_topic(
        &self,
        topic: String,
        tokens: Vec<String>,
    ) -> Result<TopicManagementResponse, TopicManagementError> {
        let req = Request::unsubscribe(format!("/topics/{topic}"), tokens);
        self.post_request(&format!("{BATCH_ENDPOINT}:batchRemove"), req)
            .await
    }
}

#[derive(Debug, Clone, Serialize)]
struct Request {
    #[serde(rename = "to")]
    topic: String,
    #[serde(rename = "registration_tokens")]
    tokens: Vec<String>,
}

impl Request {
    fn subscribe(topic: String, tokens: Vec<String>) -> Self {
        Self { topic, tokens }
    }
    fn unsubscribe(topic: String, tokens: Vec<String>) -> Self {
        Self { topic, tokens }
    }
}
// FIXME: better error modeling
///
/// [TopicManagementResponse] is a raw response type from iid endpoint.
///
/// example
///
/// ```json
///{
///  "results":[
///    {}, // registration suceeded
///    {"error":"NOT_FOUND"}, // registration token has been deleted or app has been uninstalled
///    {"error":"INVALID_ARGUMENT"},
///    {"error":"INTERNAL"}, // internal server error
///    {"error":"TOO_MANY_TOPICS"}, // app has too many topics
///    {},
///  ]
///}
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct TopicManagementResponse {
    pub results: Vec<HashMap<String, String>>,
}
#[derive(Debug, Clone)]
pub enum TopicManagementError {
    /// Unauthorized. Check
    /// 1. your `GOOGLE_APPLICATION_CREDENTIALS` is properly set
    /// 2. service account has sufficient permission to invoke firebase cloud messaging API
    Unauthorized(String),
    /// Request is invalid. Check
    /// 1. your topic name is correct
    InvalidRequest,
    ServerError,
    InternalRequestError,
    InternalResponseError,
    Unknown,
}

impl From<RPCError> for TopicManagementError {
    fn from(e: RPCError) -> Self {
        match e {
            RPCError::BuildRequestFailure => Self::InternalRequestError, // FIXME gulanuarity
            RPCError::HttpRequestFailure => Self::InternalRequestError,
            RPCError::DecodeFailure => Self::InternalResponseError,
            RPCError::DeserializeFailure => Self::InternalResponseError,
            RPCError::Unauthorized(msg) => Self::Unauthorized(msg),
            RPCError::InvalidRequest => Self::InvalidRequest,
            RPCError::Internal => Self::ServerError,
            RPCError::Unknown(_) => Self::Unknown,
        }
    }
}
