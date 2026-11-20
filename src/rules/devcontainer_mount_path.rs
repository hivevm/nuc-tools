//! The rule `devcontainer/mount-path` of the template's `scripts/check-devcontainer.sh` from
//! release v0.7.0 on: `workspaceFolder` and the target of `workspaceMount` of
//! `.devcontainer/devcontainer.json`, where the file sets them, are
//! `/workspaces/${localWorkspaceFolderBasename}`, the default of Dev Containers
//! (NUC-container-hides-host-daemon §5). Each value is the text from its top-level key to the next
//! one; the target is one of the mount's comma-separated options, under any of its names.

use super::{Input, Inputs, Outcome, Rule};
use crate::devcontainer::{self, CONFIG, MOUNT_PATH};
use crate::finding::Finding;

pub const RULE_ID: &str = "devcontainer/mount-path";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    reasons: &["folder", "mount"],
    explain: "workspaceFolder and the target of workspaceMount in .devcontainer/devcontainer.json, \
              where the file sets them, are '/workspaces/${localWorkspaceFolderBasename}', the \
              default of Dev Containers: a path every project shares gives every project one agent \
              memory. From template release v0.7.0 on.",
    reads: &[Input::Devcontainer, Input::Release],
    check,
};

/// The path each project is mounted under, as devcontainer.json writes it.
const OWN: &str = "/workspaces/${localWorkspaceFolderBasename}";

fn finding(reason: &'static str, message: String) -> Finding {
    Finding {
        rule: RULE_ID,
        reason,
        file: CONFIG.to_owned(),
        line: None,
        message,
    }
}

/// Whether the mount, its quotes and POSIX blanks and newlines dropped, targets the own path: it
/// is empty, or one of its options split at commas, without one leading colon, is
/// `target=`, `dst=`, or `destination=` and the path.
fn mount_own(mount: &[u8]) -> bool {
    mount.is_empty()
        || mount.split(|&b| b == b',').any(|option| {
            let option = option.strip_prefix(b":").unwrap_or(option);
            ["target=", "dst=", "destination="]
                .iter()
                .any(|name| option == format!("{name}{OWN}").as_bytes())
        })
}

fn check(inputs: &Inputs) -> Outcome {
    let Some(config) = inputs.devcontainer.config.as_deref() else {
        return Outcome::default();
    };
    if !devcontainer::binds(inputs) || !inputs.carries_at_least(MOUNT_PATH) {
        return Outcome::default();
    }
    let cited = devcontainer::cite(inputs, 5);
    let mut findings = Vec::new();
    let value = devcontainer::top_level_value(config, b"workspaceFolder");
    let folder = crate::bash::substituted(&value);
    if !folder.is_empty() && !crate::bash::contains(folder, format!("\"{OWN}\"").as_bytes()) {
        findings.push(finding(
            "folder",
            format!(
                "workspaceFolder is not '{OWN}' — a path every project shares gives every project \
                 one agent memory {cited}"
            ),
        ));
    }
    let mount: Vec<u8> = devcontainer::top_level_value(config, b"workspaceMount")
        .into_iter()
        .filter(|b| !matches!(b, b'"' | b' ' | b'\t' | b'\n'))
        .collect();
    if !mount_own(&mount) {
        findings.push(finding(
            "mount",
            format!(
                "workspaceMount does not target '{OWN}' — a path every project shares gives every \
                 project one agent memory {cited}"
            ),
        ));
    }
    findings.into()
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

    const OWN_FOLDER: &str = r#""workspaceFolder": "/workspaces/${localWorkspaceFolderBasename}","#;

    // bash: a workspaceFolder every project shares
    #[test]
    fn a_workspace_folder_every_project_shares_is_reported() {
        // check-devcontainer.txt: a workspaceFolder every project shares
        let mut tree = Tree::base();
        assert!(keys(&tree).is_empty());
        tree.extra(r#""workspaceFolder": "/workspace","#);
        assert_eq!(
            keys(&tree),
            [".devcontainer/devcontainer.json: workspaceFolder is not \
              '/workspaces/${localWorkspaceFolderBasename}' — a path every project shares gives \
              every project one agent memory (NUC-container-hides-host-daemon \u{a7}5) [folder]"]
        );
    }

    // bash: a project mounted under a path of its own
    // bash: a mount whose target comes last, under another of its names
    #[test]
    fn a_project_mounted_under_a_path_of_its_own_passes() {
        // check-devcontainer.txt: a project mounted under a path of its own
        let mut tree = Tree::base();
        tree.extra(OWN_FOLDER).extra(
            r#""workspaceMount": "source=${localWorkspaceFolder},target=/workspaces/${localWorkspaceFolderBasename},type=bind","#,
        );
        assert!(keys(&tree).is_empty());
        // check-devcontainer.txt: a mount whose target comes last, under another of its names
        let mut tree = Tree::base();
        tree.extra(
            r#""workspaceMount": "source=${localWorkspaceFolder},type=bind,dst=/workspaces/${localWorkspaceFolderBasename}","#,
        );
        assert!(keys(&tree).is_empty());
    }

    // bash: a workspaceMount that targets a path every project shares
    #[test]
    fn a_workspace_mount_that_targets_a_shared_path_is_reported() {
        // check-devcontainer.txt: a workspaceMount that targets a path every project shares
        let mut tree = Tree::base();
        tree.extra(OWN_FOLDER).extra(
            r#""workspaceMount": "source=${localWorkspaceFolder},target=/workspace,type=bind","#,
        );
        assert_eq!(
            keys(&tree),
            [
                ".devcontainer/devcontainer.json: workspaceMount does not target \
              '/workspaces/${localWorkspaceFolderBasename}' — a path every project shares gives \
              every project one agent memory (NUC-container-hides-host-daemon \u{a7}5) [mount]"
            ]
        );
        // The target first, after the key's colon, holds.
        assert!(mount_own(
            b":target=/workspaces/${localWorkspaceFolderBasename},type=bind,"
        ));
        // The last key of the object keeps the closing brace on its last option.
        assert!(!mount_own(
            b":type=bind,target=/workspaces/${localWorkspaceFolderBasename}}"
        ));
    }

    #[test]
    fn a_release_before_v0_7_0_has_no_such_check_and_one_before_v0_9_0_cites_adr_0004() {
        let mut tree = Tree::base();
        tree.extra(r#""workspaceFolder": "/workspace","#);
        tree.release = Some(Version(0, 6, 2));
        assert!(keys(&tree).is_empty());
        tree.release = Some(Version(0, 7, 0));
        assert_eq!(
            keys(&tree),
            [format!(
                ".devcontainer/devcontainer.json: workspaceFolder is not \
                 '/workspaces/${{localWorkspaceFolderBasename}}' — a path every project shares \
                 gives every project one agent memory ({}) [folder]",
                crate::testing::adr_id(4)
            )]
        );
    }
}
