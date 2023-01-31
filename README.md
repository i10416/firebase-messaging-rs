## firebase-messaging-rs

[![ci](https://github.com/i10416/firebase-messaging-rs/actions/workflows/ci.yaml/badge.svg)](https://github.com/i10416/firebase-messaging-rs/actions/workflows/ci.yaml)

## Install

Cargo.toml

```toml
firebase-messaging-rs  = {git = "ssh://git@github.com/i10416/firebase-messaging-rs.git", branch = "main", version = "0.4"}

# wip: firebase-messaging-rs = "0.2"

```

## Overview

This crate implements some wrapper functions for google FCM APIs and Instant ID APIs.

General features like authorization are delegated to [gcloud-sdk-rs](https://github.com/abdolence/gcloud-sdk-rs) under the hood.


## Features

You can choose tls backend from native-tls or rustls.

```toml
firebase-messaging-rs  = {git = "ssh://git@github.com/i10416/firebase-messaging-rs.git", branch = "main", version = "0.4", features = ["rustls"] }

# wip: firebase-messaging-rs = { version = "<version>", features = ["rustls"] }
```

## Example


### register token to topic and un-register token from topic

```rust
use firebase_messaging_rs::FCMClient;
use firebase_messaging_rs::topic::TopicManagementSupport;

// you need export GOOGLE_APPLICATION_CREDENTIALS env to authenticate to Firebase.
let client = FCMClient::new().await.unwrap();


let topic_name = "topic_name";
// bulk register tokens
let res = client.register_tokens_to_topic(
  topic_name.into(),
  vec![token_0,token_1,...]
).await.unwrap();

println!("{res:?}");
// => TopicManagementResponse {results: [{}, {"error": "INVALID_ARGUMENT"}, ...] }

let sts = c.get_info_by_iid_token(token_0, true).await.unwrap();
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
let res = client.unregister_token_from_topic(
  topic_name,
  token_0
).await.unwrap();
// => Ok(TopicManagementResponse { results: [{}] })


```


## License

Licensed under either of [Apache License, Version 2.0](https://github.com/abdolence/gcloud-sdk-rs/blob/master/LICENSE-APACHE) or [MIT license](https://github.com/abdolence/gcloud-sdk-rs/blob/master/LICENSE-MIT) at your option.
