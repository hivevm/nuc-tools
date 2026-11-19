//! The rule `devcontainer/settings` of the template's `scripts/check-devcontainer.sh`:
//! `.vscode/settings.json` pins the container-management extension to the host side,
//! `"ms-azuretools.vscode-containers": ["ui"]` inside the `remote.extensionKind` object
//! (NUC-container-hides-host-daemon §4). The file is read flattened, its full-line comments,
//! spaces, tabs, and newlines dropped, from the key up to the first closing brace. From release
//! v0.9.0 on only a tree with both `.devcontainer/` and `.vscode/` is held to it, since the bash
//! passes a tree without `.devcontainer/` as unbound before it looks at the settings; before, every
//! tree, with a Dev Container or not. Before release v0.8.0 the bash pipes the file through
//! `tr -d` into `grep -q`; the flattened text is one line, which grep reads to its end before it
//! can match, so the pipe is drained and the result holds on a file of any length.

use super::{Input, Inputs, Outcome, Rule};
use crate::devcontainer::{self, SETTINGS};
use crate::finding::Finding;

pub const RULE_ID: &str = "devcontainer/settings";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    reasons: &["absent", "unpinned"],
    explain: "In a tree with .devcontainer/ and .vscode/, .vscode/settings.json pins the container-management \
              extension to the host side: '\"ms-azuretools.vscode-containers\": [\"ui\"]' inside \
              the 'remote.extensionKind' object. Without it the extension runs in the container \
              and asks for the host's socket. Before template release v0.9.0, every tree is held \
              to it.",
    reads: &[Input::Devcontainer, Input::VscodeSettings, Input::Release],
    check,
};

fn finding(reason: &'static str, message: String) -> Finding {
    Finding {
        rule: RULE_ID,
        reason,
        file: SETTINGS.to_owned(),
        line: None,
        message,
    }
}

/// Whether the flattened settings pin the extension, as
/// `"remote\.extensionKind":\{[^}]*"ms-azuretools\.vscode-containers":\["ui"\]` matches them.
fn pinned(flat: &[u8]) -> bool {
    static PIN: std::sync::OnceLock<regex::bytes::Regex> = std::sync::OnceLock::new();
    PIN.get_or_init(|| {
        regex::bytes::Regex::new(
            r#"(?-u)"remote\.extensionKind":\{[^}]*"ms-azuretools\.vscode-containers":\["ui"\]"#,
        )
        .expect("a valid pattern")
    })
    .is_match(flat)
}

fn check(inputs: &Inputs) -> Outcome {
    if !devcontainer::settings_bind(inputs) {
        return Outcome::default();
    }
    let cited = devcontainer::cite(inputs, 4);
    let Some(settings) = inputs.devcontainer.settings.as_deref() else {
        return vec![finding(
            "absent",
            format!(
                "not found — it pins the container-management extension to the host side {cited}"
            ),
        )]
        .into();
    };
    let flat: Vec<u8> = devcontainer::uncommented(settings)
        .iter()
        .flat_map(|line| line.iter().copied())
        .filter(|b| !matches!(b, b' ' | b'\t'))
        .collect();
    if pinned(&flat) {
        return Outcome::default();
    }
    vec![finding(
        "unpinned",
        format!(
            "does not pin 'ms-azuretools.vscode-containers' to [\"ui\"] under \
                 'remote.extensionKind' — without it the extension runs in the container and asks \
                 for the socket {cited}"
        ),
    )]
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Version;
    use crate::devcontainer::fixture::Tree;

    fn keys(tree: &Tree) -> Vec<String> {
        check(&tree.inputs())
            .findings
            .into_iter()
            .map(|f| format!("{}: {} [{}]", f.file, f.message, f.reason))
            .collect()
    }

    const UNPINNED: &str = ".vscode/settings.json: does not pin 'ms-azuretools.vscode-containers' \
                            to [\"ui\"] under 'remote.extensionKind' — without it the extension \
                            runs in the container and asks for the socket \
                            (NUC-container-hides-host-daemon \u{a7}4) [unpinned]";

    // bash: the container-management extension not pinned to the host
    #[test]
    fn the_extension_pinned_to_the_host_passes_and_on_the_workspace_is_unpinned() {
        assert!(keys(&Tree::base()).is_empty());
        // check-devcontainer.txt: the container-management extension not pinned to the host
        let mut tree = Tree::base();
        tree.settings(r#"{ "ms-azuretools.vscode-containers": ["workspace"] }"#);
        assert_eq!(keys(&tree), [UNPINNED]);
        // Written out over lines, with another extension before it.
        tree.settings("{\n\t\"a.b\": [\"workspace\"],\n    \"ms-azuretools.vscode-containers\": [ \"ui\" ]\n  }");
        assert!(keys(&tree).is_empty());
    }

    // bash: the pair under a key other than remote.extensionKind
    #[test]
    fn the_pair_under_another_key_is_unpinned() {
        // check-devcontainer.txt: the pair under a key other than remote.extensionKind
        let mut tree = Tree::base();
        tree.files.settings = Some(
            b"{\n  \"some.other.setting\": { \"ms-azuretools.vscode-containers\": [\"ui\"] },\n  \
              \"remote.extensionKind\": { \"ms-azuretools.vscode-docker\": [\"ui\"] }\n}\n"
                .to_vec(),
        );
        assert_eq!(keys(&tree), [UNPINNED]);
        // A carriage return is no space the bash drops.
        tree.files.settings = Some(
            b"{\r\n\"remote.extensionKind\":\r\n{\"ms-azuretools.vscode-containers\":[\"ui\"]}}"
                .to_vec(),
        );
        assert_eq!(keys(&tree), [UNPINNED]);
    }

    #[test]
    fn vs_code_without_its_settings_is_absent_and_no_vs_code_binds_nothing_from_v0_9_0_on() {
        // check-devcontainer.txt: VS Code without its settings file
        let mut tree = Tree::base();
        tree.files.settings = None;
        assert_eq!(
            keys(&tree),
            [
                ".vscode/settings.json: not found — it pins the container-management extension to \
              the host side (NUC-container-hides-host-daemon \u{a7}4) [absent]"
            ]
        );
        // check-devcontainer.txt: a Dev Container without VS Code, where the host-side clause
        // binds nothing
        tree.files.vscode = false;
        assert!(keys(&tree).is_empty());
        tree.release = Some(Version(0, 8, 1));
        assert_eq!(
            keys(&tree),
            [format!(
                ".vscode/settings.json: not found — it pins the container-management extension \
                 to the host side ({}) [absent]",
                crate::testing::adr_id(4)
            )]
        );
    }

    #[test]
    fn no_dev_container_binds_no_settings_from_v0_9_0_on_and_every_tree_before() {
        // check-devcontainer.txt: a project without a Dev Container
        let mut tree = Tree::base();
        tree.files.dir = false;
        tree.files.config = None;
        tree.files.lock = None;
        tree.settings(r#"{ "ms-azuretools.vscode-containers": ["workspace"] }"#);
        tree.release = Some(Version(0, 9, 0));
        assert!(keys(&tree).is_empty());
        tree.release = Some(Version(0, 8, 1));
        assert_eq!(keys(&tree).len(), 1);
        assert!(keys(&tree)[0].ends_with("[unpinned]"));
    }

    #[test]
    fn long_settings_are_decided_on_every_release() {
        let mut tree = Tree::base();
        let pad = format!("\"pad\": \"{}\",", "z".repeat(70_000));
        tree.settings(&format!(
            "{{ \"ms-azuretools.vscode-containers\": [\"ui\"] }},\n  {pad}"
        ));
        assert!(keys(&tree).is_empty());
        // Before v0.8.0 grep reads the one flattened line to its end, so it drains the pipe.
        tree.release = Some(Version(0, 7, 0));
        assert_eq!(check(&tree.inputs()), Outcome::default());
        tree.settings(&format!("{{}},\n  {pad}"));
        assert_eq!(keys(&tree).len(), 1);
    }
}
