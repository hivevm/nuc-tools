//! The shell's reading of the tree: `Tree` reads every file and lists every directory the rules
//! need, each resolved through [`Tree::resolve`], which refuses one outside the repository.

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use super::git;
use crate::adr;
use crate::glob;
use crate::markdown;
use crate::rules;

/// The repository `nuc` checks: its root as git names it, and the root resolved, within which
/// every file `nuc` reads must resolve. No rule reads beyond the tree it checks: a path that a
/// link, `..`, or a symbolic link carries outside is not decided.
pub(super) struct Tree {
    pub(super) root: PathBuf,
    inside: PathBuf,
}

impl Tree {
    pub(super) fn new(root: PathBuf) -> Result<Self, String> {
        let inside =
            fs::canonicalize(&root).map_err(|e| format!("{}: unreadable — {e}", root.display()))?;
        Ok(Self { root, inside })
    }

    /// The path as the file system resolves it, or the error it gives; a path that resolves
    /// outside the repository is refused as not decided, with the message `outside` writes. Every
    /// file is read and every directory listed through it, which the structural test holds.
    fn resolve(
        &self,
        path: &Path,
        outside: impl FnOnce() -> String,
    ) -> Result<io::Result<PathBuf>, String> {
        match fs::canonicalize(path) {
            Ok(real) if !real.starts_with(&self.inside) => Err(outside()),
            resolved => Ok(resolved),
        }
    }

    /// The bytes of the file, `None` when it does not exist, or why it cannot be read; a file
    /// that resolves outside the repository is not decided.
    pub(super) fn read_bytes(&self, path: &Path) -> Result<Option<Vec<u8>>, String> {
        let real = match self.resolve(path, || outside(path))? {
            Ok(real) => real,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(format!("{}: unreadable — {e}", path.display())),
        };
        match fs::read(&real) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!("{}: unreadable — {e}", path.display())),
        }
    }

    /// The text of a file `nuc` itself reads, which must be UTF-8.
    pub(super) fn read(&self, path: &Path) -> Result<Option<String>, String> {
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
    pub(super) fn read_checked(&self, path: &Path) -> Result<Option<String>, String> {
        self.read_record_file(path)
            .map(|bytes| bytes.map(|bytes| String::from_utf8_lossy(&bytes).into_owned()))
    }

    /// The files check 13 reads for the template's placeholders, each that is a file.
    pub(super) fn read_identity(&self) -> Result<BTreeMap<String, Vec<u8>>, String> {
        let mut identity = BTreeMap::new();
        for &file in rules::docs_template_identity::FILES {
            let path = self.root.join(file);
            if path.is_file()
                && let Some(bytes) = self.read_record_file(&path)?
            {
                identity.insert(file.to_owned(), bytes);
            }
        }
        Ok(identity)
    }

    /// The files that carry a `Verifies:` marker, as the traceability check reads them: each path
    /// git lists as tracked or new and not ignored, separated by NUL bytes so that no path is
    /// quoted, that is a file as `-f` finds it and not Markdown. A NUL byte in the first buffer
    /// grep reads makes the file binary to `grep -I`, which leaves it out; a later one, in a file
    /// that holds the marker, is binary to grep or not depending on its buffers, and not decided.
    pub(super) fn read_markers(&self) -> Result<BTreeMap<String, Vec<u8>>, String> {
        let listing = git::run_bytes(
            &self.root,
            &[
                "ls-files",
                "--cached",
                "--others",
                "--exclude-standard",
                "-z",
            ],
        )?;
        let mut files = BTreeMap::new();
        for raw in listing.split(|&b| b == 0).filter(|path| !path.is_empty()) {
            let Some(full) = path_of(&self.root, raw) else {
                continue;
            };
            if raw.ends_with(b".md") || !full.is_file() {
                continue;
            }
            let Some(bytes) = self.read_bytes(&full)? else {
                continue;
            };
            if !adr::contains(&bytes, b"Verifies:") {
                continue;
            }
            // A path a finding could not name is read only where it holds no marker.
            let Ok(path) = std::str::from_utf8(raw) else {
                return Err(format!(
                    "{}: a path that is no UTF-8 holds a 'Verifies:' marker, which a finding could not name — not decided",
                    String::from_utf8_lossy(raw)
                ));
            };
            match bytes.iter().position(|&b| b == 0) {
                Some(at) if at < GREP_FIRST_BUFFER => {}
                Some(_) => {
                    return Err(format!(
                        "{path}: holds a 'Verifies:' marker and a NUL byte past its first {GREP_FIRST_BUFFER} bytes, which grep reads as text or as binary depending on its buffers — not decided"
                    ));
                }
                None => {
                    files.insert(path.to_owned(), bytes);
                }
            }
        }
        Ok(files)
    }

    /// The bytes of the specification when it is a file, as `-f` finds it, read as `read` reads
    /// it.
    pub(super) fn read_specification(&self) -> Result<Option<Vec<u8>>, String> {
        let path = self.root.join(rules::traceability_criteria::SPECIFICATION);
        if path.is_file() {
            self.read_record_file(&path)
        } else {
            Ok(None)
        }
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
    /// and `grep` do; empty when the directory does not exist. A directory that resolves outside
    /// the repository is not listed, and not decided.
    pub(super) fn read_adr(&self, dir: &Path) -> Result<adr::Dir, String> {
        let entries = match self.resolve(dir, || outside(dir))?.and_then(fs::read_dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(adr::Dir::new()),
            Err(e) => return Err(format!("{}: unreadable — {e}", dir.display())),
        };
        let mut files = adr::Dir::new();
        for entry in entries {
            let entry = entry.map_err(|e| format!("{}: unreadable — {e}", dir.display()))?;
            let path = dir.join(entry.file_name());
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                return Err(format!(
                    "{}: a file name that is no UTF-8 — not decided",
                    path.display()
                ));
            };
            let own_file = entry.file_type().is_ok_and(|kind| kind.is_file());
            let wanted = (own_file && adr::is_adr_name(&name))
                || ((name == "README.md" || name == "template.md") && path.is_file());
            let bytes = if wanted {
                self.read_record_file(&path)?
                    .map(|bytes| unit_separator_free(&path, bytes))
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
    pub(super) fn read_adr_targets(
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
            let resolved = self.resolve(&path, || {
                format!(
                    "docs/adr/README.md: a row links '{target}', outside the repository — not decided"
                )
            })?;
            let bytes = match resolved {
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
    pub(super) fn read_text_files(&self) -> Result<BTreeMap<String, Vec<u8>>, String> {
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
    pub(super) fn read_link_targets(
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

/// The refusal of a path that resolves outside the repository.
fn outside(path: &Path) -> String {
    format!(
        "{}: resolves outside the repository — not decided",
        path.display()
    )
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

/// The path under the root of the bytes git names: on Unix any bytes, elsewhere only UTF-8,
/// which is what git writes there.
fn path_of(root: &Path, raw: &[u8]) -> Option<PathBuf> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        Some(root.join(std::ffi::OsStr::from_bytes(raw)))
    }
    #[cfg(not(unix))]
    {
        std::str::from_utf8(raw).ok().map(|path| root.join(path))
    }
}

impl Tree {
    /// The names of the entries of a directory of the repository, sorted by their bytes as a glob
    /// in the C locale sorts them; empty when it is no directory. A directory that resolves
    /// outside the repository is not listed, and a name that is no UTF-8 is not decided.
    fn list(&self, dir: &Path) -> Result<Vec<String>, String> {
        let real = match self.resolve(dir, || outside(dir))? {
            Ok(real) => real,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(format!("{}: unreadable — {e}", dir.display())),
        };
        let entries = match fs::read_dir(&real) {
            Ok(entries) => entries,
            Err(e) if matches!(e.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) => {
                return Ok(Vec::new());
            }
            Err(e) => return Err(format!("{}: unreadable — {e}", real.display())),
        };
        let mut names = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| format!("{}: unreadable — {e}", real.display()))?;
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

    /// The files of check 9: `checks.yml`, `check-all.sh`, the workflows as the bash's globs
    /// list them, and the scripts as `find -type f` lists them.
    pub(super) fn read_check_lists(&self) -> Result<rules::CheckLists, String> {
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
    pub(super) fn read_layout(&self, readme: Option<&str>) -> Result<rules::Layout, String> {
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
                    self.resolve(dir, || {
                        format!(
                            "README.md: the Project Layout pattern '{pattern}' looks outside the repository — not decided"
                        )
                    })?
                    .map_err(|e| format!("{}: unreadable — {e}", dir.display()))?;
                }
                for name in self.list(dir)? {
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
    pub(super) fn read_landing(&self) -> Result<rules::docs_no_draft_on_main::Landing, String> {
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
                    // The bash reads find's list by lines, so a newline splits such a name in two.
                    if name.contains('\n') {
                        return Err(format!(
                            "docs/adr/{}: a draft whose name holds a newline, which the bash reads as two — not decided",
                            name.escape_debug()
                        ));
                    }
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
    pub(super) fn read_skills(&self) -> Result<rules::Skills, String> {
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
                // Resolved as `cd -P` resolves it, to compare and show, never read: a pointer
                // outside the repository is a finding, not a refusal.
                let real_dir = |path: &Path| fs::canonicalize(path).ok().filter(|r| r.is_dir());
                match real_dir(&p) {
                    None => rules::Entry::Dangles,
                    Some(target) => {
                        let want = real_dir(&skills.join(&name));
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
