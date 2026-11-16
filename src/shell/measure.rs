//! `nuc measure`: reads a harness's transcripts from one directory outside the repository and
//! prints the tokens of the Measures line (API-measure-reads-harness-transcripts). The one read
//! outside the repository is `read_transcripts`, which follows no symbolic link and writes
//! nothing; the counting is the library's.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::measure::{self, Harness, Request};

use super::{PASSED, git};

/// The options of `nuc measure`, each at most once: the harness, Claude Code where none is named,
/// and the directory of its transcripts, the harness's own where none is named.
struct Options<'a> {
    harness: Harness,
    transcripts: Option<&'a str>,
}

/// `nuc measure`: the tokens the harness's sessions on the current branch read and wrote, written
/// as `tokens <n>` on the standard output.
pub(super) fn run(options: &[&str]) -> Result<u8, String> {
    let options = parse(options).ok_or_else(super::usage)?;
    let here =
        env::current_dir().map_err(|e| format!("the current directory is unreadable: {e}"))?;
    let root = git::toplevel(&here)?;
    let branch = git::probe(&root, &["symbolic-ref", "--short", "-q", "HEAD"])?
        .map(|branch| branch.trim_end().to_owned())
        .ok_or("HEAD is on no branch, so no session's branch can be chosen — not decided")?;
    let dir = match options.transcripts {
        Some(dir) => PathBuf::from(dir),
        None => default_dir(options.harness, &root)?,
    };
    let mut requests: Vec<Request> = Vec::new();
    for (name, bytes) in read_transcripts(&dir)? {
        requests.extend(options.harness.requests(&name, &bytes)?);
    }
    let tokens = measure::tokens(&requests, &branch).ok_or_else(|| {
        format!(
            "no request of the harness on the branch {branch} in {} — not decided",
            dir.display()
        )
    })?;
    println!("{}", measure::line(tokens));
    Ok(PASSED)
}

/// The options, or none where one is unknown, repeated, or has no value.
fn parse<'a>(options: &[&'a str]) -> Option<Options<'a>> {
    let (mut harness, mut transcripts) = (None, None);
    let mut options = options.iter();
    while let Some(&option) = options.next() {
        let value = *options.next()?;
        match option {
            "--harness" if harness.is_none() => harness = Some(Harness::parse(value)?),
            "--transcripts" if transcripts.is_none() => transcripts = Some(value),
            _ => return None,
        }
    }
    Some(Options {
        harness: harness.unwrap_or(Harness::ClaudeCode),
        transcripts,
    })
}

/// The harness's directory of the transcripts of the working tree at `root`. Claude Code keeps
/// them under `projects/` of its configuration directory, which `CLAUDE_CONFIG_DIR` names or is
/// `.claude` in the home directory, in a directory named for the working tree's path, each byte
/// but an ASCII letter or digit replaced by `-`.
fn default_dir(harness: Harness, root: &Path) -> Result<PathBuf, String> {
    match harness {
        Harness::ClaudeCode => {
            let config = match env::var_os("CLAUDE_CONFIG_DIR") {
                Some(dir) => PathBuf::from(dir),
                None => env::var_os("HOME")
                    .or_else(|| env::var_os("USERPROFILE"))
                    .map(|home| PathBuf::from(home).join(".claude"))
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

/// The transcripts in `dir`, each named by its path under `dir` and read whole: every `*.jsonl`
/// file of the directory, and every `agent-*.jsonl` of `<session>/subagents/` beside it, in the
/// order of their names. A symbolic link or a file that is not regular, where a transcript would
/// be read, is refused, so that nothing outside `dir` is read.
fn read_transcripts(dir: &Path) -> Result<Vec<(String, Vec<u8>)>, String> {
    let mut files = Vec::new();
    for name in entries(dir)? {
        let path = dir.join(&name);
        let what = kind(&path)?;
        if name.ends_with(".jsonl") {
            files.push((name, path, what));
        } else if what == Kind::Dir {
            let subagents = path.join("subagents");
            match kind_if_any(&subagents)? {
                None => {}
                Some(Kind::Dir) => {
                    for agent in entries(&subagents)? {
                        if agent.starts_with("agent-") && agent.ends_with(".jsonl") {
                            let at = subagents.join(&agent);
                            let agent_kind = kind(&at)?;
                            files.push((format!("{name}/subagents/{agent}"), at, agent_kind));
                        }
                    }
                }
                Some(_) => return Err(not_regular(&subagents)),
            }
        }
    }
    files
        .into_iter()
        .map(|(name, path, kind)| {
            if kind != Kind::Regular {
                return Err(not_regular(&path));
            }
            let bytes = fs::read(&path)
                .map_err(|e| format!("{}: unreadable: {e} — not decided", path.display()))?;
            Ok((name, bytes))
        })
        .collect()
}

/// What a path is, without following a symbolic link.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Regular,
    Dir,
    Other,
}

/// The kind of the path, none where nothing is there.
fn kind_if_any(path: &Path) -> Result<Option<Kind>, String> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => Ok(Some(Kind::Other)),
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
    for entry in fs::read_dir(dir).map_err(|e| format!("{}: unreadable: {e}", dir.display()))? {
        let entry = entry.map_err(|e| format!("{}: unreadable: {e}", dir.display()))?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| format!("{}: a name that is no UTF-8 — not decided", dir.display()))?;
        names.push(name);
    }
    names.sort();
    Ok(names)
}
