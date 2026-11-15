use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use streamdeck_plugin::{export_ts, TS};

/// Settings stored on every action.
///
/// Field names stay snake_case so profiles saved by the v1 Property Inspector
/// keep working. Every field is read leniently: a malformed value falls back to
/// its v1 default instead of discarding the rest of the settings.
#[derive(Clone, Debug, PartialEq, Serialize, TS)]
pub struct ActionSettings {
    /// Legacy actions only. New per-method actions ignore it.
    pub method: Option<String>,
    /// URL called when a key is pressed.
    pub url_press: String,
    /// URL called when a key or dial is released.
    pub url_release: String,
    /// URL called when a dial turns left.
    pub url_left: String,
    /// URL called when a dial turns right.
    pub url_right: String,
    /// URL called when a dial is pressed.
    pub url_push: String,
    /// URL called when the touch strip is tapped.
    pub url_touch: String,
    /// Request body.
    pub body: String,
    /// HTTP Basic user name.
    pub basic_auth_id: String,
    /// HTTP Basic password.
    pub basic_auth_password: String,
    /// Verbatim `Authorization` header value.
    pub auth_header: String,
    /// Show the OK notification after a request.
    pub show_ok: bool,
    /// Show the warning notification when a request fails.
    pub show_alert: bool,
}

impl Default for ActionSettings {
    fn default() -> Self {
        Self {
            method: None,
            url_press: String::new(),
            url_release: String::new(),
            url_left: String::new(),
            url_right: String::new(),
            url_push: String::new(),
            url_touch: String::new(),
            body: String::new(),
            basic_auth_id: String::new(),
            basic_auth_password: String::new(),
            auth_header: String::new(),
            show_ok: true,
            show_alert: true,
        }
    }
}

impl ActionSettings {
    fn from_value(value: &Value) -> Self {
        let Value::Object(map) = value else {
            return Self::default();
        };
        Self {
            method: read_text(map.get("method")),
            url_press: read_text(map.get("url_press")).unwrap_or_default(),
            url_release: read_text(map.get("url_release")).unwrap_or_default(),
            url_left: read_text(map.get("url_left")).unwrap_or_default(),
            url_right: read_text(map.get("url_right")).unwrap_or_default(),
            url_push: read_text(map.get("url_push")).unwrap_or_default(),
            url_touch: read_text(map.get("url_touch")).unwrap_or_default(),
            body: read_text(map.get("body")).unwrap_or_default(),
            basic_auth_id: read_text(map.get("basic_auth_id")).unwrap_or_default(),
            basic_auth_password: read_text(map.get("basic_auth_password")).unwrap_or_default(),
            auth_header: read_text(map.get("auth_header")).unwrap_or_default(),
            show_ok: read_bool(map.get("show_ok")).unwrap_or(true),
            show_alert: read_bool(map.get("show_alert")).unwrap_or(true),
        }
    }
}

impl<'de> Deserialize<'de> for ActionSettings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Ok(Self::from_value(&value))
    }
}

export_ts!(ActionSettings);

fn read_text(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(text)) => Some(text.clone()),
        Some(Value::Null) | None => None,
        Some(other) => Some(other.to_string()),
    }
}

fn read_bool(value: Option<&Value>) -> Option<bool> {
    match value {
        Some(Value::Bool(value)) => Some(*value),
        Some(Value::String(text)) => match text.trim().to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" => Some(true),
            "false" | "0" | "no" => Some(false),
            _ => None,
        },
        Some(Value::Number(number)) => number.as_i64().map(|value| value != 0),
        Some(Value::Null) | None => None,
        Some(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_v1_key_settings() {
        let settings: ActionSettings = serde_json::from_str(
            r#"{
                "method": "GET",
                "url_press": "https://example.com/press",
                "url_release": "",
                "body": "",
                "basic_auth_id": "user",
                "basic_auth_password": "pass",
                "auth_header": "Bearer token",
                "show_ok": true,
                "show_alert": false
            }"#,
        )
        .expect("legacy settings");

        assert_eq!(settings.method.as_deref(), Some("GET"));
        assert_eq!(settings.url_press, "https://example.com/press");
        assert_eq!(settings.basic_auth_id, "user");
        assert_eq!(settings.basic_auth_password, "pass");
        assert_eq!(settings.auth_header, "Bearer token");
        assert!(settings.show_ok);
        assert!(!settings.show_alert);
    }

    #[test]
    fn parses_v1_dial_settings() {
        let settings: ActionSettings = serde_json::from_str(
            r#"{
                "method": "POST",
                "url_left": "https://example.com/left",
                "url_right": "https://example.com/right",
                "url_push": "https://example.com/push",
                "url_release": "https://example.com/release",
                "url_touch": "https://example.com/touch",
                "body": "{}",
                "show_ok": true,
                "show_alert": true
            }"#,
        )
        .expect("legacy settings");

        assert_eq!(settings.method.as_deref(), Some("POST"));
        assert_eq!(settings.url_left, "https://example.com/left");
        assert_eq!(settings.url_right, "https://example.com/right");
        assert_eq!(settings.url_push, "https://example.com/push");
        assert_eq!(settings.url_release, "https://example.com/release");
        assert_eq!(settings.url_touch, "https://example.com/touch");
        assert_eq!(settings.body, "{}");
    }

    #[test]
    fn missing_fields_use_v1_defaults() {
        let settings: ActionSettings = serde_json::from_str(r#"{"method":""}"#).expect("settings");
        assert_eq!(settings.method.as_deref(), Some(""));
        assert_eq!(settings.url_press, "");
        assert_eq!(settings.body, "");
        assert!(settings.show_ok);
        assert!(settings.show_alert);
    }

    #[test]
    fn booleans_accept_strings_and_numbers() {
        let settings: ActionSettings = serde_json::from_str(
            r#"{"show_ok":"false","show_alert":0,"url_press":null,"method":null}"#,
        )
        .expect("settings");
        assert!(!settings.show_ok);
        assert!(!settings.show_alert);
        assert_eq!(settings.url_press, "");
        assert_eq!(settings.method, None);
    }

    #[test]
    fn bodies_keep_whitespace() {
        let settings: ActionSettings =
            serde_json::from_str(r#"{"body":"  { \"a\": 1 }  "}"#).expect("settings");
        assert_eq!(settings.body, "  { \"a\": 1 }  ");
    }

    #[test]
    fn non_object_settings_fall_back_to_defaults() {
        for text in ["null", "[]", "\"nope\"", "42"] {
            let settings: ActionSettings = serde_json::from_str(text).expect("settings");
            assert_eq!(settings, ActionSettings::default(), "{text}");
        }
    }
}
