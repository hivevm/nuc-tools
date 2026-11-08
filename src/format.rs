//! What `nuc check` writes on the standard output, in the format `--format` names
//! (ADR-0027): the lines of ADR-0020, one JSON document, or one SARIF 2.1.0 log.
//! Built here, without a crate, so that the same findings give the same bytes; the shell only
//! writes the result.

use std::fmt::Write as _;

use crate::finding::Finding;

/// A format `nuc check --format` takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// One line per finding, `<file>[:<line>]: <rule ID>: <message>` (ADR-0020); the default.
    Text,
    /// One JSON document that names its format and version.
    Json,
    /// One SARIF 2.1.0 log, as GitHub's code scanning reads it.
    Sarif,
}

impl Format {
    /// The format named `name` on the command line, if `nuc` knows it.
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "text" => Some(Self::Text),
            "json" => Some(Self::Json),
            "sarif" => Some(Self::Sarif),
            _ => None,
        }
    }
}

/// What a run of `nuc check` decided, as every format writes it.
pub struct Report<'a> {
    /// The findings, in the order the rules gave them.
    pub findings: &'a [Finding],
    /// The notes of the rules, which are no findings.
    pub notes: &'a [String],
    /// The rules the run turned on, in the order it ran them: each ID with its reason.
    pub rules: &'a [(&'static str, &'static str)],
    /// The version of the running `nuc`.
    pub version: &'a str,
}

/// The standard output of a run that decided.
pub fn document(format: Format, report: &Report) -> String {
    match format {
        Format::Text => report.findings.iter().map(|f| format!("{f}\n")).collect(),
        Format::Json => json(report.findings, report.notes),
        Format::Sarif => sarif(report),
    }
}

/// The SARIF 2.1.0 log: one run, its tool with every rule the run turned on, and one result per
/// finding, located by a URI relative to the root of the working tree and a start line, 1 where
/// the finding names none.
fn sarif(report: &Report) -> String {
    let mut out = String::from(
        r#"{"$schema":"https://json.schemastore.org/sarif-2.1.0.json","version":"2.1.0","runs":[{"tool":{"driver":{"name":"nuc","semanticVersion":"#,
    );
    string(&mut out, report.version);
    out.push_str(r#","rules":["#);
    for (i, (id, reason)) in report.rules.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(r#"{"id":"#);
        string(&mut out, id);
        for key in ["shortDescription", "fullDescription", "help"] {
            write!(out, r#","{key}":{{"text":"#).expect("a String takes every write");
            string(&mut out, reason);
            out.push('}');
        }
        out.push('}');
    }
    out.push_str(r#"]}},"results":["#);
    for (i, finding) in report.findings.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(r#"{"ruleId":"#);
        string(&mut out, finding.rule);
        let index = report
            .rules
            .iter()
            .position(|(id, _)| *id == finding.rule)
            .expect("the shell refuses a finding a rule reports under another ID");
        write!(out, r#","ruleIndex":{index}"#).expect("a String takes every write");
        out.push_str(r#","level":"error","message":{"text":"#);
        string(&mut out, &finding.message);
        out.push_str(r#"},"locations":[{"physicalLocation":{"artifactLocation":{"uri":"#);
        string(&mut out, &uri(&finding.file));
        write!(
            out,
            r#"}},"region":{{"startLine":{}}}}}}}]}}"#,
            finding.line.unwrap_or(1)
        )
        .expect("a String takes every write");
    }
    out.push_str("]}]}\n");
    out
}

/// `path` as a URI reference, as RFC 3986 asks: `/` and the unreserved characters stay, and
/// every other byte of its UTF-8 is percent-encoded in upper-case hex, so that no path reads as
/// a scheme, a fragment, or a query.
fn uri(path: &str) -> String {
    let mut out = String::new();
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~/".contains(&byte) {
            out.push(char::from(byte));
        } else {
            write!(out, "%{byte:02X}").expect("a String takes every write");
        }
    }
    out
}

/// The JSON document: its format name and version, the findings, and the notes, on one line.
fn json(findings: &[Finding], notes: &[String]) -> String {
    let mut out = String::from(r#"{"format":"nuc-findings","version":1,"findings":["#);
    for (i, finding) in findings.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(r#"{"rule":"#);
        string(&mut out, finding.rule);
        out.push_str(r#","file":"#);
        string(&mut out, &finding.file);
        out.push_str(r#","line":"#);
        match finding.line {
            Some(line) => write!(out, "{line}").expect("a String takes every write"),
            None => out.push_str("null"),
        }
        out.push_str(r#","message":"#);
        string(&mut out, &finding.message);
        out.push('}');
    }
    out.push_str(r#"],"notes":["#);
    for (i, note) in notes.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        string(&mut out, note);
    }
    out.push_str("]}\n");
    out
}

/// `text` as a JSON string, escaped as RFC 8259 asks: `"` and `\` with a backslash, the five
/// control characters that have a short escape with it, every other one below U+0020 as `\u00XX`
/// in lower-case hex, and every other character as it is.
fn string(out: &mut String, text: &str) {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => {
                write!(out, "\\u{:04x}", u32::from(c)).expect("a String takes every write")
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    const RULES: &[(&str, &str)] = &[("docs/adr-index", "an index"), ("docs/links", "a \"link\"")];

    fn report<'a>(findings: &'a [Finding], notes: &'a [String]) -> Report<'a> {
        Report {
            findings,
            notes,
            rules: RULES,
            version: "1.2.3",
        }
    }

    fn quoted(text: &str) -> String {
        let mut out = String::new();
        string(&mut out, text);
        out
    }

    #[test]
    fn strings_escape_what_rfc_8259_asks_and_nothing_else() {
        assert_eq!(quoted(r#"a"b\c"#), r#""a\"b\\c""#);
        assert_eq!(quoted("\u{8}\u{c}\n\r\t"), r#""\b\f\n\r\t""#);
        assert_eq!(quoted("\u{0}\u{1f}\u{b}"), r#""\u0000\u001f\u000b""#);
        // DEL, U+2028, a slash, and letters beyond ASCII need no escape.
        assert_eq!(quoted("\u{7f}\u{2028}/é—"), "\"\u{7f}\u{2028}/é—\"");
    }

    #[test]
    fn the_json_document_holds_each_finding_and_note_in_order() {
        let finding = |file: &str, line, message: &str| Finding {
            rule: "docs/links",
            file: file.to_owned(),
            line,
            message: message.to_owned(),
        };
        let findings = [finding("a.md", Some(3), "x"), finding("b.md", None, "y")];
        let notes = ["n".to_owned()];
        assert_eq!(
            document(Format::Json, &report(&findings, &notes)),
            "{\"format\":\"nuc-findings\",\"version\":1,\"findings\":[\
             {\"rule\":\"docs/links\",\"file\":\"a.md\",\"line\":3,\"message\":\"x\"},\
             {\"rule\":\"docs/links\",\"file\":\"b.md\",\"line\":null,\"message\":\"y\"}],\
             \"notes\":[\"n\"]}\n"
        );
        assert_eq!(
            document(Format::Json, &report(&[], &[])),
            "{\"format\":\"nuc-findings\",\"version\":1,\"findings\":[],\"notes\":[]}\n"
        );
        assert_eq!(
            document(Format::Text, &report(&findings, &[])),
            "a.md:3: docs/links: x\nb.md: docs/links: y\n"
        );
    }

    #[test]
    fn only_the_formats_of_this_nuc_parse() {
        assert_eq!(Format::parse("text"), Some(Format::Text));
        assert_eq!(Format::parse("json"), Some(Format::Json));
        assert_eq!(Format::parse("sarif"), Some(Format::Sarif));
        for name in ["", "JSON", "yaml", "sarif "] {
            assert_eq!(Format::parse(name), None, "{name:?}");
        }
    }

    #[test]
    fn a_uri_keeps_slashes_and_unreserved_characters_and_encodes_every_other_byte() {
        assert_eq!(uri("docs/a-b_c.d~e/F9.md"), "docs/a-b_c.d~e/F9.md");
        assert_eq!(uri("parity: variant.txt"), "parity%3A%20variant.txt");
        assert_eq!(uri("a#b?c%d"), "a%23b%3Fc%25d");
        // A letter beyond ASCII is each byte of its UTF-8, in upper-case hex (RFC 3986, 2.1).
        assert_eq!(uri("é"), "%C3%A9");
        assert_eq!(uri("a\\b\"c"), "a%5Cb%22c");
    }

    #[test]
    fn the_sarif_log_lists_the_rules_and_one_located_result_per_finding() {
        let finding = |file: &str, line, message: &str| Finding {
            rule: "docs/links",
            file: file.to_owned(),
            line,
            message: message.to_owned(),
        };
        let findings = [finding("a b.md", Some(3), "x"), finding("c.md", None, "y")];
        let rule = |id: &str, reason: &str| {
            format!(
                "{{\"id\":\"{id}\",\"shortDescription\":{{\"text\":{reason}}},\"fullDescription\":{{\"text\":{reason}}},\"help\":{{\"text\":{reason}}}}}"
            )
        };
        let result = |uri: &str, line: usize, message: &str| {
            format!(
                "{{\"ruleId\":\"docs/links\",\"ruleIndex\":1,\"level\":\"error\",\"message\":{{\"text\":\"{message}\"}},\"locations\":[{{\"physicalLocation\":{{\"artifactLocation\":{{\"uri\":\"{uri}\"}},\"region\":{{\"startLine\":{line}}}}}}}]}}"
            )
        };
        assert_eq!(
            document(Format::Sarif, &report(&findings, &["n".to_owned()])),
            format!(
                "{{\"$schema\":\"https://json.schemastore.org/sarif-2.1.0.json\",\"version\":\"2.1.0\",\
                 \"runs\":[{{\"tool\":{{\"driver\":{{\"name\":\"nuc\",\"semanticVersion\":\"1.2.3\",\
                 \"rules\":[{},{}]}}}},\"results\":[{},{}]}}]}}\n",
                rule("docs/adr-index", "\"an index\""),
                rule("docs/links", "\"a \\\"link\\\"\""),
                result("a%20b.md", 3, "x"),
                result("c.md", 1, "y"),
            )
        );
    }

    /// GitHub's code scanning takes no empty string for a rule's descriptions and at most 1024
    /// characters for its short and its full description, both of which hold the reason:
    /// <https://docs.github.com/en/code-security/code-scanning/integrating-with-code-scanning/sarif-support-for-code-scanning>.
    const DESCRIPTION_LIMIT: usize = 1024;

    /// Verifies: ADR-0027
    #[test]
    fn every_reason_fits_the_descriptions_github_takes() {
        for rule in crate::rules::ALL {
            let length = rule.explain.chars().count();
            assert!(
                (1..=DESCRIPTION_LIMIT).contains(&length),
                "{}: its reason has {length} characters, and GitHub takes 1 to {DESCRIPTION_LIMIT}",
                rule.id
            );
        }
    }
}
