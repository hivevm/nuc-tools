//! The crate's own sources as the structural tests read them: every `.rs` file under a directory,
//! the rule modules, and the rule ID a module declares. One definition, so that no copy reads a
//! path by its display, which differs on Windows.

use std::fs;
use std::path::{Path, PathBuf};

/// The root of the crate: the directory cargo fixed when it built this test binary, which must be
/// the checkout cargo runs the test in, as it names it in `CARGO_MANIFEST_DIR` at run time; a
/// binary built from another checkout into the same target would read that checkout's tree, or
/// one that is gone. A binary started without cargo names no checkout and is not compared.
pub fn root() -> &'static Path {
    let built = Path::new(env!("CARGO_MANIFEST_DIR"));
    if let Some(running) = std::env::var_os("CARGO_MANIFEST_DIR") {
        same_checkout(built, Path::new(&running));
    }
    built
}

/// Fails where the checkout a test binary was built from is not the one it runs in.
pub fn same_checkout(built: &Path, running: &Path) {
    let same = matches!(
        (built.canonicalize(), running.canonicalize()),
        (Ok(built), Ok(running)) if built == running
    );
    assert!(
        same,
        "this test binary was built from {} but runs in {}; build it again from here: \
         cargo clean -p nuc-tools",
        built.display(),
        running.display()
    );
}

/// The `.rs` files under `dir`, recursively and sorted, except those under `skip`.
pub fn rust_files(dir: &Path, skip: Option<&Path>) -> Vec<PathBuf> {
    let mut found = Vec::new();
    walk(dir, skip, &mut found);
    found.sort();
    found
}

fn walk(dir: &Path, skip: Option<&Path>, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("the directory is readable") {
        let path = entry.expect("a directory entry is readable").path();
        if skip.is_some_and(|skip| path.starts_with(skip)) {
            continue;
        }
        if path.is_dir() {
            walk(&path, skip, found);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            found.push(path);
        }
    }
}

/// The rule modules: the `.rs` files under `src/rules/` but its `mod.rs`, compared by path, never
/// by how a path displays.
pub fn rule_modules() -> Vec<PathBuf> {
    let rules = root().join("src/rules");
    rust_files(&rules, Some(&rules.join("mod.rs")))
}

/// The string the `RULE_ID` constant is set to: the first quoted string on the first line that
/// names `const RULE_ID`, as the structural test counts the declaration by those words. A comment
/// line naming them comes first if it stands above the declaration; no rule module has one.
pub fn rule_id(source: &str) -> Option<String> {
    let line = source.lines().find(|line| line.contains("const RULE_ID"))?;
    line.split('"').nth(1).map(str::to_owned)
}
