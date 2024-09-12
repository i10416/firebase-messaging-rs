use std::{collections::HashMap, time::Duration};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
/// Android specific options for messages sent through FCM connection server.
pub mod android;
/// Apple Push Notification Service specific options.
pub mod ios;
/// Webpush protocol options.
pub mod webpush;
use crate::{GenericGoogleRestAPISupport, RPCError};

use android::AndroidConfig;
use ios::ApnsConfig;
use webpush::WebPushConfig;

#[async_trait]
/// [FCMApi] trait supports APIs in <https://firebase.google.com/docs/reference/fcm/rest>
/// This trait provides firebase cloud messaging utilities.
pub trait FCMApi: GenericGoogleRestAPISupport {
    fn post_endpoint(project_id: &str) -> String {
        format!("https://fcm.googleapis.com/v1/projects/{project_id}/messages:send")
    }
    /// Send the message to firebase messaging API.
    async fn send(&self, message: &Message) -> Result<MessageOutput, FCMError> {
        let payload = MessagePayload {
            validate_only: false,
            message,
        };
        self.post_request(&Self::post_endpoint(&self.project_id()), &payload)
            .await
    }
    /// Send the message to firebase messaging API with dry run option.
    async fn validate(&self, message: &Message) -> Result<MessageOutput, FCMError> {
        let payload = MessagePayload {
            validate_only: true,
            message,
        };
        self.post_request(&Self::post_endpoint(&self.project_id()), &payload)
            .await
    }
}

#[derive(Debug, Serialize)]
/// Message payload sent to firebase messaging API.
pub(crate) struct MessagePayload<'a> {
    validate_only: bool,
    message: &'a Message,
}

#[derive(Debug, Deserialize, Clone)]
pub enum FCMError {
    InternalRequestError { reason: String },
    InternalResponseError { reason: String },
    Unauthorized(String),
    InvalidRequestDescriptive { reason: String },
    InvalidRequest,
    RetryableInternal { retry_after: Duration },
    Internal,
    Unknown { code: u16, hint: Option<String> },
}

impl From<RPCError> for FCMError {
    fn from(value: RPCError) -> Self {
        match value {
            RPCError::BuildRequestFailure(reason) => Self::InternalRequestError { reason },
            RPCError::Unauthorized(reason) => Self::Unauthorized(reason),
            RPCError::HttpRequestFailure => Self::InternalRequestError {
                reason: "unable to process http request".to_string(),
            },
            RPCError::DecodeFailure => Self::InternalResponseError {
                reason: "unable to decode response body bytes".to_string(),
            },
            RPCError::DeserializeFailure { reason, source } => Self::InternalResponseError {
                reason: format!("unable to deserialize response body to type: {reason}: {source}"),
            },
            RPCError::InvalidRequest {
                details: Some(details),
            } => Self::InvalidRequestDescriptive { reason: details },
            RPCError::InvalidRequest { details: None } => Self::InvalidRequest,
            RPCError::Internal {
                retry_after: Some(retry_after),
            } => Self::RetryableInternal { retry_after },
            RPCError::Internal { retry_after: None } => Self::Internal,
            RPCError::Unknown(code) => Self::Unknown { code, hint: None },
        }
    }
}
/// Low-level type representing FCM Message type.
/// See <https://fcm.googleapis.com/$discovery/rest?version=v1> for details.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Message {
    Token {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<HashMap<String, String>>,
        /// Registration token to send a message to.
        token: String,
        /// Template for FCM SDK feature options to use across all platforms.
        #[serde(skip_serializing_if = "Option::is_none")]
        fcm_options: Option<FcmOptions>,
        /// Basic notification template to use across all platforms.
        #[serde(skip_serializing_if = "Option::is_none")]
        notification: Option<Notification>,
        /// Android specific options for messages sent through [FCM connection server](https://goo.gl/4GLdUl).
        #[serde(skip_serializing_if = "Option::is_none")]
        android: Option<AndroidConfig>,
        /// [Webpush protocol](https://tools.ietf.org/html/rfc8030) options.
        #[serde(skip_serializing_if = "Option::is_none")]
        webpush: Option<WebPushConfig>,

        /// [Apple Push Notification Service](https://goo.gl/MXRTPa) specific options.
        #[serde(skip_serializing_if = "Option::is_none")]
        apns: Option<ApnsConfig>,
    },
    Topic {
        /// Topic name to send a message to, e.g. "weather". Note: "/topics/" prefix should not be provided.
        topic: String,
        /// Template for FCM SDK feature options to use across all platforms.
        #[serde(skip_serializing_if = "Option::is_none")]
        fcm_options: Option<FcmOptions>,
        /// Basic notification template to use across all platforms.
        #[serde(skip_serializing_if = "Option::is_none")]
        notification: Option<Notification>,
        /// Android specific options for messages sent through [FCM connection server](https://goo.gl/4GLdUl).
        #[serde(skip_serializing_if = "Option::is_none")]
        android: Option<AndroidConfig>,

        /// [Webpush protocol](https://tools.ietf.org/html/rfc8030) options.
        #[serde(skip_serializing_if = "Option::is_none")]
        webpush: Option<WebPushConfig>,

        /// [Apple Push Notification Service](https://goo.gl/MXRTPa) specific options.
        #[serde(skip_serializing_if = "Option::is_none")]
        apns: Option<ApnsConfig>,
    },
    Condition {
        /// "Condition to send a message to, e.g. "'foo' in topics && 'bar' in topics".
        condition: String,
        /// Template for FCM SDK feature options to use across all platforms.
        #[serde(skip_serializing_if = "Option::is_none")]
        fcm_options: Option<FcmOptions>,
        /// Basic notification template to use across all platforms.
        #[serde(skip_serializing_if = "Option::is_none")]
        notification: Option<Notification>,
        /// Android specific options for messages sent through [FCM connection server](https://goo.gl/4GLdUl).
        #[serde(skip_serializing_if = "Option::is_none")]
        android: Option<AndroidConfig>,
        /// [Webpush protocol](https://tools.ietf.org/html/rfc8030) options.
        #[serde(skip_serializing_if = "Option::is_none")]
        webpush: Option<WebPushConfig>,

        /// [Apple Push Notification Service](https://goo.gl/MXRTPa) specific options.
        #[serde(skip_serializing_if = "Option::is_none")]
        apns: Option<ApnsConfig>,
    },
}

#[derive(Debug, Serialize, Default)]
/// Platform independent options for features provided by the FCM SDKs.
pub struct FcmOptions {
    /// Label associated with the message's analytics data.
    #[serde(skip_serializing_if = "Option::is_none")]
    analytics_label: Option<String>,
}
impl FcmOptions {
    pub fn new(analytics_label: &str) -> Self {
        Self {
            analytics_label: Some(analytics_label.to_string()),
        }
    }
}

#[derive(Debug, Serialize, Default)]
///  Basic notification template to use across all platforms.
pub struct Notification {
    /// The notification title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// The notification's body text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// Contains the URL of an image that is going to be downloaded on the device
    /// and displayed in a notification. JPEG, PNG, BMP have full support across platforms.
    /// Animated GIF and video only work on iOS. WebP and HEIF have varying levels of
    /// support across platforms and platform versions. Android has 1MB image size limit.
    /// Quota usage and implications/costs for hosting image on Firebase Storage: <https://firebase.google.com/pricing>
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
/// Payload returned from firebase messaging API.
pub struct MessageOutput {
    /// "Output Only. The identifier of the message sent, in the format of `projects/*/messages/{message_id}`."
    pub name: String,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{Message, Notification};
    use crate::fcm::ApnsConfig;
    #[test]
    pub fn ios_background_notification() {
        let background_notification = Message::Topic {
            topic: "background_channel".to_string(),
            fcm_options: None,
            notification: Some(Notification {
                title: Some("example".to_string()),
                ..Default::default()
            }),
            android: None,
            webpush: None,
            apns: Some(ApnsConfig::ios_background_notification(HashMap::from_iter(
                [("message".to_string(), "Hello, World!".to_string())],
            ))),
        };
        let result = serde_json::to_value(&background_notification).expect("should always succeed");
        let expected = serde_json::json!({
            "topic": "background_channel",
            "notification": {
                "title": "example"
            },
            "apns": {
                "payload": {
                    "aps": {
                        "content-available": 1
                    },
                    "message": "Hello, World!"
                },
                "headers": {
                    "apns-push-type": "background",
                    "apns-priority": "5"
                }
            }
        });
        assert_eq!(result, expected)
    }
}
