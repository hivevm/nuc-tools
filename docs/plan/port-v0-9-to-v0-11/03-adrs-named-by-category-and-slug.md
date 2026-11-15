# Ticket: ADRs are named by category and slug from v0.9.0

## What to build

`nuc` reads the record of a tree on v0.9.0 or later as that release names it: an ADR's identifier
is its file name `<CAT>-<slug>`, the category fixed (`NUC`, `SEC`, `OPS`, `API`) or listed under
`### Categories` in the index. The model of the record reads both forms by release, so each rule
takes the identifier from it. On such a tree:

- `docs/adr-identifiers` reports as the bash of v0.9.1 does: a category not fixed or listed, a slug
  out of form, a heading that does not open with the identifier, a numbered file, and an
  identifier the first-parent history deleted, naming the commit; the shell reads that history;
- `docs/adr-numbering`, `docs/no-draft-on-main`, and `docs/inherited-adrs` report nothing.

v0.9.0 and v0.9.1 join the releases at which a ported check changed. A tree on v0.8.1 or earlier
is checked exactly as before.

## Acceptance criteria

- [ ] Given each case of the copied tables for `docs/adr-identifiers` not keyed `adr-numbering`,
      when the runner runs, then it agrees and leaves the pending list.
- [ ] Given a tree on v0.9.0 with an ADR whose identifier the history deleted, when `nuc check`
      runs, then it reports the identifier and the commit as the bash does.
- [ ] Given a tree on v0.8.1, when `nuc check` runs, then its findings are those of today, and
      the parity harness agrees on the corpus.
- [ ] Given a tree on v0.9.0 and a `nuc.toml` listing the retired rules, when `nuc check` runs,
      then they report nothing and the run is not refused.
- [ ] Given `docs/adr-identifiers`, when the contract test runs, then the ID is listed and
      `nuc explain` gives its reason.

## Read first

- Touches: `src/adr.rs`, `src/rules/mod.rs`, `src/rules/docs_adr_numbering.rs`,
  `src/rules/docs_no_draft_on_main.rs`, `src/rules/docs_inherited_adrs.rs`, a new rule module,
  `src/shell/mod.rs`, `src/shell/git.rs`, `src/config.rs`, `tests/contract.rule-ids`,
  `tests/parity/releases.tsv`, `tests/git_cli.neutralized`
- The template's `scripts/check-docs.sh` and `scripts/lib/adr.sh` at v0.8.1 and v0.9.1, diffed with
  `git diff`, never run; the v0.9.0 tag message.
- Golden path: a rule that dispatches by release today, such as docs/adr-supersessions.
- Seam: the table runner of ticket 12; the parity harness.

## Blocked by

- docs/plan/port-v0-9-to-v0-11/12-the-rules-run-against-the-template-tables.md

## Mode

with the human: it changes the parser of ADR files and adds a git command to the shell, both
protected parts under DEV-one-agent-one-ticket.
