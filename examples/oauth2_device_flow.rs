use std::fmt;

use oauth2::{
    ClientId, ClientSecret, DeviceAuthorizationUrl, HttpRequest, HttpResponse, Scope,
    StandardDeviceAuthorizationResponse, TokenUrl,
    basic::BasicClient,
    http::{self, Response},
};
use reqwest::Client;

use oauth2_http_client::{HttpInterface, OAuth2Client};

#[derive(Clone)]
enum HttpClient {
    Reqwest(Client),
    // Placeholder for other HTTP client implementations (e.g., Hyper, Surf, etc.)
    // You can add the HTTP client you want to use, curl, hyper, surf, etc.
}

#[derive(Debug)]
struct ExampleError(String);

impl fmt::Display for ExampleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ExampleError: {}", self.0)
    }
}

impl From<reqwest::Error> for ExampleError {
    fn from(err: reqwest::Error) -> Self {
        ExampleError(format!("Reqwest error: {}", err))
    }
}

impl From<http::Error> for ExampleError {
    fn from(err: http::Error) -> Self {
        ExampleError(format!("HTTP error: {}", err))
    }
}

impl std::error::Error for ExampleError {}

#[async_trait::async_trait]
impl HttpInterface for HttpClient {
    type Error = ExampleError;

    async fn perform(&self, request: HttpRequest) -> Result<HttpResponse, Self::Error> {
        match &self {
            HttpClient::Reqwest(client) => {
                let mut req_builder = client
                    .request(request.method().clone(), request.uri().to_string())
                    .body(request.body().clone());

                for (name, value) in request.headers().iter() {
                    req_builder = req_builder.header(name, value);
                }

                let resp = req_builder.send().await?;
                let status = resp.status();
                let headers = resp.headers().clone();
                let body = resp.bytes().await?.to_vec();

                let mut builder = Response::builder().status(status);
                for (name, value) in headers.iter() {
                    builder = builder.header(name, value);
                }

                Ok(builder.body(body)?)
            }
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let async_http_callback = OAuth2Client::new(HttpClient::Reqwest(Client::new()));
    let client = BasicClient::new(ClientId::new("test-client-id".into()));

    let device_auth_response: StandardDeviceAuthorizationResponse = client
        .set_client_secret(ClientSecret::new("Client-secret".into()))
        .set_auth_type(oauth2::AuthType::RequestBody)
        .set_token_uri(TokenUrl::new("https://localhost:8080/token".into()).unwrap())
        .set_device_authorization_url(
            DeviceAuthorizationUrl::new("https://localhost:8080/device".into()).unwrap(),
        )
        .exchange_device_code()
        .add_scopes(vec![
            Scope::new("scope1".into()),
            Scope::new("scope2".into()),
        ])
        .request_async(&async_http_callback)
        .await
        .unwrap();

    println!(
        "Device Code: {}",
        device_auth_response.device_code().secret()
    );
    println!("User Code: {}", device_auth_response.user_code().secret());
    println!(
        "Verification URI: {}",
        device_auth_response.verification_uri()
    );
    println!(
        "Expires In: {} seconds",
        device_auth_response.expires_in().as_secs()
    );
    println!(
        "Interval: {} seconds",
        device_auth_response.interval().as_secs()
    );
    Ok(())
}
