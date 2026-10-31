//! The structural test of the shape of the code. Verifies: ADR-draft-functional-core
//!
//! The library outside `src/shell/` names nothing of `functional_core.forbidden`, the list kept
//! beside this test, and `src/main.rs` does nothing but call the shell. The source is read as
//! text: comments and literals are dropped, and the paths of a grouped `use` are expanded, so
//! `use std::{fs, io::stdout}` names `std::fs` and `std::io::stdout`. An opening, a glob as
//! `use std::*` or a rename as `use std as s`, counts as naming every entry it can bring into
//! scope. A rename inside a group, as `use std::{fs as f}`, is read as the path it renames.
//!
//! Each module under `src/rules/` is one rule: it declares exactly one `RULE_ID`, an
//! `area/name` unique across the crate, and the list of every rule names its `RULE`.

use std::fs;
use std::path::{Path, PathBuf};

const FORBIDDEN: &str = include_str!("functional_core.forbidden");

/// The forms `src/main.rs` may take, with comments and whitespace removed.
const MAIN_FORMS: [&str; 2] = [
    "usestd::process::ExitCode;fnmain()->ExitCode{nuc::shell::main()}",
    "fnmain()->std::process::ExitCode{nuc::shell::main()}",
];

/// The entries of the forbidden list: its lines without comments and blank lines.
fn entries(list: &str) -> Vec<&str> {
    list.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

/// The source with comments, string literals, and character literals replaced by a space.
fn code(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let mut i = 0;
    while i < chars.len() {
        let skipped = skip_comment(&chars, i)
            .or_else(|| skip_string(&chars, i))
            .or_else(|| skip_char(&chars, i));
        match skipped {
            Some(end) => {
                out.push(' ');
                i = end;
            }
            None => {
                out.push(chars[i]);
                i += 1;
            }
        }
    }
    out
}

/// The index after a line or block comment that starts at `i`, if one does.
fn skip_comment(chars: &[char], i: usize) -> Option<usize> {
    match (chars.get(i), chars.get(i + 1)) {
        (Some('/'), Some('/')) => Some(
            (i..chars.len())
                .find(|&j| chars[j] == '\n')
                .unwrap_or(chars.len()),
        ),
        (Some('/'), Some('*')) => {
            let mut depth = 0;
            let mut j = i;
            while j < chars.len() {
                match (chars[j], chars.get(j + 1)) {
                    ('/', Some('*')) => (depth, j) = (depth + 1, j + 2),
                    ('*', Some('/')) => (depth, j) = (depth - 1, j + 2),
                    _ => j += 1,
                }
                if depth == 0 {
                    return Some(j);
                }
            }
            Some(chars.len())
        }
        _ => None,
    }
}

/// The index after a string literal, plain, byte, C, or raw, that starts at `i`, if one does.
fn skip_string(chars: &[char], i: usize) -> Option<usize> {
    let ident_before = i > 0 && is_ident(chars[i - 1]);
    let mut j = i;
    if matches!(chars.get(j), Some('b' | 'c')) {
        j += 1;
    }
    let raw = chars.get(j) == Some(&'r');
    if raw {
        j += 1;
    }
    let hashes = chars[j.min(chars.len())..]
        .iter()
        .take_while(|&&c| c == '#')
        .count();
    j += hashes;
    if chars.get(j) != Some(&'"') || (ident_before && j > i) || (!raw && hashes > 0) {
        return None;
    }
    j += 1;
    while j < chars.len() {
        match chars[j] {
            '\\' if !raw => j += 2,
            '"' if chars[j + 1..].iter().take_while(|&&c| c == '#').count() >= hashes => {
                return Some(j + 1 + hashes);
            }
            _ => j += 1,
        }
    }
    Some(chars.len())
}

/// The index after a character literal that starts at `i`, if one does; a lifetime is none.
fn skip_char(chars: &[char], i: usize) -> Option<usize> {
    if chars.get(i) != Some(&'\'') {
        return None;
    }
    match chars.get(i + 1) {
        Some('\\') => (i + 2..chars.len())
            .find(|&j| chars[j] == '\'')
            .map(|j| j + 1),
        Some(_) if chars.get(i + 2) == Some(&'\'') => Some(i + 3),
        _ => None,
    }
}

fn is_ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Every path the code names, the paths of a grouped `use` expanded with their prefix, a
/// trailing `::self` dropped.
fn paths(code: &str) -> Vec<String> {
    let compact: Vec<char> = code.chars().collect();
    let mut found = Vec::new();
    // One frame per open brace: the prefix of a `use` group, or None for any other block.
    let mut frames: Vec<Option<String>> = Vec::new();
    let mut current = String::new();
    let mut i = 0;
    while i < compact.len() {
        let c = compact[i];
        if c.is_whitespace() {
            // Whitespace beside `::` lies inside a path; anywhere else it ends one.
            let next = (i..compact.len())
                .find(|&k| !compact[k].is_whitespace())
                .unwrap_or(compact.len());
            let joins = current.ends_with("::")
                || (compact.get(next) == Some(&':') && compact.get(next + 1) == Some(&':'));
            if !joins {
                emit(
                    &mut found,
                    frames.last().cloned().flatten().as_deref(),
                    &current,
                );
                current.clear();
            }
            i = next;
            continue;
        }
        if is_ident(c) || (c == ':' && compact.get(i + 1) == Some(&':')) {
            let step = if c == ':' { 2 } else { 1 };
            current.extend(&compact[i..i + step]);
            i += step;
            continue;
        }
        let prefix = frames.last().cloned().flatten();
        if c == '{' && current.ends_with("::") {
            let group = format!("{}{current}", prefix.unwrap_or_default());
            frames.push(Some(group));
        } else {
            emit(&mut found, prefix.as_deref(), &current);
            match c {
                '{' => frames.push(None),
                '}' => drop(frames.pop()),
                _ => {}
            }
        }
        current.clear();
        i += 1;
    }
    emit(&mut found, None, &current);
    found
}

fn emit(found: &mut Vec<String>, prefix: Option<&str>, path: &str) {
    if path.is_empty() {
        return;
    }
    // A rename keeps its keyword, unprefixed, so that the path before it reads as an opening.
    if path == "as" {
        found.push(path.to_owned());
        return;
    }
    let full = format!("{}{path}", prefix.unwrap_or_default());
    found.push(full.strip_suffix("::self").unwrap_or(&full).to_owned());
}

/// Whether the segments of `entry` appear, in order and adjacent, among those of `path`.
fn names(path: &str, entry: &str) -> bool {
    let path: Vec<&str> = path.split("::").collect();
    let entry: Vec<&str> = entry.split("::").collect();
    path.windows(entry.len())
        .any(|window| window == entry.as_slice())
}

/// Whether an opening, a glob `path::*` or a rename `path as x`, can bring the entry into scope
/// under a shorter or another name: the entry starts with the last segments of the path.
fn opens(path: &str, entry: &str) -> bool {
    let path: Vec<&str> = path.trim_end_matches("::").split("::").collect();
    let entry: Vec<&str> = entry.trim_end_matches('!').split("::").collect();
    (1..=path.len().min(entry.len())).any(|k| entry[..k] == path[path.len() - k..])
}

/// The openings among the paths: each glob, and each path a rename follows.
fn openings(paths: &[String]) -> Vec<&str> {
    paths
        .iter()
        .enumerate()
        .filter(|(i, path)| path.ends_with("::") || paths.get(i + 1).is_some_and(|n| n == "as"))
        .map(|(_, path)| path.as_str())
        .collect()
}

/// Whether the code invokes the macro, as `name!` with no identifier character before it.
fn invokes(code: &str, mac: &str) -> bool {
    code.match_indices(mac)
        .any(|(at, _)| !code[..at].chars().next_back().is_some_and(is_ident))
}

/// The entries of the list the source names.
fn violations(source: &str, list: &str) -> Vec<String> {
    let code = code(source);
    let paths = paths(&code);
    let openings = openings(&paths);
    entries(list)
        .into_iter()
        .filter(|entry| {
            let named = match entry.strip_suffix('!') {
                Some(_) => invokes(&code, entry),
                None => paths.iter().any(|path| names(path, entry)),
            };
            named || openings.iter().any(|opening| opens(opening, entry))
        })
        .map(str::to_owned)
        .collect()
}

/// Whether `src/main.rs` does nothing but call the shell.
fn main_is_thin(source: &str) -> bool {
    let compact: String = code(source)
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    MAIN_FORMS.contains(&compact.as_str())
}

/// The `.rs` files under `dir`, recursively, except those under `skip`.
fn sources(dir: &Path, skip: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("src/ is readable") {
        let path = entry.expect("a directory entry is readable").path();
        if path.starts_with(skip) {
            continue;
        }
        if path.is_dir() {
            sources(&path, skip, found);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            found.push(path);
        }
    }
}

/// What is wrong with a rule module's ID: how many `RULE_ID` constants its code declares, and
/// whether the one it declares is an `area/name`. `None` when it declares one well-formed ID.
fn rule_id_problem(source: &str) -> Option<String> {
    let compact: String = code(source)
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let declared = compact.matches("constRULE_ID:").count();
    if declared != 1 {
        return Some(format!("declares {declared} RULE_ID constants, not one"));
    }
    let id = rule_id(source).unwrap_or_default();
    (!is_area_name(&id)).then(|| format!("declares the rule ID '{id}', which is no area/name"))
}

/// The string the `RULE_ID` constant is set to, as written on its line.
fn rule_id(source: &str) -> Option<String> {
    let line = source.lines().find(|line| line.contains("const RULE_ID"))?;
    line.split('"').nth(1).map(str::to_owned)
}

/// Whether the ID is two lowercase segments, `area/name`, the name in kebab case.
fn is_area_name(id: &str) -> bool {
    let word = |s: &str| {
        !s.is_empty()
            && s.bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
    };
    match id.split_once('/') {
        Some((area, name)) => word(area) && name.split('-').all(word),
        None => false,
    }
}

/// Whether the list of every rule, the code of `src/rules/mod.rs` without whitespace, has the
/// entry `<module>::RULE`, compared entry by entry.
fn listed(list: &str, module: &str) -> bool {
    let entries = list
        .split_once("ALL:&[Rule]=&[")
        .and_then(|(_, rest)| rest.split_once(']'))
        .map_or("", |(entries, _)| entries);
    entries
        .split(',')
        .any(|entry| entry == format!("{module}::RULE"))
}

#[test]
fn every_rule_module_declares_one_unique_rule_id_and_is_listed() {
    let rules = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/rules");
    let list = fs::read_to_string(rules.join("mod.rs")).expect("src/rules/mod.rs is readable");
    let list: String = code(&list).chars().filter(|c| !c.is_whitespace()).collect();
    let mut modules = Vec::new();
    sources(&rules, &rules.join("mod.rs"), &mut modules);
    assert!(!modules.is_empty(), "no rule module under src/rules/");
    let mut seen: Vec<String> = Vec::new();
    let mut problems = Vec::new();
    for path in &modules {
        let source = fs::read_to_string(path).expect("a rule module is readable");
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if let Some(problem) = rule_id_problem(&source) {
            problems.push(format!("{}: {problem}", path.display()));
        } else if let Some(id) = rule_id(&source) {
            if seen.contains(&id) {
                problems.push(format!(
                    "{}: the rule ID '{id}' is used twice",
                    path.display()
                ));
            }
            seen.push(id);
        }
        if !listed(&list, stem) {
            problems.push(format!("{}: not in the list of every rule", path.display()));
        }
    }
    assert!(
        problems.is_empty(),
        "the rules break their shape:\n{}",
        problems.join("\n")
    );
}

#[test]
fn a_rule_module_without_exactly_one_well_formed_id_is_found() {
    assert_eq!(
        rule_id_problem("pub const RULE_ID: &str = \"docs/template-release\";"),
        None
    );
    for source in [
        "pub const OTHER: &str = \"docs/x\";",
        "pub const RULE_ID: &str = \"docs/a\";\npub const RULE_ID: &str = \"docs/b\";",
        "pub const RULE_ID: &str = \"Docs/A\";",
        "pub const RULE_ID: &str = \"template-release\";",
        "pub const RULE_ID: &str = \"docs/template--release\";",
        "// pub const RULE_ID: &str = \"docs/a\";",
    ] {
        assert!(rule_id_problem(source).is_some(), "not found in: {source}");
    }
}

#[test]
fn the_core_names_no_effect() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    sources(&src, &src.join("shell"), &mut files);
    files.retain(|path| path != &src.join("main.rs"));
    assert!(!files.is_empty(), "no library source outside src/shell/");
    let found: Vec<String> = files
        .iter()
        .flat_map(|path| {
            let source = fs::read_to_string(path).expect("a source is readable");
            violations(&source, FORBIDDEN)
                .into_iter()
                .map(move |entry| format!("{}: names '{entry}'", path.display()))
        })
        .collect();
    assert!(
        found.is_empty(),
        "the core has effects only src/shell/ may have:\n{}",
        found.join("\n")
    );
}

#[test]
fn the_binary_only_calls_the_shell() {
    let main = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/main.rs");
    let source = fs::read_to_string(main).expect("src/main.rs is readable");
    assert!(
        main_is_thin(&source),
        "src/main.rs holds more than the call into the shell"
    );
}

#[test]
fn the_list_names_the_effects_of_the_decision() {
    let list = entries(FORBIDDEN);
    for entry in [
        "std::fs",
        "std::process",
        "std::env",
        "std::net",
        "io::stdout",
        "println!",
    ] {
        assert!(list.contains(&entry), "the forbidden list lacks '{entry}'");
    }
}

#[test]
fn a_path_is_found_in_every_form() {
    for source in [
        "use std::fs;",
        "fn f() { std::fs::read(\"x\"); }",
        "use std::{collections::HashMap, fs};",
        "use std::{io::{self, Write}, fs::File};",
        "use ::std::fs::File;",
        "use std::io; fn f() { io::stdout(); }",
        "use std :: { env } ;",
        "use std::*; fn f() { fs::read(\"x\"); }",
        "use std::io::*; fn f() { stdout(); }",
        "use std as s; fn f() { s::fs::read(\"x\"); }",
        "extern crate std as s;",
        "use std::io as sio; fn f() { sio::stdout(); }",
        "use std::{fs as f};",
        "use std::println as say;",
    ] {
        assert!(
            !violations(source, FORBIDDEN).is_empty(),
            "not found in: {source}"
        );
    }
}

#[test]
fn a_macro_is_found() {
    for source in [
        "fn f() { println!(\"x\"); }",
        "fn f() { std::eprintln!(\"x\"); }",
        "fn f() { dbg!(1); }",
    ] {
        assert!(
            !violations(source, FORBIDDEN).is_empty(),
            "not found in: {source}"
        );
    }
}

#[test]
fn comments_literals_and_lookalikes_are_no_effect() {
    for source in [
        "// std::fs::read and println!\nfn f() {}",
        "/* std::process /* nested */ std::env */ fn f() {}",
        "const S: &str = \"std::fs and println!(x)\";",
        "const R: &str = r#\"std::net \"quoted\" \"#;",
        "const C: char = '\"'; fn f<'a>(x: &'a str) -> &'a str { x }",
        "use std::io::Write; use std::collections::HashMap; fn f() { sprintln(); }",
        "mod fsck {} fn f() { my::std_fs::read(); }",
        "use super::*; use std::collections::*; use crate::finding::Finding as F;",
        "fn f() { let r = io::Result::<()>::Ok(()); }",
    ] {
        assert_eq!(
            violations(source, FORBIDDEN),
            Vec::<String>::new(),
            "in: {source}"
        );
    }
}

#[test]
fn a_string_does_not_hide_the_code_after_it() {
    let source = "fn f() { let u = \"https://example.org\"; std::fs::read(u); }";
    assert_eq!(violations(source, FORBIDDEN), ["std::fs"]);
}

#[test]
fn only_the_call_into_the_shell_is_thin() {
    assert!(main_is_thin(
        "use std::process::ExitCode;\n\n// Comment.\nfn main() -> ExitCode {\n    nuc::shell::main()\n}\n"
    ));
    assert!(main_is_thin(
        "fn main() -> std::process::ExitCode { nuc::shell::main() }"
    ));
    assert!(!main_is_thin(
        "use std::process::ExitCode;\nfn main() -> ExitCode {\n    let _ = 1;\n    nuc::shell::main()\n}\n"
    ));
    assert!(!main_is_thin("fn main() { nuc::shell::main(); }"));
}

#[test]
fn a_rule_is_listed_only_by_its_own_entry() {
    let list = "pubconstALL:&[Rule]=&[docs_template_release::RULE,other::RULE];";
    assert!(listed(list, "docs_template_release"));
    assert!(listed(list, "other"));
    assert!(!listed(list, "release"));
    assert!(!listed(list, "template_release"));
    assert!(!listed("pubconstALL:&[Rule]=&[];", "other"));
}
