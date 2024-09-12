#[cfg(feature = "fcm")]
pub use serde_json;
#[cfg(feature = "fcm")]
pub mod fcm;
#[cfg(feature = "topic-management")]
pub mod topic;

use async_trait::async_trait;
use gcloud_sdk::{GoogleAuthTokenGenerator, TokenSourceType, GCP_DEFAULT_SCOPES};
use http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE},
    HeaderName, Request, Response, StatusCode,
};
use hyper::{client::HttpConnector, Body};
#[cfg(feature = "hyper-rustls")]
use hyper_rustls::HttpsConnector;
#[cfg(feature = "hyper-tls")]
use hyper_tls::HttpsConnector;
use serde::Deserialize;
use std::{env, sync::Arc, time::Duration};

/// [FCMClient] implements some wrapper functions for google FCM APIs and Instant ID APIs.
///
/// On [FCMClient] initialization, it tries to load authorization info from well-known locations.
///
/// For example, if you set `GOOGLE_APPLICATION_CREDENTIALS` environment variable pointing to your service account json file,
/// FCMClient tries to read the file from there.
///
/// For details, see https://google.aip.dev/auth/4110
///
/// ```no_run
/// use firebase_messaging_rs::FCMClient;
/// use firebase_messaging_rs::fcm::*;
/// use firebase_messaging_rs::topic::*;
///
/// #[tokio::main]
/// async fn main() {
///   let client = FCMClient::new().await.unwrap();
///   let _ = client.register_tokens_to_topic(
///     "topic_name".into(),
///       vec!["token_0".to_string(),"token_1".to_string()]
///     ).await;
///   // => TopicManagementResponse {results: [{}, {"error": "INVALID_ARGUMENT"}, ...] }
/// }
/// ```
#[derive(Clone)]
pub struct FCMClient {
    http_client: hyper::Client<HttpsConnector<HttpConnector>>,
    token_gen: Arc<GoogleAuthTokenGenerator>,
    project_id: String,
}

impl FCMClient {
    fn google_cloud_project() -> Option<String> {
        env::var("GOOGLE_CLOUD_PROJECT")
            .or_else(|_| env::var("GCP_PROJECT"))
            .ok()
    }
    pub async fn new() -> Result<Self, String> {
        #[cfg(feature = "fcm")]
        let project_id = Self::google_cloud_project().ok_or(
            "Cannot detect google project id from env. Provide project id by GOOGLE_CLOUD_PROJECT env var.".to_string(),
        )?;
        #[cfg(not(feature = "fcm"))]
        let project_id = "dummy id for compatibility".to_string();
        FCMClient::with_scope(&project_id, &GCP_DEFAULT_SCOPES).await
    }
    pub async fn new_with_project(project_id: &str) -> Result<Self, String> {
        FCMClient::with_scope(project_id, &GCP_DEFAULT_SCOPES).await
    }

    pub async fn with_scope(project_id: &str, scopes: &[String]) -> Result<Self, String> {
        #[cfg(feature = "hyper-tls")]
        let connector = HttpsConnector::new();

        #[cfg(feature = "hyper-rustls")]
        let connector = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .map_err(|_| "unable to load native roots for https connector".to_string())?
            .https_or_http()
            .enable_http1()
            .build();

        let token_gen = GoogleAuthTokenGenerator::new(TokenSourceType::Default, scopes.to_vec())
            .await
            .map_err(|_| "unable to initialize token generator")?;
        Ok(Self {
            token_gen: Arc::new(token_gen),
            http_client: hyper::Client::builder().build::<_, Body>(connector),
            project_id: project_id.to_string(),
        })
    }
}

#[cfg(feature = "topic-management")]
impl crate::topic::TopicManagementSupport for FCMClient {}
#[cfg(feature = "fcm")]
impl crate::fcm::FCMApi for FCMClient {}

#[async_trait]
impl GenericGoogleRestAPISupport for FCMClient {
    fn get_http_client(&self) -> hyper::Client<HttpsConnector<HttpConnector>, Body> {
        self.http_client.clone()
    }
    fn project_id(&self) -> String {
        self.project_id.to_string()
    }
    async fn get_header_token(&self) -> Result<String, gcloud_sdk::error::Error> {
        let token = self.token_gen.create_token().await?;
        Ok(token.header_value())
    }
}

#[async_trait]
pub trait GenericGoogleRestAPISupport {
    async fn get_header_token(&self) -> Result<String, gcloud_sdk::error::Error>;
    fn project_id(&self) -> String;
    fn get_http_client(&self) -> hyper::Client<HttpsConnector<HttpConnector>, Body>;
    async fn post_request<
        P: serde::Serialize + Send + Sync,
        R: for<'a> Deserialize<'a> + Clone,
        E: From<RPCError>,
    >(
        &self,
        endpoint: &str,
        payloadable: P,
    ) -> Result<R, E> {
        self.post_request_with(endpoint, payloadable, &[]).await
    }

    async fn post_request_with<
        P: serde::Serialize + Send + Sync,
        R: for<'a> Deserialize<'a> + Clone,
        E: From<RPCError>,
    >(
        &self,
        endpoint: &str,
        payloadable: P,
        extra_headers: &[(&str, &str)],
    ) -> Result<R, E> {
        let auth_header_value = self
            .get_header_token()
            .await
            .map_err(|_| RPCError::Unauthorized("unable to get header token".into()))
            .map_err(E::from)?;
        let payload = serde_json::to_vec(&payloadable).unwrap();
        let mut builder = Request::builder()
            .uri(endpoint)
            .method("POST")
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .header(AUTHORIZATION, auth_header_value)
            .header(CONTENT_LENGTH, format!("{}", payload.len() as u64));
        for (key, value) in extra_headers {
            builder = builder.header(*key, *value)
        }
        let req = builder
            .body(Body::from(payload))
            .map_err(|e| RPCError::BuildRequestFailure(format!("{e:?}")))
            .map_err(E::from)?;
        let res = self
            .get_http_client()
            .request(req)
            .await
            .map_err(|_| RPCError::HttpRequestFailure) // FIXME: propagate error info
            .map_err(E::from)?;
        Self::handle_response_body(res).await
    }

    async fn get_request<R: for<'a> Deserialize<'a> + Clone, E: From<RPCError>>(
        &self,
        endpoint: &str,
    ) -> Result<R, E> {
        let auth_header_value = self
            .get_header_token()
            .await
            .map_err(|_| RPCError::Unauthorized("unable to get header token".into()))
            .map_err(E::from)?;
        let req = Request::builder()
            .uri(endpoint)
            .method("GET")
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .header(AUTHORIZATION, auth_header_value)
            .body(Body::empty()) // NOTE: what is difference between Body::empty() and ()?
            .map_err(|e| RPCError::BuildRequestFailure(format!("{e:?}")))
            .map_err(E::from)?;
        let res = self
            .get_http_client()
            .request(req)
            .await
            .map_err(|_| RPCError::HttpRequestFailure) // FIXME: don't swallow error! propagate error info
            .map_err(E::from)?;
        Self::handle_response_body(res).await
    }
    async fn get_request_with<R: for<'a> Deserialize<'a> + Clone, E: From<RPCError>>(
        &self,
        endpoint: &str,
        extra_headers: &[(&str, &str)],
    ) -> Result<R, E> {
        let auth_header_value = self
            .get_header_token()
            .await
            .map_err(|_| RPCError::Unauthorized("unable to get header token".into()))
            .map_err(E::from)?;
        let mut builder = Request::builder()
            .uri(endpoint)
            .method("GET")
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .header(AUTHORIZATION, auth_header_value);
        for (key, value) in extra_headers {
            builder = builder.header(*key, *value)
        }
        let req = builder
            .body(Body::empty()) // NOTE: what is difference between Body::empty() and ()?
            .map_err(|e| RPCError::BuildRequestFailure(format!("{e:?}")))
            .map_err(E::from)?;
        let res = self
            .get_http_client()
            .request(req)
            .await
            .map_err(|_| RPCError::HttpRequestFailure) // FIXME: don't swallow error! propagate error info
            .map_err(E::from)?;
        Self::handle_response_body(res).await
    }

    async fn handle_response_body<R: for<'a> Deserialize<'a> + Clone, E: From<RPCError>>(
        mut res: Response<Body>,
    ) -> Result<R, E> {
        match res.status() {
            StatusCode::OK => {
                let buf = hyper::body::to_bytes(res)
                    .await
                    .map_err(|_| RPCError::DecodeFailure)
                    .map_err(E::from)?;
                let text = std::str::from_utf8(&buf).unwrap_or_default();
                serde_json::from_slice::<R>(&buf)
                    .map_err(|e| RPCError::DeserializeFailure {
                        reason: format!("{e:?}"),
                        source: text.to_string(),
                    })
                    .map_err(E::from)
            }
            StatusCode::UNAUTHORIZED => {
                Err(RPCError::Unauthorized(
                    "unable to access firebase resource".into(),
                ))
            }
            .map_err(E::from),
            StatusCode::BAD_REQUEST => {
                let data = hyper::body::to_bytes(res.body_mut())
                    .await
                    .map_err(|_| RPCError::DecodeFailure)?;
                let data = String::from_utf8(data.to_vec()).ok();
                Err(E::from(RPCError::InvalidRequest { details: data }))
            }
            e if e.is_client_error() => Err(E::from(RPCError::invalid_request())),
            e if e.is_server_error() => {
                if let Some(retry_after_sec) = res
                    .headers()
                    .get(HeaderName::from_static("Retry-After"))
                    .and_then(|h| h.to_str().ok().and_then(|s| s.parse::<u64>().ok()))
                {
                    Err(E::from(RPCError::retryable_internal(Duration::from_secs(
                        retry_after_sec,
                    ))))
                } else {
                    Err(E::from(RPCError::internal()))
                }
            }
            e => Err(E::from(RPCError::Unknown(e.as_u16()))),
        }
    }
}

/// [RPCError] is internal error types. Please use dedicated error types like [topic::TopicManagementError] in general.
#[derive(Debug, Clone)]
pub enum RPCError {
    Unauthorized(String),
    BuildRequestFailure(String),
    HttpRequestFailure,
    DecodeFailure,
    DeserializeFailure { reason: String, source: String },
    InvalidRequest { details: Option<String> },
    Internal { retry_after: Option<Duration> },
    Unknown(u16),
}
impl RPCError {
    pub fn invalid_request() -> Self {
        Self::InvalidRequest { details: None }
    }
    pub fn invalid_request_descriptive(data: &str) -> Self {
        Self::InvalidRequest {
            details: Some(data.to_string()),
        }
    }
    pub fn internal() -> Self {
        RPCError::Internal { retry_after: None }
    }
    pub fn retryable_internal(retry_after: Duration) -> Self {
        RPCError::Internal {
            retry_after: Some(retry_after),
        }
    }
}

#[cfg(test)]

mod tests {

    #[cfg(feature = "fcm")]
    use crate::fcm::android::*;
    #[cfg(feature = "fcm")]
    use crate::fcm::ios::*;
    #[cfg(feature = "fcm")]
    use crate::fcm::webpush::*;
    #[cfg(feature = "fcm")]
    use crate::fcm::*;
    #[cfg(feature = "topic-management")]
    use crate::topic::*;
    use crate::FCMClient;
    use std::collections::HashMap;
    #[cfg(feature = "fcm")]
    #[tokio::test{flavor = "multi_thread"}]
    async fn full_message_payload_should_pass_validation() {
        let client = FCMClient::new().await.unwrap();
        let aps = Aps {
            alert: Some(Alert::Structural(
                RichAlert {
                    title: Some("example".to_string()),
                    subtitle: Some("example".to_string()),
                    body: Some("example".to_string()),
                    launch_image: Some("example".to_string()),
                    title_loc_key: Some("example".to_string()),
                    title_loc_args: Some(vec!["example".to_string()]),
                    subtitle_loc_key: Some("example".to_string()),
                    subtitle_loc_args: Some(vec!["example".to_string()]),
                    loc_key: Some("example".to_string()),
                    loc_args: Some(vec!["example".to_string()]),
                }
                .into(),
            )),
            badge: Some(42),
            thread_id: Some("example".to_string()),
            content_available: Some(ContentAvailable::On),
            mutable_content: Some(MutableContent::On),
            timestamp: Some(0),
            event: Some("example".to_string()),
            dismissal_date: Some(0),
            attributes_type: Some("example".to_string()),
        };
        let headers = ApnsHeaders {
            authorization: Some("example".to_string()),
            apns_id: Some("example".to_string()),
            apns_push_type: Some(ApnsPushType::Alert),
            apns_expiration: Some(ios::Duration::from_secs(3600)),
            apns_priority: Some(ApnsPriority::RespectEnergySavingMode),
            apns_topic: Some("example".to_string()),
            apns_collapse_id: Some("example".to_string()),
        };
        let msg = Message::Topic {
            topic: "example".to_string(),
            fcm_options: Some(FcmOptions::new("example")),
            notification: Some(Notification {
                title: Some("example".to_string()),
                body: Some("example".to_string()),
                image: Some("https://example.com/example.png".to_string()),
            }),
            android: Some(AndroidConfig {
                fcm_options: Some(AndroidFcmOptions::new("example")),
                priority: Some(AndroidMessagePriority::Normal),
                notification: Some(AndroidNotification {
                    local_only: Some(true),
                    default_light_settings: Some(false),
                    default_sound: Some(false),
                    image: Some("https://example.com/example.png".to_string()),
                    tag: Some("example".to_string()),
                    default_vibrate_timings: Some(false),
                    notification_count: Some(1),
                    title_loc_key: Some("example".to_string()),
                    bypass_proxy_notification: Some(false),
                    click_action: Some("example".to_string()),
                    sound: Some("default".to_string()),
                    // FIXME
                    event_time: Some("1970-01-01T00:00:00Z".to_string()),
                    title: Some("example".to_string()),
                    vibrate_timings: Some(vec![android::Duration::from_secs(10.0)]),
                    body_loc_key: Some("example".to_string()),
                    body: Some("example".to_string()),
                    icon: Some("https://example.com/example.ico".to_string()),
                    title_loc_args: Some(vec!["example".to_string()]),
                    color: Some("#FFFFFF".to_string()),
                    body_loc_args: Some(vec!["example".to_string()]),
                    sticky: Some(true),
                    proxy: Some(Proxy::ProxyUnspecified),
                    ticker: Some("example".to_string()),
                    notification_priority: Some(NotificationPriority::PriorityDefault),
                    visibility: Some(android::Visibility::VisibilityUnspecified),
                    channel_id: Some("example".to_string()),
                    light_settings: Some(LightSettings {
                        color: Color {
                            red: 255.0,
                            green: 255.0,
                            blue: 255.0,
                            alpha: 1.0,
                        },
                        light_on_duration: Some(android::Duration::from_secs(10.0)),
                        light_off_duration: Some(android::Duration::from_secs(10.0)),
                    }),
                }),
                data: Some(HashMap::from_iter([("foo".to_string(), "bar".to_string())])),
                restricted_package_name: Some("com.example.app".to_string()),
                ttl: Some(android::Duration::from_secs(3.5)),
                direct_boot_ok: Some(true),
                collapse_key: Some("example".to_string()),
            }),
            webpush: Some(WebPushConfig {
                headers: Some(HashMap::from_iter([("foo".to_string(), "bar".to_string())])),
                data: Some(HashMap::from_iter([("foo".to_string(), "bar".to_string())])),
                notification: None,
                fcm_options: Some(WebPushFcmOptions {
                    analytics_label: Some("example".to_string()),
                    link: Some("example".to_string()),
                }),
            }),
            apns: Some(ApnsConfig::new(&aps, &HashMap::default(), Some(headers))),
        };
        let res = client.send(&msg).await;
        println!("{res:?}")
    }
    #[cfg(feature = "topic-management")]
    #[tokio::test{flavor = "multi_thread"}]
    async fn it_returns_errors_for_invalid_token() {
        let res = FCMClient::new()
            .await
            .expect("FCMClient initialization failed. Did you set GOOGLE_APPLICATION_CREDENTIALS?")
            .register_token_to_topic("topic_name".into(), "")
            .await;
        assert!(matches!(res, Err(TopicManagementError::InvalidRequest)));
    }
    #[cfg(feature = "topic-management")]
    #[tokio::test{flavor = "multi_thread"}]
    async fn it_returns_errors_for_invalid_tokens() {
        let res = FCMClient::new()
            .await
            .expect("FCMClient initialization failed. Did you set GOOGLE_APPLICATION_CREDENTIALS?")
            .register_tokens_to_topic("topic_name".into(), vec!["".into(), "".into(), "".into()])
            .await
            .expect("Request Failed Due to: ");
        let error_results = res.results;
        assert!(error_results.iter().all(|result| {
            match result.get("error") {
                Some(msg) => msg == "INVALID_ARGUMENT",
                _ => false,
            }
        }));
    }
    #[allow(unused)]
    #[cfg(feature = "topic-management")]
    #[tokio::test{ flavor = "multi_thread"}]
    async fn test_for_interactive_debug() {
        let topic_name =
            std::env::var("TEST_FIREBASE_TOPIC_NAME").expect("TEST_FIREBASE_TOPIC_NAME not found.");
        let tkn =
            std::env::var("TEST_FIREBASE_IID_TOKEN").expect("TEST_FIREBASE_IID_TOKEN not found");
        let c = FCMClient::new()
            .await
            .expect("FCMClient initialization failed. Did you set GOOGLE_APPLICATION_CREDENTIALS?");
        let sts = c.get_info_by_iid_token(&tkn, true).await;
        let res = c.register_token_to_topic(&topic_name, &tkn).await;
        let res = c
            .unregister_tokens_from_topic(&topic_name, vec![tkn.clone().into()])
            .await;
        let sts = c.get_info_by_iid_token(&tkn, true).await;
    }
}
