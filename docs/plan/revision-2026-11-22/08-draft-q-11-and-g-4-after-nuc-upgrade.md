# Ticket: Draft Q-11 and G-4 after nuc upgrade

## What to build

A drafted change of `docs/SPECIFICATION.md`, which the human lands or declines
(NUC-documents-are-fixed §2). Two criteria the spec of `nuc upgrade` served no longer say all the
tests decide:

- **Q-11** reads "No network in checks and hooks: with network access blocked, every check and
  hook reports the same findings as with it." `nuc upgrade` reaches no network either, by
  API-upgrade-by-blob-toml-optional: the maintainer fetches the tag, and the ADR names
  fetching a tag as "the first network access of `nuc` outside a sensor that names it". Q-11 does
  not cover it.
- **G-4** reads "`nuc upgrade` applies its mechanical migrations and lists what needs judgment." The
  work decided against migrations coded per pair of releases. The command writes every change the
  two releases and the project's tree decide, and lists every other as a finding. The glossary now
  defines **migration** that way, so G-4 holds as written; a sharper wording names the exit codes
  the tests decide.

The draft:

- **Q-11** — No network in checks, hooks, and take-ups: with network access blocked, every check
  and hook reports the same findings as with it, and `nuc upgrade` writes the same files and
  lists the same findings.
- **G-4** — When a project takes up a template release, `nuc upgrade` writes into the working tree
  every migration the two releases and the project's tree decide, lists each judgment as a
  finding, and exits 0 without one and 1 with one.

Accepted ADRs the change moves: none. API-upgrade-by-blob-toml-optional decides both already. If
Q-11 is landed, ticket 03's tests gain a case run without the network, as
`tests/isolation.rs` runs `nuc check`.

## Acceptance criteria

- [ ] Given the human's decision, when the session lands it, then `docs/SPECIFICATION.md` reads
  the drafted Q-11 and G-4, or this ticket is deleted with the human's reason.
- [ ] Given a landed Q-11, when `tests/isolation.rs` runs, then `nuc upgrade` without the network
  writes and lists what it does with it, cited with `Verifies: Q-11`.

## Read first

- Touches: `docs/SPECIFICATION.md`, `tests/isolation.rs`
- Q-11 and G-4 in `docs/SPECIFICATION.md`; the glossary's **take-up**, **migration**, and
  **judgment**; API-upgrade-by-blob-toml-optional (*Decision*, *Alternatives considered*)

## Blocked by

- None (can start immediately)

## Mode

with the human: the specification is the human's.
