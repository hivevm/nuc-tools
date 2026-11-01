//! Check 4 of the template's `scripts/check-docs.sh` at v0.5.0: every section reference names a
//! numbered `## N.` section of `AGENTS.md`, whose sections run 1..N; in Markdown it is a link to
//! the section's anchor. Superseded and rejected ADRs are not read.

use super::{Input, Inputs, Rule};
use crate::adr::{self, show};
use crate::bash;
use crate::finding::Finding;
use crate::markdown;

pub const RULE_ID: &str = "docs/section-refs";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    explain: "Every section reference names a numbered '## N.' heading of AGENTS.md, whose \
              headings run 1..N in order: the section sign and a number in Markdown, YAML, \
              shell scripts, and the git hooks, and 'AGENTS.md, section N' in every text file \
              but Markdown. In Markdown a section is cited as a link to its anchor, so that a \
              renumbering breaks the link instead of leaving the number pointing elsewhere.",
    reads: &[Input::Adr, Input::TextFiles, Input::Agents],
    check,
};

const SECTION: &str = "§";

fn finding(file: &str, line: Option<usize>, message: String) -> Finding {
    Finding {
        rule: RULE_ID,
        file: file.to_owned(),
        line,
        message,
    }
}

/// The numbered sections of AGENTS.md, as `sed -nE 's/^## ([0-9]+)\..*/\1/p'` lists them.
fn sections(agents: &[u8]) -> Vec<String> {
    bash::awk_lines(agents)
        .into_iter()
        .filter_map(|line| {
            let rest = line.strip_prefix(b"## ")?;
            let digits = rest.iter().take_while(|b| b.is_ascii_digit()).count();
            (digits > 0 && rest.get(digits) == Some(&b'.')).then(|| show(&rest[..digits]))
        })
        .collect()
}

/// The anchor of each numbered section, the last heading of a number winning.
fn section_slugs(agents: &[u8], valid: &[String]) -> Vec<(String, String)> {
    let mut slugs: Vec<(String, String)> = Vec::new();
    for slug in markdown::heading_slugs(agents) {
        let digits = slug.iter().take_while(|b| b.is_ascii_digit()).count();
        if digits == 0 || slug.get(digits) != Some(&b'-') {
            continue;
        }
        let n = show(&slug[..digits]);
        if valid.contains(&n) {
            slugs.retain(|(k, _)| *k != n);
            slugs.push((n, show(&slug)));
        }
    }
    slugs
}

fn check(inputs: &Inputs) -> Vec<Finding> {
    let Some(agents) = &inputs.agents else {
        return vec![finding(
            "AGENTS.md",
            None,
            "AGENTS.md not found in the repository root".to_owned(),
        )];
    };
    let valid = sections(agents);
    if valid.is_empty() {
        return vec![finding(
            "AGENTS.md",
            None,
            "AGENTS.md: no numbered '## N.' section headings found".to_owned(),
        )];
    }
    let mut found = Vec::new();
    if let Some((i, n)) = valid
        .iter()
        .enumerate()
        .find(|(i, n)| **n != (i + 1).to_string())
    {
        found.push(finding("AGENTS.md", None, format!(
            "AGENTS.md: numbered sections must run 1..N in order — expected '## {}.' at position {}, found '## {n}.'",
            i + 1,
            i + 1
        )));
    }
    let slugs = section_slugs(agents, &valid);
    let inactive = adr::inactive_paths(&inputs.adr);
    for (file, text) in &inputs.text_files {
        if inactive.contains(file.as_str()) {
            continue;
        }
        if !file.ends_with(".md") {
            found.extend(word_form(file, text, &valid));
        }
        if file.ends_with(".md") {
            found.extend(markdown_refs(file, text, &valid, &slugs));
        } else if is_doc_file(file) {
            found.extend(sign_refs(file, text, &valid));
        }
    }
    found
}

/// The anchor of section `n`, empty when no heading of it has one.
fn slug_of<'a>(slugs: &'a [(String, String)], n: &str) -> &'a str {
    slugs
        .iter()
        .find(|(k, _)| k == n)
        .map_or("", |(_, s)| s.as_str())
}

/// Whether section signs in the file are references into AGENTS.md by convention: Markdown,
/// YAML, shell scripts, and the git hooks.
fn is_doc_file(file: &str) -> bool {
    [".md", ".yml", ".yaml", ".sh"]
        .iter()
        .any(|ext| file.ends_with(ext))
        || file.starts_with(".githooks/")
}

/// Each `AGENTS.md, section N` or `AGENTS.md section N` of the file whose N names no section.
fn word_form(file: &str, text: &[u8], valid: &[String]) -> Vec<Finding> {
    let mut found = Vec::new();
    for (i, line) in bash::awk_lines(text).into_iter().enumerate() {
        let mut at = 0;
        while let Some(start) = bash::find(&line[at..], b"AGENTS.md") {
            let rest = &line[at + start + b"AGENTS.md".len()..];
            let rest = rest.strip_prefix(b",").unwrap_or(rest);
            let digits_at = rest
                .strip_prefix(b" section ")
                .map(|r| (r, r.iter().take_while(|b| b.is_ascii_digit()).count()));
            match digits_at {
                Some((r, digits)) if digits > 0 => {
                    let n = show(&r[..digits]);
                    if !valid.contains(&n) {
                        found.push(finding(file, Some(i + 1), format!(
                            "reference 'AGENTS.md, section {n}' matches no numbered section in AGENTS.md"
                        )));
                    }
                    at = line.len() - r.len() + digits;
                }
                _ => at += start + 1,
            }
        }
    }
    found
}

/// Each section sign and number on the line, in order, as a `§[0-9]+` search finds them.
fn signs(line: &[u8]) -> Vec<String> {
    let sign = SECTION.as_bytes();
    let mut found = Vec::new();
    let mut at = 0;
    while let Some(start) = bash::find(&line[at..], sign) {
        let after = at + start + sign.len();
        let digits = line[after..]
            .iter()
            .take_while(|b| b.is_ascii_digit())
            .count();
        if digits > 0 {
            found.push(show(&line[after..after + digits]));
        }
        at = after + digits;
    }
    found
}

/// Each `§N` of a YAML file, shell script, or hook whose N names no section.
fn sign_refs(file: &str, text: &[u8], valid: &[String]) -> Vec<Finding> {
    let mut found = Vec::new();
    for (i, line) in bash::awk_lines(text).into_iter().enumerate() {
        for n in signs(line) {
            if !valid.contains(&n) {
                found.push(finding(
                    file,
                    Some(i + 1),
                    format!("reference '§{n}' matches no numbered section in AGENTS.md"),
                ));
            }
        }
    }
    found
}

/// The line without its inline code spans, from a backquote to the next; an unpaired one stays.
fn without_code_spans(line: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut rest = line;
    while let Some(open) = rest.iter().position(|&b| b == b'`') {
        let Some(close) = rest[open + 1..].iter().position(|&b| b == b'`') else {
            break;
        };
        out.extend_from_slice(&rest[..open]);
        rest = &rest[open + 1 + close + 1..];
    }
    out.extend_from_slice(rest);
    out
}

/// The section references of a Markdown file's lines outside fences: each `§N` in a link must
/// name a section and link its anchor; one outside every link is a reference nothing verifies.
fn markdown_refs(
    file: &str,
    text: &[u8],
    valid: &[String],
    slugs: &[(String, String)],
) -> Vec<Finding> {
    let slug_of = |n: &str| slug_of(slugs, n).to_owned();
    let mut found = Vec::new();
    for (n, raw, read) in markdown::scanned_lines(text) {
        if !read || !bash::contains(raw, SECTION.as_bytes()) {
            continue;
        }
        let line = without_code_spans(raw);
        let mut links = Vec::new();
        let mut clean = Vec::new();
        let mut rest = line.as_slice();
        while let Some((start, length)) = markdown::leftmost_link(rest) {
            clean.extend_from_slice(&rest[..start]);
            links.push(&rest[start..start + length]);
            rest = &rest[start + length..];
        }
        clean.extend_from_slice(rest);
        let at = |message: String| finding(file, Some(n), message);
        for link in links {
            let Some(s) = signs(link).into_iter().next() else {
                continue;
            };
            if !valid.contains(&s) {
                found.push(at(format!(
                    "reference '§{s}' matches no numbered section in AGENTS.md"
                )));
                continue;
            }
            let target = markdown::link_target(link);
            let fragment = target
                .iter()
                .position(|&b| b == b'#')
                .map(|i| show(&target[i + 1..]))
                .filter(|f| !f.is_empty());
            match fragment {
                None => found.push(at(format!(
                    "'§{s}' is cited without an anchor — point the link at '#{}'",
                    slug_of(&s)
                ))),
                Some(f) if f != slug_of(&s) => found.push(at(format!(
                    "'§{s}' links to anchor '#{f}', but section {s} of AGENTS.md anchors as '#{}'",
                    slug_of(&s)
                ))),
                Some(_) => {}
            }
        }
        for s in signs(&clean) {
            found.push(at(if valid.contains(&s) {
                format!("'§{s}' stands outside a link — cite a section as a link to its anchor, e.g. [`AGENTS.md` §{s}](AGENTS.md#{})", slug_of(&s))
            } else {
                format!("reference '§{s}' matches no numbered section in AGENTS.md")
            }));
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    const AGENTS: &str = "# Agents\n\n## 1. Principles\n\n## 2. Start here\n\n## 3. ADR rules\n";

    fn messages(
        agents: Option<&str>,
        files: &[(&str, &str)],
    ) -> Vec<(String, Option<usize>, String)> {
        let mut inputs = Inputs {
            agents: agents.map(|a| a.as_bytes().to_vec()),
            ..Inputs::default()
        };
        for (path, text) in files {
            inputs
                .text_files
                .insert((*path).to_owned(), text.as_bytes().to_vec());
        }
        check(&inputs)
            .into_iter()
            .map(|f| (f.file, f.line, f.message))
            .collect()
    }

    fn texts(agents: Option<&str>, files: &[(&str, &str)]) -> Vec<String> {
        messages(agents, files)
            .into_iter()
            .map(|(_, _, m)| m)
            .collect()
    }

    #[test]
    fn references_as_links_to_their_anchors_pass() {
        let readme =
            "See [`AGENTS.md` §3](AGENTS.md#3-adr-rules) and `§9` in code.\n```\n§9\n```\n";
        let files = [
            ("README.md", readme),
            ("scripts/x.sh", "# AGENTS.md, section 2 and §1\n"),
            ("x.rs", "// §9 is a statute\n"),
        ];
        assert_eq!(texts(Some(AGENTS), &files), Vec::<String>::new());
    }

    #[test]
    fn an_empty_anchor_is_cited_without_one() {
        assert_eq!(
            texts(Some(AGENTS), &[("README.md", "[§3](AGENTS.md#)\n")]),
            ["'§3' is cited without an anchor — point the link at '#3-adr-rules'"]
        );
    }

    #[test]
    fn a_missing_agents_or_sections_fail() {
        assert_eq!(
            texts(None, &[]),
            ["AGENTS.md not found in the repository root"]
        );
        assert_eq!(
            texts(Some("# A\n"), &[]),
            ["AGENTS.md: no numbered '## N.' section headings found"]
        );
        assert_eq!(
            texts(Some("## 1. A\n## 3. C\n"), &[]),
            [
                "AGENTS.md: numbered sections must run 1..N in order — expected '## 2.' at position 2, found '## 3.'"
            ]
        );
    }

    #[test]
    fn each_kind_of_bad_reference_fails_at_its_line() {
        let readme =
            "x\n[s](AGENTS.md#3-rules) §3 bare, [n](AGENTS.md) §2, [z](AGENTS.md#9-x) §9.\n";
        let readme = readme
            .replace("[s](", "[§3](")
            .replace("[n](", "[§2](")
            .replace("[z](", "[§9](");
        let found = messages(
            Some(AGENTS),
            &[
                ("README.md", &readme),
                ("x.yml", "a: §7\n"),
                ("docs/n.txt", concat!("AGENTS.md, sec", "tion 8\n")),
            ],
        );
        let at: Vec<(&str, Option<usize>, &str)> = found
            .iter()
            .map(|(f, l, m)| (f.as_str(), *l, m.as_str()))
            .collect();
        assert!(at.contains(&("README.md", Some(2), "'§3' links to anchor '#3-rules', but section 3 of AGENTS.md anchors as '#3-adr-rules'")), "{at:?}");
        assert!(
            at.contains(&(
                "README.md",
                Some(2),
                "'§2' is cited without an anchor — point the link at '#2-start-here'"
            )),
            "{at:?}"
        );
        assert!(
            at.contains(&(
                "README.md",
                Some(2),
                "reference '§9' matches no numbered section in AGENTS.md"
            )),
            "{at:?}"
        );
        assert!(
            at.iter()
                .any(|(f, _, m)| *f == "README.md" && m.starts_with("'§3' stands outside a link")),
            "{at:?}"
        );
        assert!(
            at.contains(&(
                "x.yml",
                Some(1),
                "reference '§7' matches no numbered section in AGENTS.md"
            )),
            "{at:?}"
        );
        assert!(
            at.contains(&(
                "docs/n.txt",
                Some(1),
                concat!(
                    "reference 'AGENTS.md, sec",
                    "tion 8' matches no numbered section in AGENTS.md"
                )
            )),
            "{at:?}"
        );
        assert_eq!(at.len(), 8, "{at:?}");
    }

    #[test]
    fn a_code_span_hides_a_sign_and_an_unpaired_backquote_stays() {
        assert_eq!(without_code_spans(b"a `\xc2\xa71` b `c"), b"a  b `c");
    }
}
