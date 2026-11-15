//! The test of the Dev Container's definition. Verifies: OPS-dev-container-carries-and-reaches
//!
//! `.devcontainer/devcontainer.json` names the template's Debian base image and the Rust Feature,
//! and neither Compose nor a Feature that brings a container engine (clauses 1 and 4). Each
//! `cargo install` it runs carries `--locked` and installs each tool at the version the deps job of
//! `.github/workflows/checks.yml` installs (clause 2). The file allows comments, so full-line `//`
//! comments are dropped and it is read as text, the way `scripts/check-devcontainer.sh` reads it;
//! the pin and the lock of every Feature, and the host socket, are that check's
//! (NUC-container-hides-host-daemon).

mod support;

use std::fs;

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

/// The line up to a comment that starts with `marker` outside a string quoted by one of `quotes`.
/// A `//` outside a string always starts one, since JSON has none; a `#` only at the start of the
/// line or after a blank, as in YAML. A backslash in a string escapes the next character.
fn without_comment<'a>(line: &'a str, marker: &str, quotes: &[char]) -> &'a str {
    let mut quote = None;
    let mut escaped = false;
    let mut previous = ' ';
    for (at, c) in line.char_indices() {
        match quote {
            Some(_) if escaped => escaped = false,
            Some(_) if c == '\\' => escaped = true,
            Some(open) if c == open => quote = None,
            Some(_) => {}
            None if quotes.contains(&c) => quote = Some(c),
            None if (marker == "//" || previous.is_whitespace())
                && line[at..].starts_with(marker) =>
            {
                return &line[..at];
            }
            None => {}
        }
        previous = c;
    }
    line
}

/// The definition without its `//` comments, whole lines and trailing ones.
fn definition_code(definition: &str) -> String {
    definition
        .lines()
        .map(|line| without_comment(line, "//", &['"']))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The deps job of the workflow, from its `  deps:` line to the next job, without `#` comments.
fn deps_job(workflow: &str) -> String {
    let mut lines = workflow
        .lines()
        .skip_while(|line| without_comment(line, "#", &['"', '\'']).trim_end() != "  deps:");
    let head = lines.next().into_iter();
    let body = lines.take_while(|line| {
        !(line.starts_with("  ") && !line.starts_with("   ") && line.trim_end().ends_with(':'))
    });
    head.chain(body)
        .map(|line| without_comment(line, "#", &['"', '\'']))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The `cargo install` commands of a text: each from `cargo install` to the end of its command,
/// at a `"`, a `&&`, a `;`, or the end of the line.
fn cargo_installs(text: &str) -> Vec<&str> {
    text.match_indices("cargo install")
        .map(|(start, _)| {
            let rest = &text[start..];
            let end = ["\"", "&&", ";", "\n"]
                .iter()
                .filter_map(|stop| rest.find(stop))
                .min()
                .unwrap_or(rest.len());
            rest[..end].trim_end()
        })
        .collect()
}

/// The options of `cargo install` that take a value as the next word; any other option takes none.
/// An option missing here shows its value as a crate at no pinned version, which fails.
const VALUED: [&str; 24] = [
    "--version",
    "--vers",
    "--git",
    "--branch",
    "--tag",
    "--rev",
    "--path",
    "--root",
    "--index",
    "--registry",
    "-F",
    "--features",
    "-j",
    "--jobs",
    "--profile",
    "--target",
    "--target-dir",
    "--config",
    "-Z",
    "--color",
    "--bin",
    "--example",
    "--message-format",
    "--lockfile-path",
];

/// The crates a `cargo install` command names, the values of its options left out.
fn crates_of(install: &str) -> Vec<&str> {
    let mut crates = Vec::new();
    let mut words = install.split_whitespace().skip(2);
    while let Some(word) = words.next() {
        if !word.starts_with('-') {
            crates.push(word);
        } else if VALUED.contains(&word) {
            words.next();
        }
    }
    crates
}

/// Whether a crate is named at one exact version: `<crate>@<major>.<minor>.<patch>`, with an
/// optional `=` before the version and an optional pre-release or build after it.
fn is_pinned(name: &str) -> bool {
    let Some((_, version)) = name.split_once('@') else {
        return false;
    };
    let version = version.strip_prefix('=').unwrap_or(version);
    let core = version.split(['-', '+']).next().unwrap_or_default();
    let parts: Vec<&str> = core.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
}

/// What breaks clause 2 in the `cargo install` commands of one side, and the crates it names,
/// sorted.
fn cargo_side<'a>(side: &str, text: &'a str, found: &mut Vec<String>) -> Vec<&'a str> {
    let mut all = Vec::new();
    for install in cargo_installs(text) {
        if !install.split_whitespace().any(|word| word == "--locked") {
            found.push(format!("{side}: '{install}' does not carry --locked"));
        }
        for name in crates_of(install) {
            if !is_pinned(name) {
                found.push(format!(
                    "{side}: '{name}' is installed at no pinned version"
                ));
            }
            all.push(name);
        }
    }
    all.sort_unstable();
    all
}

/// What breaks clause 2 for the cargo tools, one message each; empty when the container and the
/// deps job install the same tools, each with `--locked` at one exact version.
fn cargo_violations(definition: &str, workflow: &str) -> Vec<String> {
    let (code, deps) = (definition_code(definition), deps_job(workflow));
    let mut found = Vec::new();
    if deps.is_empty() {
        found.push("the workflow has no deps job".to_owned());
    }
    let container = cargo_side("the container", &code, &mut found);
    let ci = cargo_side("the deps job", &deps, &mut found);
    if container.is_empty() {
        found.push("the container installs no cargo tool".to_owned());
    }
    if container != ci {
        found.push(format!(
            "the container installs {container:?}, the deps job {ci:?}"
        ));
    }
    found
}

/// A workflow whose deps job installs the cargo tools the fixture names.
const WORKFLOW: &str =
    "  deps:\n    steps:\n      - run: cargo install --locked cargo-deny@1.0.0 cargo-audit@2.0.0\n";

/// The `postCreateCommand` entry of a fixture that installs the cargo tools as `WORKFLOW` does.
const CARGO_TOOLS: &str = "  \"postCreateCommand\": { \"cargo-tools\": \"cargo install --locked cargo-deny@1.0.0 cargo-audit@2.0.0\" },";

/// A definition that keeps the decision, with `extra` added before the Features.
fn fixture(extra: &str) -> String {
    format!(
        "{{\n  // A comment naming docker-in-docker is no Feature.\n  \"image\": \"{IMAGE}\",\n{extra}\n  \"features\": {{\n    \"ghcr.io/devcontainers/features/github-cli:1\": {{}},\n    \"{RUST_FEATURE}\": {{}}\n  }}\n}}\n"
    )
}

#[test]
fn the_definition_keeps_the_decision() {
    let path = support::sources::root().join(".devcontainer/devcontainer.json");
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

#[test]
fn the_container_installs_the_cargo_tools_of_ci() {
    let root = support::sources::root();
    let definition = fs::read_to_string(root.join(".devcontainer/devcontainer.json"))
        .expect(".devcontainer/devcontainer.json is readable");
    let workflow = fs::read_to_string(root.join(".github/workflows/checks.yml"))
        .expect(".github/workflows/checks.yml is readable");
    let found = cargo_violations(&definition, &workflow);
    assert!(
        found.is_empty(),
        "the Dev Container's cargo tools differ from CI's:\n{}",
        found.join("\n")
    );
}

#[test]
fn the_cargo_tools_of_the_workflow_pass() {
    assert_eq!(
        cargo_violations(&fixture(CARGO_TOOLS), WORKFLOW),
        Vec::<String>::new()
    );
}

#[test]
fn a_cargo_tool_without_locked_fails() {
    let definition = fixture(&CARGO_TOOLS.replace("--locked ", ""));
    assert_eq!(
        cargo_violations(&definition, WORKFLOW),
        [
            "the container: 'cargo install cargo-deny@1.0.0 cargo-audit@2.0.0' does not carry --locked"
        ]
    );
}

#[test]
fn a_cargo_tool_at_another_version_fails() {
    let definition = fixture(&CARGO_TOOLS.replace("cargo-audit@2.0.0", "cargo-audit@2.0.1"));
    assert_eq!(
        cargo_violations(&definition, WORKFLOW),
        [
            "the container installs [\"cargo-audit@2.0.1\", \"cargo-deny@1.0.0\"], the deps job [\"cargo-audit@2.0.0\", \"cargo-deny@1.0.0\"]"
        ]
    );
}

#[test]
fn a_cargo_tool_without_a_version_fails() {
    let definition = fixture(&CARGO_TOOLS.replace("cargo-audit@2.0.0", "cargo-audit"));
    let found = cargo_violations(&definition, WORKFLOW);
    assert!(
        found
            .contains(&"the container: 'cargo-audit' is installed at no pinned version".to_owned()),
        "{found:?}"
    );
}

#[test]
fn a_cargo_tool_ci_does_not_install_fails() {
    let definition =
        fixture(&CARGO_TOOLS.replace("cargo-audit@2.0.0", "cargo-audit@2.0.0 cargo-nextest@0.9.0"));
    assert_eq!(
        cargo_violations(&definition, WORKFLOW),
        [
            "the container installs [\"cargo-audit@2.0.0\", \"cargo-deny@1.0.0\", \"cargo-nextest@0.9.0\"], the deps job [\"cargo-audit@2.0.0\", \"cargo-deny@1.0.0\"]"
        ]
    );
}

#[test]
fn no_cargo_tool_fails() {
    assert_eq!(
        cargo_violations(&fixture(""), WORKFLOW),
        [
            "the container installs no cargo tool",
            "the container installs [], the deps job [\"cargo-audit@2.0.0\", \"cargo-deny@1.0.0\"]"
        ]
    );
}

#[test]
fn the_deps_job_without_locked_fails() {
    let workflow = WORKFLOW.replace("--locked ", "");
    assert_eq!(
        cargo_violations(&fixture(CARGO_TOOLS), &workflow),
        [
            "the deps job: 'cargo install cargo-deny@1.0.0 cargo-audit@2.0.0' does not carry --locked"
        ]
    );
}

#[test]
fn a_version_range_is_not_pinned() {
    let definition = fixture(&CARGO_TOOLS.replace("cargo-deny@1.0.0", "cargo-deny@^1.0"));
    let workflow = WORKFLOW.replace("cargo-deny@1.0.0", "cargo-deny@^1.0");
    assert_eq!(
        cargo_violations(&definition, &workflow),
        [
            "the container: 'cargo-deny@^1.0' is installed at no pinned version",
            "the deps job: 'cargo-deny@^1.0' is installed at no pinned version"
        ]
    );
}

#[test]
fn an_exact_version_with_an_equals_sign_is_pinned() {
    let definition = fixture(&CARGO_TOOLS.replace("cargo-deny@1.0.0", "cargo-deny@=1.0.0"));
    let workflow = WORKFLOW.replace("cargo-deny@1.0.0", "cargo-deny@=1.0.0");
    assert_eq!(
        cargo_violations(&definition, &workflow),
        Vec::<String>::new()
    );
}

#[test]
fn a_cargo_install_in_a_comment_of_the_definition_is_no_install() {
    let definition = fixture(
        "  \"postCreateCommand\": { \"cargo-tools\": \"cargo binstall cargo-deny@9.0.0\" }, // cargo install --locked cargo-deny@1.0.0 cargo-audit@2.0.0",
    );
    assert_eq!(
        cargo_violations(&definition, WORKFLOW),
        [
            "the container installs no cargo tool",
            "the container installs [], the deps job [\"cargo-audit@2.0.0\", \"cargo-deny@1.0.0\"]"
        ]
    );
}

#[test]
fn a_cargo_install_in_a_comment_of_the_deps_job_is_no_install() {
    let workflow = "  deps:\n    steps:\n      - uses: example/install@v1 # was: cargo install --locked cargo-deny@1.0.0 cargo-audit@2.0.0\n";
    assert_eq!(
        cargo_violations(&fixture(CARGO_TOOLS), workflow),
        ["the container installs [\"cargo-audit@2.0.0\", \"cargo-deny@1.0.0\"], the deps job []"]
    );
}

#[test]
fn a_cargo_install_of_another_job_is_not_the_deps_job() {
    let workflow = format!(
        "  build:\n    steps:\n      - run: cargo install --locked cargo-audit@2.0.0\n{}  later:\n    steps:\n      - run: cargo install other@1.0.0\n",
        WORKFLOW.replace(" cargo-audit@2.0.0", "")
    );
    assert_eq!(
        cargo_violations(&fixture(CARGO_TOOLS), &workflow),
        [
            "the container installs [\"cargo-audit@2.0.0\", \"cargo-deny@1.0.0\"], the deps job [\"cargo-deny@1.0.0\"]"
        ]
    );
}

#[test]
fn the_value_of_an_option_is_no_crate() {
    let definition = fixture(&CARGO_TOOLS.replace("--locked", "--locked --root /opt/cargo"));
    assert_eq!(
        cargo_violations(&definition, WORKFLOW),
        Vec::<String>::new()
    );
}

#[test]
fn an_option_without_a_value_hides_no_crate() {
    let definition = fixture(&CARGO_TOOLS.replace(
        "--locked cargo-deny@1.0.0",
        "--locked --frozen cargo-deny@9.9.9",
    ));
    let workflow = WORKFLOW.replace("--locked cargo-deny", "--locked --frozen cargo-deny");
    assert_eq!(
        cargo_violations(&definition, &workflow),
        [
            "the container installs [\"cargo-audit@2.0.0\", \"cargo-deny@9.9.9\"], the deps job [\"cargo-audit@2.0.0\", \"cargo-deny@1.0.0\"]"
        ]
    );
}

#[test]
fn a_comment_right_after_a_string_is_no_install() {
    let definition = fixture(
        "  \"postCreateCommand\": { \"cargo-tools\": \"echo \\\"hi\\\" && cargo binstall cargo-deny@9.0.0\" },// cargo install --locked cargo-deny@1.0.0 cargo-audit@2.0.0",
    );
    assert_eq!(
        cargo_violations(&definition, WORKFLOW),
        [
            "the container installs no cargo tool",
            "the container installs [], the deps job [\"cargo-audit@2.0.0\", \"cargo-deny@1.0.0\"]"
        ]
    );
}

#[test]
fn a_workflow_without_a_deps_job_fails() {
    let workflow = WORKFLOW.replace("  deps:", "  audit:");
    assert_eq!(
        cargo_violations(&fixture(CARGO_TOOLS), &workflow),
        [
            "the workflow has no deps job",
            "the container installs [\"cargo-audit@2.0.0\", \"cargo-deny@1.0.0\"], the deps job []"
        ]
    );
}

#[test]
fn a_deps_job_with_a_comment_on_its_line_is_read() {
    let workflow = WORKFLOW.replace("  deps:", "  deps: # the audit");
    assert_eq!(
        cargo_violations(&fixture(CARGO_TOOLS), &workflow),
        Vec::<String>::new()
    );
}
