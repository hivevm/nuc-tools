//! `nuc measure`: reads a harness's transcripts from one directory outside the repository and
//! prints the tokens of the Measures line (API-measure-reads-harness-transcripts). The reads
//! outside the repository are `entries` and `read`, which follow no symbolic link and write
//! nothing; the counting is the library's.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use std::collections::BTreeMap;

use crate::measure::{self, Harness, Lines, Selection, Source};
use crate::rules;

use super::tree::Tree;
use super::{PASSED, git};

/// The options of `nuc measure`: the harness, Claude Code where none is named; the directory of
/// its transcripts, the harness's own where none is named; the branches each `--branch` adds to the
/// current one; the time range; whether to break the tokens down; and whether to draft a template
/// finding. Each but `--branch` at most once.
#[derive(Default)]
struct Options<'a> {
    harness: Option<Harness>,
    transcripts: Option<&'a str>,
    branches: Vec<String>,
    since: Option<String>,
    until: Option<String>,
    breakdown: bool,
    finding: bool,
}

/// A transcript as the shell read it: its name under the directory, where it was written, its
/// bytes, and for a subagent the bytes of the harness's description of it, where there is one.
struct Transcript {
    name: String,
    source: Source,
    bytes: Vec<u8>,
    meta: Option<Vec<u8>>,
}

/// `nuc measure`: the tokens the harness's sessions on the selected branches read and wrote,
/// written as `tokens <n>` on the standard output, after their breakdown and a draft of a template
/// finding where they are asked for.
pub(super) fn run(options: &[&str]) -> Result<u8, String> {
    let options = parse(options).ok_or_else(super::usage)?;
    let harness = options.harness.unwrap_or(Harness::ClaudeCode);
    let here =
        env::current_dir().map_err(|e| format!("the current directory is unreadable: {e}"))?;
    let root = git::toplevel(&here)?;
    // The current branch and each one named; on a detached HEAD, the named ones alone.
    let current = git::probe(&root, &["symbolic-ref", "--short", "-q", "HEAD"])?
        .map(|branch| branch.trim_end().to_owned());
    if current.is_none() && options.branches.is_empty() {
        return Err(
            "HEAD is on no branch: name the branches with --branch — not decided".to_owned(),
        );
    }
    let mut branches: Vec<String> = current.into_iter().chain(options.branches).collect();
    let mut seen = std::collections::BTreeSet::new();
    branches.retain(|branch| seen.insert(branch.clone()));
    // Components drop a trailing separator, through which a link to a directory would be
    // followed rather than seen.
    let dir: PathBuf = match options.transcripts {
        Some(dir) => Path::new(dir).components().collect(),
        None => default_dir(harness, &root)?,
    };
    let mut lines = Lines::default();
    let mut subagents = BTreeMap::new();
    for transcript in read_transcripts(&dir)? {
        let read = harness.lines(&transcript.name, &transcript.bytes, &transcript.source)?;
        lines.requests.extend(read.requests);
        lines.tool_outputs.extend(read.tool_outputs);
        if let Some(agent) = transcript.meta.and_then(|meta| harness.subagent(&meta)) {
            subagents.insert(transcript.source, agent);
        }
    }
    let selection = Selection {
        branches,
        since: options.since,
        until: options.until,
    };
    let measures = measure::measure(&lines, &selection)?.ok_or_else(|| {
        format!(
            "no request of the harness on the branch {} in {} — not decided",
            selection.branches.join(", "),
            dir.display()
        )
    })?;
    // Built whole before a byte is printed, so that a refusal leaves the standard output empty.
    let mut out = String::new();
    if options.breakdown {
        out.push_str(&measure::breakdown(&measures, &subagents));
    }
    if options.finding {
        out.push_str(&measure::finding(&measures, release(&root)?.as_deref()));
    }
    println!("{out}{}", measure::line(measures.total.total()));
    Ok(PASSED)
}

/// The Template release the README of the working tree names, `vX.Y.Z` or `unreleased`, read
/// through the tree as every command reads the repository; none where the README or its line is
/// missing or names none.
fn release(root: &Path) -> Result<Option<String>, String> {
    let tree = Tree::new(root.to_path_buf())?;
    let Some(readme) = tree.read_checked(&root.join("README.md"))? else {
        return Ok(None);
    };
    Ok(match rules::docs_template_release::carried(&readme) {
        Some(rules::Carried::Release(version)) => Some(format!("v{version}")),
        Some(rules::Carried::Unreleased) => Some("unreleased".to_owned()),
        None => None,
    })
}

/// The options, or none where one is unknown, repeated, has no value, or a time is of no form
/// `measure::instant` reads.
fn parse<'a>(options: &[&'a str]) -> Option<Options<'a>> {
    let mut parsed = Options::default();
    let mut options = options.iter();
    while let Some(&option) = options.next() {
        if option == "--breakdown" && !parsed.breakdown {
            parsed.breakdown = true;
            continue;
        }
        if option == "--finding" && !parsed.finding {
            parsed.finding = true;
            continue;
        }
        let value = *options.next()?;
        match option {
            "--harness" if parsed.harness.is_none() => {
                parsed.harness = Some(Harness::parse(value)?)
            }
            "--transcripts" if parsed.transcripts.is_none() => parsed.transcripts = Some(value),
            "--branch" if !value.is_empty() => parsed.branches.push(value.to_owned()),
            "--since" if parsed.since.is_none() => parsed.since = Some(measure::instant(value)?),
            "--until" if parsed.until.is_none() => parsed.until = Some(measure::instant(value)?),
            _ => return None,
        }
    }
    // A range that ends before it starts selects nothing: a mistake of the command line.
    if let (Some(since), Some(until)) = (&parsed.since, &parsed.until)
        && since >= until
    {
        return None;
    }
    Some(parsed)
}

/// The harness's directory of the transcripts of the working tree at `root`. Claude Code keeps
/// them under `projects/` of its configuration directory, which `CLAUDE_CONFIG_DIR` names or is
/// `.claude` in the home directory, in a directory named for the working tree's path, each
/// character but an ASCII letter or digit replaced by `-`. The directories above it are followed
/// as the system resolves them; only what is read inside it must be no symbolic link.
fn default_dir(harness: Harness, root: &Path) -> Result<PathBuf, String> {
    match harness {
        Harness::ClaudeCode => {
            let config = match env::var_os("CLAUDE_CONFIG_DIR") {
                Some(dir) => PathBuf::from(dir),
                None => home()
                    .map(|home| home.join(".claude"))
                    .ok_or("neither CLAUDE_CONFIG_DIR nor a home directory is set — name the transcripts with --transcripts")?,
            };
            let key: String = root
                .to_string_lossy()
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
                .collect();
            Ok(config.join("projects").join(key))
        }
    }
}

/// The home directory: `USERPROFILE` on Windows, where Claude Code takes the system's home and a
/// shell's `HOME` may be no native path, and `HOME` elsewhere, each the other where it is unset.
fn home() -> Option<PathBuf> {
    let (first, second) = if cfg!(windows) {
        ("USERPROFILE", "HOME")
    } else {
        ("HOME", "USERPROFILE")
    };
    env::var_os(first)
        .or_else(|| env::var_os(second))
        .map(PathBuf::from)
}

/// The transcripts in `dir`, each read whole: every `<session>.jsonl` of the directory, and every
/// `agent-<id>.jsonl` of `<session>/subagents/` with its `agent-<id>.meta.json` where there is one,
/// in the order of their names. A symbolic link anywhere in `dir` or a `subagents/` directory,
/// and a file that is not regular where a transcript is read, is refused, so that nothing outside
/// `dir` is read and nothing in it is passed over unseen.
fn read_transcripts(dir: &Path) -> Result<Vec<Transcript>, String> {
    let mut found = Vec::new();
    for name in entries(dir)? {
        let path = dir.join(&name);
        let what = kind(&path)?;
        if what == Kind::Link {
            return Err(not_regular(&path));
        }
        if let Some(session) = name.strip_suffix(".jsonl") {
            let source = Source {
                session: session.to_owned(),
                subagent: None,
            };
            found.push(Transcript {
                name: name.clone(),
                source,
                bytes: read(&path)?,
                meta: None,
            });
        } else if what == Kind::Dir {
            found.extend(read_subagents(&name, &path.join("subagents"))?);
        }
    }
    Ok(found)
}

/// The transcripts of the subagents of `session` in its `subagents/` directory, none where it has
/// none.
fn read_subagents(session: &str, dir: &Path) -> Result<Vec<Transcript>, String> {
    match kind_if_any(dir)? {
        None => return Ok(Vec::new()),
        Some(Kind::Dir) => {}
        Some(_) => return Err(not_regular(dir)),
    }
    let names = entries(dir)?;
    let mut found = Vec::new();
    for name in &names {
        let path = dir.join(name);
        if kind(&path)? == Kind::Link {
            return Err(not_regular(&path));
        }
        let Some(agent) = name
            .strip_suffix(".jsonl")
            .filter(|a| a.starts_with("agent-"))
        else {
            continue;
        };
        let meta_name = format!("{agent}.meta.json");
        let meta = if names.contains(&meta_name) {
            Some(read(&dir.join(&meta_name))?)
        } else {
            None
        };
        found.push(Transcript {
            name: format!("{session}/subagents/{name}"),
            source: Source {
                session: session.to_owned(),
                subagent: Some(agent.to_owned()),
            },
            bytes: read(&path)?,
            meta,
        });
    }
    Ok(found)
}

/// The bytes of a regular file the listing found.
fn read(path: &Path) -> Result<Vec<u8>, String> {
    if kind(path)? != Kind::Regular {
        return Err(not_regular(path));
    }
    fs::read(path).map_err(|e| format!("{}: unreadable: {e} — not decided", path.display()))
}

/// What a path is, without following a symbolic link.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Regular,
    Dir,
    Link,
    Other,
}

/// The kind of the path, none where nothing is there.
fn kind_if_any(path: &Path) -> Result<Option<Kind>, String> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => Ok(Some(Kind::Link)),
        Ok(meta) if meta.is_file() => Ok(Some(Kind::Regular)),
        Ok(meta) if meta.is_dir() => Ok(Some(Kind::Dir)),
        Ok(_) => Ok(Some(Kind::Other)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("{}: unreadable: {e} — not decided", path.display())),
    }
}

/// The kind of a path that is there.
fn kind(path: &Path) -> Result<Kind, String> {
    kind_if_any(path)?.ok_or_else(|| format!("{}: gone while read — not decided", path.display()))
}

/// The refusal of a symbolic link or another file that is not regular where a transcript is read.
fn not_regular(path: &Path) -> String {
    format!(
        "{}: a symbolic link or no regular file where a transcript is read — not decided",
        path.display()
    )
}

/// The names in the directory, in order; the directory itself must be one, not a link to one.
fn entries(dir: &Path) -> Result<Vec<String>, String> {
    match kind_if_any(dir)? {
        Some(Kind::Dir) => {}
        None => {
            return Err(format!(
                "no transcripts of the harness at {} — name them with --transcripts",
                dir.display()
            ));
        }
        Some(_) => return Err(not_regular(dir)),
    }
    let mut names = Vec::new();
    let unreadable =
        |e: std::io::Error| format!("{}: unreadable: {e} — not decided", dir.display());
    for entry in fs::read_dir(dir).map_err(unreadable)? {
        let entry = entry.map_err(unreadable)?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| format!("{}: a name that is no UTF-8 — not decided", dir.display()))?;
        names.push(name);
    }
    names.sort();
    Ok(names)
}
