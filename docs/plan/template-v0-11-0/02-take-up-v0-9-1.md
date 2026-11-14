# Ticket: Take up v0.9.1 and migrate the numbered record

## What to build

The repository works under v0.9.1. Section 8 of the normalize skill of v0.9.1 replaces ADR-0001 to
ADR-0006 by the twelve `NUC-` ADRs, `AGENTS.md`, and `docs/adr/NUC.blobs`. It renames ADR-0007 to
ADR-0028 by the map ticket 01 had confirmed, and gives the harness and toolchain ADRs their `Role:`
line. It runs `remap.sh --record` with the shipped map and the project's map. Then the other files
of v0.9.0 and v0.9.1 are taken up as the notes of both releases list. The project's own scripts and
checks stay green. Messages of `nuc` and parity fixtures that copy an old release keep their old
numbers, reverted by hand after the run. The `Template release` line moves to v0.9.1. The branch
stacks on ticket 01's and lands with it in one pull request.

## Acceptance criteria

- [ ] Given the migration, when `scripts/check-all.sh` runs, then it is green, the base-blob,
  identifier, and role rules included.
- [ ] Given the migration, when `git diff` against the base is read under `tests/parity/` and in
  the expected messages of `nuc`'s tests, then nothing changed outside a comment line.
- [ ] Given the Rust suite, when it runs, then it passes unchanged.
- [ ] Given the project's scripts that the template lacks (build, dependencies, gates, workflows,
  parity, release, bench, `gh` login), when the check runner runs them, then they pass.
- [ ] Given the commit body, when it is read, then it carries both maps and the clause table, and
  the hand-off names the repository settings the maintainer walks.

## Read first

- Touches: `docs/adr/`, `AGENTS.md`, `README.md` (Template), `scripts/`, `.githooks/`,
  `.github/`, `.agents/skills/`, `CONTRIBUTING.md`, `SECURITY.md`,
  `.devcontainer/devcontainer.json`, `.vscode/settings.json`, `.gitignore`, `docs/GLOSSARY.md`,
  `docs/CONVENTIONS.md`, `src/` and `tests/` (citations only)
- The template at v0.9.1: section 8 of the normalize skill, `remap.sh`, `map-v0.9.0.txt`.
- The release notes of v0.9.0 and v0.9.1, "Taking it up", the list of files to leave out included.
- The spec of this directory, "Existing behaviour", on what keeps its old numbers.

## Blocked by

- docs/plan/template-v0-11-0/01-propose-project-additions.md

## Mode

with the human: the migration renames every ADR the project holds, and the hooks and repository
settings it takes up are kept with the human by the harness ADR.
