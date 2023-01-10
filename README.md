## firebase-messaging-rs

## Install

Cargo.toml

```toml
firebase-messaging-rs  = {git = "ssh://git@github.com/i10416/firebase-messaging-rs.git", branch = "main", version = "0.2.0"}

# wip: firebase-messaging-rs = "0.2"

```

## Overview

This crate implements some wrapper functions for google FCM APIs and Instant ID APIs.

General features like authorization are delegated to [gcloud-sdk-rs](https://github.com/abdolence/gcloud-sdk-rs) under the hood.


## Features

You can choose tls backend from native-tls or rustls.

```toml
firebase-messaging-rs  = {git = "ssh://git@github.com/i10416/firebase-messaging-rs.git", branch = "main", version = "0.2.0", features = ["rustls"] }

# wip: firebase-messaging-rs = { version = "<version>", features = ["rustls"] }
```

## Example

```rust
let client = FCMClient::new().await.unwrap();

let res = client.register_tokens_to_topic(
  "topic_name".into(),
  vec![token_0,token_1,...]
).await.unwrap();

println!("{:?}",res);
// => TopicManagementResponse {results: [{}, {"error": "INVALID_ARGUMENT"}, ...] }
```

## License

Licensed under either of [Apache License, Version 2.0](https://github.com/abdolence/gcloud-sdk-rs/blob/master/LICENSE-APACHE) or [MIT license](https://github.com/abdolence/gcloud-sdk-rs/blob/master/LICENSE-MIT) at your option.
