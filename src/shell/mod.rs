//! The imperative shell: the only module that spawns processes, touches the file system, reads
//! the environment, or prints (ADR-0009). It reads the command line and the
//! repository, calls the rules, and reports what they return.

pub mod git;

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::process::ExitCode;

use crate::adr;
use crate::config::{self, Version};
use crate::finding::Finding;
use crate::rules::{self, Inputs};

/// Exit code of a run that found nothing.
const PASSED: u8 = 0;
/// Exit code of a check that reported findings.
const FOUND: u8 = 1;
/// Exit code of a run that could not decide, whatever the cause: it never passes (Q-1).
const UNDECIDED: u8 = 2;

const USAGE: &str = "usage: nuc check [--repo <path>] | nuc explain <rule ID> | nuc --version";

/// The entry point of the binary: runs the command named on the command line and returns the
/// exit code the process ends with.
pub fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let result = match args.as_slice() {
        ["check"] => check(None),
        ["check", "--repo", repo] => check(Some(Path::new(repo))),
        ["explain", id] => explain(id),
        ["--version"] => {
            println!("nuc {}", env!("CARGO_PKG_VERSION"));
            Ok(PASSED)
        }
        _ => Err(format!(
            "{USAGE} — a newer nuc may know the command: {}",
            config::INSTALL
        )),
    };
    match result {
        Ok(code) => ExitCode::from(code),
        Err(why) => {
            eprintln!("nuc: {why}");
            ExitCode::from(UNDECIDED)
        }
    }
}

/// The version of this `nuc`, which `nuc.toml` sets the floor for.
fn running() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("the package version is X.Y.Z")
}

/// `nuc check`: the rules `nuc.toml` turns on, run over the repository that contains `repo`, or
/// the directory `nuc` runs in.
fn check(repo: Option<&Path>) -> Result<u8, String> {
    let start = match repo {
        Some(repo) => repo.to_path_buf(),
        None => {
            env::current_dir().map_err(|e| format!("the current directory is unreadable: {e}"))?
        }
    };
    let root = git::toplevel(&start)?;
    let text = read(&root.join("nuc.toml"))?.ok_or_else(|| config::Refusal::Missing.to_string())?;
    let config = config::load(&text, running()).map_err(|refusal| refusal.to_string())?;
    let selected = config::select(&config, rules::ALL).map_err(|r| r.to_string())?;
    let adr = read_adr(&root.join("docs/adr"))?;
    let inputs = Inputs {
        readme: read_checked(&root.join("README.md"))?,
        adr_targets: read_adr_targets(&root, &adr)?,
        text_files: read_text_files(&root)?,
        adr,
    };
    let findings: Vec<Finding> = selected
        .iter()
        .flat_map(|rule| (rule.check)(&inputs))
        .collect();
    for finding in &findings {
        println!("{finding}");
    }
    if findings.is_empty() {
        eprintln!("nuc check passed ({} rules).", selected.len());
        Ok(PASSED)
    } else {
        eprintln!("nuc check: {} finding(s).", findings.len());
        Ok(FOUND)
    }
}

/// `nuc explain <id>`: the reason of the rule (G-6).
fn explain(id: &str) -> Result<u8, String> {
    let rule = rules::find(id).ok_or_else(|| format!("no rule '{id}' is known to this nuc"))?;
    println!("{}: {}", rule.id, rule.explain);
    Ok(PASSED)
}

/// The bytes of the file, `None` when it does not exist, or why it cannot be read.
fn read_bytes(path: &Path) -> Result<Option<Vec<u8>>, String> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("{}: unreadable — {e}", path.display())),
    }
}

/// The text of a file `nuc` itself reads, which must be UTF-8.
fn read(path: &Path) -> Result<Option<String>, String> {
    read_bytes(path)?
        .map(|bytes| {
            String::from_utf8(bytes)
                .map_err(|_| format!("{}: unreadable — not UTF-8", path.display()))
        })
        .transpose()
}

/// The text of a file the rules read as the bash checks do, byte by byte: a byte that is no
/// UTF-8 stands for itself, as for grep in the C locale. A NUL byte makes grep read the file as
/// binary, which it reports or not depending on where the byte falls; `nuc` does not decide
/// such a file (Q-1).
fn read_checked(path: &Path) -> Result<Option<String>, String> {
    read_bytes(path)?
        .map(|bytes| {
            if bytes.contains(&0) {
                return Err(format!(
                    "{}: holds a NUL byte, which the bash checks read as binary — not decided",
                    path.display()
                ));
            }
            Ok(String::from_utf8_lossy(&bytes).into_owned())
        })
        .transpose()
}

/// The bytes of a file awk or `read` reads for a rule of the record, `None` when it is missing; a
/// NUL byte, which they do not read as the other bytes, is not decided (Q-1).
fn read_record_file(path: &Path) -> Result<Option<Vec<u8>>, String> {
    let Some(bytes) = read_bytes(path)? else {
        return Ok(None);
    };
    if bytes.contains(&0) {
        return Err(format!(
            "{}: holds a NUL byte, which the bash checks read as binary — not decided",
            path.display()
        ));
    }
    Ok(Some(bytes))
}

/// The files of the directory the bash reads for the record: each ADR file of its own, as
/// `find -type f` lists it, and the index and the template wherever they resolve, as `-f` and
/// `grep` do; empty when the directory does not exist.
fn read_adr(dir: &Path) -> Result<adr::Dir, String> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(adr::Dir::new()),
        Err(e) => return Err(format!("{}: unreadable — {e}", dir.display())),
    };
    let mut files = adr::Dir::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("{}: unreadable — {e}", dir.display()))?;
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            return Err(format!(
                "{}: a file name that is no UTF-8 — not decided",
                entry.path().display()
            ));
        };
        let own_file = entry.file_type().is_ok_and(|kind| kind.is_file());
        let wanted = (own_file && adr::is_adr_name(&name))
            || ((name == "README.md" || name == "template.md") && entry.path().is_file());
        let bytes = if wanted {
            read_record_file(&entry.path())?
        } else {
            None
        };
        if let Some(bytes) = bytes {
            files.insert(name, bytes);
        }
    }
    Ok(files)
}

/// The file each row of the index links, resolved under `docs/adr/` as `-f` resolves it: its
/// bytes when it is a file, or none. A target that resolves outside the repository is not
/// decided: no rule reads beyond the tree it checks.
fn read_adr_targets(
    root: &Path,
    adr: &adr::Dir,
) -> Result<BTreeMap<String, Option<Vec<u8>>>, String> {
    let Some(index) = adr.get("README.md") else {
        return Ok(BTreeMap::new());
    };
    let base = root.join("docs/adr");
    let inside =
        fs::canonicalize(root).map_err(|e| format!("{}: unreadable — {e}", root.display()))?;
    let mut targets = BTreeMap::new();
    for target in rules::docs_adr_index::targets(index) {
        let path = base.join(&target);
        let bytes = match fs::canonicalize(&path) {
            Ok(real) if !real.starts_with(&inside) => {
                return Err(format!(
                    "docs/adr/README.md: a row links '{target}', outside the repository — not decided"
                ));
            }
            Ok(real) if real.is_file() => read_bytes(&real)?,
            _ => None,
        };
        targets.insert(target, bytes);
    }
    Ok(targets)
}

/// Every text file of the repository by its path, as the bash's collect_text_files lists them:
/// the lines of `git ls-files --cached --others --exclude-standard`, read as they stand, so that a
/// path git quotes names no file; each a file with a byte other than a newline and no NUL byte,
/// which `grep -Il .` keeps under `LC_ALL=C`.
fn read_text_files(root: &Path) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let listing = git::run(
        root,
        &["ls-files", "--cached", "--others", "--exclude-standard"],
    )?;
    let mut files = BTreeMap::new();
    for path in listing.lines() {
        let full = root.join(path);
        if !full.is_file() {
            continue;
        }
        let Some(bytes) = read_bytes(&full)? else {
            continue;
        };
        if !bytes.contains(&0) && bytes.iter().any(|&b| b != b'\n') {
            files.insert(path.to_owned(), bytes);
        }
    }
    Ok(files)
}
