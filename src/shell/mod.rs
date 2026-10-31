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
use crate::glob;
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
    let mut unplaced = false;
    if needs(Input::Readme) {
        inputs.readme = tree.read_checked(&tree.root.join("README.md"))?;
    }
    if needs(Input::Release) {
        let reported = selected
            .iter()
            .any(|rule| rule.id == rules::docs_template_release::RULE_ID);
        match carried_release(&tree, reported)? {
            Some(release) => inputs.release = release,
            None => unplaced = true,
        }
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
    if needs(Input::CheckLists) {
        inputs.check_lists = tree.read_check_lists()?;
    }
    if needs(Input::Layout) {
        inputs.layout = tree.read_layout(inputs.readme.as_deref())?;
    }
    if needs(Input::Skills) {
        inputs.skills = tree.read_skills()?;
    }
    if needs(Input::SetupFile) {
        inputs.setup_file = tree.root.join("TEMPLATE-SETUP.md").is_file();
    }
    if needs(Input::Landing) {
        inputs.landing = tree.read_landing()?;
    }
    // While the setup file exists the identity rule is silent, so it reads nothing (check 13).
    if needs(Input::Identity) && !inputs.setup_file {
        for &file in rules::docs_template_identity::FILES {
            let path = tree.root.join(file);
            if path.is_file()
                && let Some(bytes) = tree.read_record_file(&path)?
            {
                inputs.identity.insert(file.to_owned(), bytes);
            }
        }
    }
    let findings: Vec<Finding> = selected
        .iter()
        .flat_map(|rule| {
            if unplaced && rule.reads.contains(&Input::Release) {
                on_every_release(rule, &inputs)
            } else {
                (rule.check)(&inputs)
            }
        })
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
const EARLIEST_RELEASE: Version = rules::RELEASES[0];

/// The findings of a rule that checks by the release on a tree whose README names none: those it
/// reports on every release it knows, which no release could take back.
fn on_every_release(rule: &rules::Rule, inputs: &Inputs) -> Vec<Finding> {
    let runs: Vec<Vec<Finding>> = rules::RELEASES
        .iter()
        .map(|&release| {
            let inputs = Inputs {
                release: Some(release),
                ..inputs.clone()
            };
            (rule.check)(&inputs)
        })
        .collect();
    let (first, rest) = runs.split_first().expect("the rules know a release");
    first
        .iter()
        .filter(|finding| rest.iter().all(|run| run.contains(finding)))
        .cloned()
        .collect()
}

/// The template release the tree carries, for the rules whose bash check changed between
/// releases: none for `unreleased`, checked as the newest release. A release before the earliest
/// is not decided; so is a README that names none, unless the rule `reported` on, the Template
/// release rule, fails the run on it: then a rule reports only what it reports on every release
/// (ADR-draft-rules-follow-the-carried-release). The outer none is that case, a release the
/// README does not name.
fn carried_release(tree: &Tree, reported: bool) -> Result<Option<Option<Version>>, String> {
    let readme = tree.read_checked(&tree.root.join("README.md"))?;
    match readme.as_deref().and_then(rules::docs_template_release::carried) {
        Some(Carried::Unreleased) => Ok(Some(None)),
        // The Template release rule reports a README that names no release, never a missing one.
        None if reported && readme.is_some() => Ok(None),
        Some(Carried::Release(release)) if release >= EARLIEST_RELEASE => Ok(Some(Some(release))),
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
                    .map(|bytes| unit_separator_free(&entry.path(), bytes))
                    .transpose()?
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

/// The bytes of an ADR file without the unit separator, byte 0x1F, which the bash splits its awk
/// output on, so that a field holding it reads differently there; such a file is not decided
/// (Q-1).
fn unit_separator_free(path: &Path, bytes: Vec<u8>) -> Result<Vec<u8>, String> {
    if bytes.contains(&0x1F) {
        return Err(format!(
            "{}: holds the byte 0x1F, which the bash reads the record's fields apart by — not decided",
            path.display()
        ));
    }
    Ok(bytes)
}

/// The bytes within which `grep -I` finds a NUL byte however its buffers fall: GNU grep reads
/// at least this much before it decides.
const GREP_FIRST_BUFFER: usize = 32 * 1024;

/// The names of the directory's entries, sorted by their bytes as a glob in the C locale sorts
/// them; empty when it is no directory. A name that is no UTF-8 is not decided.
fn entry_names(dir: &Path) -> Result<Vec<String>, String> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if matches!(e.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) => {
            return Ok(Vec::new());
        }
        Err(e) => return Err(format!("{}: unreadable — {e}", dir.display())),
    };
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("{}: unreadable — {e}", dir.display()))?;
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            return Err(format!(
                "{}: a file name that is no UTF-8 — not decided",
                entry.path().display()
            ));
        };
        names.push(name);
    }
    names.sort();
    Ok(names)
}

impl Tree {
    /// The names of the entries of a directory of the repository, as [`entry_names`] gives them;
    /// a directory that resolves outside the repository is not listed, and not decided.
    fn list(&self, dir: &Path) -> Result<Vec<String>, String> {
        let real = match fs::canonicalize(dir) {
            Ok(real) => real,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(format!("{}: unreadable — {e}", dir.display())),
        };
        if !real.starts_with(&self.inside) {
            return Err(format!(
                "{}: resolves outside the repository — not decided",
                dir.display()
            ));
        }
        entry_names(&real)
    }

    /// The files of check 9: `checks.yml`, `check-all.sh`, the workflows as the bash's globs
    /// list them, and the scripts as `find -type f` lists them.
    fn read_check_lists(&self) -> Result<rules::CheckLists, String> {
        let file = |path: &str| -> Result<Option<Vec<u8>>, String> {
            let full = self.root.join(path);
            if full.is_file() {
                self.read_record_file(&full)
            } else {
                Ok(None)
            }
        };
        let mut lists = rules::CheckLists {
            checks: file(".github/workflows/checks.yml")?,
            check_all: file("scripts/check-all.sh")?,
            ..rules::CheckLists::default()
        };
        let dir = self.root.join(".github/workflows");
        let names = self.list(&dir)?;
        lists.workflow_files = names
            .iter()
            .filter(|name| dir.join(name).is_file())
            .cloned()
            .collect();
        for suffix in [".yml", ".yaml"] {
            for name in names
                .iter()
                .filter(|name| !name.starts_with('.') && name.ends_with(suffix))
            {
                let rel = format!(".github/workflows/{name}");
                if let Some(bytes) = file(&rel)? {
                    lists.workflows.push((rel, bytes));
                }
            }
        }
        let scripts = self.root.join("scripts");
        for name in self.list(&scripts)? {
            let regular = fs::symlink_metadata(scripts.join(&name)).is_ok_and(|m| m.is_file());
            if regular && name.ends_with(".sh") {
                lists.scripts.push(name);
            }
        }
        Ok(lists)
    }

    /// What check 10 tests: each path of the Project Layout block, as `compgen -G`, `-d`, or
    /// `-f` tests it, and the first part of every tracked path.
    fn read_layout(&self, readme: Option<&str>) -> Result<rules::Layout, String> {
        let mut layout = rules::Layout::default();
        for path in readme
            .map(rules::docs_project_layout::listed)
            .unwrap_or_default()
        {
            if path.contains('\u{FFFD}') {
                return Err(format!(
                    "README.md: the Project Layout lists a path that is no UTF-8, '{path}' — not decided"
                ));
            }
            let there = if rules::docs_project_layout::is_pattern(&path) {
                self.glob_matches(&path)?
            } else if let Some(dir) = path.strip_suffix('/') {
                // Joined as text, as the bash joins it: an absolute path stays under the root.
                Path::new(&format!("{}/{dir}", self.root.display())).is_dir()
            } else {
                Path::new(&format!("{}/{path}", self.root.display())).is_file()
            };
            layout.found.insert(path, there);
        }
        let listing = git::run(&self.root, &["ls-files", "--cached"])?;
        let mut top: Vec<String> = listing
            .split('\n')
            .filter(|path| !path.is_empty())
            .map(|path| path.split('/').next().unwrap_or(path).to_owned())
            .collect();
        top.sort();
        top.dedup();
        layout.top_level = top;
        Ok(layout)
    }

    /// Whether the pattern matches a path under the root, as bash's pathname expansion matches
    /// it. A directory the walk would list outside the repository is not decided.
    fn glob_matches(&self, pattern: &str) -> Result<bool, String> {
        let (pattern, want_dir) = match pattern.strip_suffix('/') {
            Some(p) => (p, true),
            None => (pattern, false),
        };
        let segments: Vec<&str> = pattern.split('/').collect();
        let mut current = vec![self.root.clone()];
        for (i, segment) in segments.iter().enumerate() {
            let last = i + 1 == segments.len();
            let mut next = Vec::new();
            for dir in &current {
                if !glob::has_magic(segment) {
                    let path = dir.join(glob::unescape(segment));
                    let exists = if last && !want_dir {
                        fs::symlink_metadata(&path).is_ok()
                    } else {
                        path.is_dir()
                    };
                    if exists {
                        next.push(path);
                    }
                    continue;
                }
                if !dir.as_os_str().is_empty() {
                    let real = fs::canonicalize(dir)
                        .map_err(|e| format!("{}: unreadable — {e}", dir.display()))?;
                    if !real.starts_with(&self.inside) {
                        return Err(format!(
                            "README.md: the Project Layout pattern '{pattern}' looks outside the repository — not decided"
                        ));
                    }
                }
                for name in entry_names(dir)? {
                    let matched = glob::matches(segment, &name).ok_or_else(|| {
                        format!(
                            "README.md: the Project Layout pattern '{pattern}' uses a bracket nuc does not read — not decided"
                        )
                    })?;
                    let path = dir.join(&name);
                    if matched && (!(want_dir || !last) || path.is_dir()) {
                        next.push(path);
                    }
                }
            }
            current = next;
        }
        Ok(!current.is_empty())
    }

    /// What CI says about the change, from the variables the bash reads, and the drafts as
    /// `find docs/adr -maxdepth 1 -type f -name 'draft-*.md'` lists them: regular files only,
    /// and none when `docs/adr` is itself a symbolic link, which `find` does not follow. A
    /// variable that is no UTF-8 is not decided.
    fn read_landing(&self) -> Result<rules::docs_no_draft_on_main::Landing, String> {
        let var = |key: &str| match env::var(key) {
            Ok(value) => Ok(Some(value)),
            Err(env::VarError::NotPresent) => Ok(None),
            Err(env::VarError::NotUnicode(_)) => {
                Err(format!("{key}: holds no UTF-8 — not decided"))
            }
        };
        let dir = self.root.join("docs/adr");
        let is_dir = fs::symlink_metadata(&dir).is_ok_and(|m| m.is_dir());
        let mut drafts = Vec::new();
        if is_dir {
            for name in self.list(&dir)? {
                let regular = fs::symlink_metadata(dir.join(&name)).is_ok_and(|m| m.is_file());
                if regular && name.starts_with("draft-") && name.ends_with(".md") {
                    drafts.push(name);
                }
            }
        }
        Ok(rules::docs_no_draft_on_main::Landing {
            base_ref: var("GITHUB_BASE_REF")?,
            actions: var("GITHUB_ACTIONS")?.as_deref() == Some("true"),
            ref_name: var("GITHUB_REF_NAME")?,
            draft_pull_request: var("PULL_REQUEST_DRAFT")?.as_deref() == Some("true"),
            drafts,
        })
    }

    /// What check 11 tests: each skill and what stands at its pointer, and each entry of the
    /// pointer directory, resolved as `cd -P` resolves it.
    fn read_skills(&self) -> Result<rules::Skills, String> {
        let skills = self.root.join(".agents/skills");
        let pointers = self.root.join(".claude/skills");
        let has_skill = |name: &str| skills.join(name).join("SKILL.md").is_file();
        let mut found = rules::Skills::default();
        for name in self.list(&skills)? {
            if name.starts_with('.') || !skills.join(&name).is_dir() || !has_skill(&name) {
                continue;
            }
            let pointer = match fs::symlink_metadata(pointers.join(&name)) {
                Ok(m) if m.file_type().is_symlink() => rules::Pointer::Symlink,
                Ok(_) => rules::Pointer::Other,
                Err(_) => rules::Pointer::Missing,
            };
            found.skills.push((name, pointer));
        }
        for name in self.list(&pointers)? {
            if name.starts_with('.') {
                continue;
            }
            let p = pointers.join(&name);
            let symlink = fs::symlink_metadata(&p).is_ok_and(|m| m.file_type().is_symlink());
            let entry = if !symlink {
                rules::Entry::NotSymlink {
                    skill: has_skill(&name),
                }
            } else {
                let resolve = |path: &Path| fs::canonicalize(path).ok().filter(|r| r.is_dir());
                match resolve(&p) {
                    None => rules::Entry::Dangles,
                    Some(target) => {
                        let want = resolve(&skills.join(&name));
                        let at_skill = want.as_ref() == Some(&target) && has_skill(&name);
                        rules::Entry::Points {
                            target: self.shown(&target),
                            at_skill,
                        }
                    }
                }
            };
            found.pointers.push((name, entry));
        }
        Ok(found)
    }

    /// A resolved path as the bash shows it: relative to the root when under it, `/` between its
    /// parts, and whole otherwise.
    fn shown(&self, real: &Path) -> String {
        match real.strip_prefix(&self.inside) {
            Ok(rel) if !rel.as_os_str().is_empty() => rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/"),
            _ => real.display().to_string(),
        }
    }
}
