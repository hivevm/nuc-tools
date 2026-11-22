//! `nuc upgrade <tag>`: the shell of the take-up (API-upgrade-takes-up-by-blob). It reads
//! `nuc.toml` as every command does, the release the README names, the two template tags under
//! `refs/template/`, and the trees of both and of `HEAD`; the core decides the plan; the shell
//! refuses before the first write what the plan or the working tree does not allow, reads every
//! blob the plan writes, then writes the plan, moves the release line, and prints the judgments.

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use crate::config;
use crate::upgrade::{self, Change, Entry, Found, Kind, Plan, Refusal, Tree};

use super::tree::Tree as Repo;
use super::{FOUND, PASSED, git};

/// `nuc upgrade <tag>`: the take-up of the release `<tag>` into the working tree of the repository
/// `nuc` runs in. `nuc.toml` is judged first, as for every command (G-5).
pub(super) fn run(args: &[&str]) -> Result<u8, String> {
    let [argument] = args else {
        return Err(refusal("usage", "usage: nuc upgrade <tag>"));
    };
    let start =
        env::current_dir().map_err(|e| format!("the current directory is unreadable: {e}"))?;
    let root = git::toplevel(&start)?;
    let repo = Repo::new(root)?;
    let text = repo
        .read(&repo.root.join("nuc.toml"))
        .map_err(config::unreadable)?
        .ok_or_else(|| config::Refusal::Missing.to_string())?;
    config::read(&text, super::running()).map_err(|refusal| refusal.to_string())?;
    let root = repo.root.clone();
    let branch = git::probe(&root, &["symbolic-ref", "-q", "--short", "HEAD"])?;
    if branch.as_deref().map(str::trim) == Some("main") {
        return Err(refusal(
            "main",
            "main is checked out; take a release up on a branch of its own",
        ));
    }
    if !super::hook::clean(&root)? {
        return Err(refusal(
            "changed",
            "the working tree or the index differs from HEAD, an untracked file included, or nuc \
             cannot tell without running a command of the repository; commit or stash first",
        ));
    }
    let readme = read_readme(&repo)?;
    let (old, new) = upgrade::releases(&String::from_utf8_lossy(&readme), argument)
        .map_err(|r| r.to_string())?;
    let (old_ref, new_ref) = (template_ref(&root, old)?, template_ref(&root, new)?);
    refuse_newer_nuc(&root, &new_ref, new)?;
    let (was, now, head) = (
        read_tree(&root, &old_ref)?,
        read_tree(&root, &new_ref)?,
        read_tree(&root, "HEAD")?,
    );
    let mut plan = upgrade::plan(&was, &now, &head, carries_mode(&root)?);
    upgrade::refuse_paths(&plan, &head).map_err(|r| r.to_string())?;
    let found = upgrade::asked(&plan)
        .into_iter()
        .map(|path| {
            let at = found(&root, &path);
            (path, at)
        })
        .collect();
    let ignores_case = git::probe(&root, &["config", "--type=bool", "core.ignoreCase"])?
        .is_some_and(|value| value.trim() == "true");
    upgrade::settle(&mut plan, &head, &found, ignores_case).map_err(|r| r.to_string())?;
    let blobs = read_blobs(&root, &plan)?;
    let moved = upgrade::move_release_line(&readme, old, new).ok_or_else(|| {
        refusal(
            "release",
            format!(
                "README.md: the Template release line, read as UTF-8, names neither {} nor {}",
                upgrade::tag(old),
                upgrade::tag(new)
            ),
        )
    })?;
    let readme = (readme != moved).then_some(moved);
    write_plan(&root, &plan, &blobs, readme.as_deref())?;
    for judgment in &plan.judgments {
        println!("{judgment}");
    }
    let (written, deleted) = plan
        .changes
        .values()
        .fold((0, 0), |(w, d), change| match change {
            Change::Write(_) => (w + 1, d),
            Change::Delete => (w, d + 1),
        });
    eprintln!(
        "nuc upgrade {} -> {}: {} written, {deleted} deleted, {} listed; read the notes of {}: git \
         cat-file tag {new_ref}.",
        upgrade::tag(old),
        upgrade::tag(new),
        written + usize::from(readme.is_some()),
        plan.judgments.len(),
        upgrade::tag(new),
    );
    Ok(if plan.judgments.is_empty() {
        PASSED
    } else {
        FOUND
    })
}

/// The README's bytes as they are, which the release line is spliced into, every other byte kept;
/// only a regular file, which a write cannot follow out of the tree.
fn read_readme(repo: &Repo) -> Result<Vec<u8>, String> {
    let root = &repo.root;
    match found(root, "README.md") {
        Found::Nothing => None,
        Found::Regular => repo.read_bytes(&root.join("README.md"))?,
        _ => {
            return Err(refusal(
                "release",
                "README.md: no regular file, so no release line nuc upgrade reads and writes",
            ));
        }
    }
    .ok_or_else(|| {
        refusal(
            "release",
            "README.md: not found, so no release to take up from",
        )
    })
}

/// The refusal keyed `[upgrade/run:<reason>]`, as the shell returns it.
fn refusal(reason: &'static str, message: impl Into<String>) -> String {
    Refusal::new(reason, message).to_string()
}

/// Whether the working tree carries an executable bit: never on a platform without one, and
/// elsewhere where `core.fileMode` is not false, as git reads it.
fn carries_mode(root: &Path) -> Result<bool, String> {
    if cfg!(not(unix)) {
        return Ok(false);
    }
    let value = git::probe(root, &["config", "--type=bool", "core.fileMode"])?;
    Ok(value.as_deref().map(str::trim) != Some("false"))
}

/// The ref of the release, `refs/template/<tag>`, refused unless it is an annotated tag of that
/// name, with the command that fetches it where it is missing.
fn template_ref(root: &Path, release: config::Version) -> Result<String, String> {
    let tag = upgrade::tag(release);
    let name = format!("refs/template/{tag}");
    let out = git::run(
        root,
        &[
            "for-each-ref",
            "--format=%(refname) %(objecttype) %(tag)",
            &name,
        ],
    )?;
    let expected = format!("{name} tag {tag}");
    if out.lines().any(|line| line == expected) {
        return Ok(name);
    }
    if out
        .lines()
        .any(|line| line.starts_with(&format!("{name} ")))
    {
        return Err(refusal(
            "ref",
            format!("{name}: no annotated tag named {tag}; fetch the template's tag {tag} there"),
        ));
    }
    Err(refusal(
        "ref",
        format!(
            "{name}: missing; fetch it with 'git fetch --no-tags <template> \
             refs/tags/{tag}:{name}'"
        ),
    ))
}

/// Refused where the new release's `nuc.toml` asks for a `nuc` newer than this one, or cannot be
/// read by it (G-5).
fn refuse_newer_nuc(root: &Path, new_ref: &str, new: config::Version) -> Result<(), String> {
    let tag = upgrade::tag(new);
    let text = git::blob_at(root, new_ref, "nuc.toml")?
        .ok_or_else(|| refusal("version", format!("{tag} carries no nuc.toml")))?;
    let text = String::from_utf8(text)
        .map_err(|_| refusal("version", format!("{tag}: its nuc.toml is no UTF-8")))?;
    config::read(&text, super::running()).map_err(|why| {
        refusal(
            "version",
            format!("{tag}: its nuc.toml asks for another nuc: {why}"),
        )
    })?;
    Ok(())
}

/// The tree of the revision, each path it holds with its entry; refused where a path is no UTF-8
/// or a mode names no entry `nuc upgrade` knows.
fn read_tree(root: &Path, rev: &str) -> Result<Tree, String> {
    let listing = git::run_bytes(root, &["ls-tree", "-r", "-z", "--full-tree", rev])?;
    let mut tree = Tree::new();
    // '<mode> <type> <object>\t<path>\0' for each entry.
    for entry in listing.split(|&b| b == 0).filter(|e| !e.is_empty()) {
        let unread = || {
            refusal(
                "path",
                format!(
                    "{rev}: an entry nuc upgrade cannot read: {}",
                    String::from_utf8_lossy(entry)
                ),
            )
        };
        let tab = entry.iter().position(|&b| b == b'\t').ok_or_else(unread)?;
        let head = std::str::from_utf8(&entry[..tab]).map_err(|_| unread())?;
        let path = std::str::from_utf8(&entry[tab + 1..]).map_err(|_| unread())?;
        let fields: Vec<&str> = head.split(' ').collect();
        let [mode, _, object] = fields.as_slice() else {
            return Err(unread());
        };
        let kind = Kind::of_mode(mode).ok_or_else(unread)?;
        tree.insert(
            path.to_owned(),
            Entry {
                kind,
                object: (*object).to_owned(),
            },
        );
    }
    Ok(tree)
}

/// What the working tree holds at the path, read without following a link.
fn found(root: &Path, path: &str) -> Found {
    match fs::symlink_metadata(root.join(path)) {
        Err(_) => Found::Nothing,
        Ok(meta) if meta.file_type().is_symlink() => Found::Link,
        Ok(meta) if meta.is_file() => Found::Regular,
        Ok(meta) if meta.is_dir() => Found::Dir,
        Ok(_) => Found::Other,
    }
}

/// The bytes of every blob the plan writes, read before the first write, so that a blob the clone
/// lacks refuses the run with nothing written (SEC-git-cleared-without-lazy-fetch).
fn read_blobs(root: &Path, plan: &Plan) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let mut blobs = BTreeMap::new();
    for change in plan.changes.values() {
        let Change::Write(entry) = change else {
            continue;
        };
        if blobs.contains_key(&entry.object) {
            continue;
        }
        let (code, _) = git::status_and_output(root, &["cat-file", "-e", &entry.object])?;
        if code != 0 {
            return Err(format!(
                "the object {} is not in the repository, as in a partial clone, and nuc fetches \
                 none — fetch it first — not decided",
                entry.object
            ));
        }
        let bytes = git::run_bytes(root, &["cat-file", "blob", &entry.object])?;
        blobs.insert(entry.object.clone(), bytes);
    }
    Ok(blobs)
}

/// The plan written into the working tree at `root`, and nowhere else: the deletions first, then
/// each directory they leave empty, then each file with the blob's bytes as git stores them, its
/// missing directories made, then the README with the release line moved. A write that fails
/// after the first refuses the run, `write`, naming what was written.
fn write_plan(
    root: &Path,
    plan: &Plan,
    blobs: &BTreeMap<String, Vec<u8>>,
    readme: Option<&[u8]>,
) -> Result<(), String> {
    let mut done: Vec<&str> = Vec::new();
    let failed = |path: &str, e: std::io::Error, done: &[&str]| {
        refusal(
            "write",
            format!(
                "{path}: {e}; written before it: {}; 'git restore .' and 'git clean' undo them",
                if done.is_empty() {
                    "nothing".to_owned()
                } else {
                    done.join(", ")
                }
            ),
        )
    };
    for (path, change) in &plan.changes {
        if *change == Change::Delete {
            fs::remove_file(root.join(path)).map_err(|e| failed(path, e, &done))?;
            done.push(path);
            let mut dir = Path::new(path.as_str()).parent();
            while let Some(parent) = dir.filter(|d| !d.as_os_str().is_empty()) {
                match fs::remove_dir(root.join(parent)) {
                    Ok(()) => dir = parent.parent(),
                    Err(e) if e.kind() == ErrorKind::DirectoryNotEmpty => break,
                    Err(e) if e.kind() == ErrorKind::NotFound => dir = parent.parent(),
                    Err(e) => return Err(failed(path, e, &done)),
                }
            }
        }
    }
    for (path, change) in &plan.changes {
        let Change::Write(entry) = change else {
            continue;
        };
        let file = root.join(path);
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent).map_err(|e| failed(path, e, &done))?;
        }
        // A file there is removed, not written over, so that a hard link to a file outside the
        // tree carries no byte out (Q-8), as git replaces a file it checks out.
        match fs::remove_file(&file) {
            Err(e) if e.kind() != ErrorKind::NotFound => return Err(failed(path, e, &done)),
            _ => {}
        }
        fs::write(&file, &blobs[&entry.object]).map_err(|e| failed(path, e, &done))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = if entry.kind == Kind::Executable {
                0o755
            } else {
                0o644
            };
            fs::set_permissions(&file, fs::Permissions::from_mode(mode))
                .map_err(|e| failed(path, e, &done))?;
        }
        done.push(path);
    }
    if let Some(readme) = readme {
        let file = root.join("README.md");
        let mode = fs::symlink_metadata(&file).map(|meta| meta.permissions());
        fs::remove_file(&file).map_err(|e| failed("README.md", e, &done))?;
        fs::write(&file, readme).map_err(|e| failed("README.md", e, &done))?;
        if let Ok(mode) = mode {
            fs::set_permissions(&file, mode).map_err(|e| failed("README.md", e, &done))?;
        }
    }
    Ok(())
}
