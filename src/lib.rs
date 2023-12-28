pub mod topic;
use async_trait::async_trait;
pub use gcloud_sdk::tonic;
use gcloud_sdk::{GoogleAuthTokenGenerator, TokenSourceType, GCP_DEFAULT_SCOPES};
use http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE},
    Request, Response, StatusCode,
};
use hyper::{client::HttpConnector, Body};
use hyper_tls::HttpsConnector;
use serde::Deserialize;
use std::sync::Arc;
use topic::TopicManagementSupport;

/// [FCMClient] implements some wrapper functions for google FCM APIs and Instant ID APIs.
///
/// On [FCMClient] initialization, it tries to load authorization info from well-known locations.
///
/// For example, if you set `GOOGLE_APPLICATION_CREDENTIALS` environment variable pointing to your service account json file,
/// FCMClient tries to read the file from there.
///
/// ```rust
/// let client = FCMClient::new().await.unwrap();
///
/// let res = client.register_tokens_to_topic(
///   "topic_name".into(),
///     vec![token_0,token_1,...]
///   ).await.unwrap();
/// println!("{:?}",res);
/// // => TopicManagementResponse {results: [{}, {"error": "INVALID_ARGUMENT"}, ...] }
/// ```
#[derive(Clone)]
pub struct FCMClient {
    http_client: hyper::Client<HttpsConnector<HttpConnector>>,
    token_gen: Arc<GoogleAuthTokenGenerator>,
}

impl FCMClient {
    pub async fn new() -> Result<Self, String> {
        #[cfg(feature = "hyper-tls")]
        let connector = HttpsConnector::new();

        #[cfg(feature = "hyper-rustls")]
        let connector = HttpsConnector::with_native_roots();

        let token_gen =
            GoogleAuthTokenGenerator::new(TokenSourceType::Default, GCP_DEFAULT_SCOPES.clone())
                .await
                .map_err(|_| "unable to initialize token generator")?;
        Ok(Self {
            token_gen: Arc::new(token_gen),
            http_client: hyper::Client::builder().build::<_, Body>(connector),
        })
    }
}

impl TopicManagementSupport for FCMClient {}

#[async_trait]
impl GenericGoogleRestAPISupport for FCMClient {
    fn get_http_client(&self) -> hyper::Client<HttpsConnector<HttpConnector>, Body> {
        self.http_client.clone()
    }
    async fn get_header_token(&self) -> Result<String, gcloud_sdk::error::Error> {
        let token = self.token_gen.create_token().await?;
        Ok(token.header_value())
    }
}

#[async_trait]
pub trait GenericGoogleRestAPISupport {
    async fn get_header_token(&self) -> Result<String, gcloud_sdk::error::Error>;
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
        let auth_header_value = self
            .get_header_token()
            .await
            .map_err(|_| RPCError::Unauthorized("unable to get header token".into()))
            .map_err(E::from)?;
        let payload = serde_json::to_vec(&payloadable).unwrap();
        let req = Request::builder()
            .uri(endpoint)
            .method("POST")
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .header(AUTHORIZATION, auth_header_value)
            .header("access_token_auth", "true")
            // `access_token_auth` enables authorization based on oauth2 access_token. Without this, We must use unsafe serverKey.
            // https://github.com/firebase/firebase-admin-go/blob/beaa6ae763d2fb57650760b9703cd91cc7c14b9b/messaging/topic_mgt.go#L69
            .header(CONTENT_LENGTH, format!("{}", payload.len() as u64))
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
            .header("access_token_auth", "true")
            // `access_token_auth` enables authorization based on oauth2 access_token. Without this, We must use unsafe serverKey.
            // https://github.com/firebase/firebase-admin-go/blob/beaa6ae763d2fb57650760b9703cd91cc7c14b9b/messaging/topic_mgt.go#L69
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
        res: Response<Body>,
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
            e if e.is_client_error() => Err(E::from(RPCError::InvalidRequest)),
            e if e.is_server_error() => Err(E::from(RPCError::Internal)),
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
    InvalidRequest,
    Internal,
    Unknown(u16),
}

#[cfg(test)]

mod tests {

    use crate::{
        topic::{TopicManagementError, TopicManagementSupport},
        FCMClient,
    };
    #[tokio::test{flavor = "multi_thread"}]

    async fn it_returns_errors_for_invalid_token() {
        let res = FCMClient::new()
            .await
            .expect(
                "FCMClient initialization failed. Did you export GOOGLE_APPLICATION_CREDENTIALS?",
            )
            .register_token_to_topic("topic_name".into(), "")
            .await;
        assert!(matches!(res, Err(TopicManagementError::InvalidRequest)));
    }

    #[tokio::test{flavor = "multi_thread"}]
    async fn it_returns_errors_for_invalid_tokens() {
        let res = FCMClient::new()
            .await
            .expect(
                "FCMClient initialization failed. Did you export GOOGLE_APPLICATION_CREDENTIALS?",
            )
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
    #[tokio::test{ flavor = "multi_thread"}]
    async fn testit() {
        let topic_name =
            std::env::var("TEST_FIREBASE_TOPIC_NAME").expect("TEST_FIREBASE_TOPIC_NAME not found.");
        let tkn =
            std::env::var("TEST_FIREBASE_IID_TOKEN").expect("TEST_FIREBASE_IID_TOKEN not found");
        let c = FCMClient::new().await.expect(
            "FCMClient initialization failed. Did you export GOOGLE_APPLICATION_CREDENTIALS?",
        );
        let sts = c.get_info_by_iid_token(&tkn, true).await;
        let res = c.register_token_to_topic(&topic_name, &tkn).await;
        let res = c
            .unregister_tokens_from_topic(&topic_name, vec![tkn.clone().into()])
            .await;
        let sts = c.get_info_by_iid_token(&tkn, true).await;
    }
}
