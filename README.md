# OAuth2 HTTP Client

A flexible Rust library providing a generic HTTP interface for OAuth2 operations. This library enables seamless integration with the `oauth2` crate by offering an async-first implementation compatible with various HTTP clients.

## Features

- ✨ **Async/await support** using `async-trait`
- 🔧 **Generic HTTP interface** - works with any HTTP client implementation
- 🛡️ **Type-safe error handling** with custom error types
- 🔗 **Full OAuth2 compatibility** with the `oauth2` crate's `AsyncHttpClient` trait
- 📦 **Zero-copy request/response handling**

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
oauth2-http-client = "0.1.0"
async-trait = "0.1"
oauth2 = { version = "5.0", default-features = false }
```

## How to use
```rust
use oauth2_http_client::{HttpInterface, OAuth2Client};
use reqwest::Client;

// Implement HttpInterface for your HTTP client
struct MyHttpClient {
    client: Client,
}

#[async_trait::async_trait]
impl HttpInterface for MyHttpClient {
    type Error = Box<dyn std::error::Error>;

    async fn perform(&self, req: oauth2::HttpRequest) -> Result<oauth2::HttpResponse, Self::Error> {
        // Your implementation here
        Ok(response)
    }
}

#[tokio::main]
async fn main() {
    let http_client = MyHttpClient { client: Client::new() };
    let oauth2_client = OAuth2Client::new(http_client);
    // Use oauth2_client with OAuth2 flows
}
```