//! What `nuc check` writes on the standard output, in the format `--format` names
//! (ADR-draft-output-formats): the lines of ADR-0020, or one JSON document. Built here, without a
//! crate, so that the same findings give the same bytes; the shell only writes the result.

use std::fmt::Write as _;

use crate::finding::Finding;

/// A format `nuc check --format` takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// One line per finding, `<file>[:<line>]: <rule ID>: <message>` (ADR-0020); the default.
    Text,
    /// One JSON document that names its format and version.
    Json,
}

impl Format {
    /// The format named `name` on the command line, if `nuc` knows it.
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "text" => Some(Self::Text),
            "json" => Some(Self::Json),
            _ => None,
        }
    }
}

/// The standard output of a run that decided, with these findings in the order the rules gave
/// them and these notes.
pub fn document(format: Format, findings: &[Finding], notes: &[String]) -> String {
    match format {
        Format::Text => findings.iter().map(|f| format!("{f}\n")).collect(),
        Format::Json => json(findings, notes),
    }
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
        assert_eq!(
            document(Format::Json, &findings, &["n".to_owned()]),
            "{\"format\":\"nuc-findings\",\"version\":1,\"findings\":[\
             {\"rule\":\"docs/links\",\"file\":\"a.md\",\"line\":3,\"message\":\"x\"},\
             {\"rule\":\"docs/links\",\"file\":\"b.md\",\"line\":null,\"message\":\"y\"}],\
             \"notes\":[\"n\"]}\n"
        );
        assert_eq!(
            document(Format::Json, &[], &[]),
            "{\"format\":\"nuc-findings\",\"version\":1,\"findings\":[],\"notes\":[]}\n"
        );
        assert_eq!(
            document(Format::Text, &findings, &[]),
            "a.md:3: docs/links: x\nb.md: docs/links: y\n"
        );
    }

    #[test]
    fn only_the_formats_of_this_nuc_parse() {
        assert_eq!(Format::parse("text"), Some(Format::Text));
        assert_eq!(Format::parse("json"), Some(Format::Json));
        for name in ["", "JSON", "yaml", "sarif "] {
            assert_eq!(Format::parse(name), None, "{name:?}");
        }
    }
}
