use advanced_http_client::HttpEngine;
use streamdeck_plugin::PluginService;

/// State shared by every action instance.
///
/// The Stream Deck host owns one `AppState` and passes it to each action, so all
/// keys and dials reuse the same connection pool.
#[derive(Clone, Default)]
pub struct AppState {
    /// Shared HTTP client.
    pub http: HttpEngine,
}

impl PluginService for AppState {}
