/// HTTP methods exposed as Stream Deck actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Head,
    Options,
    Query,
    Post,
    Put,
    Delete,
    Patch,
}

impl HttpMethod {
    /// Every method, in the order the Property Inspector lists them.
    pub const ALL: [Self; 8] = [
        Self::Get,
        Self::Head,
        Self::Options,
        Self::Query,
        Self::Post,
        Self::Put,
        Self::Delete,
        Self::Patch,
    ];

    /// Wire name, uppercase.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
            Self::Query => "QUERY",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
        }
    }

    /// Parse a user- or action-supplied method name. Case-insensitive.
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_uppercase().as_str() {
            "GET" => Some(Self::Get),
            "HEAD" => Some(Self::Head),
            "OPTIONS" => Some(Self::Options),
            "QUERY" => Some(Self::Query),
            "POST" => Some(Self::Post),
            "PUT" => Some(Self::Put),
            "DELETE" => Some(Self::Delete),
            "PATCH" => Some(Self::Patch),
            _ => None,
        }
    }

    /// Whether the Property Inspector should show a BODY field for this method.
    ///
    /// DELETE and QUERY may carry a body, so only the bodiless methods are
    /// excluded. Custom legacy methods keep the body field.
    pub const fn has_body(self) -> bool {
        !matches!(self, Self::Get | Self::Head | Self::Options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_method_round_trips_through_its_wire_name() {
        for method in HttpMethod::ALL {
            assert_eq!(HttpMethod::parse(method.as_str()), Some(method));
        }
    }

    #[test]
    fn parsing_is_case_insensitive_and_trims() {
        assert_eq!(HttpMethod::parse(" get "), Some(HttpMethod::Get));
        assert_eq!(HttpMethod::parse("Query"), Some(HttpMethod::Query));
        assert_eq!(HttpMethod::parse(""), None);
        assert_eq!(HttpMethod::parse("BREW"), None);
    }

    #[test]
    fn only_bodiless_methods_hide_the_body_field() {
        assert!(!HttpMethod::Get.has_body());
        assert!(!HttpMethod::Head.has_body());
        assert!(!HttpMethod::Options.has_body());
        assert!(HttpMethod::Query.has_body());
        assert!(HttpMethod::Post.has_body());
        assert!(HttpMethod::Put.has_body());
        assert!(HttpMethod::Delete.has_body());
        assert!(HttpMethod::Patch.has_body());
    }
}
