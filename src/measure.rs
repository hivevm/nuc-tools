//! The counting of `nuc measure`: the tokens a harness's sessions read and wrote, as the Measures
//! line of a hand-off counts them, and where they went (API-measure-reads-harness-transcripts).
//! Pure: the shell reads the transcripts and hands their bytes over, an adapter per harness turns
//! them into requests and tool outputs, and the counting knows no harness.

pub mod claude_code;

use std::collections::BTreeMap;
use std::fmt::Write as _;

/// The tokens of one request as the harness reports them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Usage {
    /// The input tokens not read from the cache.
    pub input: u64,
    /// The input tokens read from the cache.
    pub cache_read: u64,
    /// The input tokens written to the cache.
    pub cache_written: u64,
    /// The output tokens.
    pub output: u64,
}

impl Usage {
    /// What the request read and wrote: the four counts summed.
    #[must_use]
    pub fn total(self) -> u64 {
        // `add` refuses a usage whose four counts sum past u64, so this sum does not saturate on a
        // usage `measure` returns.
        self.input
            .saturating_add(self.cache_read)
            .saturating_add(self.cache_written)
            .saturating_add(self.output)
    }

    /// Each count the larger of the two: the lines of one request may report it as it streams,
    /// so the largest count of each is the request's.
    #[must_use]
    fn largest(self, other: Self) -> Self {
        Self {
            input: self.input.max(other.input),
            cache_read: self.cache_read.max(other.cache_read),
            cache_written: self.cache_written.max(other.cache_written),
            output: self.output.max(other.output),
        }
    }

    /// The counts of both summed, none where a sum, or the total of the four, overflows.
    fn add(self, other: Self) -> Option<Self> {
        let sum = Self {
            input: self.input.checked_add(other.input)?,
            cache_read: self.cache_read.checked_add(other.cache_read)?,
            cache_written: self.cache_written.checked_add(other.cache_written)?,
            output: self.output.checked_add(other.output)?,
        };
        sum.input
            .checked_add(sum.cache_read)?
            .checked_add(sum.cache_written)?
            .checked_add(sum.output)?;
        Some(sum)
    }
}

/// Where a line was written: the session, and the subagent of that session, none for the
/// session's own lines.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Source {
    pub session: String,
    pub subagent: Option<String>,
}

/// One line that reports a request of the model: its identifier, the branch the session was on,
/// its time, where it was written, and the usage it reports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub id: String,
    /// The transcript and line it stands at, `<name>:<line>`, for a refusal.
    pub at: String,
    pub branch: String,
    pub time: Option<String>,
    pub source: Source,
    pub usage: Usage,
}

/// The output of one tool call, by the tool's name and its size in bytes, never its text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolOutput {
    pub tool: String,
    /// The transcript and line it stands at, `<name>:<line>`, for a refusal.
    pub at: String,
    pub size: u64,
    pub branch: String,
    pub time: Option<String>,
    pub source: Source,
}

/// What an adapter reads out of a transcript.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Lines {
    pub requests: Vec<Request>,
    pub tool_outputs: Vec<ToolOutput>,
}

/// A subagent as its harness names it: its type and the description it was started with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Subagent {
    pub kind: String,
    pub description: String,
}

/// The harnesses whose transcripts `nuc measure` reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Harness {
    ClaudeCode,
}

impl Harness {
    /// The harness an option names, none for a name `nuc` knows no adapter for.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        (name == "claude-code").then_some(Self::ClaudeCode)
    }

    /// The requests and tool outputs the lines of one transcript report, written at `source`.
    /// `name` names the transcript in a refusal.
    ///
    /// # Errors
    ///
    /// The refusal, which names the file and the line and nothing of the transcript's text.
    pub fn lines(self, name: &str, bytes: &[u8], source: &Source) -> Result<Lines, String> {
        match self {
            Self::ClaudeCode => claude_code::lines(name, bytes, source),
        }
    }

    /// The subagent its harness describes in `bytes`, none where they do not describe one.
    #[must_use]
    pub fn subagent(self, bytes: &[u8]) -> Option<Subagent> {
        match self {
            Self::ClaudeCode => claude_code::subagent(bytes),
        }
    }
}

/// An instant in UTC, `YYYY-MM-DD` or `YYYY-MM-DDTHH:MM[:SS[.f…]]Z`, in the one form
/// `YYYY-MM-DDTHH:MM:SS.fff` that orders as its text does; none for another form.
#[must_use]
pub fn instant(text: &str) -> Option<String> {
    let form = b"dddd-dd-ddTdd:dd:dd.";
    let bytes = text.as_bytes();
    let fits = |n: usize| {
        n <= bytes.len()
            && bytes[..n].iter().zip(form).all(|(&b, &f)| {
                if f == b'd' {
                    b.is_ascii_digit()
                } else {
                    b == f
                }
            })
    };
    let two = |at: usize| (bytes[at] - b'0') * 10 + (bytes[at + 1] - b'0');
    let real = |n: usize| {
        (1..=12).contains(&two(5))
            && (1..=days_in(two(5), &bytes[..4])).contains(&two(8))
            && (n < 16 || (two(11) < 24 && two(14) < 60))
            && (n < 19 || two(17) < 60)
    };
    if bytes.len() == 10 && fits(10) {
        return real(10).then(|| format!("{text}T00:00:00.000"));
    }
    let time = text.strip_suffix('Z')?;
    let canonical = match time.len() {
        16 if fits(16) => Some(format!("{time}:00.000")),
        19 if fits(19) => Some(format!("{time}.000")),
        n if n > 20 && fits(20) && time[20..].bytes().all(|b| b.is_ascii_digit()) => {
            let fraction: String = time[20..].chars().chain("000".chars()).take(3).collect();
            Some(format!("{}{fraction}", &time[..20]))
        }
        _ => None,
    }?;
    real(time.len()).then_some(canonical)
}

/// The days of the month in the year, its four digits given.
fn days_in(month: u8, year: &[u8]) -> u8 {
    let year = year.iter().fold(0u32, |n, &b| n * 10 + u32::from(b - b'0'));
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// The requests and tool outputs that count: on one of the branches, and inside the time range,
/// from `since` on and before `until`, where one is given, each in the form `instant` gives.
#[derive(Clone, Debug, Default)]
pub struct Selection {
    pub branches: Vec<String>,
    pub since: Option<String>,
    pub until: Option<String>,
}

impl Selection {
    /// Whether a line on `branch` at `time` counts.
    ///
    /// # Errors
    ///
    /// A time range is given and the line has no time of the form `instant` reads.
    fn takes(&self, branch: &str, time: Option<&str>, at: &str) -> Result<bool, String> {
        if !self.branches.iter().any(|b| b == branch) {
            return Ok(false);
        }
        if self.since.is_none() && self.until.is_none() {
            return Ok(true);
        }
        let time = time.and_then(instant).ok_or_else(|| {
            format!(
                "{at}: a line without a time of the form 2026-01-31T12:00:00.000Z, so the time range cannot be applied — not decided"
            )
        })?;
        let after = self
            .since
            .as_deref()
            .is_none_or(|since| time.as_str() >= since);
        let before = self
            .until
            .as_deref()
            .is_none_or(|until| time.as_str() < until);
        Ok(after && before)
    }
}

/// What the selected requests and tool outputs read, wrote, and returned.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Measures {
    /// The tokens of every request, each counted once.
    pub total: Usage,
    /// The tokens by where they were written: a session's own, and each of its subagents'.
    pub by_source: BTreeMap<Source, Usage>,
    /// By tool name, its calls and the bytes of their outputs.
    pub by_tool: BTreeMap<String, (u64, u64)>,
    /// Each tool output, the largest first.
    pub outputs: Vec<ToolOutput>,
}

/// The measures of the selection, each request counted once with the largest usage its lines
/// report; its first line decides whether it is selected and where it was written. None where no
/// request counts.
///
/// # Errors
///
/// A line the selection cannot place in its time range.
pub fn measure(lines: &Lines, selection: &Selection) -> Result<Option<Measures>, String> {
    let mut requests: BTreeMap<&str, (&Request, Usage)> = BTreeMap::new();
    for request in &lines.requests {
        let entry = requests
            .entry(&request.id)
            .or_insert((request, Usage::default()));
        entry.1 = entry.1.largest(request.usage);
    }
    let mut selected = Vec::new();
    for (first, usage) in requests.into_values() {
        if selection.takes(&first.branch, first.time.as_deref(), &first.at)? {
            selected.push((&first.source, usage));
        }
    }
    if selected.is_empty() {
        return Ok(None);
    }
    let overflow =
        || "the counts of the selection sum past what nuc can count — not decided".to_owned();
    let mut measures = Measures::default();
    for (source, usage) in selected {
        measures.total = measures.total.add(usage).ok_or_else(overflow)?;
        let entry = measures.by_source.entry(source.clone()).or_default();
        *entry = entry.add(usage).ok_or_else(overflow)?;
    }
    for output in &lines.tool_outputs {
        if selection.takes(&output.branch, output.time.as_deref(), &output.at)? {
            let (calls, bytes) = measures.by_tool.entry(output.tool.clone()).or_default();
            *calls += 1;
            *bytes = bytes.saturating_add(output.size);
            measures.outputs.push(output.clone());
        }
    }
    measures
        .outputs
        .sort_by(|a, b| b.size.cmp(&a.size).then(a.source.cmp(&b.source)));
    Ok(Some(measures))
}

/// How many of the costliest subagents and the largest tool outputs the breakdown names.
const NAMED: usize = 3;

/// The breakdown of the measures: each session's own tokens and each subagent's, in the four parts
/// the harness reports; each tool's calls and output bytes; the costliest subagents and the
/// largest tool outputs. Subagents are named by type and description, as `subagents` gives them
/// by source, and nothing else of a transcript is named.
#[must_use]
pub fn breakdown(measures: &Measures, subagents: &BTreeMap<Source, Subagent>) -> String {
    let mut text = String::new();
    let name = |source: &Source| {
        let session = printable(&source.session);
        match (&source.subagent, subagents.get(source)) {
            (None, _) => format!("session {session}"),
            (Some(id), None) => format!("  subagent {} in session {session}", printable(id)),
            (Some(_), Some(agent)) => format!(
                "  subagent {} \"{}\" in session {session}",
                printable(&agent.kind),
                printable(&agent.description)
            ),
        }
    };
    for (source, usage) in &measures.by_source {
        let _ = writeln!(
            text,
            "{}: input {} cache read {} cache written {} output {}",
            name(source),
            usage.input,
            usage.cache_read,
            usage.cache_written,
            usage.output
        );
    }
    for (tool, (calls, bytes)) in &measures.by_tool {
        let _ = writeln!(
            text,
            "tool {}: calls {calls} output {bytes} bytes",
            printable(tool)
        );
    }
    let mut costliest: Vec<(&Source, &Usage)> = measures
        .by_source
        .iter()
        .filter(|(source, _)| source.subagent.is_some())
        .collect();
    costliest.sort_by(|a, b| b.1.total().cmp(&a.1.total()).then(a.0.cmp(b.0)));
    let _ = writeln!(text, "costliest subagents:");
    for (source, usage) in costliest.into_iter().take(NAMED) {
        let _ = writeln!(text, "{}: {}", name(source), usage.total());
    }
    let _ = writeln!(text, "largest tool outputs:");
    for output in measures.outputs.iter().take(NAMED) {
        let _ = writeln!(
            text,
            "  {} in session {}: {} bytes",
            printable(&output.tool),
            printable(&output.source.session),
            output.size
        );
    }
    text
}

/// A draft of a template finding in the five fields of the template's issue form for findings,
/// for the human to complete and send. `release` is the Template release the README names, none
/// where it names none. Measures carries the tokens, their four parts, and the tools and sizes of
/// the largest outputs, an MCP tool as `mcp`, and nothing else: no session, subagent, branch, path,
/// or MCP server leaves with it (NUC-template-learns-from-projects).
#[must_use]
pub fn finding(measures: &Measures, release: Option<&str>) -> String {
    let total = measures.total;
    let largest: Vec<String> = measures
        .outputs
        .iter()
        .take(NAMED)
        .map(|output| format!("{} {} bytes", public_tool(&output.tool), output.size))
        .collect();
    let largest = if largest.is_empty() {
        "none".to_owned()
    } else {
        largest.join(", ")
    };
    let release = release.map_or_else(
        || "none: the README names no Template release".to_owned(),
        printable,
    );
    format!(
        "## Template release\n\n{release}\n\n\
         ## Concerns\n\n<!-- The rule, file, or skill of the template. -->\n\n\
         ## Finding\n\n<!-- What the work met. -->\n\n\
         ## Measures\n\n\
         {} (input {}, cache read {}, cache written {}, output {})\n\
         largest tool outputs: {largest}\n\n\
         ## Proposal\n\n<!-- What the template changes. -->\n\n",
        line(total.total()),
        total.input,
        total.cache_read,
        total.cache_written,
        total.output,
    )
}

/// A tool's name as a finding may carry it: an MCP tool, `mcp__<server>__<tool>`, as `mcp`, since
/// its server is the project's own; a harness's built-in tool by its name.
fn public_tool(tool: &str) -> String {
    if tool.starts_with("mcp__") {
        "mcp".to_owned()
    } else {
        printable(tool)
    }
}

/// The text with every control character a space, so that a description stays on its line.
fn printable(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}

/// The part of the Measures line `nuc measure` prints, ready to paste.
#[must_use]
pub fn line(tokens: u64) -> String {
    format!("tokens {tokens}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(session: &str, subagent: Option<&str>) -> Source {
        Source {
            session: session.to_owned(),
            subagent: subagent.map(str::to_owned),
        }
    }

    fn request(id: &str, branch: &str, time: &str, at: Source, output: u64) -> Request {
        Request {
            id: id.to_owned(),
            at: format!("t.jsonl:{id}"),
            branch: branch.to_owned(),
            time: Some(time.to_owned()),
            source: at,
            usage: Usage {
                input: 1,
                cache_read: 10,
                cache_written: 100,
                output,
            },
        }
    }

    fn on(branches: &[&str]) -> Selection {
        Selection {
            branches: branches.iter().map(|b| (*b).to_owned()).collect(),
            ..Selection::default()
        }
    }

    const T: &str = "2026-11-16T10:00:00.000Z";

    fn total(lines: &Lines, selection: &Selection) -> Option<u64> {
        measure(lines, selection)
            .expect("placed")
            .map(|m| m.total.total())
    }

    #[test]
    fn a_request_over_several_lines_counts_once_at_its_largest_usage() {
        let s = source("s", None);
        let requests = vec![
            request("r", "b", T, s.clone(), 8),
            request("r", "b", T, s.clone(), 935),
            request("r", "b", T, s, 900),
        ];
        let lines = Lines {
            requests,
            tool_outputs: Vec::new(),
        };
        assert_eq!(total(&lines, &on(&["b"])), Some(1 + 10 + 100 + 935));
    }

    #[test]
    fn the_branches_and_the_time_range_select() {
        let s = source("s", None);
        let requests = vec![
            request("r1", "a", "2026-11-15T23:59:59.999Z", s.clone(), 1),
            request("r2", "b", "2026-11-16T00:00:00.000Z", s.clone(), 2),
            request("r3", "c", "2026-11-17T00:00:00.000Z", s, 3),
        ];
        let lines = Lines {
            requests,
            tool_outputs: Vec::new(),
        };
        assert_eq!(total(&lines, &on(&["a", "c"])), Some(111 * 2 + 1 + 3));
        assert_eq!(total(&lines, &on(&["d"])), None);
        let day = Selection {
            since: instant("2026-11-16"),
            until: instant("2026-11-17"),
            ..on(&["a", "b", "c"])
        };
        assert_eq!(total(&lines, &day), Some(111 + 2));
    }

    #[test]
    fn a_request_whose_lines_straddle_the_range_is_decided_by_its_first() {
        let s = source("s", None);
        let requests = vec![
            request("r", "b", "2026-11-16T23:59:59.000Z", s.clone(), 1),
            request("r", "b", "2026-11-17T00:00:01.000Z", s, 50),
        ];
        let lines = Lines {
            requests,
            tool_outputs: Vec::new(),
        };
        let day = Selection {
            until: instant("2026-11-17"),
            ..on(&["b"])
        };
        assert_eq!(total(&lines, &day), Some(111 + 50));
        let next = Selection {
            since: instant("2026-11-17"),
            ..on(&["b"])
        };
        assert_eq!(total(&lines, &next), None);
    }

    #[test]
    fn a_time_range_refuses_a_line_without_its_time() {
        let mut r = request("r", "b", T, source("s", None), 1);
        r.time = None;
        let lines = Lines {
            requests: vec![r],
            tool_outputs: Vec::new(),
        };
        let range = Selection {
            since: instant("2026-01-01"),
            ..on(&["b"])
        };
        let refusal = measure(&lines, &range).expect_err("refused");
        assert!(
            refusal.starts_with("t.jsonl:r: a line without a time"),
            "{refusal}"
        );
        assert_eq!(total(&lines, &on(&["b"])), Some(112));
    }

    #[test]
    fn an_instant_is_a_date_or_a_utc_time() {
        for (text, form) in [
            ("2026-11-16", "2026-11-16T00:00:00.000"),
            ("2026-11-16T10:00Z", "2026-11-16T10:00:00.000"),
            ("2026-11-16T10:00:05Z", "2026-11-16T10:00:05.000"),
            ("2026-11-16T10:00:05.5Z", "2026-11-16T10:00:05.500"),
            (T, "2026-11-16T10:00:00.000"),
        ] {
            assert_eq!(instant(text).as_deref(), Some(form), "{text}");
        }
        assert!(instant("2026-11-16T10:00Z") > instant("2026-11-16T09:59:59.999Z"));
        for bad in [
            "2026-02-29",
            "2026-04-31",
            "2026-13-01",
            "2026-11-32",
            "2026-11-16T24:00Z",
            "2026-11-16T10:60Z",
            "2026-11-6",
            "2026-11-16T10:00",
            "2026-11-16T10:00:00+01:00",
            "yesterday",
            "",
        ] {
            assert_eq!(instant(bad), None, "{bad}");
        }
    }

    #[test]
    fn the_breakdown_sums_to_the_total_and_names_the_costliest() {
        let (own, agent) = (source("s", None), source("s", Some("agent-x")));
        let requests = vec![
            request("r1", "b", T, own.clone(), 1),
            request("r2", "b", T, agent.clone(), 1000),
        ];
        let tool_outputs = vec![ToolOutput {
            tool: "Bash".to_owned(),
            at: "t.jsonl:3".to_owned(),
            size: 42,
            branch: "b".to_owned(),
            time: Some(T.to_owned()),
            source: own,
        }];
        let lines = Lines {
            requests,
            tool_outputs,
        };
        let measures = measure(&lines, &on(&["b"]))
            .expect("placed")
            .expect("counted");
        let sum: u64 = measures.by_source.values().map(|u| u.total()).sum();
        assert_eq!(sum, measures.total.total());
        let names = BTreeMap::from([(
            agent,
            Subagent {
                kind: "review".to_owned(),
                description: "line\none".to_owned(),
            },
        )]);
        let text = breakdown(&measures, &names);
        assert!(
            text.contains("session s: input 1 cache read 10 cache written 100 output 1\n"),
            "{text}"
        );
        assert!(text.contains(
            "  subagent review \"line one\" in session s: input 1 cache read 10 cache written 100 output 1000\n"
        ));
        assert!(
            text.contains("tool Bash: calls 1 output 42 bytes\n"),
            "{text}"
        );
        assert!(
            text.contains(
                "costliest subagents:\n  subagent review \"line one\" in session s: 1111\n"
            )
        );
        assert!(
            text.contains("largest tool outputs:\n  Bash in session s: 42 bytes\n"),
            "{text}"
        );
    }

    #[test]
    fn the_finding_fills_the_five_fields_and_carries_no_source() {
        let agent = source("session-x", Some("agent-y"));
        let lines = Lines {
            requests: vec![request("r1", "b", T, agent.clone(), 4)],
            tool_outputs: vec![ToolOutput {
                tool: "Read".to_owned(),
                at: "t.jsonl:2".to_owned(),
                size: 9,
                branch: "b".to_owned(),
                time: None,
                source: agent,
            }],
        };
        let measures = measure(&lines, &on(&["b"]))
            .expect("placed")
            .expect("counted");
        let text = finding(&measures, Some("v0.11.1"));
        let headings: Vec<&str> = text.lines().filter(|l| l.starts_with("## ")).collect();
        assert_eq!(
            headings,
            [
                "## Template release",
                "## Concerns",
                "## Finding",
                "## Measures",
                "## Proposal"
            ]
        );
        assert!(text.contains("## Template release\n\nv0.11.1\n"), "{text}");
        let measures_field = "## Measures\n\n\
                              tokens 115 (input 1, cache read 10, cache written 100, output 4)\n\
                              largest tool outputs: Read 9 bytes\n\n## Proposal";
        assert!(text.contains(measures_field), "{text}");
        assert_eq!(public_tool("mcp__acme__query"), "mcp");
        assert!(
            !text.contains("session-x") && !text.contains("agent-y") && !text.contains("t.jsonl")
        );
        assert!(finding(&measures, None).contains("none: the README names no Template release"));
    }

    #[test]
    fn counts_that_sum_past_u64_are_refused() {
        let s = source("s", None);
        let mut big = request("r1", "b", T, s.clone(), u64::MAX - 200);
        big.usage.input = 0;
        let lines = Lines {
            requests: vec![big, request("r2", "b", T, s, 1)],
            tool_outputs: Vec::new(),
        };
        assert!(measure(&lines, &on(&["b"])).is_err());
        assert_eq!(
            instant("2024-02-29").as_deref(),
            Some("2024-02-29T00:00:00.000")
        );
    }

    #[test]
    fn the_line_is_the_measures_part_to_paste() {
        assert_eq!(line(1234), "tokens 1234");
    }

    #[test]
    fn only_claude_code_is_a_harness() {
        assert_eq!(Harness::parse("claude-code"), Some(Harness::ClaudeCode));
        assert_eq!(Harness::parse("Claude Code"), None);
    }
}
