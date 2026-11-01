//! The structural test of the shape of the code. Verifies: ADR-0025
//!
//! The library outside `src/shell/` names nothing of `functional_core.forbidden`, the list kept
//! beside this test, and `src/main.rs` does nothing but call the shell. The source is read as
//! text: comments and literals are dropped, and the paths of a grouped `use` are expanded, so
//! `use std::{fs, io::stdout}` names `std::fs` and `std::io::stdout`. An opening, a glob as
//! `use std::*` or a rename as `use std as s`, counts as naming every entry it can bring into
//! scope. A rename inside a group, as `use std::{fs as f}`, is read as the path it renames.
//!
//! Inside `src/shell/`, a call that reads a file, lists a directory, or resolves a path, as
//! `functional_core.reads` lists them, stands only in a function `functional_core.guarded` names
//! for that call: those that resolve the path through `Tree::resolve` first, so that nothing
//! outside the repository is read, as the convention "The shell reads, the rules decide" of
//! `docs/CONVENTIONS.md` asks. A test of existence is free.
//!
//! The readings of the bash's tools that `functional_core.bash` names, as `awk_lines` and
//! `is_space`, are defined in `src/bash.rs` and nowhere else, so that no port writes a copy of its
//! own that reads a byte otherwise, as the convention "A port copies the bash, quirks included" of
//! `docs/CONVENTIONS.md` asks. The test reads names: a copy under another name, or a closure,
//! passes it.
//!
//! The shell carries out every rule's outcome alike: no source of `src/shell/` names what
//! `functional_core.outcomes` lists on a rule module, as its `RULE`, whose check it would call by
//! name. It may name a rule module for the types of its inputs and for the reading of them.
//!
//! Each module under `src/rules/` is one rule: it declares exactly one `RULE_ID`, an
//! `area/name` unique across the crate, and the list of every rule names its `RULE`.

use std::fs;
use std::path::{Path, PathBuf};

const FORBIDDEN: &str = include_str!("functional_core.forbidden");
const READS: &str = include_str!("functional_core.reads");
const GUARDED: &str = include_str!("functional_core.guarded");
const BASH_READERS: &str = include_str!("functional_core.bash");
const OUTCOMES: &str = include_str!("functional_core.outcomes");

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

/// Whether the word stands at `i` in the code, with no identifier character on either side.
fn word_at(chars: &[char], i: usize, word: &str) -> bool {
    let len = word.chars().count();
    chars
        .get(i..i + len)
        .is_some_and(|w| w.iter().copied().eq(word.chars()))
        && !(i > 0 && is_ident(chars[i - 1]))
        && !chars.get(i + len).copied().is_some_and(is_ident)
}

/// The identifier that starts at the first non-space character from `i`, and the index after it.
fn ident_from(chars: &[char], i: usize) -> Option<(String, usize)> {
    let start = (i..chars.len()).find(|&k| !chars[k].is_whitespace())?;
    let end = (start..chars.len())
        .find(|&k| !is_ident(chars[k]))
        .unwrap_or(chars.len());
    (end > start).then(|| (chars[start..end].iter().collect(), end))
}

/// The type an `impl` block is for, from the text between `impl` and its brace: the type after
/// `for` when it implements a trait, without its generics, references, and lifetimes.
fn impl_type(header: &str) -> String {
    // The header with every generic list, however nested, left out.
    let mut flat = String::new();
    let mut depth = 0usize;
    for c in header.chars() {
        match c {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            _ if depth == 0 => flat.push(c),
            _ => {}
        }
    }
    let ty = flat.rsplit(" for ").next().unwrap_or(&flat);
    ty.split_whitespace()
        .map(|word| word.trim_start_matches('&'))
        .find(|word| {
            !word.is_empty() && !word.starts_with('\'') && *word != "mut" && *word != "dyn"
        })
        .unwrap_or_default()
        .split(|c: char| !is_ident(c) && c != ':')
        .next()
        .unwrap_or_default()
        .to_owned()
}

/// The code of each function of the source, its signature and body, by its name: `Type::name`
/// inside an `impl Type` block, the bare name elsewhere. The code of a nested function is its
/// own; the code outside every function is under `<module>`.
fn functions(source: &str) -> Vec<(String, String)> {
    enum Block {
        Impl(String),
        Fn(String),
        Other,
    }
    let chars: Vec<char> = code(source).chars().collect();
    let mut found: Vec<(String, String)> = vec![("<module>".to_owned(), String::new())];
    let mut stack: Vec<Block> = Vec::new();
    // The function whose signature is being read, the brackets open in it, and where an `impl`
    // header started.
    let mut pending: Option<String> = None;
    let mut open = 0usize;
    let mut header: Option<usize> = None;
    let mut i = 0;
    while i < chars.len() {
        let owner = |stack: &[Block], pending: &Option<String>| {
            pending.clone().or_else(|| {
                stack.iter().rev().find_map(|block| match block {
                    Block::Fn(name) => Some(name.clone()),
                    _ => None,
                })
            })
        };
        if word_at(&chars, i, "fn")
            && let Some((name, end)) = ident_from(&chars, i + 2)
        {
            let name = match stack.last() {
                Some(Block::Impl(ty)) => format!("{ty}::{name}"),
                _ => name,
            };
            found.push((name.clone(), String::new()));
            pending = Some(name);
            open = 0;
            i = end;
            continue;
        }
        if pending.is_none() && word_at(&chars, i, "impl") {
            header = Some(i + 4);
        }
        match chars[i] {
            '{' => {
                stack.push(if let Some(name) = pending.take() {
                    Block::Fn(name)
                } else if let Some(start) = header {
                    Block::Impl(impl_type(&chars[start..i].iter().collect::<String>()))
                } else {
                    Block::Other
                });
                header = None;
            }
            '}' => drop(stack.pop()),
            '(' | '[' if pending.is_some() => open += 1,
            ')' | ']' if pending.is_some() => open = open.saturating_sub(1),
            // A declaration without a body ends at its `;`; one inside `[u8; 4]` is no end.
            ';' if open == 0 => pending = None,
            _ => {}
        }
        let name = owner(&stack, &pending).unwrap_or_else(|| "<module>".to_owned());
        if let Some((_, code)) = found.iter_mut().rev().find(|(n, _)| *n == name) {
            code.push(chars[i]);
        }
        i += 1;
    }
    found
}

/// The entries of the guarded list: the file, the function, and the calls it may make.
fn guarded_entries(guarded: &str) -> Vec<(&str, &str, Vec<&str>)> {
    entries(guarded)
        .into_iter()
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            Some((words.next()?, words.next()?, words.collect()))
        })
        .collect()
}

/// The calls of a source under `src/shell/`, named `file` there, that read, list, or resolve
/// in a function the guarded list does not allow them, each with its function.
fn unguarded(file: &str, source: &str, guarded: &str) -> Vec<String> {
    let allowed = guarded_entries(guarded);
    functions(source)
        .into_iter()
        .flat_map(|(name, code)| {
            let may: Vec<&str> = allowed
                .iter()
                .filter(|(f, n, _)| *f == file && *n == name)
                .flat_map(|(_, _, calls)| calls.iter().copied())
                .collect();
            violations(&code, READS)
                .into_iter()
                .filter(move |entry| !may.contains(&entry.as_str()))
                .map(move |entry| format!("{file}: {name} calls '{entry}'"))
        })
        .collect()
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

#[test]
fn the_shell_reads_only_through_the_guard() {
    let shell = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/shell");
    let mut files = Vec::new();
    sources(&shell, &shell.join(".none"), &mut files);
    assert!(!files.is_empty(), "no source under src/shell/");
    let mut found = Vec::new();
    let mut defined = Vec::new();
    for path in &files {
        let file = path
            .strip_prefix(&shell)
            .expect("a source lies under src/shell/")
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        let source = fs::read_to_string(path).expect("a source is readable");
        found.extend(unguarded(&file, &source, GUARDED));
        defined.extend(
            functions(&source)
                .into_iter()
                .map(|(name, _)| format!("{file} {name}")),
        );
    }
    assert!(
        found.is_empty(),
        "the shell reads past Tree::resolve:\n{}",
        found.join("\n")
    );
    for (file, name, calls) in guarded_entries(GUARDED) {
        let entry = format!("{file} {name}");
        let times = defined.iter().filter(|d| **d == entry).count();
        assert_eq!(
            times, 1,
            "functional_core.guarded names '{entry}', which src/shell/ defines {times} times, not once"
        );
        for call in calls {
            assert!(
                entries(READS).contains(&call),
                "functional_core.guarded lets '{entry}' call '{call}', which functional_core.reads does not list"
            );
        }
    }
}

#[test]
fn a_read_outside_the_guarded_functions_is_found_by_its_function() {
    let guarded = "tree.rs Tree::read_bytes fs::read\n";
    for (source, function) in [
        ("fn reader() { std::fs::read(\"x\"); }", "reader"),
        (
            "impl Tree { fn other(&self) { fs::read_dir(p); } }",
            "Tree::other",
        ),
        (
            "impl Tree { fn other(&self) { p.read_dir(); } }",
            "Tree::other",
        ),
        ("fn open() -> fs::File { todo!() }", "open"),
        ("fn f() { fn inner() { fs::canonicalize(p); } }", "inner"),
        ("use std::fs::read_to_string;", "<module>"),
        ("use std::fs::{self, read};", "<module>"),
        (
            "impl<T> Show for Tree<T> { fn read_bytes(&self) { fs::read(p); } }",
            "Tree::read_bytes",
        ),
        ("fn helper(x: [u8; 4]) { fs::read(p); }", "helper"),
        (
            "impl<T: Into<Vec<u8>>> Wrap<T> { fn get(&self) { fs::read(p); } }",
            "Wrap::get",
        ),
        (
            "impl<'a> Show for &'a Tree { fn other(&self) { fs::read(p); } }",
            "Tree::other",
        ),
    ] {
        let found = unguarded("other.rs", source, guarded);
        assert!(
            found
                .iter()
                .any(|f| f.starts_with(&format!("other.rs: {function} calls"))),
            "{function} not found in: {source}, found {found:?}"
        );
    }
    assert_eq!(
        unguarded(
            "tree.rs",
            "impl Tree { fn read_bytes(&self) { fs::read(p); } }",
            guarded
        ),
        Vec::<String>::new()
    );
    assert_eq!(
        unguarded(
            "tree.rs",
            "impl Tree { fn read_bytes(&self) {} fn other(&self) { fs::read(p); } }",
            guarded
        ),
        ["tree.rs: Tree::other calls 'fs::read'"]
    );
    // A guarded function may make only the calls its entry names.
    assert_eq!(
        unguarded(
            "tree.rs",
            "impl Tree { fn read_bytes(&self) { fs::read(p); fs::read_dir(d); } }",
            guarded
        ),
        ["tree.rs: Tree::read_bytes calls 'read_dir'"]
    );
    // A function nested in a guarded one is not covered by its entry.
    assert_eq!(
        unguarded(
            "tree.rs",
            "impl Tree { fn read_bytes(&self) { fn h(a: [u8; 1]) { fs::read(p); } } }",
            guarded
        ),
        ["tree.rs: h calls 'fs::read'"]
    );
}

#[test]
fn a_test_of_existence_is_no_read() {
    for source in [
        "fn f() { p.is_file(); p.is_dir(); fs::metadata(p); fs::symlink_metadata(p); }",
        "fn f() { tree.read(p); tree.read_text_files(); Input::SetupFile; }",
        "fn f() { let s = \"fs::read(x)\"; } // fs::read_dir",
    ] {
        assert_eq!(
            unguarded("tree.rs", source, ""),
            Vec::<String>::new(),
            "in: {source}"
        );
    }
}

/// The functions of the source named as a reader of `functional_core.bash`, by their name: a
/// method by its last part, so that `Type::trim` counts as `trim`.
fn bash_readers(source: &str, readers: &str) -> Vec<String> {
    let names = entries(readers);
    functions(source)
        .into_iter()
        .map(|(name, _)| name)
        .filter(|name| names.contains(&name.rsplit("::").next().unwrap_or(name)))
        .collect()
}

#[test]
fn the_readings_of_the_bash_s_tools_are_defined_once() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    sources(&src, &src.join("bash.rs"), &mut files);
    let found: Vec<String> = files
        .iter()
        .flat_map(|path| {
            let source = fs::read_to_string(path).expect("a source is readable");
            bash_readers(&source, BASH_READERS)
                .into_iter()
                .map(move |name| format!("{}: defines {name}", path.display()))
        })
        .collect();
    assert!(
        found.is_empty(),
        "a reading of the bash's tools defined outside src/bash.rs:\n{}",
        found.join("\n")
    );
    let bash = fs::read_to_string(src.join("bash.rs")).expect("src/bash.rs is readable");
    for name in entries(BASH_READERS) {
        assert_eq!(
            bash_readers(&bash, name),
            [name],
            "src/bash.rs defines no '{name}' once, which functional_core.bash names"
        );
    }
}

#[test]
fn a_reader_of_the_bash_s_tools_defined_elsewhere_is_found_by_its_name() {
    let readers = "awk_lines\nis_space\ntrim\n";
    assert_eq!(
        bash_readers("fn lines(t: &[u8]) {} fn awk_lines(t: &[u8]) {}", readers),
        ["awk_lines"]
    );
    assert_eq!(
        bash_readers("impl Text { fn trim(&self) {} }", readers),
        ["Text::trim"]
    );
    assert_eq!(
        bash_readers("fn f() { fn is_space(b: u8) -> bool { b == 32 } }", readers),
        ["is_space"]
    );
    assert!(bash_readers("fn is_spaced() {} // fn trim()", readers).is_empty());
}

/// The names of the rule modules under `src/rules/`.
fn rule_modules() -> Vec<String> {
    let rules = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/rules");
    let mut found = Vec::new();
    sources(&rules, &rules.join("mod.rs"), &mut found);
    found
        .iter()
        .filter_map(|path| path.file_stem()?.to_str().map(str::to_owned))
        .collect()
}

/// The calls of the source into what a rule decides: a path that names a rule module followed by
/// one of the names `outcomes` lists, or a glob that opens a rule module.
fn decides_for_a_rule(source: &str, modules: &[String], outcomes: &str) -> Vec<String> {
    let names = entries(outcomes);
    let code = code(source);
    let paths = paths(&code);
    let openings = openings(&paths);
    let mut found: Vec<String> = paths
        .iter()
        .filter(|path| {
            let segments: Vec<&str> = path.split("::").collect();
            segments
                .windows(2)
                .any(|w| modules.iter().any(|m| m == w[0]) && names.contains(&w[1]))
        })
        .cloned()
        .collect();
    found.extend(
        openings
            .into_iter()
            .filter(|opening| {
                let last = opening.trim_end_matches("::").rsplit("::").next();
                last.is_some_and(|last| modules.iter().any(|m| m == last))
            })
            .map(str::to_owned),
    );
    found
}

#[test]
fn the_shell_calls_no_rule_by_name() {
    let shell = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/shell");
    let modules = rule_modules();
    assert!(!modules.is_empty(), "no rule module under src/rules/");
    let mut files = Vec::new();
    sources(&shell, &shell.join(".none"), &mut files);
    let found: Vec<String> = files
        .iter()
        .flat_map(|path| {
            let source = fs::read_to_string(path).expect("a source is readable");
            decides_for_a_rule(&source, &modules, OUTCOMES)
                .into_iter()
                .map(move |call| format!("{}: {call}", path.display()))
        })
        .collect();
    assert!(
        found.is_empty(),
        "the shell calls what a rule decides, which its outcome carries:\n{}",
        found.join("\n")
    );
}

#[test]
fn a_call_into_what_a_rule_decides_is_found() {
    let modules = [
        "docs_prose_width".to_owned(),
        "traceability_coverage".to_owned(),
    ];
    let outcomes = "RULE\nundecided_lines\n";
    for source in [
        "fn f() { let notes = (crate::rules::traceability_coverage::RULE.check)(i).notes; }",
        "use crate::rules::docs_prose_width::undecided_lines;",
        "fn f() { (rules::traceability_coverage::RULE.check)(i); }",
        "use crate::rules::docs_prose_width::*;",
    ] {
        assert!(
            !decides_for_a_rule(source, &modules, outcomes).is_empty(),
            "not found in: {source}"
        );
    }
    for source in [
        "fn f() { rules::docs_prose_width::wide_lines(t); (rule.check)(i); }",
        "fn f() -> rules::docs_prose_width::Width { todo!() }",
        "// rules::traceability_coverage::pending",
    ] {
        assert_eq!(
            decides_for_a_rule(source, &modules, outcomes),
            Vec::<String>::new(),
            "in: {source}"
        );
    }
}
