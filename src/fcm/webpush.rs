use std::collections::HashMap;

use serde::Serialize;

/// [Webpush protocol](https://tools.ietf.org/html/rfc8030) options.,
#[derive(Debug, Serialize, Default)]
pub struct WebPushConfig {
    /// HTTP headers defined in webpush protocol. Refer to [Webpush protocol](https://tools.ietf.org/html/rfc8030#section-5) for supported headers, e.g. \"TTL\": \"15\".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, String>>,
    /// Web Notification options as a JSON object. Supports Notification instance properties as defined in
    /// [Web Notification API](https://developer.mozilla.org/en-US/docs/Web/API/Notification).
    /// If present, "title" and "body" fields override [google.firebase.fcm.v1.Notification.title] and
    /// [google.firebase.fcm.v1.Notification.body].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification: Option<serde_json::Value>,
    /// Options for features provided by the FCM SDK for Web
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fcm_options: Option<WebPushFcmOptions>,
}

#[derive(Debug, Serialize, Default)]
pub struct WebPushFcmOptions {
    /// Label associated with the message's analytics data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analytics_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The link to open when the user clicks on the notification. For all URL values, HTTPS is required.
    pub link: Option<String>,
}
