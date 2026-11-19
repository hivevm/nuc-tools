//! The rule `devcontainer/socket` of the template's `scripts/check-devcontainer.sh`: the Dev
//! Container mounts no host Docker socket, read as `docker.sock` or `/var/run/docker` on a line of
//! `.devcontainer/devcontainer.json` that is no full-line comment, and adds no Feature whose id
//! names `docker-outside-of-docker` (NUC-container-hides-host-daemon §2). Before release v0.8.0
//! the bash pipes the file into `grep -q`, which may cut the pipe off on a file longer than a pipe
//! holds; such a file with a match is not decided.

use super::{Input, Inputs, Outcome, Rule};
use crate::devcontainer::{self, CONFIG};
use crate::finding::Finding;

pub const RULE_ID: &str = "devcontainer/socket";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    reasons: &["mount", "feature"],
    explain: "The Dev Container never reaches the host's container engine: \
              .devcontainer/devcontainer.json names no 'docker.sock' and no '/var/run/docker' on \
              a line that is no full-line comment, and adds no Feature whose id names \
              'docker-outside-of-docker', the one that mounts the socket. An engine inside the \
              container, such as Docker-in-Docker, is the project's choice. Manage host \
              containers from the host-side extension instead.",
    reads: &[Input::Devcontainer, Input::Release],
    check,
};

fn finding(reason: &'static str, message: String) -> Finding {
    Finding {
        rule: RULE_ID,
        reason,
        file: CONFIG.to_owned(),
        line: None,
        message,
    }
}

fn check(inputs: &Inputs) -> Outcome {
    let Some(config) = inputs.devcontainer.config.as_deref() else {
        return Outcome::default();
    };
    if !devcontainer::binds(inputs) {
        return Outcome::default();
    }
    let mut findings = Vec::new();
    let lines = devcontainer::uncommented(config);
    let mounts = lines.iter().any(|line| {
        crate::bash::contains(line, b"docker.sock")
            || crate::bash::contains(line, b"/var/run/docker")
    });
    if mounts {
        if let Some(refusal) = devcontainer::cut_off(inputs, CONFIG, &lines) {
            return Outcome {
                refusal: Some(refusal),
                ..Outcome::default()
            };
        }
        // The citation stands inside the message, as the bash writes it.
        let cited = devcontainer::cite(inputs, 2);
        findings.push(finding(
            "mount",
            format!(
                "mounts the host Docker socket — the Dev Container never reaches the host engine \
                 {cited}; manage host containers from the host-side extension instead"
            ),
        ));
    }
    for id in devcontainer::feature_ids(config) {
        if crate::bash::contains(&id, b"docker-outside-of-docker") {
            findings.push(finding(
                "feature",
                format!(
                    "Feature '{}' would mount the host Docker socket — it is not added {}",
                    devcontainer::spelled(&id),
                    devcontainer::cite(inputs, 2)
                ),
            ));
        }
    }
    findings.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Version;
    use crate::devcontainer::fixture::{NUC, Tree};

    fn keys(tree: &Tree) -> Vec<String> {
        check(&tree.inputs())
            .findings
            .into_iter()
            .map(|f| format!("{}: {} [{}]", f.file, f.message, f.reason))
            .collect()
    }

    const MOUNT: &str =
        "\"mounts\": [\"source=/var/run/docker.sock,target=/var/run/docker.sock,type=bind\"],";

    // bash: a host socket mount
    // bash: a host socket passed as a run argument
    #[test]
    fn a_host_socket_mount_is_reported_once() {
        // check-devcontainer.txt: a host socket mount
        let mut tree = Tree::base();
        assert!(keys(&tree).is_empty());
        tree.extra(MOUNT);
        assert_eq!(
            keys(&tree),
            [
                ".devcontainer/devcontainer.json: mounts the host Docker socket — the Dev Container \
              never reaches the host engine (NUC-container-hides-host-daemon \u{a7}2); manage \
              host containers from the host-side extension instead [mount]"
            ]
        );
        // check-devcontainer.txt: a host socket passed as a run argument
        let mut tree = Tree::base();
        tree.extra(r#""runArgs": ["-v", "/var/run/docker.sock:/var/run/docker.sock"],"#);
        assert_eq!(keys(&tree).len(), 1);
        // A socket only in a full-line comment is none.
        let mut tree = Tree::base();
        tree.extra("// \"mounts\": [\"/var/run/docker.sock\"]");
        assert!(keys(&tree).is_empty());
    }

    // bash: a Feature that mounts the socket
    // bash: an engine inside the container talks to no host engine and is the project's choice
    #[test]
    fn a_feature_that_mounts_the_socket_is_reported_and_one_inside_the_container_is_not() {
        // check-devcontainer.txt: a Feature that mounts the socket
        let mut tree = Tree::base();
        tree.feature = "ghcr.io/devcontainers/features/docker-outside-of-docker:1".to_owned();
        tree.config();
        assert_eq!(
            keys(&tree),
            [".devcontainer/devcontainer.json: Feature \
              'ghcr.io/devcontainers/features/docker-outside-of-docker:1' would mount the host \
              Docker socket — it is not added (NUC-container-hides-host-daemon \u{a7}2) \
              [feature]"]
        );
        // check-devcontainer.txt: an engine inside the container talks to no host engine and is
        // the project's choice
        tree.feature = "ghcr.io/devcontainers/features/docker-in-docker:2".to_owned();
        tree.config();
        assert!(keys(&tree).is_empty());
        assert_eq!(tree.nuc.as_deref(), Some(NUC));
    }

    // bash: a features block on one line, with a string-valued Feature, is read like any other
    #[test]
    fn a_features_block_on_one_line_is_read_like_any_other() {
        let mut tree = Tree::base();
        tree.files.config = Some(
            br#"{ "image": "x", "features": { "ghcr.io/devcontainers/features/docker-outside-of-docker:1": {}, "ghcr.io/devcontainers/features/node:1": "lts" } }"#
                .to_vec(),
        );
        assert_eq!(keys(&tree).len(), 1);
        assert!(keys(&tree)[0].ends_with("[feature]"));
    }

    #[test]
    fn a_release_before_v0_9_0_cites_adr_0004_and_one_without_a_dev_container_is_bound() {
        let mut tree = Tree::base();
        tree.extra(MOUNT);
        tree.files.dir = false;
        assert!(keys(&tree).is_empty());
        tree.release = Some(Version(0, 6, 1));
        let adr = crate::testing::adr_id(4);
        assert_eq!(
            keys(&tree),
            [format!(
                ".devcontainer/devcontainer.json: mounts the host Docker socket — the Dev \
                 Container never reaches the host engine ({adr}); manage host containers from the \
                 host-side extension instead [mount]"
            )]
        );
    }

    // bash: a host socket mount in a config longer than a pipe holds
    #[test]
    fn a_long_config_with_a_socket_is_reported_from_v0_8_0_on_and_not_decided_before() {
        // check-devcontainer.txt: a host socket mount in a config longer than a pipe holds
        let mut tree = Tree::base();
        tree.extra(MOUNT);
        tree.extra(&format!(
            "\"remoteEnv\": {{ \"PAD\": \"{}\" }},",
            "z".repeat(200_000)
        ));
        assert_eq!(keys(&tree).len(), 1);
        tree.release = Some(Version(0, 8, 0));
        assert_eq!(keys(&tree).len(), 1);
        tree.release = Some(Version(0, 7, 0));
        let outcome = check(&tree.inputs());
        assert!(outcome.findings.is_empty());
        assert!(
            outcome
                .refusal
                .is_some_and(|why| why.starts_with(".devcontainer/devcontainer.json: matches")),
        );
        // Without a match, grep reads the whole pipe, and the release decides.
        let mut tree = Tree::base();
        tree.extra(&format!(
            "\"remoteEnv\": {{ \"PAD\": \"{}\" }},",
            "z".repeat(200_000)
        ));
        tree.release = Some(Version(0, 7, 0));
        assert_eq!(check(&tree.inputs()), Outcome::default());
    }
}
