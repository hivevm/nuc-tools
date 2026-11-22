//! The fixtures of `nuc upgrade`: a template repository with an annotated tag per release, and a
//! project derived from its first release with the tags fetched into `refs/template/`, as the
//! normalize skill fetches them (API-upgrade-takes-up-by-blob).

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
/// one and adds one, and changes a file outside the base, which the tracer leaves alone.
pub fn two_releases() -> Fixture {
    let readme = "# NUC\n\n- **Template release:** v0.9.0 — this release.\n";
    let readme_new = "# NUC\n\n- **Template release:** v0.10.0 — this release.\n";
    template(&[
        (
            "v0.9.0",
            &[
                ("README.md", Some(readme)),
                ("nuc.toml", Some(UPGRADE_TOML)),
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
                ("README.md", Some(readme_new)),
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
