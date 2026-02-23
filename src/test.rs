use std::{fmt, vec};

use oauth2::{
    ClientId, ClientSecret, DeviceAuthorizationUrl, HttpRequest, HttpResponse, Scope,
    StandardDeviceAuthorizationResponse, TokenUrl,
    basic::BasicClient,
    http::{self, Response},
};
use reqwest::Client;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers};

use crate::{HttpInterface, OAuth2Client};

#[derive(Clone)]
enum HttpClient {
    Reqwest(Client),
}

#[derive(Debug)]
struct MockError(String);

impl fmt::Display for MockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MockError: {}", self.0)
    }
}

impl From<reqwest::Error> for MockError {
    fn from(err: reqwest::Error) -> Self {
        MockError(format!("Reqwest error: {}", err))
    }
}

impl From<std::io::Error> for MockError {
    fn from(err: std::io::Error) -> Self {
        MockError(format!("IO error: {}", err))
    }
}

impl From<http::Error> for MockError {
    fn from(err: http::Error) -> Self {
        MockError(format!("HTTP error: {}", err))
    }
}

impl std::error::Error for MockError {}

#[async_trait::async_trait]
impl HttpInterface for HttpClient {
    type Error = MockError;

    async fn perform(&self, request: HttpRequest) -> Result<HttpResponse, Self::Error> {
        match &self {
            HttpClient::Reqwest(client) => {
                // Build the Reqwest request
                let mut req_builder = client
                    .request(request.method().clone(), request.uri().to_string())
                    .body(request.body().clone());

                // Copy headers
                for (name, value) in request.headers().iter() {
                    req_builder = req_builder.header(name, value);
                }

                // Send request
                let resp = req_builder.send().await?;

                // Extract parts for oauth2::HttpResponse
                let status = resp.status();
                let headers = resp.headers().clone();
                let body = resp.bytes().await?.to_vec();

                // Build http::Response
                let mut builder = Response::builder().status(status);

                // Insert headers
                for (name, value) in headers.iter() {
                    builder = builder.header(name, value);
                }

                Ok(builder.body(body)?)
            }
        }
    }
}

#[tokio::test]
async fn test_oauth2_client() {
    // Start WireMock server
    let mock_server = MockServer::start().await;

    // Mock device authorization endpoint
    Mock::given(matchers::method("POST"))
        .and(matchers::path("/device"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "device_code": "test-device-code",
            "user_code": "TEST-1234",
            "verification_uri": "https://localhost:8080/verify",
            "expires_in": 1800,
            "interval": 5
        })))
        .mount(&mock_server)
        .await;

    // Mock token endpoint
    Mock::given(matchers::method("POST"))
        .and(matchers::path("/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "test-access-token",
            "token_type": "Bearer",
            "expires_in": 3600
        })))
        .mount(&mock_server)
        .await;

    let async_http_callback = OAuth2Client::new(HttpClient::Reqwest(Client::new()));
    let client = BasicClient::new(ClientId::new("test-client-id".into()));

    let device_auth_response: StandardDeviceAuthorizationResponse = client
        .set_client_secret(ClientSecret::new("Client-secret".into()))
        .set_auth_type(oauth2::AuthType::RequestBody)
        .set_token_uri(TokenUrl::new(format!("{}/token", mock_server.uri())).unwrap())
        .set_device_authorization_url(
            DeviceAuthorizationUrl::new(format!("{}/device", mock_server.uri())).unwrap(),
        )
        .exchange_device_code()
        .add_scopes(vec![
            Scope::new("scope1".into()),
            Scope::new("scope2".into()),
        ])
        .request_async(&async_http_callback)
        .await
        .unwrap();

    assert_eq!(
        device_auth_response.device_code().secret(),
        "test-device-code"
    );
    assert_eq!(device_auth_response.user_code().secret(), "TEST-1234");
    assert_eq!(
        device_auth_response.verification_uri().as_str(),
        "https://localhost:8080/verify"
    );
    assert_eq!(device_auth_response.expires_in().as_secs(), 1800);
    assert_eq!(device_auth_response.interval().as_secs(), 5);
}
