# Ticket: Parity on the template's trees of v0.9.0 and v0.9.1

## What to build

The release v0.9 is done when the parity harness agrees on it. `docs/prose-width` counts a line's
width in code points from v0.9.1, not in bytes, so it no longer depends on the locale. The corpus
pins the template at the commits of v0.9.0 and v0.9.1, the table of releases gains every row the
trees show, and variants are added for each rule of v0.9 that fires on no pinned tree.

## Acceptance criteria

- [ ] Given the corpus with the template at v0.9.0 and v0.9.1, when the parity harness runs, then
      it agrees on both, their variants included.
- [ ] Given a line of 100 code points with characters beyond ASCII on a tree of v0.9.1, when
      `nuc check` runs, then `docs/prose-width` passes it; on a tree of v0.9.0 it reports it as the
      bash of v0.9.0 does.
- [ ] Given the table of releases, when tests/releases.rs runs, then each rule's changes at v0.9.0
      and v0.9.1 are covered.
- [ ] Given the pending list of the table runner, when this lands, then it holds only cases that
      v0.10.0 added and the `shell/` cases.

## Notes

- From ticket 05, for the normalization of `docs/base-blobs`: the bash names the compute failure
  `adr_blobs: git hash-object failed in <root>/docs/adr`, with the root's own path, where nuc names
  `docs/adr`; and it names the file of an unlisted or differing blob bare, `NUC-a.md:` or
  `AGENTS.md:`, where nuc names `docs/adr/NUC-a.md` and `AGENTS.md`, so a row places `docs/adr/`
  before a `NUC-` name only.

## Read first

- Touches: `tests/parity/corpus.toml`, `tests/parity/releases.tsv`, `tests/parity/variants.tsv`,
  `tests/parity/normalize.tsv`, `src/rules/docs_prose_width.rs`, `scripts/check-parity.sh`
- TEST-parity-harness, API-version-coupling-hook-defaults *The corpus*.
- The template's `check_prose_width` at v0.9.0 and v0.9.1.
- Seam: the parity harness.

## Blocked by

- docs/plan/port-v0-9-to-v0-11/04-the-record-rules-read-the-new-identifiers.md
- docs/plan/port-v0-9-to-v0-11/05-the-fixed-base-and-its-roles.md
- docs/plan/port-v0-9-to-v0-11/06-section-refs-and-skill-pointers.md
- docs/plan/port-v0-9-to-v0-11/07-traceability-of-v0-9.md

## Mode

with the human: a pull request that moves a corpus pin is reviewed like code (TEST-parity-harness),
and the corpus runs the bash of the pinned trees.
