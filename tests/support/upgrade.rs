//! The fixtures of `nuc upgrade`: a template repository with an annotated tag per release, and a
//! project derived from its first release with the tags fetched into `refs/template/`, as the
//! normalize skill fetches them (API-upgrade-by-blob-toml-optional).

use super::{Fixture, stderr};

/// The files of a template release: each path with its bytes, or none where the release removes
/// it.
pub type Release<'a> = (&'a str, &'a [(&'a str, Option<&'a str>)]);

/// A template repository with an annotated tag per release, each release's files committed on
/// top of the last's, as the template cuts its releases.
pub fn template(releases: &[Release]) -> Fixture {
    let template = Fixture::new("upgrade-template");
    for (tag, files) in releases {
        for (path, contents) in *files {
            match contents {
                Some(contents) => {
                    template.write(path, contents);
                }
                None => {
                    template.git(&["rm", "-q", path]);
                }
            }
        }
        template.commit(tag);
        template.git(&["tag", "-a", "-m", &format!("NUC {tag}"), tag]);
    }
    template
}

/// The tag of the template fetched into the project as the normalize skill fetches it, into
/// `refs/template/<tag>`.
pub fn fetch_release(project: &Fixture, template: &Fixture, tag: &str) {
    let refspec = format!("refs/tags/{tag}:refs/template/{tag}");
    let source = template.root().to_str().expect("a UTF-8 path");
    let output = project.git(&["fetch", "-q", "--no-tags", source, &refspec]);
    assert!(output.status.success(), "{}", stderr(&output));
}

pub const UPGRADE_TOML: &str = "minimum = \"0.0.0\"\nrules = []\n";

/// v0.9.0 and v0.10.0 of a template: v0.10.0 changes AGENTS.md and an ADR of the base, removes
/// one and adds one, and changes a script the project left as v0.9.0 had it. Its README stays, so
/// that a project with a README of its own gets no judgment.
pub fn two_releases() -> Fixture {
    let readme = "# NUC\n\n- **Template release:** v0.9.0 — this release.\n";
    template(&[
        (
            "v0.9.0",
            &[
                ("README.md", Some(readme)),
                ("AGENTS.md", Some("# Agents v0.9.0\n")),
                ("docs/adr/NUC.blobs", Some("blobs v0.9.0\n")),
                ("docs/adr/NUC-kept.md", Some("# NUC-kept\n")),
                ("docs/adr/NUC-changed.md", Some("# NUC-changed v0.9.0\n")),
                ("docs/adr/NUC-gone.md", Some("# NUC-gone\n")),
                ("scripts/check-docs.sh", Some("echo v0.9.0\n")),
            ],
        ),
        (
            "v0.10.0",
            &[
                ("AGENTS.md", Some("# Agents v0.10.0\n")),
                ("docs/adr/NUC.blobs", Some("blobs v0.10.0\n")),
                ("docs/adr/NUC-changed.md", Some("# NUC-changed v0.10.0\n")),
                ("docs/adr/NUC-gone.md", None),
                ("docs/adr/NUC-new.md", Some("# NUC-new\n")),
                ("scripts/check-docs.sh", Some("echo v0.10.0\n")),
            ],
        ),
    ])
}

/// A project derived from v0.9.0 of the template, its own README and ADR beside the base, on a
/// branch of its own with both tags fetched.
pub fn derived(template: &Fixture) -> Fixture {
    let readme = "# Project\n\nIt took up v0.9.0 once.\n\n- **Template release:** v0.9.0 — the \
                  release of NUC this project carries.\n";
    let project = Fixture::project("upgrade", UPGRADE_TOML, readme);
    project
        .write("AGENTS.md", "# Agents v0.9.0\n")
        .write("docs/adr/NUC.blobs", "blobs v0.9.0\n")
        .write("docs/adr/NUC-kept.md", "# NUC-kept\n")
        .write("docs/adr/NUC-changed.md", "# NUC-changed v0.9.0\n")
        .write("docs/adr/NUC-gone.md", "# NUC-gone\n")
        .write("docs/adr/API-own.md", "# API-own\n")
        .write("scripts/check-docs.sh", "echo v0.9.0\n")
        .commit("Take up v0.9.0");
    project.git(&["switch", "-q", "-c", "feat/take-up"]);
    fetch_release(&project, template, "v0.9.0");
    fetch_release(&project, template, "v0.10.0");
    project
}

/// v0.9.0 and v0.10.0 of a template whose files outside the base meet each case of the rule for
/// every other path in the project `three_way_project` derives: one the template leaves, one the
/// project left, one both changed, one the project deleted, one the template adds, one the project
/// has of its own, one the template removes, one it removes that the project changed, a file that
/// becomes a directory, a directory that becomes a file, and the README, which both change.
pub fn three_way_releases() -> Fixture {
    let readme = "# NUC\n\n- **Template release:** v0.9.0 — this release.\n";
    template(&[
        (
            "v0.9.0",
            &[
                ("README.md", Some(readme)),
                ("AGENTS.md", Some("# Agents\n")),
                ("scripts/same.sh", Some("same\n")),
                ("scripts/taken.sh", Some("taken v0.9.0\n")),
                ("scripts/both.sh", Some("both v0.9.0\n")),
                ("scripts/lost.sh", Some("lost v0.9.0\n")),
                ("scripts/gone.sh", Some("gone\n")),
                ("scripts/kept.sh", Some("kept v0.9.0\n")),
                ("swap-a", Some("a file\n")),
                ("swap-b/inner.md", Some("inner\n")),
            ],
        ),
        (
            "v0.10.0",
            &[
                ("scripts/taken.sh", Some("taken v0.10.0\n")),
                ("scripts/both.sh", Some("both v0.10.0\n")),
                ("scripts/lost.sh", Some("lost v0.10.0\n")),
                ("scripts/new.sh", Some("new\n")),
                ("scripts/own.sh", Some("own of the template\n")),
                ("scripts/gone.sh", None),
                ("scripts/kept.sh", None),
                ("swap-a", None),
                ("swap-a/inner.md", Some("now inner\n")),
                ("swap-b/inner.md", None),
                ("swap-b", Some("now a file\n")),
                (
                    "README.md",
                    Some("# NUC\n\n- **Template release:** v0.10.0 — this release.\n"),
                ),
            ],
        ),
    ])
}

/// The project derived from v0.9.0 of `three_way_releases`, with the changes of its own each case
/// needs, on a branch of its own with both tags fetched.
pub fn three_way_project(template: &Fixture) -> Fixture {
    let readme = "# Project\n\n- **Template release:** v0.9.0\n";
    let project = Fixture::project("upgrade-three-way", UPGRADE_TOML, readme);
    project
        .write("AGENTS.md", "# Agents\n")
        .write("scripts/same.sh", "same, edited\n")
        .write("scripts/taken.sh", "taken v0.9.0\n")
        .write("scripts/both.sh", "both, edited\n")
        .write("scripts/own.sh", "own of the project\n")
        .write("scripts/gone.sh", "gone\n")
        .write("scripts/kept.sh", "kept, edited\n")
        .write("swap-a", "a file\n")
        .write("swap-b/inner.md", "inner\n")
        .commit("Take up v0.9.0");
    project.git(&["switch", "-q", "-c", "feat/take-up"]);
    fetch_release(&project, template, "v0.9.0");
    fetch_release(&project, template, "v0.10.0");
    project
}
