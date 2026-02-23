#[cfg(test)]
mod test;

use std::pin::Pin;

use oauth2::{AsyncHttpClient, HttpRequest, HttpResponse};

#[async_trait::async_trait]
pub trait HttpInterface {
    type Error: std::fmt::Debug + Send + Sync + 'static;

    async fn perform(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error>;
}

pub struct OAuth2Client<HI>
where
    HI: HttpInterface + Clone + Send + Sync + 'static,
{
    interface: HI,
}

impl<HI> OAuth2Client<HI>
where
    HI: HttpInterface + Clone + Send + Sync + 'static,
{
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

    fn call(&'c self, request: HttpRequest) -> Self::Future {
        let interface = self.interface.clone();
        Box::pin(async move {
            let result = interface.perform(request).await?;
            Ok(result)
        })
    }
}
