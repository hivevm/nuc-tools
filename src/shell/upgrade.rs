//! `nuc upgrade <tag>`: the shell of the take-up (API-upgrade-takes-up-by-blob). It reads
//! `nuc.toml` as every command does, the release the README names, the two template tags under
//! `refs/template/`, and the trees of both and of `HEAD`; the core decides the plan; the shell
//! refuses before the first write what the plan or the working tree does not allow, reads every
//! blob the plan writes, then writes the plan, moves the release line, and prints the judgments.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use crate::config;
use crate::finding::Finding;
use crate::upgrade::{self, Change, Entry, Found, Kind, Plan, Refusal, Tree, WorkTree};

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
    let mut plan = upgrade::plan(&was, &now, &head, (old, new), carries_mode(&root)?);
    upgrade::refuse_paths(&plan, &head).map_err(|r| r.to_string())?;
    let work = work_tree(&root, &plan)?;
    upgrade::settle(&mut plan, &head, &work).map_err(|r| r.to_string())?;
    let (blobs, mut judgments) = read_and_cite(&root, &plan, &head, &now, new)?;
    judgments.extend(upgrade::skipped(old, new));
    // The release line is moved in the README as the plan leaves it: the new release's where the
    // plan takes it up, the project's otherwise.
    let readme = match plan.changes.get("README.md") {
        None => readme,
        Some(Change::Write(entry)) => blobs[&entry.object].clone(),
        Some(Change::Delete) => {
            return Err(refusal(
                "release",
                format!(
                    "README.md: {} removes it, and with it the release line",
                    upgrade::tag(new)
                ),
            ));
        }
    };
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
    plan.judgments.extend(judgments);
    plan.judgments
        .sort_by(|a, b| (&a.file, a.line).cmp(&(&b.file, b.line)));
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
        written + usize::from(readme.is_some() && !plan.changes.contains_key("README.md")),
        plan.judgments
            .iter()
            .map(|judgment| judgment.file.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
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

/// What `settle` needs of the working tree: what it holds at each path of the plan and each
/// directory above, each entry below a directory at a path the plan writes, and whether its file
/// system ignores case.
fn work_tree(root: &Path, plan: &Plan) -> Result<WorkTree, String> {
    let found: BTreeMap<String, Found> = upgrade::asked(plan)
        .into_iter()
        .map(|path| {
            let at = found(root, &path);
            (path, at)
        })
        .collect();
    let mut below = BTreeMap::new();
    for (path, change) in &plan.changes {
        // Listed only where the path and each directory above it is a directory, no link, so that
        // nothing outside the tree is read (Q-12).
        let real = path
            .match_indices('/')
            .map(|(at, _)| &path[..at])
            .chain(std::iter::once(path.as_str()))
            .all(|dir| found.get(dir) == Some(&Found::Dir));
        if matches!(change, Change::Write(_)) && real {
            list_below(root, path, &mut below)?;
        }
    }
    let ignores_case = git::probe(root, &["config", "--type=bool", "core.ignoreCase"])?
        .is_some_and(|value| value.trim() == "true");
    Ok(WorkTree {
        found,
        below,
        ignores_case,
    })
}

/// Each entry below the directory at `dir`, with what it is, read without following a link: a
/// directory is listed in turn, a link is not. An entry whose name is no UTF-8 counts as a file of
/// the working tree's own, which the plan deletes nowhere.
fn list_below(root: &Path, dir: &str, below: &mut BTreeMap<String, Found>) -> Result<(), String> {
    let entries = fs::read_dir(root.join(dir))
        .map_err(|e| format!("{dir}: unreadable — {e} — not decided"))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("{dir}: unreadable — {e} — not decided"))?;
        let name = entry.file_name();
        let path = format!("{dir}/{}", name.to_string_lossy());
        let kind = match entry.file_type() {
            Ok(kind) if kind.is_symlink() => Found::Link,
            Ok(kind) if kind.is_dir() => Found::Dir,
            Ok(kind) if kind.is_file() => Found::Regular,
            _ => Found::Other,
        };
        let kind = if name.to_str().is_none() {
            Found::Other
        } else {
            kind
        };
        below.insert(path.clone(), kind);
        if kind == Found::Dir {
            list_below(root, &path, below)?;
        }
    }
    Ok(())
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

/// The blobs the plan writes and the files outside the base as the plan leaves it, read in one
/// batch before the first write, and each line of those files that cites a `NUC-` ADR the new
/// release, `now` at `new`, lacks.
fn read_and_cite(
    root: &Path,
    plan: &Plan,
    head: &Tree,
    now: &Tree,
    new: config::Version,
) -> Result<(Blobs, Vec<Finding>), String> {
    let kept = upgrade::kept_files(plan, head);
    let mut objects: BTreeSet<&str> = kept.values().map(String::as_str).collect();
    objects.extend(plan.changes.values().filter_map(|change| match change {
        Change::Write(entry) => Some(entry.object.as_str()),
        Change::Delete => None,
    }));
    let blobs = read_blobs(root, &objects)?;
    let texts: BTreeMap<String, &[u8]> = kept
        .iter()
        .map(|(path, object)| (path.clone(), blobs[object].as_slice()))
        .collect();
    let judgments = upgrade::citations(&texts, &upgrade::base_identifiers(now), new);
    Ok((blobs, judgments))
}

/// The bytes of blobs, by object.
type Blobs = BTreeMap<String, Vec<u8>>;

/// The bytes of each blob, read by one `git cat-file --batch` before the first write, so that a
/// blob the clone lacks refuses the run with nothing written (SEC-git-cleared-without-lazy-fetch).
fn read_blobs(root: &Path, objects: &BTreeSet<&str>) -> Result<Blobs, String> {
    let mut input = Vec::new();
    for object in objects {
        input.extend_from_slice(object.as_bytes());
        input.push(b'\n');
    }
    let out = git::run_bytes_with_input(root, &["cat-file", "--batch"], &input)?;
    let mut blobs = BTreeMap::new();
    let mut rest = out.as_slice();
    for object in objects {
        // '<object> blob <size>\n<bytes>\n', or '<object> missing\n'.
        let end = rest.iter().position(|&b| b == b'\n').unwrap_or(rest.len());
        let head = String::from_utf8_lossy(&rest[..end]).into_owned();
        let size = match head.split(' ').collect::<Vec<_>>().as_slice() {
            [name, "blob", size] if name == object => size.parse::<usize>().ok(),
            _ => None,
        };
        let Some(size) = size.filter(|size| end + 1 + size < rest.len() + 1) else {
            return Err(format!(
                "the object {object} is not a blob in the repository, as in a partial clone, and \
                 nuc fetches none — fetch it first — not decided"
            ));
        };
        let body = &rest[end + 1..end + 1 + size];
        blobs.insert((*object).to_owned(), body.to_vec());
        rest = rest.get(end + 2 + size..).unwrap_or_default();
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
        // A directory there, which the plan's deletions left empty, is removed first.
        let removed = match fs::symlink_metadata(&file) {
            Ok(meta) if meta.is_dir() => fs::remove_dir(&file),
            _ => fs::remove_file(&file),
        };
        match removed {
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
