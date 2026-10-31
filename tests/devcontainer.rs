//! The test of the Dev Container's definition. Verifies: ADR-draft-dev-container-with-rust
//!
//! `.devcontainer/devcontainer.json` names the template's Debian base image and the Rust Feature,
//! and neither Compose nor a Feature that brings a container engine. The file allows comments, so
//! full-line `//` comments are dropped and it is read as text, the way
//! `scripts/check-devcontainer.sh` reads it; the pin and the lock of every Feature, and the host
//! socket, are that check's (ADR-0004).

use std::fs;
use std::path::Path;

const IMAGE: &str = "mcr.microsoft.com/devcontainers/base:debian";
const RUST_FEATURE: &str = "ghcr.io/devcontainers/features/rust:1";

/// Parts of a Feature id that bring a container engine into the container or reach the host's.
const ENGINE_FEATURES: [&str; 3] = ["docker-in-docker", "docker-outside-of-docker", "podman"];

/// The definition without its full-line `//` comments.
fn uncommented(definition: &str) -> String {
    definition
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The string value of the first `"<key>": "<value>"` in the text, if it has one.
fn string_value<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let quoted = format!("\"{key}\"");
    let after = text[text.find(&quoted)? + quoted.len()..].trim_start();
    let value = after.strip_prefix(':')?.trim_start().strip_prefix('"')?;
    value.find('"').map(|end| &value[..end])
}

/// What breaks the decision, one message each; empty when the definition keeps it.
fn violations(definition: &str) -> Vec<String> {
    let text = uncommented(definition);
    let mut found = Vec::new();
    match string_value(&text, "image") {
        Some(IMAGE) => {}
        Some(other) => found.push(format!("the image is '{other}', not '{IMAGE}'")),
        None => found.push(format!("no image is named; the decision keeps '{IMAGE}'")),
    }
    if !text.contains(&format!("\"{RUST_FEATURE}\"")) {
        found.push(format!("the Feature '{RUST_FEATURE}' is not added"));
    }
    if text.contains("\"dockerComposeFile\"") {
        found.push("a 'dockerComposeFile' brings Compose services".to_owned());
    }
    for engine in ENGINE_FEATURES {
        if text.contains(&format!("/{engine}")) {
            found.push(format!("a Feature '{engine}' brings a container engine"));
        }
    }
    found
}

/// A definition that keeps the decision, with `extra` added before the Features.
fn fixture(extra: &str) -> String {
    format!(
        "{{\n  // A comment naming docker-in-docker is no Feature.\n  \"image\": \"{IMAGE}\",\n{extra}\n  \"features\": {{\n    \"ghcr.io/devcontainers/features/github-cli:1\": {{}},\n    \"{RUST_FEATURE}\": {{}}\n  }}\n}}\n"
    )
}

#[test]
fn the_definition_keeps_the_decision() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(".devcontainer/devcontainer.json");
    let definition = fs::read_to_string(path).expect(".devcontainer/devcontainer.json is readable");
    let found = violations(&definition);
    assert!(
        found.is_empty(),
        "the Dev Container breaks its decision:\n{}",
        found.join("\n")
    );
}

#[test]
fn the_fixture_passes() {
    assert_eq!(violations(&fixture("")), Vec::<String>::new());
}

#[test]
fn another_image_fails() {
    let definition = fixture("").replace(IMAGE, "mcr.microsoft.com/devcontainers/rust:1");
    assert_eq!(
        violations(&definition),
        [
            "the image is 'mcr.microsoft.com/devcontainers/rust:1', not 'mcr.microsoft.com/devcontainers/base:debian'"
        ]
    );
}

#[test]
fn no_image_fails() {
    let definition = fixture("").replace(
        &format!("\"image\": \"{IMAGE}\","),
        "\"build\": { \"dockerfile\": \"Dockerfile\" },",
    );
    assert_eq!(violations(&definition).len(), 1);
}

#[test]
fn a_commented_image_is_no_image() {
    let definition = fixture("").replace("  \"image\"", "  // \"image\"");
    assert_eq!(violations(&definition).len(), 1);
}

#[test]
fn without_the_rust_feature_fails() {
    let definition = fixture("").replace(RUST_FEATURE, "ghcr.io/devcontainers/features/node:1");
    assert_eq!(
        violations(&definition),
        [format!("the Feature '{RUST_FEATURE}' is not added")]
    );
}

#[test]
fn compose_fails() {
    let definition = fixture("  \"dockerComposeFile\": \"compose.yaml\",");
    assert_eq!(
        violations(&definition),
        ["a 'dockerComposeFile' brings Compose services"]
    );
}

#[test]
fn an_engine_feature_fails() {
    for engine in ENGINE_FEATURES {
        let definition = fixture(&format!(
            "  \"features\": {{ \"ghcr.io/devcontainers/features/{engine}:2\": {{}} }},"
        ));
        assert_eq!(
            violations(&definition),
            [format!("a Feature '{engine}' brings a container engine")]
        );
    }
}
