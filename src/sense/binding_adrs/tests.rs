use super::*;

const V0_5: Version = Version(0, 5, 0);
const V0_8: Version = Version(0, 8, 0);
const NEWEST: Version = Version(0, 12, 0);

/// An ADR file with the status and the *Applies to*, and the body after the header.
fn adr(path: &str, status: &str, applies: &str, body: &str) -> Adr {
    Adr {
        path: path.as_bytes().to_vec(),
        text: format!("# X: T\n\n- **Status:** {status}\n- **Applies to:** {applies}\n\n{body}")
            .into_bytes(),
    }
}

fn record(adrs: Vec<Adr>) -> Record {
    Record {
        adrs,
        index: Vec::new(),
    }
}

fn changed(paths: &[&str]) -> Listing {
    Listing::Changed {
        range: "abc..def".to_owned(),
        paths: paths.iter().map(|p| p.as_bytes().to_vec()).collect(),
    }
}

/// The report where no path is named with checks of its own.
fn plain(release: Version, sections: bool, listing: &Listing, record: &Record) -> String {
    assert!(
        checked_paths(release, listing, record).is_empty(),
        "no file is read for checks"
    );
    let report = report(release, sections, listing, record, &Diffs::new()).expect("a report");
    String::from_utf8(report).expect("UTF-8")
}

#[test]
fn the_arguments_read_as_the_bash_of_each_release_reads_them() {
    let parse = |args: &[&'static str], release| parse(args, release);
    // Before v0.7.0 an option is a base, which names no commit.
    assert_eq!(parse(&["--sections"], Version(0, 6, 2)), None);
    assert_eq!(
        parse(&["main", "work", "extra"], V0_5),
        Some(Args {
            sections: false,
            named: Vec::new(),
            base: Some("main"),
            head: Some("work"),
        })
    );
    // `--path` without a path exits 2 on every release that reads it.
    for release in [OPTIONS, NEWEST] {
        assert_eq!(parse(&["--path"], release), None);
        assert_eq!(parse(&["--path", ""], release), None);
        assert_eq!(parse(&["--sections", "--path"], release), None);
    }
    let named = parse(
        &["--path", "./src/x.rs", "--sections", "--path", "--x"],
        NEWEST,
    )
    .unwrap();
    assert_eq!(named.named, ["src/x.rs", "--x"]);
    assert!(named.sections);
    // Beside `--path`, a base is ignored before v0.10.0 and refused from it; a third positional
    // argument likewise.
    let v0_9_1 = Version(0, 9, 1);
    let named = parse(&["--path", "a", "HEAD"], v0_9_1).unwrap();
    assert_eq!((named.named, named.base), (vec!["a"], None));
    assert_eq!(parse(&["--path", "a", "HEAD"], RULE_KEYS), None);
    assert_eq!(parse(&["a", "b", "c"], RULE_KEYS), None);
    assert!(parse(&["a", "b", "c"], v0_9_1).is_some());
    // An argument that reads as an option is refused on every release, wherever it stands, and
    // so is `--root`.
    for release in [V0_8, v0_9_1, RULE_KEYS] {
        assert_eq!(parse(&["-x"], release), None);
        assert_eq!(parse(&["a", "b", "--x"], release), None);
        assert_eq!(parse(&["--path", "a", "HEAD", "-x"], release), None);
    }
    assert_eq!(parse(&["--path", "a", "--root", "b"], V0_8), None);
    // `--root` as the value of `--path` names no path.
    for release in [OPTIONS, NEWEST] {
        assert_eq!(parse(&["--path", "--root"], release), None);
        assert_eq!(
            parse(&["--sections", "--path", "a", "--path", "--root"], release),
            None
        );
    }
}

#[test]
fn a_rust_file_names_the_directory_of_its_module_but_a_root_or_another_kind() {
    assert_eq!(module_dir(b"crates/e/src/graph.rs"), b"crates/e/src/graph/");
    assert_eq!(module_dir(b"a.b/x.rs"), b"a.b/x/");
    for none in [
        &b"src/lib.rs"[..],
        b"main.rs",
        b"x/mod.rs",
        b"build.rs",
        b".rs",
        b"tools/a.sh",
    ] {
        assert_eq!(module_dir(none), b"", "{}", String::from_utf8_lossy(none));
    }
}

#[test]
fn the_paths_of_an_applies_to_read_as_each_release_reads_them() {
    let read = |applies: &str, release| {
        bound_paths(applies.as_bytes(), release)
            .into_iter()
            .map(|b| {
                let checks = b.checks.map(|c| {
                    c.iter()
                        .map(|w| String::from_utf8_lossy(w).into_owned())
                        .collect::<Vec<_>>()
                });
                (String::from_utf8_lossy(&b.path).into_owned(), checks)
            })
            .collect::<Vec<_>>()
    };
    let applies = "checks 1, 3, and 12 of `s.sh`, the rules `t/a` and `t/b` of `k.sh`, \
                   the `pull request` template, and `docs/`";
    assert_eq!(
        read(applies, V0_8),
        [
            ("s.sh".to_owned(), None),
            ("t/a".to_owned(), None),
            ("t/b".to_owned(), None),
            ("k.sh".to_owned(), None),
            ("docs".to_owned(), None),
        ]
    );
    assert_eq!(
        read(applies, NAMED),
        [
            (
                "s.sh".to_owned(),
                Some(vec!["1".into(), "3".into(), "12".into()])
            ),
            ("t/a".to_owned(), None),
            ("t/b".to_owned(), None),
            ("k.sh".to_owned(), None),
            ("docs".to_owned(), None),
        ]
    );
    assert_eq!(
        read(applies, RULE_KEYS),
        [
            ("s.sh".to_owned(), None),
            ("k.sh".to_owned(), Some(vec!["t/a".into(), "t/b".into()])),
            ("docs".to_owned(), None),
        ]
    );
    // A path `/` printed empty: `read` takes the keys after the tab for the path.
    assert_eq!(
        read("the rule `k` of `/`, and `/`", RULE_KEYS),
        [("k".to_owned(), None), (String::new(), None)]
    );
    // A key of spaces alone is not empty to `-z`, and splits into no word.
    assert_eq!(
        read("the rule ` ` of `x`", RULE_KEYS),
        [("x".to_owned(), Some(Vec::new()))]
    );
}

#[test]
fn held_names_each_line_by_the_check_whose_function_holds_it() {
    let file = b"#   t/b (was: t/old, t/older)\nset -u\ncheck_a() {  # t/a\n  :\n}\nh() {\n}\n\
                 check_b() {  # t/b\n  :\n}\ncheck_one() {  # check 1\n  :\n}\n";
    let held_keys = held(file, RULE_KEYS);
    let text = |lines: &[Vec<u8>]| {
        lines
            .iter()
            .map(|l| String::from_utf8_lossy(l).into_owned())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        text(&held_keys),
        [
            "",
            "",
            "t/a",
            "t/a",
            "t/a",
            "",
            "",
            "t/b t/old t/older",
            "t/b t/old t/older",
            "t/b t/old t/older",
            "",
            "",
            "",
            ""
        ]
    );
    assert_eq!(
        text(&held(file, NAMED)),
        [
            "", "", "", "", "", "0", "0", "0", "0", "0", "1", "1", "1", "0"
        ]
    );
}

#[test]
fn a_hunk_head_awk_reads_inexactly_is_not_decided() {
    let diffed = Diffed {
        file: b"check_a() {  # t/a\n  :\n}\n".to_vec(),
        diff: b"@@ -1 +99999999999999999999 @@\n".to_vec(),
    };
    let refusal = held_in_hunks(&diffed, RULE_KEYS).unwrap_err();
    assert!(
        refusal.to_string().ends_with("[sense/binding-adrs:hunk]"),
        "{refusal}"
    );
    // Where awk's `start + count + 1` runs past what it holds exactly as well.
    let with = |diff: &[u8]| Diffed {
        file: diffed.file.clone(),
        diff: diff.to_vec(),
    };
    assert!(held_in_hunks(&with(b"@@ -1 +9007199254740993 @@\n"), RULE_KEYS).is_err());
    assert!(held_in_hunks(&with(b"@@ -1 +9007199254740991,1 @@\n"), RULE_KEYS).is_err());
    let held = held_in_hunks(&with(b"@@ -1 +9007199254740990,1 @@\n"), RULE_KEYS).unwrap();
    assert_eq!(held, b" all ");
}

#[test]
fn a_hunk_binds_the_checks_of_its_lines_and_the_lines_on_either_side() {
    let file = b"check_a() {  # t/a\n  :\n}\n\ncheck_b() {  # t/b\n  :\n}\n".to_vec();
    let touched = |diff: &str| {
        let diffed = Diffed {
            file: file.clone(),
            diff: diff.as_bytes().to_vec(),
        };
        String::from_utf8(held_in_hunks(&diffed, RULE_KEYS).unwrap()).unwrap()
    };
    // Line 6 changed: lines 5 to 7, all in check b.
    assert_eq!(touched("@@ -6 +6 @@\n-  :\n+  x\n"), " t/b ");
    // Line 4 changed: lines 3 to 5, the brace of a, the blank line outside, and b's opening.
    assert_eq!(touched("@@ -4 +4 @@ x\n"), " all t/a t/b ");
    // A deletion after line 2 reads lines 2 and 3.
    assert_eq!(touched("@@ -3 +2,0 @@\n"), " t/a ");
    // A hunk at the first line reads line 0, which no file has.
    assert_eq!(touched("@@ -1 +1 @@\n"), " all t/a ");
    // No hunk, as for a mode change or a binary file.
    assert_eq!(touched("Binary files a and b differ\n"), " ");
    assert!(in_checks(b" ", &[]));
    assert!(in_checks(b" t/b t/old ", &[b"t/old".to_vec()]));
    assert!(!in_checks(b" t/b t/old ", &[b"t/o".to_vec()]));
    assert!(in_checks(b" all t/a ", &[b"t/c".to_vec()]));
}

#[test]
fn prose_is_an_item_that_is_not_one_path_in_backticks() {
    assert!(!is_prose(b"`a/`, `b`, and `c` and `d`", NEWEST));
    assert!(is_prose(b"the code under `src/`", NEWEST));
    assert!(is_prose(b"", NEWEST));
    assert!(is_prose(b"`pull request`", NEWEST));
    // A split that ends in a newline: v0.7.0 reads the empty line after it, v0.8.0 drops it.
    assert!(is_prose(b"`a`, ", OPTIONS));
    assert!(!is_prose(b"`a`, ", V0_8));
}

#[test]
fn ads_head_listing_matches_the_pattern_of_the_release_and_no_quoted_name() {
    let ls_tree =
        b"docs/adr/0001-a.md\ndocs/adr/draft-b.md\ndocs/adr/Ab-c-d.md\ndocs/adr/README.md\n\
                    docs/adr/sub/0002-x.md\n\"docs/adr/0003-\\303\\274.md\"\n";
    let listed = |release| {
        listed_adrs(ls_tree, release)
            .into_iter()
            .map(|p| String::from_utf8(p).unwrap())
            .collect::<Vec<_>>()
    };
    assert_eq!(listed(V0_8), ["docs/adr/0001-a.md", "docs/adr/draft-b.md"]);
    assert_eq!(listed(NAMED), ["docs/adr/draft-b.md", "docs/adr/Ab-c-d.md"]);
    assert!(is_adr_name("0001-a.md", V0_8) && !is_adr_name("0001-a.md", NAMED));
    assert!(is_adr_name("a1-x.md", NAMED));
}

#[test]
fn nothing_to_compare_and_no_change_end_as_each_release_ends() {
    let record = record(Vec::new());
    assert_eq!(
        plain(V0_5, false, &Listing::Nothing, &record),
        "Binding ADRs sensor: nothing to compare — no base given and no main to compare with.\n"
    );
    assert_eq!(
        plain(V0_5, false, &changed(&[]), &record),
        "Binding ADRs sensor (abc..def): nothing to compare — no path changed.\n"
    );
    assert_eq!(
        plain(NEWEST, true, &changed(&[]), &record),
        "Binding ADRs sensor (abc..def): nothing to compare — no path changed. Name the paths the \
         work will touch with --path.\n\nNo ADR in force found in docs/adr/; read the index for \
         the ADRs whose Applies to is prose.\n"
    );
    assert_eq!(
        plain(V0_8, true, &Listing::Nothing, &record).lines().last(),
        Some(
            "No row found under '### Binding' in docs/adr/README.md; read the index for the ADRs \
             whose Applies to is prose."
        )
    );
}

#[test]
fn an_adr_in_force_binds_the_paths_its_applies_to_names() {
    let record = record(vec![
        adr(
            "docs/adr/0001-plan.md",
            "🟢 accepted",
            "`docs/plan/` and `README.md`",
            "",
        ),
        adr("docs/adr/0002-gone.md", "⚪ superseded", "`src/`", ""),
        adr(
            "docs/adr/draft-code.md",
            "🟡 proposed",
            "the code under `src/`",
            "",
        ),
        adr("docs/adr/0003-graph.md", "🟢 accepted", "`e/graph.rs`", ""),
    ]);
    let paths = [
        "README.md",
        "README.md.bak",
        "docs/plan/x",
        "e/graph/hit.rs",
        "src/1",
        "src/2",
    ];
    let mut listing = changed(&paths);
    if let Listing::Changed { paths, .. } = &mut listing {
        paths.extend([b"src/3".to_vec(), b"src/4".to_vec()]);
    }
    assert_eq!(
        plain(V0_5, false, &listing, &record),
        "Binding ADRs sensor (abc..def): 8 path(s) changed; 2 ADR(s) in force name one of them. \
         Read these, and the index for the ADRs whose Applies to is prose:\n  \
         - ADR-0001 (docs/adr/0001-plan.md) — README.md docs/plan/x\n  \
         - ADR-draft-code (docs/adr/draft-code.md) — src/1 src/2 src/3, and 1 more\n"
    );
    // From v0.6.2 a Rust file names its module's directory; from v0.9.0 an ADR is named by its
    // file.
    let report = plain(NAMED, false, &listing, &record);
    assert!(report.contains("  - 0003-graph (docs/adr/0003-graph.md) — e/graph/hit.rs\n"));
    // A named directory binds what an ADR names under it.
    let named = Listing::Named(vec![b"docs/".to_vec(), b"e".to_vec()]);
    assert_eq!(
        plain(OPTIONS, false, &named, &record),
        "Binding ADRs sensor (named paths): 2 path(s) named; 2 ADR(s) in force name one of them. \
         Read these, and the index for the ADRs whose Applies to is prose:\n  \
         - ADR-0001 (docs/adr/0001-plan.md) — docs/\n  \
         - ADR-0003 (docs/adr/0003-graph.md) — e\n"
    );
}

#[test]
fn sections_print_the_prose_and_of_each_adr_listed_its_header_decision_and_enforcement() {
    let body = "## Context\n\nWhy.\n\n## Decision\n\nWe will.\n\n```md\n## Notes\n```\n\nAfter.\n\n\
                ## Enforcement\n\nA sensor.\n\n\n";
    let record = record(vec![
        adr("docs/adr/FIX-a-b.md", "🟢 accepted", "`README.md`", body),
        adr("docs/adr/FIX-c-d.md", "🟡 proposed", "every ADR", ""),
        adr("docs/adr/FIX-e-f.md", "🟢 accepted", "`tools/`", ""),
    ]);
    let listing = Listing::Named(vec![b"README.md".to_vec()]);
    let header = "# X: T\n\n- **Status:** 🟢 accepted\n- **Applies to:** `README.md`\n\n";
    assert_eq!(
        plain(NEWEST, true, &listing, &record),
        format!(
            "Binding ADRs sensor (named paths): 1 path(s) named; 1 ADR(s) in force name one of \
             them. Read these, and the ADRs whose Applies to is prose, below:\n  \
             - FIX-a-b (docs/adr/FIX-a-b.md) — README.md\n\n\
             The ADRs in force no path above decides, by their Applies to; match the prose against \
             the work:\n  - FIX-c-d: every ADR\n\n\
             === FIX-a-b (docs/adr/FIX-a-b.md): the header, the Decision, and the Enforcement ===\n\
             {header}## Decision\n\nWe will.\n\n```md\n## Notes\n```\n\nAfter.\n\n\n\
             ## Enforcement\n\nA sensor.\n"
        )
    );
    // Before v0.8.1 the `## Notes` inside the fence ends the Decision.
    let record = Record {
        adrs: vec![adr(
            "docs/adr/0001-a.md",
            "🟢 accepted",
            "`README.md`",
            body,
        )],
        index: b"### Binding\n| [0001](x) | A | `README.md` |\n| [0002](y) | B | `x/`, `y` |\n"
            .to_vec(),
    };
    let report = plain(V0_8, true, &listing, &record);
    assert!(report.contains(
        "Every other ADR in force names paths alone, and none of them names one of the work.\n"
    ));
    assert!(report.ends_with(
        "=== ADR-0001 (docs/adr/0001-a.md): the header, the Decision, and the Enforcement ===\n\
         # X: T\n\n- **Status:** 🟢 accepted\n- **Applies to:** `README.md`\n\n\
         ## Decision\n\nWe will.\n\n```md\n\n## Enforcement\n\nA sensor.\n"
    ));
}

/// The first bound path that matches decides: one without checks binds and reads no diff, even
/// where a later one of the same ADR has checks; a later ADR that binds the path by its checks
/// reads it.
#[test]
fn the_first_bound_path_that_matches_decides_whether_the_diff_is_read() {
    let listing = changed(&["c.sh"]);
    let plain_first = record(vec![adr(
        "docs/adr/FIX-a-b.md",
        "🟢 accepted",
        "`c.sh` and the rule `t/a` of `c.sh`",
        "",
    )]);
    assert!(checked_paths(NEWEST, &listing, &plain_first).is_empty());
    let checks_first = record(vec![adr(
        "docs/adr/FIX-a-b.md",
        "🟢 accepted",
        "the rule `t/a` of `c.sh` and `c.sh`",
        "",
    )]);
    assert_eq!(
        checked_paths(NEWEST, &listing, &checks_first),
        [b"c.sh".to_vec()]
    );
    let later_by_checks = record(vec![
        adr("docs/adr/FIX-a-b.md", "🟢 accepted", "`c.sh`", ""),
        adr(
            "docs/adr/FIX-c-d.md",
            "🟢 accepted",
            "the rule `t/a` of `c.sh`",
            "",
        ),
    ]);
    assert_eq!(
        checked_paths(NEWEST, &listing, &later_by_checks),
        [b"c.sh".to_vec()]
    );
}

#[test]
fn a_file_named_with_checks_is_read_only_where_the_bash_runs_checked() {
    let record = record(vec![
        adr(
            "docs/adr/FIX-a-b.md",
            "🟢 accepted",
            "the rule `t/a` of `c.sh`",
            "",
        ),
        adr(
            "docs/adr/FIX-c-d.md",
            "🟢 accepted",
            "the rule `t/b` of `c.sh`",
            "",
        ),
        adr("docs/adr/FIX-e-f.md", "🟢 accepted", "`c.sh`", ""),
    ]);
    let listing = changed(&["c.sh", "d.sh"]);
    let asked = checked_paths(NEWEST, &listing, &record);
    let diffs: Diffs = asked
        .iter()
        .map(|path| {
            let diffed = Diffed {
                file: b"check_a() {  # t/a\n  :\n}\ncheck_b() {  # t/b\n  :\n}\n".to_vec(),
                diff: b"@@ -5 +5 @@\n".to_vec(),
            };
            (path.clone(), diffed)
        })
        .collect();
    let report = report(NEWEST, false, &listing, &record, &diffs).unwrap();
    let report = String::from_utf8(report).unwrap();
    assert!(!report.contains("FIX-a-b"), "{report}");
    assert!(report.contains("  - FIX-c-d (docs/adr/FIX-c-d.md) — c.sh\n"));
    assert!(report.contains("  - FIX-e-f (docs/adr/FIX-e-f.md) — c.sh\n"));
    assert_eq!(asked, [b"c.sh".to_vec()]);
    // Named, the file is not read.
    let named = Listing::Named(vec![b"c.sh".to_vec()]);
    assert!(plain(NEWEST, false, &named, &record).contains("3 ADR(s) in force"));
}
