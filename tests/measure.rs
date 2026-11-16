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
fn a_request_over_several_lines_counts_once_with_its_last_usage() {
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
        &["measure", "--branch", "b"],
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
