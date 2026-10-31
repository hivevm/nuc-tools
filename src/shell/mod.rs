//! The imperative shell: the only module that spawns processes, touches the file system, reads
//! the environment, or prints (ADR-0009). It reads the command line and the
//! repository, calls the rules, and reports what they return.

pub mod git;

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::adr;
use crate::config::{self, Version};
use crate::finding::Finding;
use crate::markdown;
use crate::rules::{self, Carried, Input, Inputs};

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
    let tree = Tree::new(root)?;
    let text = tree
        .read(&tree.root.join("nuc.toml"))?
        .ok_or_else(|| config::Refusal::Missing.to_string())?;
    let config = config::load(&text, running()).map_err(|refusal| refusal.to_string())?;
    let selected = config::select(&config, rules::ALL).map_err(|r| r.to_string())?;
    // Only what a rule turned on reads: an input no rule needs can neither refuse nor cost.
    let needs = |input: Input| selected.iter().any(|rule| rule.reads.contains(&input));
    let mut inputs = Inputs::default();
    if needs(Input::Readme) {
        inputs.readme = tree.read_checked(&tree.root.join("README.md"))?;
    }
    if needs(Input::Release) {
        let reported = selected
            .iter()
            .any(|rule| rule.id == rules::docs_template_release::RULE_ID);
        inputs.release = carried_release(&tree, reported)?;
    }
    if needs(Input::Adr) {
        inputs.adr = tree.read_adr(&tree.root.join("docs/adr"))?;
        inputs.adr_targets = tree.read_adr_targets(&inputs.adr)?;
    }
    if needs(Input::TextFiles) {
        inputs.text_files = tree.read_text_files()?;
    }
    if needs(Input::LinkTargets) {
        inputs.link_targets = tree.read_link_targets(&inputs.text_files, &inputs.adr)?;
    }
    if needs(Input::Agents) {
        let agents = tree.root.join("AGENTS.md");
        inputs.agents = if agents.is_file() {
            tree.read_bytes(&agents)?
        } else {
            None
        };
    }
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

/// The earliest template release whose bash checks the rules port.
const EARLIEST_RELEASE: Version = Version(0, 5, 0);

/// The template release the tree carries, for the rules whose bash check changed between
/// releases: none for `unreleased`, checked as the newest release. A release before the earliest
/// is not decided; so is a README that names none, unless the rule `reported` on, the Template
/// release rule, fails the run on it, and the tree is checked as the newest release
/// (ADR-draft-rules-follow-the-carried-release).
fn carried_release(tree: &Tree, reported: bool) -> Result<Option<Version>, String> {
    let readme = tree.read_checked(&tree.root.join("README.md"))?;
    match readme.as_deref().and_then(rules::docs_template_release::carried) {
        Some(Carried::Unreleased) => Ok(None),
        None if reported => Ok(None),
        Some(Carried::Release(release)) if release >= EARLIEST_RELEASE => Ok(Some(release)),
        Some(Carried::Release(release)) => Err(format!(
            "README.md: template release v{release} is before v{EARLIEST_RELEASE}, the earliest whose checks this nuc ports — not decided"
        )),
        None => Err(
            "README.md: no Template release line names the release the tree carries, which a rule turned on checks by — not decided"
                .to_owned(),
        ),
    }
}

/// `nuc explain <id>`: the reason of the rule (G-6).
fn explain(id: &str) -> Result<u8, String> {
    let rule = rules::find(id).ok_or_else(|| format!("no rule '{id}' is known to this nuc"))?;
    println!("{}: {}", rule.id, rule.explain);
    Ok(PASSED)
}

/// The repository `nuc` checks: its root as git names it, and the root resolved, within which
/// every file `nuc` reads must resolve. No rule reads beyond the tree it checks: a path that a
/// link, `..`, or a symbolic link carries outside is not decided.
struct Tree {
    root: PathBuf,
    inside: PathBuf,
}

impl Tree {
    fn new(root: PathBuf) -> Result<Self, String> {
        let inside =
            fs::canonicalize(&root).map_err(|e| format!("{}: unreadable — {e}", root.display()))?;
        Ok(Self { root, inside })
    }

    /// The bytes of the file, `None` when it does not exist, or why it cannot be read; a file
    /// that resolves outside the repository is not decided.
    fn read_bytes(&self, path: &Path) -> Result<Option<Vec<u8>>, String> {
        let real = match fs::canonicalize(path) {
            Ok(real) => real,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(format!("{}: unreadable — {e}", path.display())),
        };
        if !real.starts_with(&self.inside) {
            return Err(format!(
                "{}: resolves outside the repository — not decided",
                path.display()
            ));
        }
        match fs::read(&real) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!("{}: unreadable — {e}", path.display())),
        }
    }

    /// The text of a file `nuc` itself reads, which must be UTF-8.
    fn read(&self, path: &Path) -> Result<Option<String>, String> {
        self.read_bytes(path)?
            .map(|bytes| {
                String::from_utf8(bytes)
                    .map_err(|_| format!("{}: unreadable — not UTF-8", path.display()))
            })
            .transpose()
    }

    /// The text of a file the rules read as the bash checks do, byte by byte: a byte that is no
    /// UTF-8 stands for itself, as for grep in the C locale. A NUL byte makes grep read the file
    /// as binary, which it reports or not depending on where the byte falls; `nuc` does not
    /// decide such a file (Q-1).
    fn read_checked(&self, path: &Path) -> Result<Option<String>, String> {
        self.read_record_file(path)
            .map(|bytes| bytes.map(|bytes| String::from_utf8_lossy(&bytes).into_owned()))
    }

    /// The bytes of a file awk or `read` reads for a rule of the record, `None` when it is
    /// missing; a NUL byte, which they do not read as the other bytes, is not decided (Q-1).
    fn read_record_file(&self, path: &Path) -> Result<Option<Vec<u8>>, String> {
        let Some(bytes) = self.read_bytes(path)? else {
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
    /// `find -type f` lists it, and the index and the template wherever they resolve, as `-f`
    /// and `grep` do; empty when the directory does not exist.
    fn read_adr(&self, dir: &Path) -> Result<adr::Dir, String> {
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
                self.read_record_file(&entry.path())?
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
    /// bytes when it is a file, or none.
    fn read_adr_targets(
        &self,
        adr: &adr::Dir,
    ) -> Result<BTreeMap<String, Option<Vec<u8>>>, String> {
        let Some(index) = adr.get("README.md") else {
            return Ok(BTreeMap::new());
        };
        let base = self.root.join("docs/adr");
        let mut targets = BTreeMap::new();
        for target in rules::docs_adr_index::targets(index) {
            let path = base.join(&target);
            let bytes = match fs::canonicalize(&path) {
                Ok(real) if !real.starts_with(&self.inside) => {
                    return Err(format!(
                        "docs/adr/README.md: a row links '{target}', outside the repository — not decided"
                    ));
                }
                Ok(real) if real.is_file() => self.read_bytes(&real)?,
                _ => None,
            };
            targets.insert(target, bytes);
        }
        Ok(targets)
    }

    /// Every text file of the repository by its path, as the bash's collect_text_files lists
    /// them: the lines of `git ls-files --cached --others --exclude-standard`, read as they
    /// stand, so that a path git quotes names no file; each a file with a byte other than a
    /// newline and no NUL byte, which `grep -Il .` keeps under `LC_ALL=C`.
    fn read_text_files(&self) -> Result<BTreeMap<String, Vec<u8>>, String> {
        let listing = git::run(
            &self.root,
            &["ls-files", "--cached", "--others", "--exclude-standard"],
        )?;
        let mut files = BTreeMap::new();
        for path in listing.split('\n').filter(|path| !path.is_empty()) {
            let full = self.root.join(path);
            if !full.is_file() {
                continue;
            }
            let Some(bytes) = self.read_bytes(&full)? else {
                continue;
            };
            match bytes.iter().position(|&b| b == 0) {
                // A NUL in the first buffer grep reads makes the file binary to `grep -I`: left
                // out.
                Some(at) if at < GREP_FIRST_BUFFER => continue,
                // A later one is binary to grep or not depending on where its buffers fall.
                Some(_) => {
                    return Err(format!(
                        "{path}: holds a NUL byte past its first {GREP_FIRST_BUFFER} bytes, which grep reads as text or as binary depending on its buffers — not decided"
                    ));
                }
                None if bytes.iter().any(|&b| b != b'\n') => {
                    files.insert(path.to_owned(), bytes);
                }
                None => {}
            }
        }
        if files.is_empty() {
            return Err(format!(
                "no text files found under {} — the file list is empty, so nothing was checked",
                self.root.display()
            ));
        }
        Ok(files)
    }

    /// Each path a relative link of a Markdown file the bash reads names, tested as the bash
    /// tests it: the path is the root and the text the link names joined, never resolved by
    /// `nuc`, so that the file system resolves `..` and links as `-e` does. A file is read only
    /// where the bash reads it, for an anchor of lines or into a Markdown file. A path that is
    /// no UTF-8 is not decided, since the file system is asked for a name `nuc` cannot spell.
    fn read_link_targets(
        &self,
        text_files: &BTreeMap<String, Vec<u8>>,
        adr: &adr::Dir,
    ) -> Result<BTreeMap<String, rules::Resolved>, String> {
        let inactive = adr::inactive_paths(adr);
        let read = text_files
            .iter()
            .filter(|(file, _)| file.ends_with(".md") && !inactive.contains(file.as_str()));
        let mut resolved: BTreeMap<String, rules::Resolved> = BTreeMap::new();
        for (file, text) in read {
            for target in markdown::relative_targets(file, text) {
                let Some(path) = target.path else { continue };
                let named = target.target.split(|&b| b == b'#').next().unwrap_or(&[]);
                if std::str::from_utf8(named).is_err() {
                    return Err(format!(
                        "{file}:{}: a link names a path that is no UTF-8 — not decided",
                        target.line
                    ));
                }
                let full = PathBuf::from(format!("{}/{path}", self.root.display()));
                let entry = resolved.entry(path.clone()).or_default();
                entry.exists = fs::metadata(&full).is_ok();
                let reads = target.fragment.as_deref().is_some_and(|f| {
                    !f.is_empty() && (markdown::is_line_anchor(f) || path.ends_with(".md"))
                });
                if !reads || entry.file.is_some() || !full.is_file() {
                    continue;
                }
                entry.file = self.read_bytes(&full).map_err(|why| {
                    format!("{file}: a link's anchor looks into '{path}' — {why}")
                })?;
            }
        }
        Ok(resolved)
    }
}

/// The bytes within which `grep -I` finds a NUL byte however its buffers fall: GNU grep reads
/// at least this much before it decides.
const GREP_FIRST_BUFFER: usize = 32 * 1024;
