//! The adapter of Claude Code: its transcripts are JSON Lines, one object a line, and a line of
//! type `assistant` reports a request of the model with its `requestId`, its `gitBranch`, and the
//! usage in `message.usage` (API-measure-reads-harness-transcripts). One request spans several
//! lines as it streams, each reporting its usage so far. Every other line and every other field is
//! ignored. A refusal names the file and the line, never their text.

use serde_json::Value;

use super::{Request, Usage};

/// The requests the transcript's lines report.
///
/// A line of the harness's own, with no request identifier and no token, is no request: Claude
/// Code writes such a line for a message it made up itself, as an interrupted turn.
///
/// # Errors
///
/// A line that is no JSON object, an `assistant` line without the usage, the four counts, or the
/// branch, and a line with tokens but no request identifier.
pub fn requests(name: &str, bytes: &[u8]) -> Result<Vec<Request>, String> {
    let mut requests = Vec::new();
    for (index, line) in bytes.split(|&b| b == b'\n').enumerate() {
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        let at = format!("{name}:{}", index + 1);
        let object: Value = serde_json::from_slice(line)
            .map_err(|_| format!("{at}: a line that is no JSON — not decided"))?;
        if !object.is_object() {
            return Err(format!("{at}: a line that is no JSON object — not decided"));
        }
        if object.get("type").and_then(Value::as_str) != Some("assistant") {
            continue;
        }
        let usage = usage(&object).ok_or_else(|| {
            format!("{at}: a request without the four counts of its usage — not decided")
        })?;
        let Some(id) = object.get("requestId").and_then(Value::as_str) else {
            if usage.total() == 0 {
                continue;
            }
            return Err(format!(
                "{at}: a request with tokens but no requestId — not decided"
            ));
        };
        let branch = object
            .get("gitBranch")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("{at}: a request without its gitBranch — not decided"))?;
        requests.push(Request {
            id: id.to_owned(),
            branch: branch.to_owned(),
            usage,
        });
    }
    Ok(requests)
}

/// The four counts of the line's `message.usage`, each a whole number; none where one is missing.
fn usage(object: &Value) -> Option<Usage> {
    let usage = object.get("message")?.get("usage")?;
    let count = |key: &str| usage.get(key)?.as_u64();
    Some(Usage {
        input: count("input_tokens")?,
        cache_read: count("cache_read_input_tokens")?,
        cache_written: count("cache_creation_input_tokens")?,
        output: count("output_tokens")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINE: &str = r#"{"type":"assistant","requestId":"req_1","gitBranch":"b","message":{"usage":{"input_tokens":2,"cache_read_input_tokens":30,"cache_creation_input_tokens":400,"output_tokens":5000}}}"#;

    #[test]
    fn an_assistant_line_is_a_request_with_its_usage_and_branch() {
        let found = requests("t.jsonl", format!("{LINE}\n").as_bytes()).expect("reads");
        assert_eq!(
            found,
            [Request {
                id: "req_1".to_owned(),
                branch: "b".to_owned(),
                usage: Usage {
                    input: 2,
                    cache_read: 30,
                    cache_written: 400,
                    output: 5000,
                },
            }]
        );
    }

    #[test]
    fn other_lines_and_blank_lines_are_no_requests() {
        let text =
            "{\"type\":\"user\",\"message\":{\"content\":\"x\"}}\n\n  \n{\"type\":\"system\"}";
        assert_eq!(requests("t.jsonl", text.as_bytes()), Ok(Vec::new()));
    }

    #[test]
    fn a_line_of_the_harness_without_request_and_tokens_is_none() {
        let text = r#"{"type":"assistant","message":{"model":"<synthetic>","usage":{"input_tokens":0,"cache_read_input_tokens":0,"cache_creation_input_tokens":0,"output_tokens":0}}}"#;
        assert_eq!(requests("t.jsonl", text.as_bytes()), Ok(Vec::new()));
    }

    #[test]
    fn what_cannot_be_read_names_its_file_and_line() {
        let refused = |text: &str| requests("s/t.jsonl", text.as_bytes()).expect_err("refused");
        assert!(refused(&format!("{LINE}\n{{\"type\":")).starts_with("s/t.jsonl:2: "));
        assert!(refused("[1]").starts_with("s/t.jsonl:1: a line that is no JSON object"));
        assert!(refused("\0\0\0").starts_with("s/t.jsonl:1: a line that is no JSON"));
        assert!(
            refused(r#"{"type":"assistant","requestId":"r","gitBranch":"b","message":{}}"#)
                .contains("without the four counts")
        );
        assert!(
            refused(&LINE.replace("\"gitBranch\":\"b\",", "")).contains("without its gitBranch")
        );
        assert!(refused(&LINE.replace("\"requestId\":\"req_1\",", "")).contains("no requestId"));
    }

    #[test]
    fn a_refusal_carries_nothing_of_the_line() {
        let refusal =
            requests("t.jsonl", b"{\"type\":\"assistant\",\"secret\":\"CANARY\"").unwrap_err();
        assert!(!refusal.contains("CANARY"), "{refusal}");
    }
}
