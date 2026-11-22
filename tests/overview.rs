//! Each module of the crate is named by the architecture overview.
//! Verifies: NUC-documents-are-fixed
//!
//! A change to the structure updates `docs/ARCHITECTURE.md` in the same change
//! (NUC-documents-are-fixed §4). The modules are each `src/*.rs` but `lib.rs` and `main.rs`, each
//! `src/shell/*.rs` but `mod.rs`, and each directory directly under `src/` that is a module, with a
//! `<dir>.rs` or a `<dir>/mod.rs`. The overview names a module by its path, as `src/upgrade.rs`,
//! `src/shell/upgrade.rs`, or `src/rules/`, or by its path in Rust in a code span: a module of the
//! shell as `shell::upgrade`, another as the start of a path into it, as `upgrade::ALL`, where no
//! module of the shell has its name. A module the overview names under another name stands in
//! `overview.unnamed`, the list kept beside this test, with its reason; a line of that list whose
//! module the overview names, or that is no module, fails as stale.

mod support;

use std::fs;
use std::path::Path;

use support::sources::root;

const UNNAMED: &str = include_str!("overview.unnamed");

/// A module of the crate: its path under `src/`, a directory with a trailing slash, and its path
/// in Rust.
type Module = (String, String);

/// The modules of the crate under `src`.
fn modules(src: &Path) -> Vec<Module> {
    let names = |dir: &Path| -> Vec<(String, bool)> {
        fs::read_dir(dir)
            .expect("the directory is readable")
            .map(|entry| {
                let entry = entry.expect("a directory entry is readable");
                let name = entry.file_name().into_string().expect("a UTF-8 name");
                (name, entry.file_type().expect("a type").is_dir())
            })
            .collect()
    };
    let mut modules = Vec::new();
    for (name, is_dir) in names(src) {
        if is_dir {
            let module = src.join(format!("{name}.rs")).is_file()
                || src.join(&name).join("mod.rs").is_file();
            if module {
                modules.push((format!("{name}/"), name));
            }
        } else if let Some(stem) = name.strip_suffix(".rs")
            && !["lib", "main"].contains(&stem)
        {
            modules.push((name.clone(), stem.to_owned()));
        }
    }
    for (name, is_dir) in names(&src.join("shell")) {
        if let Some(stem) = name.strip_suffix(".rs")
            && !is_dir
            && stem != "mod"
        {
            modules.push((format!("shell/{name}"), format!("shell::{stem}")));
        }
    }
    modules.sort();
    modules
}

/// Whether the overview names the module, among `modules`, by its path or by its path in Rust.
fn named(overview: &str, modules: &[Module], (path, rust): &Module) -> bool {
    if overview.contains(&format!("src/{path}")) {
        return true;
    }
    if rust.contains("::") {
        return overview.contains(&format!("`{rust}`")) || overview.contains(&format!("`{rust}::"));
    }
    let shadowed = modules.iter().any(|(_, r)| *r == format!("shell::{rust}"));
    !shadowed && overview.contains(&format!("`{rust}::"))
}

/// The modules a list of exceptions names, each line `<path> <reason>`; an error names a line
/// without a reason.
fn exceptions(text: &str) -> Result<Vec<&str>, String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| match line.split_once(' ') {
            Some((path, reason)) if !reason.trim().is_empty() => Ok(path),
            _ => Err(format!("'{line}' gives no reason")),
        })
        .collect()
}

/// What fails: each module the overview does not name and the list does not except, and each line
/// of the list that is no module or whose module the overview names.
fn failures(overview: &str, modules: &[Module], listed: &[&str]) -> Vec<String> {
    let mut failures: Vec<String> = modules
        .iter()
        .filter(|module| !named(overview, modules, module) && !listed.contains(&module.0.as_str()))
        .map(|(path, _)| format!("docs/ARCHITECTURE.md names no module {path}"))
        .collect();
    for path in listed {
        match modules.iter().find(|(p, _)| p == path) {
            None => failures.push(format!("overview.unnamed: {path} is no module")),
            Some(module) if named(overview, modules, module) => failures.push(format!(
                "overview.unnamed: the overview names {path}, so its line is stale"
            )),
            Some(_) => {}
        }
    }
    failures
}

#[test]
fn the_overview_names_every_module_of_the_crate() {
    let overview =
        fs::read_to_string(root().join("docs/ARCHITECTURE.md")).expect("the overview is readable");
    let listed = exceptions(UNNAMED).unwrap_or_else(|e| panic!("overview.unnamed: {e}"));
    let failures = failures(&overview, &modules(&root().join("src")), &listed);
    assert!(
        failures.is_empty(),
        "{failures:#?}\nname each module in the overview, by its path under src/ or its path in \
         Rust, in the change that adds it (NUC-documents-are-fixed §4)"
    );
}

#[test]
fn a_module_the_overview_does_not_name_is_found() {
    let module = |path: &str, rust: &str| (path.to_owned(), rust.to_owned());
    let modules = vec![
        module("config.rs", "config"),
        module("git.rs", "git"),
        module("shell/git.rs", "shell::git"),
        module("shell/upgrade.rs", "shell::upgrade"),
        module("upgrade.rs", "upgrade"),
    ];
    let named = |overview: &str, m: &Module| named(overview, &modules, m);
    assert!(named("`src/upgrade.rs` decides", &modules[4]));
    // A path into a module names it, unless a module of the shell shares its name.
    assert!(named("the keys of `config::Version`", &modules[0]));
    assert!(!named("the plan of `upgrade::ALL`", &modules[4]));
    assert!(!named("the upgrade decides, in `upgrade`", &modules[4]));
    assert!(named("`shell::upgrade` reads", &modules[3]));
    assert!(!named("`src/upgrade.rs`", &modules[3]));
    // `git::diff` is the shell's module, which shares the name: it names no `src/git.rs`.
    assert!(!named("`git::diff` runs git", &modules[1]));
    assert!(named("`src/git.rs` reads", &modules[1]));
    // The list: a line without a reason, a stale line, a line that is no module.
    assert!(exceptions("  upgrade.rs\n").is_err());
    assert_eq!(
        exceptions("# x\n  upgrade.rs as `up`\n"),
        Ok(vec!["upgrade.rs"])
    );
    let overview = "`config::read`, `src/git.rs`, `shell::git`, `shell::upgrade`";
    assert_eq!(
        failures(overview, &modules, &[]),
        ["docs/ARCHITECTURE.md names no module upgrade.rs"]
    );
    assert!(failures(overview, &modules, &["upgrade.rs"]).is_empty());
    assert_eq!(
        failures(overview, &modules, &["upgrade.rs", "git.rs", "gone.rs"]),
        [
            "overview.unnamed: the overview names git.rs, so its line is stale",
            "overview.unnamed: gone.rs is no module",
        ]
    );
    // The crate: a file, a module of the shell, a directory; never lib.rs, main.rs, or mod.rs.
    let crate_modules = self::modules(&root().join("src"));
    for want in ["upgrade.rs", "shell/upgrade.rs", "rules/"] {
        assert!(
            crate_modules.iter().any(|(p, _)| p == want),
            "{want} is no module"
        );
    }
    for skipped in ["lib.rs", "main.rs", "shell/mod.rs"] {
        assert!(
            !crate_modules.iter().any(|(p, _)| p == skipped),
            "{skipped} counts"
        );
    }
}
