//! Stream Deck actions, one per HTTP method plus the v1 legacy actions.

use advanced_http_client::{HttpError, HttpMethod, Outcome, RequestSpec};
use streamdeck_plugin::{
    streamdeck_action, ActionContext, ActionPayload, DialRotatePayload, EncoderAction,
    KeypadAction, Result, TouchTapPayload,
};

use crate::contracts::ActionSettings;
use crate::request::build_spec;
use crate::state::AppState;

/// Which controller an action instance lives on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControllerKind {
    /// Keypad key.
    Keypad,
    /// Stream Deck + dial / touch strip.
    Encoder,
}

impl ControllerKind {
    /// Lowercase name used by the generated Property Inspector metadata.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Keypad => "keypad",
            Self::Encoder => "encoder",
        }
    }
}

/// One action UUID and the method/controller the inspector should show.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionMeta {
    /// Manifest action UUID.
    pub uuid: &'static str,
    /// Fixed method, or `None` for a legacy action with a METHOD selector.
    pub method: Option<HttpMethod>,
    /// Controller family.
    pub controller: ControllerKind,
    /// Whether this is the v1 "any method" action.
    pub legacy: bool,
}

/// Legacy keypad action UUID. Kept registered for existing profiles.
pub const LEGACY_KEY_UUID: &str = "dev.flowingspdg.advancedhttp.perform";
/// Legacy dial action UUID. Kept registered for existing profiles.
pub const LEGACY_DIAL_UUID: &str = "dev.flowingspdg.advancedhttp.dial";

macro_rules! define_method_actions {
    ($(($key:ident, $dial:ident, $key_uuid:literal, $dial_uuid:literal, $method:ident)),+ $(,)?) => {
        $(
            #[derive(Default)]
            pub struct $key;

            #[streamdeck_action(uuid = $key_uuid, settings = ActionSettings, state = AppState)]
            impl KeypadAction for $key {
                async fn on_key_down(
                    &mut self,
                    _payload: &ActionPayload,
                    ctx: &ActionContext<'_, ActionSettings, AppState>,
                ) -> Result<()> {
                    let url = ctx.settings().url_press;
                    execute(ctx, HttpMethod::$method.as_str(), &url).await
                }

                async fn on_key_up(
                    &mut self,
                    _payload: &ActionPayload,
                    ctx: &ActionContext<'_, ActionSettings, AppState>,
                ) -> Result<()> {
                    let url = ctx.settings().url_release;
                    execute(ctx, HttpMethod::$method.as_str(), &url).await
                }
            }

            #[derive(Default)]
            pub struct $dial;

            #[streamdeck_action(uuid = $dial_uuid, settings = ActionSettings, state = AppState)]
            impl EncoderAction for $dial {
                async fn on_dial_rotate(
                    &mut self,
                    payload: &DialRotatePayload,
                    ctx: &ActionContext<'_, ActionSettings, AppState>,
                ) -> Result<()> {
                    let settings = ctx.settings();
                    let url = if payload.ticks >= 0 {
                        settings.url_right
                    } else {
                        settings.url_left
                    };
                    rotate(ctx, HttpMethod::$method.as_str(), &url, payload.ticks).await
                }

                async fn on_dial_down(
                    &mut self,
                    _payload: &ActionPayload,
                    ctx: &ActionContext<'_, ActionSettings, AppState>,
                ) -> Result<()> {
                    let url = ctx.settings().url_push;
                    execute(ctx, HttpMethod::$method.as_str(), &url).await
                }

                async fn on_dial_up(
                    &mut self,
                    _payload: &ActionPayload,
                    ctx: &ActionContext<'_, ActionSettings, AppState>,
                ) -> Result<()> {
                    let url = ctx.settings().url_release;
                    execute(ctx, HttpMethod::$method.as_str(), &url).await
                }

                async fn on_touch_tap(
                    &mut self,
                    _payload: &TouchTapPayload,
                    ctx: &ActionContext<'_, ActionSettings, AppState>,
                ) -> Result<()> {
                    let url = ctx.settings().url_touch;
                    execute(ctx, HttpMethod::$method.as_str(), &url).await
                }
            }
        )+

        fn force_link_methods() {
            $( let _ = std::any::type_name::<$key>(); let _ = std::any::type_name::<$dial>(); )+
        }

        /// Every per-method action, in key/dial pairs.
        pub const METHOD_ACTIONS: &[ActionMeta] = &[
            $(
                ActionMeta {
                    uuid: $key_uuid,
                    method: Some(HttpMethod::$method),
                    controller: ControllerKind::Keypad,
                    legacy: false,
                },
                ActionMeta {
                    uuid: $dial_uuid,
                    method: Some(HttpMethod::$method),
                    controller: ControllerKind::Encoder,
                    legacy: false,
                },
            )+
        ];
    };
}

define_method_actions!(
    (
        GetKeyAction,
        GetDialAction,
        "dev.flowingspdg.advancedhttp.get",
        "dev.flowingspdg.advancedhttp.dial.get",
        Get
    ),
    (
        PostKeyAction,
        PostDialAction,
        "dev.flowingspdg.advancedhttp.post",
        "dev.flowingspdg.advancedhttp.dial.post",
        Post
    ),
    (
        PutKeyAction,
        PutDialAction,
        "dev.flowingspdg.advancedhttp.put",
        "dev.flowingspdg.advancedhttp.dial.put",
        Put
    ),
    (
        PatchKeyAction,
        PatchDialAction,
        "dev.flowingspdg.advancedhttp.patch",
        "dev.flowingspdg.advancedhttp.dial.patch",
        Patch
    ),
    (
        DeleteKeyAction,
        DeleteDialAction,
        "dev.flowingspdg.advancedhttp.delete",
        "dev.flowingspdg.advancedhttp.dial.delete",
        Delete
    ),
    (
        HeadKeyAction,
        HeadDialAction,
        "dev.flowingspdg.advancedhttp.head",
        "dev.flowingspdg.advancedhttp.dial.head",
        Head
    ),
    (
        OptionsKeyAction,
        OptionsDialAction,
        "dev.flowingspdg.advancedhttp.options",
        "dev.flowingspdg.advancedhttp.dial.options",
        Options
    ),
    (
        QueryKeyAction,
        QueryDialAction,
        "dev.flowingspdg.advancedhttp.query",
        "dev.flowingspdg.advancedhttp.dial.query",
        Query
    ),
);

/// v1 key action. Reads the method from settings.
#[derive(Default)]
pub struct LegacyKeyAction;

#[streamdeck_action(
    uuid = "dev.flowingspdg.advancedhttp.perform",
    settings = ActionSettings,
    state = AppState
)]
impl KeypadAction for LegacyKeyAction {
    async fn on_key_down(
        &mut self,
        _payload: &ActionPayload,
        ctx: &ActionContext<'_, ActionSettings, AppState>,
    ) -> Result<()> {
        let settings = ctx.settings();
        let method = settings.method.clone().unwrap_or_default();
        execute(ctx, &method, &settings.url_press).await
    }

    async fn on_key_up(
        &mut self,
        _payload: &ActionPayload,
        ctx: &ActionContext<'_, ActionSettings, AppState>,
    ) -> Result<()> {
        let settings = ctx.settings();
        let method = settings.method.clone().unwrap_or_default();
        execute(ctx, &method, &settings.url_release).await
    }
}

/// v1 dial action. Reads the method from settings.
#[derive(Default)]
pub struct LegacyDialAction;

#[streamdeck_action(
    uuid = "dev.flowingspdg.advancedhttp.dial",
    settings = ActionSettings,
    state = AppState
)]
impl EncoderAction for LegacyDialAction {
    async fn on_dial_rotate(
        &mut self,
        payload: &DialRotatePayload,
        ctx: &ActionContext<'_, ActionSettings, AppState>,
    ) -> Result<()> {
        let settings = ctx.settings();
        let method = settings.method.clone().unwrap_or_default();
        let url = if payload.ticks >= 0 {
            settings.url_right
        } else {
            settings.url_left
        };
        rotate(ctx, &method, &url, payload.ticks).await
    }

    async fn on_dial_down(
        &mut self,
        _payload: &ActionPayload,
        ctx: &ActionContext<'_, ActionSettings, AppState>,
    ) -> Result<()> {
        let settings = ctx.settings();
        let method = settings.method.clone().unwrap_or_default();
        execute(ctx, &method, &settings.url_push).await
    }

    async fn on_dial_up(
        &mut self,
        _payload: &ActionPayload,
        ctx: &ActionContext<'_, ActionSettings, AppState>,
    ) -> Result<()> {
        let settings = ctx.settings();
        let method = settings.method.clone().unwrap_or_default();
        execute(ctx, &method, &settings.url_release).await
    }

    async fn on_touch_tap(
        &mut self,
        _payload: &TouchTapPayload,
        ctx: &ActionContext<'_, ActionSettings, AppState>,
    ) -> Result<()> {
        let settings = ctx.settings();
        let method = settings.method.clone().unwrap_or_default();
        execute(ctx, &method, &settings.url_touch).await
    }
}

/// Every action UUID, including the two legacy actions.
pub fn all_actions() -> Vec<ActionMeta> {
    let mut actions = METHOD_ACTIONS.to_vec();
    actions.push(ActionMeta {
        uuid: LEGACY_KEY_UUID,
        method: None,
        controller: ControllerKind::Keypad,
        legacy: true,
    });
    actions.push(ActionMeta {
        uuid: LEGACY_DIAL_UUID,
        method: None,
        controller: ControllerKind::Encoder,
        legacy: true,
    });
    actions
}

/// Keep action modules linked so `inventory` registrations are not stripped.
pub fn force_link() {
    force_link_methods();
    let _ = std::any::type_name::<LegacyKeyAction>();
    let _ = std::any::type_name::<LegacyDialAction>();
}

async fn execute(
    ctx: &ActionContext<'_, ActionSettings, AppState>,
    method: &str,
    url: &str,
) -> Result<()> {
    let settings = ctx.settings();
    let Some(spec) = build_spec(method, url, &settings) else {
        return Ok(());
    };
    let spec = match spec {
        Ok(spec) => spec,
        Err(error) => {
            tracing::warn!(%error, "could not build an HTTP request");
            if settings.show_alert {
                ctx.show_alert()?;
            }
            return Ok(());
        }
    };
    let outcome = ctx.state().http.execute(&spec).await;
    notify(ctx, &settings, &spec, outcome)
}

fn notify(
    ctx: &ActionContext<'_, ActionSettings, AppState>,
    settings: &ActionSettings,
    spec: &RequestSpec,
    outcome: std::result::Result<Outcome, HttpError>,
) -> Result<()> {
    match outcome {
        Ok(outcome) => {
            tracing::info!(
                method = %spec.method,
                url = %spec.url,
                status = outcome.status.as_u16(),
                "HTTP request completed"
            );
            if settings.show_ok {
                ctx.show_ok()?;
            }
        }
        Err(error) => {
            tracing::warn!(
                method = %spec.method,
                url = %spec.url,
                %error,
                "HTTP request failed"
            );
            if settings.show_alert {
                ctx.show_alert()?;
            }
        }
    }
    Ok(())
}

async fn rotate(
    ctx: &ActionContext<'_, ActionSettings, AppState>,
    method: &str,
    url: &str,
    ticks: i32,
) -> Result<()> {
    let settings = ctx.settings();
    let Some(spec) = build_spec(method, url, &settings) else {
        return Ok(());
    };
    let spec = match spec {
        Ok(spec) => spec,
        Err(error) => {
            tracing::warn!(%error, "could not build an HTTP request");
            if settings.show_alert {
                ctx.show_alert()?;
            }
            return Ok(());
        }
    };
    let count = ticks.unsigned_abs();
    if count == 0 {
        return Ok(());
    }

    let engine = &ctx.state().http;
    let results = futures::future::join_all((0..count).map(|_| engine.execute(&spec))).await;
    let mut failures = 0usize;
    for result in results {
        if let Err(error) = result {
            failures += 1;
            tracing::debug!(
                method = %spec.method,
                url = %spec.url,
                %error,
                "HTTP dial request failed"
            );
        }
    }

    if failures == 0 {
        tracing::info!(
            method = %spec.method,
            url = %spec.url,
            requests = count,
            "HTTP dial requests completed"
        );
        if settings.show_ok {
            ctx.show_ok()?;
        }
    } else {
        tracing::warn!(
            method = %spec.method,
            url = %spec.url,
            requests = count,
            failures,
            "HTTP dial requests failed"
        );
        if settings.show_alert {
            ctx.show_alert()?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_eight_methods_and_two_legacy_actions() {
        let actions = all_actions();
        assert_eq!(actions.len(), 18);
        assert_eq!(
            actions
                .iter()
                .filter(|action| action.controller == ControllerKind::Keypad)
                .count(),
            9
        );
        assert_eq!(
            actions
                .iter()
                .filter(|action| action.controller == ControllerKind::Encoder)
                .count(),
            9
        );
        assert_eq!(actions.iter().filter(|action| action.legacy).count(), 2);
    }

    #[test]
    fn every_uuid_is_unique_and_prefixed() {
        let mut uuids: Vec<&str> = all_actions().iter().map(|action| action.uuid).collect();
        uuids.sort_unstable();
        uuids.dedup();
        assert_eq!(uuids.len(), 18);
        for uuid in uuids {
            assert!(uuid.starts_with("dev.flowingspdg.advancedhttp."), "{uuid}");
        }
    }

    #[test]
    fn every_method_has_one_key_and_one_dial_action() {
        for method in HttpMethod::ALL {
            assert_eq!(
                METHOD_ACTIONS
                    .iter()
                    .filter(|action| action.method == Some(method))
                    .count(),
                2,
                "{method:?}"
            );
        }
    }

    #[test]
    fn legacy_actions_are_present() {
        let actions = all_actions();
        assert!(actions.iter().any(|action| action.uuid == LEGACY_KEY_UUID
            && action.controller == ControllerKind::Keypad
            && action.legacy));
        assert!(actions.iter().any(|action| action.uuid == LEGACY_DIAL_UUID
            && action.controller == ControllerKind::Encoder
            && action.legacy));
    }
}
