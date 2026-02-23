//! OAuth2 HTTP Client Library
//!
//! This library provides a flexible HTTP interface for OAuth2 operations.
//! It defines traits and implementations for handling HTTP requests and responses
//! in an async-first manner, compatible with the `oauth2` crate.
//!
//! # Features
//!
//! - Async/await support using `async-trait`
//! - Generic HTTP interface implementation
//! - Type-safe error handling
//! - Compatible with `oauth2` crate's `AsyncHttpClient` trait
//!
//! # Example
//!
//! ```ignore
//! use oauth2_http_client::{HttpInterface, OAuth2Client};
//! use reqwest::Client;
//!
//! // Create an HTTP interface implementation
//! let http_client = MyHttpImpl::new(Client::new());
//!
//! // Wrap it with OAuth2Client
//! let oauth2_client = OAuth2Client::new(http_client);
//! ```

#[cfg(test)]
mod test;

use std::pin::Pin;

use oauth2::{AsyncHttpClient, HttpRequest, HttpResponse};

/// Trait for implementing HTTP request handling.
///
/// This trait defines the interface for performing HTTP requests asynchronously.
/// Implementations should handle the conversion of `HttpRequest` to `HttpResponse`,
/// managing network communication, error handling, and response parsing.
///
/// # Associated Types
///
/// * `Error` - The error type returned by the `perform` method.
///   Must implement `Debug`, `Send`, `Sync`, and have a `'static` lifetime.
///
/// # Example
///
/// ```ignore
/// impl HttpInterface for MyHttpClient {
///     type Error = MyError;
///
///     async fn perform(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
///         // Implementation using reqwest, hyper, etc.
///         Ok(response)
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait HttpInterface {
    /// The error type returned when a request fails.
    type Error: std::fmt::Debug + Send + Sync + 'static;

    /// Performs an HTTP request asynchronously.
    ///
    /// # Arguments
    ///
    /// * `req` - The HTTP request to perform
    ///
    /// # Returns
    ///
    /// Returns `Ok(HttpResponse)` on success, or `Err(Self::Error)` on failure.
    async fn perform(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error>;
}

/// OAuth2 HTTP client wrapper that implements the `oauth2` crate's `AsyncHttpClient` trait.
///
/// This struct wraps any implementation of `HttpInterface` and provides compatibility
/// with the `oauth2` crate's async HTTP client interface. It enables seamless integration
/// with OAuth2 authorization flows.
///
/// # Type Parameters
///
/// * `HI` - The HTTP interface implementation type
///
/// # Generic Constraints
///
/// * `HI` must implement `HttpInterface`, `Clone`, `Send`, and `Sync`
/// * `HI` must outlive the entire application (`'static`)
///
/// # Example
///
/// ```ignore
/// use oauth2_http_client::OAuth2Client;
///
/// let client = OAuth2Client::new(my_http_interface);
/// ```
pub struct OAuth2Client<HI>
where
    HI: HttpInterface + Clone + Send + Sync + 'static,
{
    /// The underlying HTTP interface implementation
    interface: HI,
}

impl<HI> OAuth2Client<HI>
where
    HI: HttpInterface + Clone + Send + Sync + 'static,
{
    /// Creates a new `OAuth2Client` wrapping the provided HTTP interface.
    ///
    /// # Arguments
    ///
    /// * `interface` - The HTTP interface implementation to wrap
    ///
    /// # Returns
    ///
    /// A new `OAuth2Client` instance
    ///
    /// # Example
    ///
    /// ```ignore
    /// let oauth2_client = OAuth2Client::new(my_http_client);
    /// ```
    pub fn new(interface: HI) -> Self {
        Self { interface }
    }
}

impl<'c, HI> AsyncHttpClient<'c> for OAuth2Client<HI>
where
    HI: HttpInterface + Clone + Send + Sync + 'static,
    HI::Error: std::error::Error,
{
    type Error = HI::Error;

    type Future = Pin<Box<dyn Future<Output = Result<HttpResponse, Self::Error>> + Send + 'c>>;

    /// Calls the underlying HTTP interface to perform a request.
    ///
    /// This method implements the `AsyncHttpClient` trait from the `oauth2` crate,
    /// allowing this client to be used with OAuth2 authorization flows.
    ///
    /// # Arguments
    ///
    /// * `request` - The HTTP request to perform
    ///
    /// # Returns
    ///
    /// A pinned boxed future that resolves to the HTTP response or an error
    fn call(&'c self, request: HttpRequest) -> Self::Future {
        let interface = self.interface.clone();
        Box::pin(async move {
            let result = interface.perform(request).await?;
            Ok(result)
        })
    }
}
