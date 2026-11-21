use std::collections::{BTreeMap, BTreeSet};

use super::*;

const V0_5: Version = Version(0, 5, 0);
const V0_7: Version = Version(0, 7, 0);
const V0_9: Version = Version(0, 9, 0);
const NEWEST: Version = Version(0, 12, 0);

const HEAD: &str = "cccccccccccccccccccccccccccccccccccccccc";
const MID: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const BASE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

/// A repository that answers what a test gives it, and fails the test on any other question: a
/// hand-off it was not given is not read.
#[derive(Default)]
struct Fake {
    commits: BTreeMap<String, String>,
    refs: BTreeMap<String, String>,
    ancestors: BTreeSet<(String, String)>,
    lists: BTreeMap<String, Vec<String>>,
    counts: BTreeMap<String, u64>,
    subjects: BTreeMap<String, String>,
    messages: BTreeMap<String, String>,
    patches: BTreeMap<String, String>,
    outside: BTreeMap<String, Vec<String>>,
    hand_off: Option<String>,
    asked: Vec<String>,
}

fn lines(items: &[String]) -> Vec<Vec<u8>> {
    items.iter().map(|s| s.as_bytes().to_vec()).collect()
}

impl Repo for Fake {
    fn commit(&mut self, rev: &str) -> Result<Option<String>, Refusal> {
        self.asked.push(format!("commit {rev}"));
        let rev = rev
            .strip_suffix("^{commit}")
            .unwrap_or(rev)
            .to_ascii_lowercase();
        if let Some(sha) = self.refs.get(&rev) {
            return Ok(Some(sha.clone()));
        }
        Ok(self
            .commits
            .values()
            .find(|sha| sha.starts_with(&rev))
            .cloned())
    }
    fn full(&mut self, rev: &str) -> Result<String, Refusal> {
        Ok(if rev == "HEAD" { HEAD } else { rev }.to_owned())
    }
    fn short(&mut self, rev: &str) -> Result<String, Refusal> {
        let full = self.full(rev)?;
        Ok(full[..7].to_owned())
    }
    fn is_ancestor(&mut self, ancestor: &str, rev: &str) -> Result<bool, Refusal> {
        Ok(self
            .ancestors
            .contains(&(ancestor.to_owned(), rev.to_owned())))
    }
    fn rev_list(&mut self, options: &[&str], range: &str) -> Result<Vec<Vec<u8>>, Refusal> {
        let key = format!("{} {range}", options.join(" "));
        Ok(lines(self.lists.get(&key).map_or(&[][..], Vec::as_slice)))
    }
    fn count(&mut self, options: &[&str], range: &str) -> Result<u64, Refusal> {
        let key = format!("{} {range}", options.join(" "));
        Ok(*self
            .counts
            .get(&key)
            .unwrap_or_else(|| panic!("count {key}")))
    }
    fn subjects(&mut self, range: &str) -> Result<Vec<Vec<u8>>, Refusal> {
        Ok(vec![format!("log {range}").into_bytes()])
    }
    fn subject(&mut self, commit: &str) -> Result<Vec<u8>, Refusal> {
        let subject = self.subjects.get(commit).cloned();
        Ok(subject
            .unwrap_or_else(|| format!("{} {commit}", &commit[..7.min(commit.len())]))
            .into_bytes())
    }
    fn message(&mut self, commit: &str) -> Result<Vec<u8>, Refusal> {
        self.asked.push(format!("message {commit}"));
        Ok(self
            .messages
            .get(commit)
            .cloned()
            .unwrap_or_default()
            .into_bytes())
    }
    fn stat(&mut self, from: &str, to: &str) -> Result<Vec<Vec<u8>>, Refusal> {
        Ok(vec![
            format!(" stat {} {to} | 1 +", &from[..7]).into_bytes(),
            b" 1 file changed, 1 insertion(+)".to_vec(),
        ])
    }
    fn patch(&mut self, commit: &str, pathspecs: &[&str]) -> Result<Vec<u8>, Refusal> {
        assert!(!pathspecs.is_empty());
        Ok(self
            .patches
            .get(commit)
            .cloned()
            .unwrap_or_default()
            .into_bytes())
    }
    fn outside_record(&mut self, commit: &str) -> Result<Vec<Vec<u8>>, Refusal> {
        Ok(lines(
            self.outside.get(commit).map_or(&[][..], Vec::as_slice),
        ))
    }
    fn hand_off(&mut self) -> Result<Vec<u8>, Refusal> {
        self.asked.push("hand-off".to_owned());
        Ok(self
            .hand_off
            .clone()
            .expect("the hand-off is read only where the bash reads it")
            .into_bytes())
    }
}

/// A branch of two commits on a base, MID and HEAD, the hand-off as given.
fn branch(hand_off: Option<&str>) -> Fake {
    let mut fake = Fake {
        hand_off: hand_off.map(str::to_owned),
        ..Fake::default()
    };
    let range = format!("{BASE}..HEAD");
    fake.lists.insert(
        format!("--no-merges --reverse {range}"),
        vec![MID.to_owned(), HEAD.to_owned()],
    );
    fake.lists.insert(
        format!("--first-parent --no-merges --reverse {range}"),
        vec![MID.to_owned(), HEAD.to_owned()],
    );
    fake.counts.insert(format!("--no-merges {range}"), 2);
    fake.counts.insert(format!(" {MID}..HEAD"), 1);
    for sha in [MID, HEAD] {
        fake.commits.insert(sha[..7].to_owned(), sha.to_owned());
        fake.commits.insert(sha.to_owned(), sha.to_owned());
        fake.ancestors.insert((sha.to_owned(), "HEAD".to_owned()));
    }
    fake.commits.insert(BASE[..9].to_owned(), BASE.to_owned());
    fake.ancestors.insert((BASE.to_owned(), "HEAD".to_owned()));
    fake.ancestors.insert((BASE.to_owned(), BASE.to_owned()));
    fake
}

fn run(fake: &mut Fake, hand_off: bool, release: Version) -> String {
    let report = report(fake, BASE, "HEAD", hand_off, release).expect("a report");
    String::from_utf8(report).expect("UTF-8")
}

#[test]
fn the_arguments_read_as_the_bash_of_each_release_reads_them() {
    let args = |hand_off, base, head| {
        Some(Args {
            hand_off,
            base,
            head,
        })
    };
    assert_eq!(parse(&[], NEWEST), args(None, None, None));
    assert_eq!(
        parse(&["--hand-off", "h.md", "main", "work"], NEWEST),
        args(Some("h.md"), Some("main"), Some("work"))
    );
    // `--hand-off` without a value names no file, which the shell refuses as the bash does.
    assert_eq!(parse(&["--hand-off"], NEWEST), args(Some(""), None, None));
    // A value that reads as an option is a path from where the sensor runs.
    assert_eq!(
        parse(&["--hand-off", "-h"], NEWEST),
        args(Some("-h"), None, None)
    );
    // `--hand-off` only before the base and the head.
    assert_eq!(parse(&["main", "--hand-off", "h.md"], NEWEST), None);
    assert_eq!(parse(&["--hand-off", "a", "--hand-off", "b"], V0_5), None);
    // `--root` anywhere, as API-sensor-cores-ask-the-shell refuses it.
    assert_eq!(parse(&["--hand-off", "--root"], V0_5), None);
    assert_eq!(parse(&["main", "work", "--root"], V0_5), None);
    // A third positional argument: ignored before v0.10.0, refused from it.
    assert_eq!(
        parse(&["main", "work", "x"], V0_9),
        args(None, Some("main"), Some("work"))
    );
    assert_eq!(parse(&["main", "work", "x"], NEWEST), None);
    // An argument that reads as an option is refused on every release, wherever it stands.
    for release in [V0_5, V0_9, NEWEST] {
        assert_eq!(parse(&["main", "work", "--x"], release), None);
        assert_eq!(parse(&["--hand-off", "h.md", "-x"], release), None);
    }
}

#[test]
fn the_reviewed_commit_is_cut_from_the_first_line_that_names_one() {
    let word = |message: &str, release| {
        reviewed(message.as_bytes(), release).map(|w| String::from_utf8(w).expect("UTF-8"))
    };
    assert_eq!(
        word("Reviewed at: abc1234\n", NEWEST).as_deref(),
        Some("abc1234")
    );
    assert_eq!(
        word("x\n  * Reviewed at:\t`abc1234` (head)\r\n", NEWEST).as_deref(),
        Some("abc1234")
    );
    assert_eq!(
        word("- Reviewed at:   abc1234\r", NEWEST).as_deref(),
        Some("abc1234")
    );
    // A line without a word after the colon is passed over; the first with one is read.
    assert_eq!(
        word(
            "Reviewed at:\nReviewed at: work\nReviewed at: abc1234",
            NEWEST
        )
        .as_deref(),
        Some("work")
    );
    // A comma ends the word from v0.9.0 on.
    assert_eq!(
        word("Reviewed at: abc1234, after fixes", V0_7).as_deref(),
        Some("abc1234,")
    );
    assert_eq!(
        word("Reviewed at: abc1234, after fixes", V0_9).as_deref(),
        Some("abc1234")
    );
    // Backticks alone are no word; a mark other than - or * is none the line allows.
    assert_eq!(word("Reviewed at: `` x", NEWEST), None);
    assert_eq!(word("+ Reviewed at: abc1234", NEWEST), None);
    assert_eq!(word("Reviewed at abc1234", NEWEST), None);
    assert_eq!(as_hash(b"abc123"), None);
    assert_eq!(as_hash(b"ABC1234"), Some("ABC1234"));
    assert_eq!(as_hash(&[b'a'; 41]), None);
}

#[test]
fn the_text_of_an_adr_is_any_line_but_its_status_date_and_deciders() {
    let patch = "diff --git a/docs/adr/FIX-a.md b/docs/adr/FIX-a.md\n\
                 --- a/docs/adr/FIX-a.md\n+++ b/docs/adr/FIX-a.md\n\
                 @@ -3 +3 @@\n-- **Status:** 🟡 proposed\n+- **Status:** 🟢 accepted\n\
                 diff --git a/docs/adr/FIX-b c.md b/docs/adr/FIX-b c.md\n\
                 @@ -9 +9 @@\n-old\n+new\n\
                 diff --git \"a/docs/adr/FIX-\\303\\274.md\" \"b/docs/adr/FIX-\\303\\274.md\"\n\
                 @@ -1 +1 @@\n--- a removed line read as a header\n+++ and an added one\n+text\n+more";
    let texts: Vec<String> = adr_texts(patch.as_bytes())
        .into_iter()
        .map(|t| String::from_utf8(t).expect("UTF-8"))
        .collect();
    // The flip is no text; awk's last field of a name with a space is its last word, and a quoted
    // name keeps its quotes.
    assert_eq!(texts, ["c.md", "\"b/docs/adr/FIX-\\303\\274.md\""]);
}

#[test]
fn a_branch_without_a_hand_off_is_bound_only_by_one() {
    let mut fake = branch(None);
    assert_eq!(
        run(&mut fake, false, NEWEST),
        "Review binding sensor: 2 commits since aaaaaaa, bound only by a hand-off: name the pull \
         request's description with --hand-off <file>.\n"
    );
    // Before v0.11.0 the commits landed outside a merge, and before v0.7.0 the headline counts
    // no finding.
    let mut fake = branch(None);
    assert_eq!(
        run(&mut fake, false, V0_9),
        "Review binding sensor: no merge since aaaaaaa; 1 finding below.\n  - 2 commits landed \
         outside a merge, named by no review:\n    bbbbbbb bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\n    \
         ccccccc cccccccccccccccccccccccccccccccccccccccc\n"
    );
    let mut fake = branch(None);
    assert!(
        run(&mut fake, false, V0_5).starts_with("Review binding sensor: no merge since aaaaaaa.\n")
    );
}

#[test]
fn a_hand_off_reviewed_at_the_head_covers_the_branch() {
    for (message, release) in [
        ("## Review\n\n- Reviewed at: `ccccccc`\n", NEWEST),
        ("Reviewed at: CCCCCCCCCCCC\n", NEWEST),
        ("Reviewed at: ccccccc\n", V0_5),
    ] {
        let mut fake = branch(Some(message));
        let text = run(&mut fake, true, release);
        assert!(
            text.starts_with(
                "Review binding sensor: the hand-off was reviewed at the head, ccccccc"
            ),
            "{message}: {text}"
        );
    }
    let mut fake = branch(Some("Reviewed at: ccccccc\n"));
    assert_eq!(
        run(&mut fake, true, NEWEST),
        "Review binding sensor: the hand-off was reviewed at the head, ccccccc, which covers the 2 \
         commits since aaaaaaa.\n"
    );
}

#[test]
fn a_hand_off_reviewed_earlier_lists_what_came_after() {
    let mut fake = branch(Some("Reviewed at: bbbbbbb\n"));
    assert_eq!(
        run(&mut fake, true, NEWEST),
        "Review binding sensor: 2 commits since aaaaaaa, bound only by the hand-off; 1 finding \
         below.\n  - the hand-off: 1 commit after the review at bbbbbbb, unreviewed:\n    \
         log bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb..HEAD\n    stat bbbbbbb HEAD | 1 +\n    \
         1 file changed, 1 insertion(+)\n"
    );
    // Before v0.11.0 the commits outside a merge it lists are not listed again.
    let mut fake = branch(Some("Reviewed at: bbbbbbb\n"));
    let text = run(&mut fake, true, V0_9);
    assert!(text.starts_with("Review binding sensor: no merge since aaaaaaa; 1 finding below.\n  - the hand-off: 1 commit after"), "{text}");
    assert!(!text.contains("outside a merge"), "{text}");
}

#[test]
fn a_hand_off_that_names_no_commit_of_the_branch_binds_nothing() {
    for (message, word) in [
        ("Reviewed at: work\n", "work"),
        ("Reviewed at: aaaaaaaaa\n", "aaaaaaaaa"),
        ("Reviewed at: deadbee\n", "deadbee"),
    ] {
        let mut fake = branch(Some(message));
        let text = run(&mut fake, true, NEWEST);
        assert!(
            text.ends_with(&format!(
                "  - the hand-off: reviewed at '{word}', which is no commit of the branch since the base\n"
            )),
            "{message}: {text}"
        );
    }
    let mut fake = branch(Some("No review.\n"));
    let text = run(&mut fake, true, NEWEST);
    assert!(text.ends_with("  - the hand-off: no 'Reviewed at:' commit in its message — the review is bound to nothing\n"), "{text}");
    // A name is never handed to git as a revision.
    assert_eq!(
        branch(Some("Reviewed at: work\n")).asked_after(true, NEWEST),
        ["hand-off"]
    );
}

impl Fake {
    fn asked_after(mut self, hand_off: bool, release: Version) -> Vec<String> {
        run(&mut self, hand_off, release);
        self.asked
    }
}

#[test]
fn nothing_to_read_reads_no_hand_off() {
    let mut fake = Fake {
        counts: BTreeMap::from([(format!("--no-merges {BASE}..HEAD"), 0)]),
        ..Fake::default()
    };
    assert_eq!(
        run(&mut fake, true, NEWEST),
        "Review binding sensor: no commit since aaaaaaa — nothing to read.\n"
    );
    // Before v0.11.0 the hand-off is read first.
    let mut fake = Fake {
        hand_off: Some(String::new()),
        ..Fake::default()
    };
    assert_eq!(
        run(&mut fake, true, V0_9),
        "Review binding sensor: no merge since aaaaaaa; 1 finding below.\n  - the hand-off: no \
         'Reviewed at:' commit in its message — the review is bound to nothing\n"
    );
    let mut fake = Fake::default();
    assert_eq!(
        run(&mut fake, false, V0_9),
        "Review binding sensor: no merge since aaaaaaa — nothing to read.\n"
    );
}

#[test]
fn each_merge_is_held_against_its_branch_before_v0_11_0_and_listed_from_it() {
    let merge = "dddddddddddddddddddddddddddddddddddddddd";
    let other = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
    let mut fake = Fake::default();
    let range = format!("{BASE}..HEAD");
    fake.lists.insert(
        format!("--first-parent --merges --reverse {range}"),
        vec![merge.to_owned(), other.to_owned()],
    );
    fake.lists.insert(
        format!("--merges --reverse {range}"),
        vec![merge.to_owned(), other.to_owned()],
    );
    fake.counts.insert(format!("--no-merges {range}"), 3);
    // The second merge takes up main, which the base has: it lands no branch.
    fake.ancestors
        .insert((format!("{other}^2"), BASE.to_owned()));
    fake.messages.insert(
        merge.to_owned(),
        "Merge topic\n\nReviewed at: 1234567\n".to_owned(),
    );
    fake.commits
        .insert("1234567".to_owned(), "1234567890".to_owned());
    fake.counts.insert(format!(" 1234567890..{merge}^2"), 1);
    fake.ancestors
        .insert(("1234567890".to_owned(), format!("{merge}^2")));
    let text = run(&mut fake, false, V0_9);
    assert_eq!(
        text,
        "Review binding sensor: 1 merge since aaaaaaa, 0 reviewed at the tip of its branch; 1 \
         finding below.\n  - ddddddd dddddddddddddddddddddddddddddddddddddddd: 1 commit after the review at \
         1234567, unreviewed:\n    log 1234567890..dddddddddddddddddddddddddddddddddddddddd^2\n    \
         stat 1234567 dddddddddddddddddddddddddddddddddddddddd^2 | 1 +\n    1 file changed, 1 insertion(+)\n",
        "{text}"
    );
    let text = run(&mut fake, false, NEWEST);
    assert_eq!(
        text,
        "Review binding sensor: 3 commits since aaaaaaa, bound only by a hand-off: name the pull \
         request's description with --hand-off <file>; 1 finding below.\n  - 2 merges between the \
         base and the head; a feat/ branch carries none (NUC-main-only-via-pr §5), and no review \
         is read from their messages:\n    ddddddd dddddddddddddddddddddddddddddddddddddddd\n    \
         eeeeeee eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee\n"
    );
}

#[test]
fn a_commit_mixing_adr_text_with_other_paths_is_listed_under_its_own_headline() {
    let mut fake = branch(None);
    fake.patches.insert(
        MID.to_owned(),
        "diff --git a/docs/adr/FIX-a.md b/docs/adr/FIX-a.md\n@@ -9 +9 @@\n-old\n+new\n".to_owned(),
    );
    fake.outside.insert(
        MID.to_owned(),
        ["a.py", "b.py", "c.py", "d.py", "e.py"]
            .map(str::to_owned)
            .to_vec(),
    );
    // A flip with code, and text without code, are not listed.
    fake.patches.insert(
        HEAD.to_owned(),
        "diff --git a/docs/adr/FIX-a.md b/docs/adr/FIX-a.md\n@@ -3 +3 @@\n-- **Date:** 1\n+- **Date:** 2\n"
            .to_owned(),
    );
    fake.outside
        .insert(HEAD.to_owned(), vec!["x.py".to_owned()]);
    fake.subjects
        .insert(MID.to_owned(), "bbbbbbb implement".to_owned());
    let text = run(&mut fake, false, NEWEST);
    assert!(
        text.ends_with(
            "--hand-off <file>; 1 finding below.\nReview binding sensor: 1 commit mixing ADR text \
             with other paths, which NUC-decisions-are-proposed-then-accepted §4 keeps in commits \
             apart:\n  - bbbbbbb implement: changes the text of docs/adr/FIX-a.md together with \
             a.py b.py c.py and 2 more\n"
        ),
        "{text}"
    );
}

#[test]
fn a_ref_named_like_a_hash_binds_nothing() {
    // git resolves the word to the ref, which it prefers to an object the word begins; the commit
    // the ref names does not begin with the word, so the review saw no such commit.
    let mut fake = branch(Some("Reviewed at: abcdef0\n"));
    fake.refs.insert("abcdef0".to_owned(), HEAD.to_owned());
    let text = run(&mut fake, true, NEWEST);
    assert!(
        text.ends_with(
            "  - the hand-off: reviewed at 'abcdef0', which is no commit of the branch since the base\n"
        ),
        "{text}"
    );
}
