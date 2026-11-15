//! Stream Deck plugin entry point.

use advanced_http_plugin::state::AppState;
use streamdeck_plugin::Plugin;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    advanced_http_plugin::force_link();
    Plugin::builder(std::env::args())
        .state(AppState::default())
        .add_registered_actions()
        .run()
        .await?;
    Ok(())
}
