# Ticket: A comment line of the code is held to 100 columns

<!-- A ticket of the design revision of 2026-11-14, in the shape of the plan-feature skill's
     ticket template, without a spec. Claim it by creating the branch
     ticket/revision-2026-11-14/<NN>-<slug> BEFORE any work, by the steps of the work-tickets
     skill. The branch that lands it deletes this file. Its sections are the parts a ticket
     carries, as docs/adr/NUC-large-work-is-planned.md decides (clause 3). -->

## What to build

A comment line in a Rust file, a shell script, or a workflow of the project's own is at most 100
columns, as Markdown prose is under the rule `docs/prose-width`, and a check of the project fails
on a longer one. The files the template ships and the project carries unchanged are left out: a
rewrap there is merge work at every take-up, and their long lines are a finding about the
template, not a ticket here. The check reads which files are the template's from the release the
`Template release` line names, or from a list the human chooses. The
review of #80 found a doc comment of `tests/releases.rs` at 102 columns, "which nothing enforces",
and left it; the review of the migration in #84 found comment lines over 100 columns in
`checks.yml`, `check-all.sh`, `test-check-parity.sh`, and `tests/releases.rs`, fixed by hand in
da53883. Code lines are left to the formatters; only comment lines are held.

## Acceptance criteria

- [ ] Given the tree as it stands, when the check runs, then it passes, or the hand-off lists each
      line of the project's own files it rewrapped first.
- [ ] Given a long comment line in a file the template ships, such as `scripts/check-docs.sh`,
      when the check runs, then it passes.
- [ ] Given a comment line of 101 columns in a `.rs`, `.sh`, or `.yml` file, when the check
      runs, then it fails and names the file and the line.
- [ ] Given a code line of 101 columns, when the check runs, then it passes.
- [ ] Given the check, when `scripts/check-all.sh` and `checks.yml` are read, then both run it
      and its self-test, and the rule `docs/check-lists` passes.

## Read first

- Touches: a new `scripts/check-comment-width.sh` and its self-test, `scripts/check-all.sh`,
  `.github/workflows/checks.yml`, `CONTRIBUTING.md` (required checks)
- The rule `docs/prose-width` of `scripts/check-docs.sh` for how width is counted; the
  convention "A check's cases are a table" of `docs/CONVENTIONS.md`.

## Blocked by

- None (can start immediately)

## Mode

with the human — DEV-one-agent-one-ticket allows no unattended ticket, and a new check changes
the required checks of the ruleset on `main`.
