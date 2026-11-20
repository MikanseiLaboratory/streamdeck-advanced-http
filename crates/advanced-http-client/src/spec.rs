use reqwest::header::HeaderValue;
use reqwest::{Method, Url};

use crate::HttpError;

/// Authentication applied to a request.
#[derive(Clone, Debug)]
pub enum Auth {
    /// HTTP Basic authentication.
    Basic {
        /// User name.
        username: String,
        /// Password.
        password: String,
    },
    /// A verbatim `Authorization` header value.
    Header(HeaderValue),
}

/// A validated request, ready for [`crate::HttpEngine::execute`].
#[derive(Clone, Debug)]
pub struct RequestSpec {
    /// Parsed HTTP method.
    pub method: Method,
    /// Parsed URL.
    pub url: Url,
    /// Optional request body.
    pub body: Option<String>,
    /// Optional authentication.
    pub auth: Option<Auth>,
}

impl RequestSpec {
    /// Parse `method` and `url`, rejecting values the HTTP client cannot send.
    pub fn new(method: &str, url: &str) -> Result<Self, HttpError> {
        let method = method.trim().to_ascii_uppercase();
        if method.is_empty() {
            return Err(HttpError::Method(method));
        }
        let method =
            Method::from_bytes(method.as_bytes()).map_err(|_| HttpError::Method(method))?;
        let url = Url::parse(url.trim()).map_err(|error| HttpError::Url(error.to_string()))?;
        Ok(Self {
            method,
            url,
            body: None,
            auth: None,
        })
    }

    /// Attach a body when it is not empty.
    #[must_use]
    pub fn with_body(mut self, body: &str) -> Self {
        if !body.is_empty() {
            self.body = Some(body.to_string());
        }
        self
    }

    /// Attach HTTP Basic authentication.
    #[must_use]
    pub fn with_basic_auth(mut self, username: &str, password: &str) -> Self {
        self.auth = Some(Auth::Basic {
            username: username.to_string(),
            password: password.to_string(),
        });
        self
    }

    /// Attach a verbatim `Authorization` header value.
    pub fn with_authorization(mut self, value: &str) -> Result<Self, HttpError> {
        let value = HeaderValue::try_from(value.trim()).map_err(|_| HttpError::Authorization)?;
        self.auth = Some(Auth::Header(value));
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_and_parses_method_and_url() {
        let spec = RequestSpec::new(" post ", " https://example.com/path?q=1 ").unwrap();
        assert_eq!(spec.method, Method::POST);
        assert_eq!(spec.url.as_str(), "https://example.com/path?q=1");
    }

    #[test]
    fn accepts_custom_method_tokens() {
        let spec = RequestSpec::new("QUERY", "https://example.com").unwrap();
        assert_eq!(spec.method.as_str(), "QUERY");
    }

    #[test]
    fn rejects_empty_or_invalid_methods() {
        assert!(matches!(
            RequestSpec::new("", "https://example.com"),
            Err(HttpError::Method(_))
        ));
        assert!(matches!(
            RequestSpec::new("BAD METHOD", "https://example.com"),
            Err(HttpError::Method(_))
        ));
    }

    #[test]
    fn rejects_invalid_urls() {
        assert!(matches!(
            RequestSpec::new("GET", "not a url"),
            Err(HttpError::Url(_))
        ));
    }

    #[test]
    fn empty_bodies_are_omitted() {
        let spec = RequestSpec::new("POST", "https://example.com")
            .unwrap()
            .with_body("");
        assert!(spec.body.is_none());
    }

    #[test]
    fn non_empty_bodies_are_kept_verbatim() {
        let body = "{\n  \"a\": 1\n}";
        let spec = RequestSpec::new("POST", "https://example.com")
            .unwrap()
            .with_body(body);
        assert_eq!(spec.body.as_deref(), Some(body));
    }

    #[test]
    fn authorization_rejects_invalid_header_bytes() {
        let spec = RequestSpec::new("GET", "https://example.com").unwrap();
        assert!(matches!(
            spec.with_authorization("bad\nvalue"),
            Err(HttpError::Authorization)
        ));
    }
}
