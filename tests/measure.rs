//! `nuc measure` end to end, on fixture transcripts in a directory of the test's own beside a
//! fixture repository. Verifies: API-measure-reads-harness-transcripts
//!
//! The transcripts are synthetic lines in the shape Claude Code writes them: a request spans
//! several lines as it streams, a subagent's transcript lies under its session's directory, and
//! every text field holds a canary that no output may carry.

mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

use support::{Fixture, stderr, stdout};

/// A string in every text field of the fixtures, which no output may carry.
const CANARY: &str = "CANARY-7f3c";

fn code(output: &Output) -> Option<i32> {
    output.status.code()
}

/// A directory of transcripts, outside every repository, removed with the test.
struct Transcripts {
    dir: PathBuf,
}

impl Transcripts {
    fn new(name: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir()
            .join("nuc-tests")
            .join(format!("{name}-transcripts-{}-{n}", process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("the transcripts' directory is created");
        Self { dir }
    }

    fn write(&self, path: &str, lines: &[String]) -> &Self {
        let path = self.dir.join(path);
        fs::create_dir_all(path.parent().expect("a file has a parent")).expect("directories");
        fs::write(path, lines.join("\n") + "\n").expect("the transcript is written");
        self
    }

    fn path(&self) -> &str {
        self.dir.to_str().expect("a UTF-8 path")
    }
}

impl Drop for Transcripts {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

/// A line of a request on `branch`, its four counts as given, its text fields the canary.
fn request(id: &str, branch: &str, counts: [u64; 4]) -> String {
    let [input, read, written, output] = counts;
    format!(
        r#"{{"type":"assistant","requestId":"{id}","gitBranch":"{branch}","cwd":"/{CANARY}","message":{{"model":"m","content":[{{"type":"text","text":"{CANARY}"}}],"usage":{{"input_tokens":{input},"cache_read_input_tokens":{read},"cache_creation_input_tokens":{written},"output_tokens":{output}}}}}}}"#
    )
}

/// A line the user wrote, its text the canary.
fn prompt() -> String {
    format!(r#"{{"type":"user","message":{{"content":"{CANARY}"}}}}"#)
}

/// A repository on `branch`, with a commit so that the branch exists.
fn repo_on(name: &str, branch: &str) -> Fixture {
    let fixture = Fixture::new(name);
    fixture
        .write("README.md", "# Fixture\n")
        .commit("Initial commit");
    fixture.git(&["switch", "-q", "-c", branch]);
    fixture
}

fn measure(fixture: &Fixture, transcripts: &Transcripts) -> Output {
    fixture.nuc(&["measure", "--transcripts", transcripts.path()])
}

fn assert_no_canary(output: &Output) {
    assert!(!stdout(output).contains(CANARY), "{}", stdout(output));
    assert!(!stderr(output).contains(CANARY), "{}", stderr(output));
}

#[test]
fn a_request_over_several_lines_counts_once_at_its_largest_counts() {
    let fixture = repo_on("measure-once", "b");
    let transcripts = Transcripts::new("measure-once");
    transcripts.write(
        "s1.jsonl",
        &[
            prompt(),
            request(&format!("r1-{CANARY}"), "b", [2, 1000, 300, 8]),
            request(&format!("r1-{CANARY}"), "b", [3, 999, 301, 935]),
            request(&format!("r1-{CANARY}"), "b", [2, 1001, 300, 900]),
            request(
                &format!("r2-{CANARY}"),
                &format!("other-{CANARY}"),
                [9, 9, 9, 9],
            ),
        ],
    );
    let output = measure(&fixture, &transcripts);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    // Each count the largest its lines report, as a request's lines report it while it streams.
    assert_eq!(
        stdout(&output),
        format!("tokens {}\n", 3 + 1001 + 301 + 935)
    );
    assert_no_canary(&output);
}

#[test]
fn a_subagent_counts_with_its_session_and_other_branches_do_not() {
    let fixture = repo_on("measure-sub", "b");
    let transcripts = Transcripts::new("measure-sub");
    transcripts
        .write(
            "s1.jsonl",
            &[
                request("r1", "b", [1, 0, 0, 1]),
                request("r2", "a", [50, 0, 0, 50]),
            ],
        )
        .write(
            "s1/subagents/agent-x.jsonl",
            &[request("r3", "b", [10, 0, 0, 10])],
        )
        .write(
            "s1/subagents/agent-x.meta.json",
            &[format!(r#"{{"description":"{CANARY}"}}"#)],
        )
        .write("s1/tool-results/t.txt", &[CANARY.to_owned()])
        .write("s2.jsonl", &[request("r4", "a", [7, 0, 0, 7])]);
    let output = measure(&fixture, &transcripts);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    assert_eq!(stdout(&output), "tokens 22\n");
    assert_no_canary(&output);
    let output = fixture.nuc(&[
        "measure",
        "--transcripts",
        transcripts.path(),
        "--breakdown",
    ]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    assert!(
        stdout(&output).contains("subagent agent-x in session s1: "),
        "{}",
        stdout(&output)
    );
    assert_no_canary(&output);
}

#[test]
fn what_cannot_be_read_stops_with_its_file_and_line() {
    let fixture = repo_on("measure-unread", "b");
    let cases: [(&str, String, &str); 3] = [
        (
            "broken",
            format!("{{\"type\":\"user\",\"x\":\"{CANARY}"),
            "s1.jsonl:2: ",
        ),
        (
            "no-usage",
            format!(
                r#"{{"type":"assistant","requestId":"r","gitBranch":"b","message":{{"content":"{CANARY}"}}}}"#
            ),
            "s1.jsonl:2: a request without the four counts",
        ),
        (
            "nul",
            "\0\0\0\0".to_owned(),
            "s1.jsonl:2: a line that is no JSON",
        ),
    ];
    for (name, line, expected) in cases {
        let transcripts = Transcripts::new(&format!("measure-{name}"));
        transcripts.write("s1.jsonl", &[request("r1", "b", [1, 1, 1, 1]), line]);
        let output = measure(&fixture, &transcripts);
        assert_eq!(code(&output), Some(2), "{name}");
        assert_eq!(stdout(&output), "", "{name}");
        assert!(
            stderr(&output).contains(expected),
            "{name}: {}",
            stderr(&output)
        );
        assert_no_canary(&output);
    }
}

#[test]
fn no_request_on_the_branch_stops_without_a_figure() {
    let fixture = repo_on("measure-none", "b");
    let transcripts = Transcripts::new("measure-none");
    transcripts.write("s1.jsonl", &[request("r1", "a", [1, 1, 1, 1])]);
    let output = measure(&fixture, &transcripts);
    assert_eq!(code(&output), Some(2));
    assert_eq!(stdout(&output), "");
    assert!(
        stderr(&output).contains("no request of the harness on the branch b"),
        "{}",
        stderr(&output)
    );

    let missing = fixture.nuc(&[
        "measure",
        "--transcripts",
        &format!("{}-gone", transcripts.path()),
    ]);
    assert_eq!(code(&missing), Some(2));
    assert!(
        stderr(&missing).contains("no transcripts of the harness at"),
        "{}",
        stderr(&missing)
    );
}

#[cfg(unix)]
#[test]
fn a_symbolic_link_where_a_transcript_is_read_stops() {
    let fixture = repo_on("measure-link", "b");
    let outside = Transcripts::new("measure-link-outside");
    outside.write("elsewhere.jsonl", &[request("r1", "b", [1, 1, 1, 1])]);
    let transcripts = Transcripts::new("measure-link");
    std::os::unix::fs::symlink(
        outside.dir.join("elsewhere.jsonl"),
        transcripts.dir.join("s1.jsonl"),
    )
    .expect("the link is made");
    let output = measure(&fixture, &transcripts);
    assert_eq!(code(&output), Some(2));
    assert_eq!(stdout(&output), "");
    assert!(
        stderr(&output).contains("a symbolic link or no regular file"),
        "{}",
        stderr(&output)
    );
}

#[cfg(unix)]
#[test]
fn a_symbolic_link_anywhere_in_the_directory_stops_and_nothing_is_passed_over() {
    let fixture = repo_on("measure-links", "b");
    let outside = Transcripts::new("measure-links-outside");
    outside.write(
        "s1/subagents/agent-x.jsonl",
        &[request("r9", "b", [5, 5, 5, 5])],
    );

    let session = Transcripts::new("measure-link-session");
    session.write("s0.jsonl", &[request("r1", "b", [1, 0, 0, 1])]);
    std::os::unix::fs::symlink(outside.dir.join("s1"), session.dir.join("s1"))
        .expect("the link is made");

    let subagents = Transcripts::new("measure-link-subagents");
    subagents.write("s1.jsonl", &[request("r1", "b", [1, 0, 0, 1])]);
    fs::create_dir_all(subagents.dir.join("s1")).expect("directory");
    std::os::unix::fs::symlink(
        outside.dir.join("s1/subagents"),
        subagents.dir.join("s1/subagents"),
    )
    .expect("link");

    let agent = Transcripts::new("measure-link-agent");
    agent.write(
        "s1/subagents/agent-y.jsonl",
        &[request("r1", "b", [1, 0, 0, 1])],
    );
    std::os::unix::fs::symlink(
        outside.dir.join("s1/subagents/agent-x.jsonl"),
        agent.dir.join("s1/subagents/agent-x.jsonl"),
    )
    .expect("link");

    for transcripts in [&session, &subagents, &agent] {
        let output = measure(&fixture, transcripts);
        assert_eq!(code(&output), Some(2), "{}", transcripts.path());
        assert_eq!(stdout(&output), "");
        assert!(
            stderr(&output).contains("a symbolic link"),
            "{}",
            stderr(&output)
        );
    }

    let whole = Transcripts::new("measure-link-whole");
    let to_outside = whole.dir.join("projects");
    std::os::unix::fs::symlink(&outside.dir, &to_outside).expect("link");
    let plain = to_outside.to_str().expect("UTF-8").to_owned();
    for named in [plain.clone(), format!("{plain}/")] {
        let output = fixture.nuc(&["measure", "--transcripts", &named]);
        assert_eq!(code(&output), Some(2), "{named}");
        assert!(
            stderr(&output).contains("a symbolic link"),
            "{named}: {}",
            stderr(&output)
        );
    }
}

#[test]
fn without_a_directory_named_it_reads_the_harness_s_own_for_the_working_tree() {
    let fixture = repo_on("measure-default", "b");
    let config = Transcripts::new("measure-default-config");
    let root = support::stdout(&fixture.git(&["rev-parse", "--show-toplevel"]));
    let key: String = root
        .trim_end()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    config.write(
        &format!("projects/{key}/s1.jsonl"),
        &[request("r1", "b", [3, 0, 0, 4])],
    );
    let output = fixture.nuc_with(&[("CLAUDE_CONFIG_DIR", config.path())], &["measure"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    assert_eq!(stdout(&output), "tokens 7\n");

    let home = Transcripts::new("measure-default-home");
    home.write(
        &format!(".claude/projects/{key}/s1.jsonl"),
        &[request("r1", "b", [5, 0, 0, 6])],
    );
    let output = fixture.nuc_with(
        &[("HOME", home.path()), ("USERPROFILE", home.path())],
        &["measure"],
    );
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    assert_eq!(stdout(&output), "tokens 11\n");
}

#[test]
fn an_unknown_harness_or_option_is_a_usage_error() {
    let fixture = repo_on("measure-usage", "b");
    for args in [
        &["measure", "--harness", "other"][..],
        &["measure", "--transcripts"],
        &["measure", "--since", "yesterday"],
        &["measure", "--until", "2026-11-16T10:00"],
        &["measure", "--breakdown", "--breakdown"],
        &["measure", "--finding", "--finding"],
        &["measure", "--since", "2026-11-17", "--until", "2026-11-16"],
        &["measure", "--branch", ""],
    ] {
        let output = fixture.nuc(args);
        assert_eq!(code(&output), Some(2), "{args:?}");
        assert!(
            stderr(&output).contains("usage: nuc check"),
            "{}",
            stderr(&output)
        );
    }
}

/// The directory the fixtures write is outside the repository: the read outside is the one the
/// ADR allows, and nothing is written there or in the repository.
#[test]
fn measure_writes_nothing() {
    let fixture = repo_on("measure-writes", "b");
    let transcripts = Transcripts::new("measure-writes");
    transcripts.write("s1.jsonl", &[request("r1", "b", [1, 1, 1, 1])]);
    let before = (
        support::snapshot(fixture.root()),
        support::snapshot(Path::new(transcripts.path())),
    );
    let output = measure(&fixture, &transcripts);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    let after = (
        support::snapshot(fixture.root()),
        support::snapshot(Path::new(transcripts.path())),
    );
    assert!(support::changed(&before.0, &after.0).is_empty());
    assert!(support::changed(&before.1, &after.1).is_empty());
}

#[test]
fn named_branches_and_a_time_range_select_the_requests() {
    let fixture = repo_on("measure-select", "b");
    let transcripts = Transcripts::new("measure-select");
    let at = |id: &str, branch: &str, time: &str, n: u64| {
        request(id, branch, [n, 0, 0, 0]).replacen(
            "{\"type\"",
            &format!("{{\"timestamp\":\"{time}\",\"type\""),
            1,
        )
    };
    transcripts.write(
        "s1.jsonl",
        &[
            at("r1", "a", "2026-11-15T23:59:59.999Z", 1),
            at("r2", "b", "2026-11-16T08:00:00.000Z", 10),
            at("r3", "c", "2026-11-16T12:00:00.000Z", 100),
            at("r4", "c", "2026-11-17T00:00:00.000Z", 1000),
        ],
    );
    let run = |args: &[&str]| {
        let mut all = vec!["measure", "--transcripts", transcripts.path()];
        all.extend_from_slice(args);
        fixture.nuc(&all)
    };
    // Each named branch is added to the current one, b.
    let output = run(&["--branch", "a", "--branch", "c"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    assert_eq!(stdout(&output), "tokens 1111\n");
    let output = run(&["--branch", "b"]);
    assert_eq!(
        stdout(&output),
        "tokens 10\n",
        "the current branch named again counts once"
    );
    let output = run(&[
        "--branch",
        "a",
        "--branch",
        "b",
        "--branch",
        "c",
        "--since",
        "2026-11-16",
        "--until",
        "2026-11-17",
    ]);
    assert_eq!(stdout(&output), "tokens 110\n", "{}", stderr(&output));
    let output = run(&["--since", "2026-11-16T09:00Z"]);
    assert_eq!(
        code(&output),
        Some(2),
        "only b, the current branch, whose request is earlier"
    );
    assert_no_canary(&output);
}

#[test]
fn the_breakdown_sums_to_the_tokens_and_names_no_content() {
    let fixture = repo_on("measure-breakdown", "b");
    let transcripts = Transcripts::new("measure-breakdown");
    let call = |id: &str| {
        format!(
            r#"{{"type":"assistant","requestId":"{id}","gitBranch":"b","message":{{"content":[{{"type":"tool_use","id":"t-{id}","name":"Bash","input":{{"command":"{CANARY}"}}}}],"usage":{{"input_tokens":1,"cache_read_input_tokens":2,"cache_creation_input_tokens":3,"output_tokens":4}}}}}}"#
        )
    };
    let result = |id: &str, size: usize| {
        format!(
            r#"{{"type":"user","gitBranch":"b","message":{{"content":[{{"type":"tool_result","tool_use_id":"t-{id}","content":"{}"}}]}}}}"#,
            CANARY.repeat(size)
        )
    };
    let meta = |kind: &str| {
        vec![format!(
            r#"{{"agentType":"{kind}","description":"Review the change","toolUseId":"{CANARY}"}}"#
        )]
    };
    transcripts
        .write("s1.jsonl", &[call("r1"), result("r1", 2)])
        .write(
            "s1/subagents/agent-a.jsonl",
            &[
                request("r2", "b", [100, 0, 0, 900]),
                call("r3"),
                result("r3", 5),
            ],
        )
        .write("s1/subagents/agent-a.meta.json", &meta("reviewer"))
        .write(
            "s1/subagents/agent-b.jsonl",
            &[
                request("r5", "b", [1, 0, 0, 0]),
                call("r6"),
                result("r6", 1),
            ],
        )
        .write("s1/subagents/agent-b.meta.json", &meta("explore"))
        .write("s2.jsonl", &[request("r4", "b", [10, 20, 30, 40])])
        .write(
            "s2/subagents/agent-c.jsonl",
            &[
                request("r7", "b", [500, 0, 0, 0]),
                call("r8"),
                result("r8", 3),
            ],
        )
        .write(
            "s2/subagents/agent-d.jsonl",
            &[request("r9", "b", [50, 0, 0, 0])],
        );
    let output = fixture.nuc(&[
        "measure",
        "--transcripts",
        transcripts.path(),
        "--breakdown",
    ]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    let text = stdout(&output);
    let tokens = 10 + 1010 + 1 + 10 + 100 + 510 + 50;
    assert!(text.ends_with(&format!("tokens {tokens}\n")), "{text}");
    for expected in [
        "session s1: input 1 cache read 2 cache written 3 output 4\n",
        "  subagent reviewer \"Review the change\" in session s1: input 101 cache read 2 cache written 3 output 904\n",
        "session s2: input 10 cache read 20 cache written 30 output 40\n",
        "  subagent agent-d in session s2: input 50 cache read 0 cache written 0 output 0\n",
        "tool Bash: calls 4 output 121 bytes\n",
        "costliest subagents:\n  subagent reviewer \"Review the change\" in session s1: 1010\n  subagent agent-c in session s2: 510\n  subagent agent-d in session s2: 50\nlargest tool outputs:\n",
        "largest tool outputs:\n  Bash in session s1: 55 bytes\n  Bash in session s2: 33 bytes\n  Bash in session s1: 22 bytes\ntokens ",
    ] {
        assert!(text.contains(expected), "{expected:?} in\n{text}");
    }
    let sum: u64 = text
        .lines()
        .filter(|line| line.contains(": input "))
        .map(|line| {
            line.rsplit(": input ")
                .next()
                .expect("counts")
                .split(' ')
                .filter_map(|word| word.parse::<u64>().ok())
                .sum::<u64>()
        })
        .sum();
    assert_eq!(sum, tokens);
    assert_no_canary(&output);
}

#[test]
fn a_time_range_stops_at_a_line_without_its_time_and_names_it() {
    let fixture = repo_on("measure-no-time", "b");
    let transcripts = Transcripts::new("measure-no-time");
    transcripts.write(
        "s1/subagents/agent-x.jsonl",
        &[prompt(), request("r1", "b", [1, 1, 1, 1])],
    );
    let output = fixture.nuc(&[
        "measure",
        "--transcripts",
        transcripts.path(),
        "--since",
        "2026-01-01",
    ]);
    assert_eq!(code(&output), Some(2));
    assert_eq!(stdout(&output), "");
    assert!(
        stderr(&output).contains("s1/subagents/agent-x.jsonl:2: a line without a time"),
        "{}",
        stderr(&output)
    );
    assert_no_canary(&output);
}

#[cfg(unix)]
#[test]
fn a_linked_description_of_a_subagent_stops() {
    let fixture = repo_on("measure-link-meta", "b");
    let outside = Transcripts::new("measure-link-meta-outside");
    outside.write(
        "meta.json",
        &[r#"{"agentType":"x","description":"y"}"#.to_owned()],
    );
    let transcripts = Transcripts::new("measure-link-meta");
    transcripts.write(
        "s1/subagents/agent-x.jsonl",
        &[request("r1", "b", [1, 1, 1, 1])],
    );
    std::os::unix::fs::symlink(
        outside.dir.join("meta.json"),
        transcripts.dir.join("s1/subagents/agent-x.meta.json"),
    )
    .expect("the link is made");
    let output = measure(&fixture, &transcripts);
    assert_eq!(code(&output), Some(2));
    assert!(
        stderr(&output).contains("a symbolic link"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn the_finding_fills_the_five_fields_and_carries_nothing_of_the_project() {
    let branch = format!("b-{CANARY}");
    let fixture = Fixture::new("measure-finding");
    fixture
        .write(
            "README.md",
            "# Fixture\n\n- **Template release:** v0.11.1\n",
        )
        .commit("Initial commit");
    fixture.git(&["switch", "-q", "-c", &branch]);
    let transcripts = Transcripts::new(&format!("measure-finding-{CANARY}"));
    let session = format!("s-{CANARY}");
    let call = format!(
        r#"{{"type":"assistant","requestId":"r1","gitBranch":"{branch}","cwd":"/{CANARY}","message":{{"content":[{{"type":"tool_use","id":"t1","name":"Read","input":{{"file_path":"/{CANARY}"}}}},{{"type":"tool_use","id":"t2","name":"mcp__{CANARY}__query","input":{{}}}}],"usage":{{"input_tokens":1,"cache_read_input_tokens":2,"cache_creation_input_tokens":3,"output_tokens":4}}}}}}"#
    );
    let result = format!(
        r#"{{"type":"user","gitBranch":"{branch}","message":{{"content":[{{"type":"tool_result","tool_use_id":"t1","content":"{CANARY}"}},{{"type":"tool_result","tool_use_id":"t2","content":"abcd"}}]}}}}"#
    );
    transcripts
        .write(&format!("{session}.jsonl"), &[call, result])
        .write(
            &format!("{session}/subagents/agent-{CANARY}.jsonl"),
            &[request("r2", &branch, [10, 0, 0, 0])],
        )
        .write(
            &format!("{session}/subagents/agent-{CANARY}.meta.json"),
            &[format!(
                r#"{{"agentType":"reviewer","description":"{CANARY}"}}"#
            )],
        );
    let output = fixture.nuc(&["measure", "--transcripts", transcripts.path(), "--finding"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    let text = stdout(&output);
    let headings: Vec<&str> = text
        .lines()
        .filter(|line| line.starts_with("## "))
        .collect();
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
    assert!(
        text.starts_with("## Template release\n\nv0.11.1\n"),
        "{text}"
    );
    let measures_field = "## Measures\n\n\
                          tokens 20 (input 11, cache read 2, cache written 3, output 4)\n\
                          largest tool outputs: Read 11 bytes, mcp 4 bytes\n\n## Proposal";
    assert!(text.contains(measures_field), "{text}");
    assert!(text.ends_with("tokens 20\n"), "{text}");
    assert_no_canary(&output);
}

#[test]
fn the_finding_says_where_the_readme_names_no_release() {
    let fixture = repo_on("measure-finding-none", "b");
    let transcripts = Transcripts::new("measure-finding-none");
    transcripts.write("s1.jsonl", &[request("r1", "b", [1, 1, 1, 1])]);
    let output = fixture.nuc(&["measure", "--transcripts", transcripts.path(), "--finding"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    assert!(
        stdout(&output)
            .contains("## Template release\n\nnone: the README names no Template release\n"),
        "{}",
        stdout(&output)
    );
}

#[test]
fn the_breakdown_and_the_finding_stand_before_the_line() {
    let fixture = repo_on("measure-both", "b");
    let transcripts = Transcripts::new("measure-both");
    transcripts.write("s1.jsonl", &[request("r1", "b", [1, 1, 1, 1])]);
    let args = [
        "measure",
        "--transcripts",
        transcripts.path(),
        "--finding",
        "--breakdown",
    ];
    let output = fixture.nuc(&args);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    let text = stdout(&output);
    let breakdown = text.find("session s1: ").expect("the breakdown");
    let draft = text.find("## Template release").expect("the draft");
    assert!(breakdown < draft, "{text}");
    assert!(text.ends_with("\ntokens 4\n"), "{text}");
}

#[cfg(unix)]
#[test]
fn a_readme_the_tree_refuses_leaves_the_standard_output_empty() {
    let fixture = repo_on("measure-readme-link", "b");
    let outside = Transcripts::new("measure-readme-outside");
    outside.write("README.md", &["- **Template release:** v0.11.1".to_owned()]);
    fs::remove_file(fixture.root().join("README.md")).expect("removed");
    std::os::unix::fs::symlink(
        outside.dir.join("README.md"),
        fixture.root().join("README.md"),
    )
    .expect("the link is made");
    let transcripts = Transcripts::new("measure-readme-link");
    transcripts.write("s1.jsonl", &[request("r1", "b", [1, 1, 1, 1])]);
    let args = [
        "measure",
        "--transcripts",
        transcripts.path(),
        "--breakdown",
        "--finding",
    ];
    let output = fixture.nuc(&args);
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
    assert_eq!(stdout(&output), "");
    assert!(stderr(&output).contains("README.md"), "{}", stderr(&output));
}

#[test]
fn the_finding_says_where_there_is_no_readme() {
    let fixture = repo_on("measure-no-readme", "b");
    fixture.git(&["rm", "-q", "README.md"]);
    let transcripts = Transcripts::new("measure-no-readme");
    transcripts.write("s1.jsonl", &[request("r1", "b", [1, 1, 1, 1])]);
    let output = fixture.nuc(&["measure", "--transcripts", transcripts.path(), "--finding"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    assert!(stdout(&output).contains("none: the README names no Template release"));
}
