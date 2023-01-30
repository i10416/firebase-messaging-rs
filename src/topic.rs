use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{GenericGoogleRestAPISupport, RPCError};
use async_trait::async_trait;

const INFO_ENDPOINT: &str = "https://iid.googleapis.com/iid/info"; // + IID_TOKEN

const BATCH_ENDPOINT: &str = "https://iid.googleapis.com/iid/v1";

/// [TopicManagementSupport] trait support APIs in https://developers.google.com/instance-id/reference/server
/// This trait provides topic management utilities.
#[async_trait]
pub(crate) trait TopicManagementSupport: GenericGoogleRestAPISupport {
    fn put_endpoint(iid_token: &str, topic_name: &str) -> String {
        format!("https://iid.googleapis.com/iid/v1/{iid_token}/rel/topics/{topic_name}")
    }
    /// [register_token_to_topic] registers a token to topic.
    /// * topic - topic to follow. You don't need to add `/topics/` prefix.
    /// * token - registration token to be associated with the topic.
    ///
    /// NOTE
    ///
    /// Be careful that Google does not provide official API to retrieve tokens from topic.
    /// In addition, Google does not automatically remove inactive or expired tokens.
    ///
    /// Therefore, it is recommended that developers keep track of token and topic relation (e.g. storing relation in database)
    /// with its modification timestamp so that they can get more control over firebase cloud messaging.
    async fn register_token_to_topic(
        &self,
        topic: &str,
        token: &str,
    ) -> Result<HashMap<String, String>, TopicManagementError> {
        self.post_request(&Self::put_endpoint(token, topic), ())
            .await
    }

    /// [register_tokens_to_topic] registers tokens to topic.
    /// * topic - topic to follow. You don't need to add `/topics/` prefix.
    /// * tokens - registration tokens to be associated with the topic.
    async fn register_tokens_to_topic(
        &self,
        topic: String,
        tokens: Vec<String>,
    ) -> Result<TopicManagementResponse, TopicManagementError> {
        let req = Request::subscribe(format!("/topics/{topic}"), tokens);
        self.post_request(&format!("{BATCH_ENDPOINT}:batchAdd"), req)
            .await
    }
    /// [unregister_tokens_from_topic] unregisters tokens from topic.
    /// * topic - topic to follow. You don't need to add `/topics/` prefix.
    /// * tokens - registration tokens to be unregistered from the topic.
    async fn unregister_tokens_from_topic(
        &self,
        topic: &str,
        tokens: Vec<String>,
    ) -> Result<TopicManagementResponse, TopicManagementError> {
        let req = Request::unsubscribe(format!("/topics/{topic}"), tokens);
        self.post_request(&format!("{BATCH_ENDPOINT}:batchRemove"), req)
            .await
    }
    /// [get_info_by_iid_token] get information about topics accosiated to the given token. Information may contain application id, authorized_entity, platform, etc.
    ///
    /// See [TopicInfoResponse] for more detail.
    ///
    /// * token - get information for this token
    /// * details - response contains `rel` field if and only if `details` flag is true. `rel` field contains all the topics that the `token` is accosiated to.
    ///
    async fn get_info_by_iid_token(
        &self,
        token: &str,
        details: bool,
    ) -> Result<TopicInfoResponse, TopicManagementError> {
        let request_url = if details {
            format!("{INFO_ENDPOINT}/{token}?details=true")
        } else {
            format!("{INFO_ENDPOINT}/{token}")
        };
        self.get_request(&request_url).await
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
///    {"error":"INVALID_ARGUMENT"}, // registration token is invalid
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

#[derive(Debug, Clone, Deserialize)]
pub struct TopicInfoResponse {
    /// example: "com.iid.example"
    pub application: String,
    /// example: "123456782354"
    #[serde(rename = "authorizedEntity")]
    pub authorized_entity: String,

    /// example: "Android", "ANDROID"
    pub platform: String,
    /// example: "1a2bc3d4e5"
    #[serde(rename = "appSigner")]
    pub app_signer: String,
    /// If and only if user specifies `details` flag on request, this field may `Some<Rel>`.
    pub rel: Option<Rel>,
}

/// example
/// ```json
/// {
///    "topics":{
///       "topicname1":{ "addDate":"2015-07-30"},
///       "topicname2":{ "addDate":"2015-07-30"},
///       "topicname3":{"addDate":"2015-07-30"},
///       "topicname4":{"addDate":"2015-07-30"}
///     }
///  }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct Rel {
    pub topics: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportRequest {
    /// example: "com.google.FCMTestApp"
    application: String,
    /// whether is environment sandbox or production?
    sandbox: bool,
    /// example:
    /// ```json
    /// [
    ///   "368dde283db539abc4a6419b1795b6131194703b816e4f624ffa12",
    ///   "76b39c2b2ceaadee8400b8868c2f45325ab9831c1998ed70859d86"
    /// ]
    /// ```
    apns_tokens: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportResponse {
    pub results: Vec<ImportResult>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportResult {
    /// example: "368dde283db539abc4a6419b1795b6131194703b816e4f624ffa12"
    pub apn_token: String,
    /// example: "OK", "Internal Server Error"
    pub status: String,
    /// registration_token exists only if registration succeeds
    /// example: "nKctODamlM4:CKrh_PC8kIb7O...clJONHoA"
    pub registration_token: Option<String>,
}
