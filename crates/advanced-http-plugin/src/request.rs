use advanced_http_client::{HttpError, RequestSpec};

use crate::contracts::ActionSettings;

/// Translate action settings into a request.
///
/// Returns `None` when there is nothing to send, which is the v1 behavior for an
/// empty URL. Legacy actions with no method fall back to GET.
pub fn build_spec(
    method: &str,
    url: &str,
    settings: &ActionSettings,
) -> Option<Result<RequestSpec, HttpError>> {
    let url = url.trim();
    if url.is_empty() {
        return None;
    }
    Some(build(method, url, settings))
}

fn build(method: &str, url: &str, settings: &ActionSettings) -> Result<RequestSpec, HttpError> {
    let method = method.trim();
    let method = if method.is_empty() { "GET" } else { method };
    let mut spec = RequestSpec::new(method, url)?.with_body(&settings.body);

    // v1 applied Basic auth when both fields were set and ignored the
    // Authorization Header field entirely. Keep Basic winning so old profiles
    // send the same request, and use the header when Basic is incomplete.
    if !settings.basic_auth_id.is_empty() && !settings.basic_auth_password.is_empty() {
        spec = spec.with_basic_auth(&settings.basic_auth_id, &settings.basic_auth_password);
    } else if !settings.auth_header.trim().is_empty() {
        spec = spec.with_authorization(settings.auth_header.trim())?;
    }

    Ok(spec)
}

#[cfg(test)]
mod tests {
    use advanced_http_client::Auth;

    use super::*;

    fn default_settings() -> ActionSettings {
        ActionSettings::default()
    }

    #[test]
    fn empty_urls_are_skipped() {
        assert!(build_spec("GET", "", &default_settings()).is_none());
        assert!(build_spec("GET", "   \n", &default_settings()).is_none());
    }

    #[test]
    fn missing_method_falls_back_to_get() {
        let spec = build_spec("", "https://example.com", &default_settings())
            .expect("spec")
            .expect("valid");
        assert_eq!(spec.method.as_str(), "GET");
        let spec = build_spec("   ", "https://example.com", &default_settings())
            .expect("spec")
            .expect("valid");
        assert_eq!(spec.method.as_str(), "GET");
    }

    #[test]
    fn urls_are_trimmed() {
        let spec = build_spec("GET", "  https://example.com/a  ", &default_settings())
            .expect("spec")
            .expect("valid");
        assert_eq!(spec.url.as_str(), "https://example.com/a");
    }

    #[test]
    fn body_is_attached_only_when_present() {
        let mut settings = default_settings();
        settings.body = "{\"a\":1}".into();
        let spec = build_spec("POST", "https://example.com", &settings)
            .expect("spec")
            .expect("valid");
        assert_eq!(spec.body.as_deref(), Some("{\"a\":1}"));

        let spec = build_spec("POST", "https://example.com", &default_settings())
            .expect("spec")
            .expect("valid");
        assert!(spec.body.is_none());
    }

    #[test]
    fn basic_auth_wins_when_both_credentials_and_a_header_are_set() {
        let mut settings = default_settings();
        settings.basic_auth_id = "user".into();
        settings.basic_auth_password = "pass".into();
        settings.auth_header = "Bearer token".into();
        let spec = build_spec("GET", "https://example.com", &settings)
            .expect("spec")
            .expect("valid");
        assert!(matches!(
            spec.auth,
            Some(Auth::Basic {
                username,
                password
            }) if username == "user" && password == "pass"
        ));
    }

    #[test]
    fn authorization_header_is_used_when_basic_auth_is_incomplete() {
        let mut settings = default_settings();
        settings.basic_auth_id = "user".into();
        settings.auth_header = "Bearer token".into();
        let spec = build_spec("GET", "https://example.com", &settings)
            .expect("spec")
            .expect("valid");
        assert!(matches!(spec.auth, Some(Auth::Header(value)) if value == "Bearer token"));
    }

    #[test]
    fn no_auth_when_everything_is_empty() {
        let spec = build_spec("GET", "https://example.com", &default_settings())
            .expect("spec")
            .expect("valid");
        assert!(spec.auth.is_none());
    }

    #[test]
    fn invalid_authorization_header_is_reported() {
        let mut settings = default_settings();
        settings.auth_header = "bad\nvalue".into();
        let result = build_spec("GET", "https://example.com", &settings).expect("spec");
        assert!(matches!(result, Err(HttpError::Authorization)));
    }

    #[test]
    fn invalid_methods_and_urls_are_reported() {
        let settings = default_settings();
        let result = build_spec("BAD METHOD", "https://example.com", &settings).expect("spec");
        assert!(matches!(result, Err(HttpError::Method(_))));

        let result = build_spec("GET", "not a url", &settings).expect("spec");
        assert!(matches!(result, Err(HttpError::Url(_))));
    }
}
