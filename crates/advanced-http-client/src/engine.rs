use std::time::Duration;

use futures::StreamExt;
use reqwest::header::AUTHORIZATION;
use reqwest::{Client, StatusCode};

use crate::{Auth, HttpError, RequestSpec};

/// Result of a completed HTTP exchange.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Outcome {
    /// Response status. Any response, including 4xx and 5xx, is an outcome.
    pub status: StatusCode,
}

/// Shared HTTP client used by every action.
#[derive(Clone)]
pub struct HttpEngine {
    client: Client,
}

impl HttpEngine {
    /// Requests are abandoned after this long.
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

    /// Build an engine with [`Self::DEFAULT_TIMEOUT`].
    pub fn new() -> Result<Self, HttpError> {
        Self::with_timeout(Self::DEFAULT_TIMEOUT)
    }

    /// Build an engine with an explicit timeout. Tests use short values.
    pub fn with_timeout(timeout: Duration) -> Result<Self, HttpError> {
        let client = Client::builder()
            .timeout(timeout)
            .user_agent(concat!("advanced-http/", env!("CARGO_PKG_VERSION")))
            .build()?;
        Ok(Self { client })
    }

    /// Send one request and discard the response body.
    ///
    /// The body is drained as a stream so a large response does not have to fit
    /// in memory. The status is returned even when it is an error status.
    pub async fn execute(&self, spec: &RequestSpec) -> Result<Outcome, HttpError> {
        let mut request = self.client.request(spec.method.clone(), spec.url.clone());
        match &spec.auth {
            Some(Auth::Basic { username, password }) => {
                request = request.basic_auth(username, Some(password));
            }
            Some(Auth::Header(value)) => {
                request = request.header(AUTHORIZATION, value);
            }
            None => {}
        }
        if let Some(body) = &spec.body {
            request = request.body(body.clone());
        }

        let response = request.send().await?;
        let status = response.status();
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            chunk?;
        }
        Ok(Outcome { status })
    }
}

impl Default for HttpEngine {
    fn default() -> Self {
        Self::new().expect("failed to build the HTTP client")
    }
}
