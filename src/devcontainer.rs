//! The Dev Container files as the template's `scripts/check-devcontainer.sh` reads them
//! (NUC-container-hides-host-daemon): `.devcontainer/devcontainer.json` and its lock as text with
//! their full-line `//` comments dropped, the keys of a `features` object found by the depth of
//! its braces, the value of a top-level key read to the next one, and `.vscode/settings.json`
//! flattened. The rules `devcontainer/*`, one per rule ID of the bash, share the reading here, so
//! that no rule module calls another. The bash's readings are the same from release v0.5.0, the
//! earliest `nuc` ports, to v0.12.0; what changed between them is what its rules check, and what
//! their messages cite.

use crate::config::Version;
use crate::rules::{CATEGORY_IDENTIFIERS, Inputs};

/// The definition of the Dev Container, from the root of the repository.
pub const CONFIG: &str = ".devcontainer/devcontainer.json";
/// The lock of its Features.
pub const LOCK: &str = ".devcontainer/devcontainer-lock.json";
/// The settings of VS Code.
pub const SETTINGS: &str = ".vscode/settings.json";

/// The release from which the bash checks the path the project is mounted under.
pub const MOUNT_PATH: Version = Version(0, 7, 0);

/// The release from which the bash reads `devcontainer.json` and the settings from a here-string,
/// where earlier releases pipe them into `grep -q`: under `pipefail`, a `grep -q` that matches and
/// exits before the pipe is drained fails the pipeline. Only the socket's pipe can be cut so: the
/// settings reach grep flattened to one line, which it reads to its end before it can match.
pub const HERE_STRINGS: Version = Version(0, 8, 0);

/// What a pipe holds before its writer waits for the reader: Linux's default, 64 KiB. A text that
/// fits reached `grep -q` whole in every run tried, so in practice it is decided; a longer one may
/// be cut off. The bound is no exact one: where grep's buffers fall decides.
const PIPE: usize = 64 * 1024;

/// What the shell reads of the Dev Container for the rules: whether `.devcontainer/` and
/// `.vscode/` are directories, as `-d` finds them, and the bytes of each file where it is a file,
/// as `-f` finds it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Devcontainer {
    pub dir: bool,
    pub config: Option<Vec<u8>>,
    pub lock: Option<Vec<u8>>,
    pub vscode: bool,
    pub settings: Option<Vec<u8>>,
}

/// Whether the Dev Container binds the tree checked as `inputs`' release: from v0.9.0 on, only
/// where `.devcontainer/` exists (NUC-container-hides-host-daemon §1); before, always, so that a
/// tree without it lacks its definition.
pub fn binds(inputs: &Inputs) -> bool {
    inputs.devcontainer.dir || !inputs.carries_at_least(CATEGORY_IDENTIFIERS)
}

/// Whether the settings of VS Code bind the tree: from v0.9.0 on, only where `.devcontainer/` and
/// `.vscode/` both exist, since the bash ends as unbound without `.devcontainer/` before it looks
/// at the settings (§1, §4); before, always.
pub fn settings_bind(inputs: &Inputs) -> bool {
    binds(inputs) && (inputs.devcontainer.vscode || !inputs.carries_at_least(CATEGORY_IDENTIFIERS))
}

/// The citation that ends a message of clause `section` of the decision: its `NUC-` ADR from
/// v0.9.0 on, and the template's ADR-0004 before.
pub fn cite(inputs: &Inputs, section: u8) -> String {
    if inputs.carries_at_least(CATEGORY_IDENTIFIERS) {
        format!("(NUC-container-hides-host-daemon \u{a7}{section})")
    } else {
        format!("({}-0004)", "ADR")
    }
}

/// Why a match of `grep -q` on the text of `path` is not decided: the release pipes the text into
/// it, and the text is longer than a pipe holds, so the pipeline fails or not depending on when
/// `grep` exits. None where the release reads a here-string or the text fits.
pub fn cut_off(inputs: &Inputs, path: &str, lines: &[&[u8]]) -> Option<String> {
    let written: usize = lines.iter().map(|line| line.len() + 1).sum();
    (!inputs.carries_at_least(HERE_STRINGS) && written > PIPE).then(|| {
        format!(
            "{path}: matches, and is longer than a pipe holds, which the bash of this release \
             reads through a pipe that grep -q may cut off — not decided"
        )
    })
}

/// The lines of the text without its full-line `//` comments, as `grep -vE '^[[:space:]]*//'`
/// writes them in the C locale.
pub fn uncommented(text: &[u8]) -> Vec<&[u8]> {
    crate::bash::awk_lines(text)
        .into_iter()
        .filter(|line| {
            let rest = line
                .iter()
                .position(|&b| !crate::bash::is_space(b))
                .map_or(&[][..], |at| &line[at..]);
            !rest.starts_with(b"//")
        })
        .collect()
}

/// The keys directly inside the `features` object, as `feature_ids` of the bash finds them: each
/// line split after an opening brace, before a closing one, and after a comma; a fragment that
/// matches `"features"[[:space:]]*:[[:space:]]*\{` opens the object at the depth before it; a
/// fragment one level deeper that opens with a quoted key and a colon names it. A brace counts
/// inside a string too.
pub fn feature_ids(text: &[u8]) -> Vec<Vec<u8>> {
    let mut ids = Vec::new();
    let (mut depth, mut base, mut in_features) = (0_i64, 0_i64, false);
    for line in uncommented(text) {
        let mut split = Vec::with_capacity(line.len());
        for &b in line {
            match b {
                b'{' => split.extend_from_slice(b"{\n"),
                b'}' => split.extend_from_slice(b"\n}"),
                b',' => split.extend_from_slice(b",\n"),
                _ => split.push(b),
            }
        }
        for fragment in split.split(|&b| b == b'\n') {
            if opens_features(fragment) {
                base = depth;
                in_features = true;
            } else if in_features
                && depth == base + 1
                && let Some(key) = leading_key(fragment)
            {
                ids.push(key.to_vec());
            }
            for &b in fragment {
                if b == b'{' {
                    depth += 1;
                } else if b == b'}' {
                    depth -= 1;
                    if in_features && depth == base {
                        in_features = false;
                    }
                }
            }
        }
    }
    ids
}

/// Whether the fragment holds `"features"`, POSIX space, a colon, POSIX space, and a brace.
fn opens_features(fragment: &[u8]) -> bool {
    let needle = b"\"features\"";
    let mut from = 0;
    while let Some(at) = crate::bash::find(&fragment[from..], needle).map(|at| at + from) {
        let rest = skip_space(&fragment[at + needle.len()..]);
        if let Some(rest) = rest.strip_prefix(b":")
            && skip_space(rest).starts_with(b"{")
        {
            return true;
        }
        from = at + 1;
    }
    false
}

fn skip_space(text: &[u8]) -> &[u8] {
    let at = text
        .iter()
        .position(|&b| !crate::bash::is_space(b))
        .unwrap_or(text.len());
    &text[at..]
}

/// The key a fragment opens with, as `^[[:space:]]*"[^"]+"[[:space:]]*:` matches it.
fn leading_key(fragment: &[u8]) -> Option<&[u8]> {
    let rest = skip_space(fragment).strip_prefix(b"\"")?;
    let end = rest.iter().position(|&b| b == b'"')?;
    (end > 0 && skip_space(&rest[end + 1..]).starts_with(b":")).then_some(&rest[..end])
}

/// The text after the top-level key `want` up to the next top-level key or the end, as
/// `top_level_value` of the bash prints it: a top-level key is a string at depth 1 that opens the
/// object or follows a comma there; a backslash in a string takes the byte after it along, so a
/// brace inside a string counts for nothing. The text is the uncommented lines, each ended by a
/// newline. Where the key stands twice, both values are printed.
pub fn top_level_value(text: &[u8], want: &[u8]) -> Vec<u8> {
    let mut joined = Vec::with_capacity(text.len());
    for line in uncommented(text) {
        joined.extend_from_slice(line);
        joined.push(b'\n');
    }
    let text = joined;
    let mut out = Vec::new();
    let (mut in_string, mut is_key, mut expect_key, mut capture) = (false, false, false, false);
    let mut depth = 0_i64;
    let mut s: Vec<u8> = Vec::new();
    let mut i = 0;
    while i < text.len() {
        let c = text[i];
        if in_string {
            if c == b'\\' {
                s.push(c);
                if let Some(&next) = text.get(i + 1) {
                    s.push(next);
                }
                i += 1;
            } else if c == b'"' {
                in_string = false;
                if is_key {
                    capture = s == want;
                    is_key = false;
                    i += 1;
                    continue;
                }
            } else {
                s.push(c);
            }
            if capture && !is_key {
                if c == b'\\' {
                    out.extend_from_slice(&text[i - 1..(i + 1).min(text.len())]);
                } else {
                    out.push(c);
                }
            }
            i += 1;
            continue;
        }
        match c {
            b'"' => {
                in_string = true;
                s.clear();
                is_key = depth == 1 && expect_key;
                expect_key = false;
            }
            b'{' | b'[' => {
                depth += 1;
                if depth == 1 {
                    expect_key = true;
                }
            }
            b'}' | b']' => depth -= 1,
            b',' if depth == 1 => expect_key = true,
            _ => {}
        }
        if capture && !is_key {
            out.push(c);
        }
        i += 1;
    }
    out
}

/// The bytes as a message names them, each that is no UTF-8 as U+FFFD.
pub fn spelled(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// The fixture of the template's table `scripts/fixtures/check-devcontainer.txt` of v0.12.0, for
/// the unit tests of the rules: a tree that passes every rule, and the steps that change it.
#[cfg(test)]
pub mod fixture {
    use super::Devcontainer;
    use crate::config::Version;
    use crate::rules::Inputs;

    /// The postStartCommand of the base.
    pub const POST: &str = "{ \"hooks\": \"git config core.hooksPath .githooks\", \"merge\": \
                            \"git config merge.ff false && git config pull.ff only\", \"index\": \
                            \"git config merge.adr-index.driver '.githooks/merge-adr-index %O %A \
                            %B %S %X %Y'\" }";
    /// The first Feature of the base.
    pub const FEATURE: &str = "ghcr.io/devcontainers/features/github-cli:1";
    /// The second Feature of the base, nuc's.
    pub const NUC: &str = "ghcr.io/hivevm/features/nuc:0";

    /// The tree of a case, built as the table's steps build it.
    #[derive(Debug, Clone)]
    pub struct Tree {
        pub feature: String,
        pub nuc: Option<String>,
        pub post: Option<String>,
        pub extras: Vec<String>,
        pub files: Devcontainer,
        pub release: Option<Version>,
    }

    impl Tree {
        /// The base tree of the table, which passes every rule.
        pub fn base() -> Self {
            let mut tree = Self {
                feature: FEATURE.to_owned(),
                nuc: Some(NUC.to_owned()),
                post: Some(POST.to_owned()),
                extras: Vec::new(),
                files: Devcontainer {
                    dir: true,
                    vscode: true,
                    ..Devcontainer::default()
                },
                release: None,
            };
            tree.config();
            tree.lock(&[FEATURE, NUC]);
            tree.settings("{ \"ms-azuretools.vscode-containers\": [\"ui\"] }");
            tree
        }
    }

    impl Tree {
        /// devcontainer.json written again from the Features, the postStartCommand, and the
        /// extra lines.
        pub fn config(&mut self) -> &mut Self {
            let mut text = String::from(
                "{\n  \"name\": \"fixture\",\n  // A comment: the container has no socket mount.\n  \
                 \"image\": \"mcr.microsoft.com/devcontainers/base:debian\",\n",
            );
            if let Some(post) = &self.post {
                text.push_str(&format!("  \"postStartCommand\": {post},\n"));
            }
            for extra in &self.extras {
                text.push_str(&format!("  {extra}\n"));
            }
            text.push_str("  \"features\": {\n    // The Features.\n");
            match &self.nuc {
                Some(nuc) => text.push_str(&format!(
                    "    \"{}\": {{}},\n    \"{nuc}\": {{}}\n",
                    self.feature
                )),
                None => text.push_str(&format!("    \"{}\": {{}}\n", self.feature)),
            }
            text.push_str(
                "  },\n  \"customizations\": { \"vscode\": { \"extensions\": [] } }\n}\n",
            );
            self.files.config = Some(text.into_bytes());
            self
        }

        /// One more top-level line of devcontainer.json.
        pub fn extra(&mut self, line: &str) -> &mut Self {
            self.extras.push(line.to_owned());
            self.config()
        }

        /// The lock written again, resolving each id.
        pub fn lock(&mut self, ids: &[&str]) -> &mut Self {
            let mut text = String::from("{\n  \"features\": {\n");
            for id in ids {
                text.push_str(&format!(
                    "    \"{id}\": {{\n      \"version\": \"1.0.0\",\n      \"resolved\": \
                     \"example@sha256:0000\",\n      \"integrity\": \"sha256:0000\"\n    }},\n"
                ));
            }
            text.push_str("  }\n}\n");
            self.files.lock = Some(text.into_bytes());
            self
        }

        /// The settings written again, the json as the value of remote.extensionKind.
        pub fn settings(&mut self, kind: &str) -> &mut Self {
            self.files.settings = Some(
                format!(
                    "{{\n  // Keep the container-management extension on the host side.\n  \
                     \"remote.extensionKind\": {kind}\n}}\n"
                )
                .into_bytes(),
            );
            self
        }

        /// The inputs of the rules.
        pub fn inputs(&self) -> Inputs {
            Inputs {
                devcontainer: self.files.clone(),
                release: self.release,
                ..Inputs::default()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG_TEXT: &str = r#"{
  "name": "fixture",
  // A comment: the container has no socket mount.
  "image": "mcr.microsoft.com/devcontainers/base:debian",
  "postStartCommand": { "hooks": "git config core.hooksPath .githooks" },
  "features": {
    // The Features.
    "ghcr.io/devcontainers/features/github-cli:1": {},
    "ghcr.io/hivevm/features/nuc:0": {}
  },
  "customizations": { "vscode": { "extensions": [] } }
}
"#;

    fn ids(text: &str) -> Vec<String> {
        feature_ids(text.as_bytes())
            .iter()
            .map(|id| spelled(id))
            .collect()
    }

    #[test]
    fn a_full_line_comment_is_dropped_and_a_trailing_one_kept() {
        let text = b"a\n  // b\n\t//c\nd // e\n//";
        assert_eq!(uncommented(text), [&b"a"[..], b"d // e"]);
    }

    #[test]
    fn the_features_are_the_keys_directly_inside_their_object() {
        assert_eq!(
            ids(CONFIG_TEXT),
            [
                "ghcr.io/devcontainers/features/github-cli:1",
                "ghcr.io/hivevm/features/nuc:0"
            ]
        );
        // check-devcontainer.txt: a features block on one line, with a string-valued Feature, is
        // read like any other
        assert_eq!(
            ids(r#"{ "image": "x", "features": { "a:1": { "o": { "p": 1 } }, "b:1": "lts" } }"#),
            ["a:1", "b:1"]
        );
        // A brace inside a string counts, as awk counts it, and a commented Feature is none.
        assert_eq!(
            ids("{\n\"features\": {\n// \"c:1\": {},\n\"a:1\": \"{\",\n\"b:1\": {}\n}\n}\n"),
            ["a:1"]
        );
        assert!(ids("{ \"image\": \"x\" }").is_empty());
    }

    #[test]
    fn crlf_line_endings_and_a_last_line_without_a_newline_read_as_awk_reads_them() {
        let crlf = CONFIG_TEXT.replace('\n', "\r\n");
        let crlf = crlf.strip_suffix("\r\n").expect("a last line");
        assert_eq!(ids(crlf), ids(CONFIG_TEXT));
        // A carriage return stays in the value, a byte the bash keeps as well.
        let value = top_level_value(crlf.as_bytes(), b"name");
        assert_eq!(value, b": \"fixture\",\r\n  ");
        assert_eq!(
            top_level_value(b"{ \"a\": 1 }", b"a"),
            top_level_value(b"{ \"a\": 1 }\n", b"a")
        );
    }

    #[test]
    fn a_top_level_value_reads_to_the_next_top_level_key_with_strings_skipped() {
        let value = top_level_value(CONFIG_TEXT.as_bytes(), b"postStartCommand");
        assert_eq!(
            spelled(&value),
            ": { \"hooks\": \"git config core.hooksPath .githooks\" },\n  "
        );
        let text = br#"{ "a": "x\"y", "list": ["a]b", "${b}"], "b": 1 }"#;
        assert_eq!(top_level_value(text, b"a"), br#": "x\"y", "#);
        assert_eq!(top_level_value(text, b"list"), br#": ["a]b", "${b}"], "#);
        assert_eq!(top_level_value(text, b"b"), b": 1 }\n");
        assert!(top_level_value(text, b"hooks").is_empty());
        // A key inside a nested object is not top-level.
        assert!(top_level_value(br#"{ "o": { "a": 1 } }"#, b"a").is_empty());
    }
}
