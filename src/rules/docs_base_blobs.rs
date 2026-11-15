//! The check `docs/base-blobs` of the template's `scripts/check-docs.sh` from v0.9.0 on: each
//! `NUC-` ADR and `AGENTS.md`, the fixed base, is the blob its line of `docs/adr/NUC.blobs` names,
//! and the list names no other file (NUC-base-is-fixed §2). A tree of an earlier release has no
//! such check, and this rule reports nothing there.

use super::{CATEGORY_IDENTIFIERS, Input, Inputs, Rule};
use crate::finding::Finding;
use std::collections::BTreeMap;

pub const RULE_ID: &str = "docs/base-blobs";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    reasons: &["compute", "absent", "gone", "unlisted", "differs"],
    explain: "Each NUC- ADR of docs/adr/ and AGENTS.md is the blob its line of docs/adr/NUC.blobs \
              names, and the list names no file that is gone. The base is fixed: a derived \
              project restores the file of its release and narrows a rule by an ADR of its own. \
              A fork names the release it forked on the Template release line, 'vX.Y.Z, forked', \
              and needs no list.",
    reads: &[Input::Base, Input::Readme, Input::Release],
    check: |inputs| check(inputs).into(),
};

/// What the shell reads of the base: the blob of each `NUC-` file and of `AGENTS.md`, by name, as
/// `adr_blobs` computes them, or why git could not; and the bytes of `docs/adr/NUC.blobs`, if it
/// is a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Base {
    pub blobs: Result<Vec<(String, String)>, String>,
    pub list: Option<Vec<u8>>,
}

impl Default for Base {
    fn default() -> Self {
        Self {
            blobs: Ok(Vec::new()),
            list: None,
        }
    }
}

const LIST: &str = "docs/adr/NUC.blobs";

const FIX: &str = "a derived project restores the file of its release and narrows a rule by an ADR \
                   of its own; the template rewrites the list: bash -c 'source scripts/lib/adr.sh \
                   && adr_blobs docs/adr' > docs/adr/NUC.blobs (NUC-base-is-fixed §2)";

/// Whether a line of the README names a forked release, as the bash's `grep -qE` reads it:
/// `^[[:space:]]*([-*] )?\*\*Template release:\*\*[[:space:]]*` an optional `[` or backtick, a
/// `vX.Y.Z`, an optional `]` or backtick, an optional `(…)`, then `, forked` and no letter or
/// digit.
fn forked(readme: &str) -> bool {
    readme.lines().any(|line| {
        let line = line.trim_start_matches([' ', '\t', '\r', '\x0b', '\x0c']);
        let line = line
            .strip_prefix("- ")
            .or_else(|| line.strip_prefix("* "))
            .unwrap_or(line);
        let Some(rest) = line.strip_prefix("**Template release:**") else {
            return false;
        };
        let rest = rest.trim_start_matches([' ', '\t', '\r', '\x0b', '\x0c']);
        let rest = rest.strip_prefix(['[', '`']).unwrap_or(rest);
        let Some(rest) = rest.strip_prefix('v') else {
            return false;
        };
        let mut rest = rest;
        for i in 0..3 {
            let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
            if digits == 0 {
                return false;
            }
            rest = &rest[digits..];
            if i < 2 {
                let Some(r) = rest.strip_prefix('.') else {
                    return false;
                };
                rest = r;
            }
        }
        let rest = rest.strip_prefix([']', '`']).unwrap_or(rest);
        let rest = match rest.strip_prefix('(') {
            Some(r) => match r.find(')') {
                Some(end) => &r[end + 1..],
                None => rest,
            },
            None => rest,
        };
        rest.strip_prefix(", forked")
            .is_some_and(|r| !r.chars().next().is_some_and(|c| c.is_ascii_alphanumeric()))
    })
}

/// The lines of the list as `while read -r blob name` reads them: each line a line break ends, a
/// last one without it dropped as `read` drops it; the first field the blob, and the rest the name,
/// space and tab trimmed from both, and a carriage return kept as part of the name.
fn listed(list: &[u8]) -> BTreeMap<String, String> {
    let text = String::from_utf8_lossy(list);
    let mut lines: Vec<&str> = text.split('\n').collect();
    lines.pop();
    let mut want = BTreeMap::new();
    for line in lines {
        let line = line.trim_matches([' ', '\t']);
        let (blob, name) = line.split_once([' ', '\t']).unwrap_or((line, ""));
        let name = name.trim_matches([' ', '\t']);
        if !name.is_empty() {
            want.insert(name.to_owned(), blob.to_owned());
        }
    }
    want
}

/// The file a name of the base stands for: `AGENTS.md` at the root, a `NUC-` file in the record.
fn file_of(name: &str) -> String {
    if name == "AGENTS.md" {
        name.to_owned()
    } else {
        format!("docs/adr/{name}")
    }
}

fn finding(reason: &'static str, file: String, message: String) -> Finding {
    Finding {
        rule: RULE_ID,
        reason,
        file,
        line: None,
        message,
    }
}

fn check(inputs: &Inputs) -> Vec<Finding> {
    if !inputs.carries_at_least(CATEGORY_IDENTIFIERS) {
        return Vec::new();
    }
    let have: BTreeMap<&str, &str> = match &inputs.base.blobs {
        Ok(blobs) => blobs
            .iter()
            .map(|(n, b)| (n.as_str(), b.as_str()))
            .collect(),
        Err(why) => {
            return vec![finding(
                "compute",
                LIST.to_owned(),
                format!("no blob identifiers computed: {why}"),
            )];
        }
    };
    let Some(list) = &inputs.base.list else {
        if have.is_empty() || inputs.readme.as_deref().is_some_and(forked) {
            return Vec::new();
        }
        return vec![finding(
            "absent",
            LIST.to_owned(),
            "not found — it names each NUC- file and AGENTS.md with its blob; a fork names the \
             release it forked on the Template release line of README.md, 'vX.Y.Z, forked' \
             (NUC-base-is-fixed §1)"
                .to_owned(),
        )];
    };
    let want = listed(list);
    let mut found = Vec::new();
    for name in want.keys() {
        if !have.contains_key(name.as_str()) {
            found.push(finding(
                "gone",
                LIST.to_owned(),
                format!("names {name}, which does not exist — {FIX}"),
            ));
        }
    }
    for (name, blob) in &have {
        match want.get(*name) {
            None => found.push(finding(
                "unlisted",
                file_of(name),
                format!("has no line in docs/adr/NUC.blobs — {FIX}"),
            )),
            Some(listed) if listed != blob => found.push(finding(
                "differs",
                file_of(name),
                format!("differs from its line in docs/adr/NUC.blobs — {FIX}"),
            )),
            Some(_) => {}
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs(blobs: &[(&str, &str)], list: Option<&str>, readme: &str) -> Inputs {
        Inputs {
            base: Base {
                blobs: Ok(blobs
                    .iter()
                    .map(|(n, b)| ((*n).to_owned(), (*b).to_owned()))
                    .collect()),
                list: list.map(|l| l.as_bytes().to_vec()),
            },
            readme: Some(readme.to_owned()),
            ..Inputs::default()
        }
    }

    fn reasons(inputs: &Inputs) -> Vec<(&'static str, String)> {
        check(inputs)
            .into_iter()
            .map(|f| (f.reason, f.file))
            .collect()
    }

    #[test]
    fn each_cause_is_reported_with_its_reason_and_file() {
        let base = [("NUC-a.md", "1"), ("NUC-b.md", "2"), ("AGENTS.md", "3")];
        let list = "9 NUC-a.md\n3 AGENTS.md\n4 NUC-gone.md\n";
        assert_eq!(
            reasons(&inputs(&base, Some(list), "")),
            [
                ("gone", LIST.to_owned()),
                ("differs", "docs/adr/NUC-a.md".to_owned()),
                ("unlisted", "docs/adr/NUC-b.md".to_owned()),
            ]
        );
    }

    #[test]
    fn the_list_is_read_as_read_reads_it() {
        let want = listed(b"1 NUC-a.md\n2 NUC-b.md\r\n  3\tname with space  \n4 NUC-last.md");
        assert_eq!(want.get("NUC-a.md").map(String::as_str), Some("1"));
        assert_eq!(want.get("NUC-b.md\r").map(String::as_str), Some("2"));
        assert_eq!(want.get("name with space").map(String::as_str), Some("3"));
        assert!(!want.contains_key("NUC-last.md"));
    }

    #[test]
    fn a_missing_list_fails_unless_the_base_is_empty_or_forked() {
        let base = [("NUC-a.md", "1")];
        assert_eq!(
            reasons(&inputs(&base, None, "")),
            [("absent", LIST.to_owned())]
        );
        assert!(reasons(&inputs(&[], None, "")).is_empty());
        for line in [
            "- **Template release:** v0.11.0, forked\n",
            "**Template release:** [v0.11.0](https://x.invalid), forked.\n",
            "  * **Template release:** `v1.2.3`, forked\n",
        ] {
            assert!(reasons(&inputs(&base, None, line)).is_empty(), "{line}");
        }
        for line in [
            "- **Template release:** v0.11.0, forkedx\n",
            "- **Template release:** v0.11, forked\n",
        ] {
            assert_eq!(reasons(&inputs(&base, None, line)).len(), 1, "{line}");
        }
    }

    #[test]
    fn a_blob_git_cannot_compute_and_an_earlier_release_are_told_apart() {
        let mut failing = inputs(&[], None, "");
        failing.base.blobs = Err("fatal: x".to_owned());
        assert_eq!(reasons(&failing), [("compute", LIST.to_owned())]);
        failing.release = Some(crate::config::Version(0, 8, 1));
        assert!(reasons(&failing).is_empty());
    }
}
