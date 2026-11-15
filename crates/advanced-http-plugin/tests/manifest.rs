use std::path::PathBuf;

use serde_json::Value;

const PLUGIN_ID: &str = "dev.flowingspdg.advancedhttp";

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../plugin/dev.flowingspdg.advancedhttp.sdPlugin/manifest.json")
}

fn manifest() -> Value {
    let text = std::fs::read_to_string(manifest_path()).expect("read manifest.json");
    serde_json::from_str(&text).expect("parse manifest.json")
}

fn manifest_uuids(manifest: &Value) -> Vec<String> {
    manifest["Actions"]
        .as_array()
        .expect("Actions array")
        .iter()
        .map(|action| {
            action["UUID"]
                .as_str()
                .expect("action UUID")
                .to_ascii_lowercase()
        })
        .collect()
}

#[test]
fn manifest_matches_the_action_catalog() {
    advanced_http_plugin::force_link();
    let manifest = manifest();

    assert_eq!(manifest["UUID"].as_str(), Some(PLUGIN_ID));
    let expected_version = format!("{}.0", env!("CARGO_PKG_VERSION"));
    assert_eq!(
        manifest["Version"].as_str(),
        Some(expected_version.as_str()),
        "manifest Version must match the Cargo version with a fourth component"
    );

    let actions = manifest["Actions"].as_array().expect("Actions array");
    let declared = manifest_uuids(&manifest);
    let catalog = advanced_http_plugin::actions::all_actions();

    assert_eq!(catalog.len(), 18);
    assert_eq!(declared.len(), catalog.len());
    assert_eq!(
        declared
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        catalog.len(),
        "manifest UUIDs must be unique"
    );

    for action in &catalog {
        assert!(
            declared.iter().any(|uuid| uuid.as_str() == action.uuid),
            "manifest is missing {}",
            action.uuid
        );
    }
    for uuid in &declared {
        assert!(
            catalog.iter().any(|action| action.uuid == uuid.as_str()),
            "catalog is missing {uuid}"
        );
    }

    for action in actions {
        let uuid = action["UUID"].as_str().expect("action UUID");
        assert!(
            uuid.starts_with("dev.flowingspdg.advancedhttp."),
            "{uuid} is not under the plugin UUID"
        );
        assert!(
            action["Name"].as_str().is_some_and(|name| !name.is_empty()),
            "{uuid} has no Name"
        );
        assert!(action["Icon"].as_str().is_some(), "{uuid} has no Icon");
        assert!(
            action["States"]
                .as_array()
                .is_some_and(|states| !states.is_empty()),
            "{uuid} has no States"
        );
        assert!(
            action["States"]
                .as_array()
                .expect("States")
                .iter()
                .all(|state| state["Image"].as_str().is_some()),
            "{uuid} has a state without an Image"
        );
    }

    for uuid in [
        "dev.flowingspdg.advancedhttp.perform",
        "dev.flowingspdg.advancedhttp.dial",
    ] {
        assert!(
            declared.iter().any(|declared| declared.as_str() == uuid),
            "legacy action {uuid} must stay registered"
        );
    }

    let plugin_dir = manifest_path()
        .parent()
        .expect("plugin directory")
        .to_path_buf();
    assert!(plugin_dir.join("images/icon.png").is_file());
    assert!(plugin_dir.join("images/icon@2x.png").is_file());
}

#[test]
fn registry_matches_the_action_catalog() {
    advanced_http_plugin::force_link();

    let mut registry = streamdeck_plugin::Registry::new();
    registry.add_registered();
    let mut registered: Vec<&str> = registry.uuids();
    registered.sort_unstable();

    let mut catalog: Vec<&str> = advanced_http_plugin::actions::all_actions()
        .iter()
        .map(|action| action.uuid)
        .collect();
    catalog.sort_unstable();

    assert_eq!(registered, catalog);
}
