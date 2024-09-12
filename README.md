## firebase-messaging-rs

[![ci](https://github.com/i10416/firebase-messaging-rs/actions/workflows/ci.yaml/badge.svg)](https://github.com/i10416/firebase-messaging-rs/actions/workflows/ci.yaml)

## Install

Cargo.toml

```toml
firebase-messaging-rs  = {git = "ssh://git@github.com/i10416/firebase-messaging-rs.git", branch = "main", version = "0.8"}

# wip: firebase-messaging-rs = "0.8"

```

## Overview

This crate implements some wrapper functions for google FCM APIs and Instant ID APIs.

General features like authorization are delegated to [gcloud-sdk-rs](https://github.com/abdolence/gcloud-sdk-rs) under the hood, so you can use oauth2 and OIDC instead of API_KEY.


## Features

You can choose tls backend from native-tls or rustls.

```toml
firebase-messaging-rs  = {git = "ssh://git@github.com/i10416/firebase-messaging-rs.git", branch = "main", version = "0.4", features = ["rustls"] }

# firebase-messaging-rs = { version = "<version>", features = ["fcm", "topic", "rustls"] }
```

## Required GCP roles

Your service account needs following GCP role(s).

- roles/firebase.sdkAdminServiceAgent.

If you need fine-grained permissions, see the table bellow and grant required roles for each api.


| api                            | roles                              |
| ------------------------------ | ---------------------------------- |
| [https://iid.googleapis.com/iid](https://iid.googleapis.com/iid) | roles/identityplatform.admin       |
|                                | roles/firebasecloudmessaging.admin |
|                                | roles/cloudconfig.admin            |
## Example


### Register Token to Topic and Un-Register Token from Topic

```rust no_run
use firebase_messaging_rs::FCMClient;
use firebase_messaging_rs::fcm::*;
use firebase_messaging_rs::topic::*;
use firebase_messaging_rs::fcm::android::*;
use firebase_messaging_rs::fcm::ios::*;
use firebase_messaging_rs::fcm::webpush::*;

// you need to have application_default_credentials.json at $HOME/.config/gcloud directory
// or export GOOGLE_APPLICATION_CREDENTIALS env to authenticate to Firebase.
#[tokio::main]
async fn main() {
  let client = FCMClient::new().await.unwrap();


  let topic_name = "topic_name";
  // bulk register tokens
  let res = client.register_tokens_to_topic(
    topic_name.into(),
    vec!["token_0".to_string(),"token_1".to_string()]
  ).await.unwrap();

  println!("{res:?}");
  // => TopicManagementResponse {results: [{}, {"error": "INVALID_ARGUMENT"}, ...] }

  let sts = client
    .get_info_by_iid_token("token_0", true)
    .await
    .unwrap();
  println!("{sts:?}");
  // => Ok(
  //  TopicInfoResponse {
  //    application: "com.example.app.name",
  //    authorized_entity: "123456789012",
  //    platform: "ANDROID",
  //    app_signer: "....",
  //    rel: Some(
  //      topics: {"topic_name" : { "addDate: : "yyyy-MM-dd" }}
  //    )
  //  }
  //)

  // un-register token from topic
  let res = client
    .unregister_tokens_from_topic(
      topic_name,
      vec!["token_0".to_string()]
    ).await
    .unwrap();
  // => Ok(TopicManagementResponse { results: [{}] })
}

```

### Send Notification to Topic

```rust no_run
use std::collections::HashMap;
use firebase_messaging_rs::FCMClient;
use firebase_messaging_rs::fcm::*;
use firebase_messaging_rs::fcm::android::*;
use firebase_messaging_rs::fcm::ios::*;
use firebase_messaging_rs::fcm::webpush::*;

#[tokio::main]
async fn main() {
  let client = FCMClient::new().await.unwrap();

  let message = Message::Topic {
    topic: "example".to_string(),
    fcm_options: Some(FcmOptions::new("example")),
    notification: Some(Notification {
      title: Some("example".to_string()),
      body: Some("example".to_string()),
      ..Default::default()
    }),
    android: Some(AndroidConfig {
       priority: Some(AndroidMessagePriority::High),
       ..Default::default()
    }),
    webpush: None,
    apns: Some(ApnsConfig::new(
      &Aps {
        content_available: Some(ContentAvailable::On),
        ..Default::default()
      },
      &HashMap::from_iter([
        ("foo".to_string(),"bar".to_string())
      ]),
      Some(
        ApnsHeaders {
          apns_push_type: Some(ApnsPushType::Alert),
          ..Default::default()
        }
      )
    ))
  };
  let res = client.validate(&message).await;
  // => Ok(MessageOutput { name: "projects/{project-id}/messages/{id}" })
}

```


## License

Licensed under either of [Apache License, Version 2.0](https://github.com/abdolence/gcloud-sdk-rs/blob/master/LICENSE-APACHE) or [MIT license](https://github.com/abdolence/gcloud-sdk-rs/blob/master/LICENSE-MIT) at your option.
