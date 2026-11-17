//! The adapter of Claude Code: its transcripts are JSON Lines, one object a line
//! (API-measure-reads-harness-transcripts).
//!
//! - A line of type `assistant` reports a request of the model with its `requestId`, its
//!   `gitBranch`, its `timestamp`, and the usage in `message.usage`. One request spans several
//!   lines, each reporting its usage. Its `message.content` names the tool calls it makes, each
//!   `tool_use` with its `id` and `name`.
//! - A line of type `user` returns a call's output, a `tool_result` of `message.content` that names
//!   the call by `tool_use_id`. Only its size is read.
//! - A subagent's `agent-*.meta.json` names its `agentType` and `description`.
//!
//! Every other line and field is ignored. A refusal names the file and the line, never their text.

use std::collections::BTreeMap;

use serde_json::Value;

use super::{Lines, Request, Source, Subagent, ToolOutput, Usage};

/// The requests and tool outputs the transcript's lines report, written at `source`.
///
/// A line of the harness's own, with no request identifier and no token, is no request: Claude
/// Code writes such a line for a message it made up itself, as an interrupted turn.
///
/// # Errors
///
/// A line that is no JSON object, an `assistant` line without the usage, the four counts, or the
/// branch, and a line with tokens but no request identifier.
pub fn lines(name: &str, bytes: &[u8], source: &Source) -> Result<Lines, String> {
    let mut found = Lines::default();
    let mut tools: BTreeMap<String, String> = BTreeMap::new();
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
        let text = |key: &str| object.get(key).and_then(Value::as_str).map(str::to_owned);
        match object.get("type").and_then(Value::as_str) {
            Some("assistant") => {
                for (id, tool) in calls(&object) {
                    tools.insert(id, tool);
                }
                if let Some(request) = request(&object, &at, source)? {
                    found.requests.push(request);
                }
            }
            Some("user") => {
                let results = results(&object);
                if results.is_empty() {
                    continue;
                }
                let branch = text("gitBranch").ok_or_else(|| {
                    format!("{at}: a tool's output without its gitBranch — not decided")
                })?;
                for (id, size) in results {
                    found.tool_outputs.push(ToolOutput {
                        tool: tools
                            .get(&id)
                            .cloned()
                            .unwrap_or_else(|| "unknown".to_owned()),
                        at: at.clone(),
                        size,
                        branch: branch.clone(),
                        time: text("timestamp"),
                        source: source.clone(),
                    });
                }
            }
            _ => {}
        }
    }
    Ok(found)
}

/// The request an `assistant` line reports, none for a line of the harness's own.
fn request(object: &Value, at: &str, source: &Source) -> Result<Option<Request>, String> {
    let usage = usage(object).ok_or_else(|| {
        format!("{at}: a request without the four counts of its usage — not decided")
    })?;
    let text = |key: &str| object.get(key).and_then(Value::as_str);
    let Some(id) = text("requestId") else {
        if usage.total() == 0 {
            return Ok(None);
        }
        return Err(format!(
            "{at}: a request with tokens but no requestId — not decided"
        ));
    };
    let branch = text("gitBranch")
        .ok_or_else(|| format!("{at}: a request without its gitBranch — not decided"))?;
    Ok(Some(Request {
        id: id.to_owned(),
        at: at.to_owned(),
        branch: branch.to_owned(),
        time: text("timestamp").map(str::to_owned),
        source: source.clone(),
        usage,
    }))
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

/// The items of the line's `message.content` of the type given.
fn content<'a>(object: &'a Value, kind: &'a str) -> impl Iterator<Item = &'a Value> {
    object
        .get("message")
        .and_then(|message| message.get("content"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(move |item| item.get("type").and_then(Value::as_str) == Some(kind))
}

/// The tool calls the line makes, by their identifier and the tool's name.
fn calls(object: &Value) -> Vec<(String, String)> {
    content(object, "tool_use")
        .filter_map(|item| {
            let field = |key: &str| item.get(key).and_then(Value::as_str).map(str::to_owned);
            Some((field("id")?, field("name")?))
        })
        .collect()
}

/// The tool outputs the line returns, by the call's identifier and the bytes of their text.
fn results(object: &Value) -> Vec<(String, u64)> {
    content(object, "tool_result")
        .filter_map(|item| {
            let id = item.get("tool_use_id").and_then(Value::as_str)?.to_owned();
            let size = match item.get("content") {
                Some(Value::String(text)) => text.len(),
                Some(Value::Array(parts)) => parts
                    .iter()
                    .filter_map(|part| part.get("text").and_then(Value::as_str))
                    .map(str::len)
                    .sum(),
                _ => 0,
            };
            Some((id, u64::try_from(size).unwrap_or(u64::MAX)))
        })
        .collect()
}

/// The longest directory name Claude Code keeps whole; a longer one is cut there and given a hash.
const KEY_LENGTH: usize = 200;

/// The name Claude Code gives the directory of a working tree's transcripts under `projects/`, as
/// its release 2.1.325 does: each UTF-16 unit of the path but an ASCII letter or digit replaced by
/// `-`; a name longer than 200 units cut there and followed by `-` and the hash of the path, the
/// 32-bit `h * 31 + unit` over its UTF-16 units, without its sign, in base 36. `path` is the
/// working tree's path as the system writes it, with `\` on Windows.
#[must_use]
pub fn directory_key(path: &str) -> String {
    let units: Vec<u16> = path.encode_utf16().collect();
    let key: String = units
        .iter()
        .map(|&unit| match u8::try_from(unit) {
            Ok(byte) if byte.is_ascii_alphanumeric() => char::from(byte),
            _ => '-',
        })
        .collect();
    if key.len() <= KEY_LENGTH {
        return key;
    }
    let hash = units.iter().fold(0i32, |h, &unit| {
        h.wrapping_shl(5)
            .wrapping_sub(h)
            .wrapping_add(i32::from(unit))
    });
    format!(
        "{}-{}",
        &key[..KEY_LENGTH],
        base36(i64::from(hash).unsigned_abs())
    )
}

/// The number in base 36, in lower-case letters, as JavaScript's `toString(36)` writes it.
fn base36(mut n: u64) -> String {
    let mut digits = Vec::new();
    loop {
        let digit = u32::try_from(n % 36).expect("below 36");
        digits.push(char::from_digit(digit, 36).expect("a digit of base 36"));
        n /= 36;
        if n == 0 {
            break;
        }
    }
    digits.iter().rev().collect()
}

/// The subagent its `agent-*.meta.json` describes, none where it names no type.
#[must_use]
pub fn subagent(bytes: &[u8]) -> Option<Subagent> {
    let object: Value = serde_json::from_slice(bytes).ok()?;
    let field = |key: &str| object.get(key).and_then(Value::as_str).map(str::to_owned);
    Some(Subagent {
        kind: field("agentType")?,
        description: field("description").unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINE: &str = r#"{"type":"assistant","requestId":"req_1","gitBranch":"b","timestamp":"2026-11-16T10:00:00.000Z","message":{"content":[{"type":"tool_use","id":"t1","name":"Bash","input":{"command":"CANARY"}}],"usage":{"input_tokens":2,"cache_read_input_tokens":30,"cache_creation_input_tokens":400,"output_tokens":5000}}}"#;

    fn read(text: &str) -> Result<Lines, String> {
        lines("s/t.jsonl", text.as_bytes(), &Source::default())
    }

    #[test]
    fn an_assistant_line_is_a_request_with_its_usage_branch_and_time() {
        let found = read(&format!("{LINE}\n")).expect("reads");
        assert_eq!(
            found.requests,
            [Request {
                id: "req_1".to_owned(),
                at: "s/t.jsonl:1".to_owned(),
                branch: "b".to_owned(),
                time: Some("2026-11-16T10:00:00.000Z".to_owned()),
                source: Source::default(),
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
    fn a_tool_result_is_measured_by_its_tool_and_size() {
        let result = r#"{"type":"user","gitBranch":"b","message":{"content":[{"type":"tool_result","tool_use_id":"t1","content":"12345"},{"type":"tool_result","tool_use_id":"t9","content":[{"type":"text","text":"abc"}]}]}}"#;
        let found = read(&format!("{LINE}\n{result}\n")).expect("reads");
        let outputs: Vec<(&str, u64)> = found
            .tool_outputs
            .iter()
            .map(|o| (o.tool.as_str(), o.size))
            .collect();
        assert_eq!(outputs, [("Bash", 5), ("unknown", 3)]);
    }

    #[test]
    fn a_tool_output_without_its_branch_is_refused() {
        let result = r#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t1","content":"x"}]}}"#;
        assert!(
            read(result)
                .unwrap_err()
                .contains("a tool's output without its gitBranch")
        );
    }

    #[test]
    fn other_lines_and_blank_lines_are_no_requests() {
        let text =
            "{\"type\":\"user\",\"message\":{\"content\":\"x\"}}\n\n  \n{\"type\":\"system\"}";
        assert_eq!(read(text), Ok(Lines::default()));
    }

    #[test]
    fn a_line_of_the_harness_without_request_and_tokens_is_none() {
        let text = r#"{"type":"assistant","message":{"model":"<synthetic>","usage":{"input_tokens":0,"cache_read_input_tokens":0,"cache_creation_input_tokens":0,"output_tokens":0}}}"#;
        assert_eq!(read(text), Ok(Lines::default()));
    }

    #[test]
    fn what_cannot_be_read_names_its_file_and_line() {
        let refused = |text: &str| read(text).expect_err("refused");
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
        let refusal = read("{\"type\":\"assistant\",\"secret\":\"CANARY\"").unwrap_err();
        assert!(!refusal.contains("CANARY"), "{refusal}");
    }

    /// The names Claude Code 2.1.325 gives these paths, computed with the functions of its
    /// `extension.js` (`mS`, `_S`, `o3`) run verbatim under Node, not with the code under test.
    #[test]
    fn the_directory_key_is_the_one_claude_code_writes() {
        let long = format!(
            "/home/user/{}project",
            "a-very-long-directory-name/".repeat(9)
        );
        let cases = [
            ("/workspaces/nuc-tools", "-workspaces-nuc-tools".to_owned()),
            (
                "C:\\Users\\Ana\\nuc.tools",
                "C--Users-Ana-nuc-tools".to_owned(),
            ),
            (
                "/private/var/folders/x/T/nuc",
                "-private-var-folders-x-T-nuc".to_owned(),
            ),
            ("/home/jürgen/projekt", "-home-j-rgen-projekt".to_owned()),
            ("/home/u/proj😀", "-home-u-proj--".to_owned()),
            (
                long.as_str(),
                format!(
                    "-home-user-{}-yj6nb2",
                    "a-very-long-directory-name-".repeat(7)
                ),
            ),
        ];
        for (path, key) in &cases {
            assert_eq!(directory_key(path), *key, "{path}");
        }
        let z = format!("/home/u/{}", "z".repeat(300));
        assert_eq!(
            directory_key(&z),
            format!("-home-u-{}-hbww3f", "z".repeat(192))
        );
    }

    #[test]
    fn a_subagent_is_named_by_its_type_and_description() {
        let meta = br#"{"agentType":"general-purpose","description":"Review","toolUseId":"x"}"#;
        assert_eq!(
            subagent(meta),
            Some(Subagent {
                kind: "general-purpose".to_owned(),
                description: "Review".to_owned()
            })
        );
        assert_eq!(subagent(b"{}"), None);
    }
}
