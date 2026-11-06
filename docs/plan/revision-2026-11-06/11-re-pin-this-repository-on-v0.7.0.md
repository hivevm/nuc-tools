# Ticket: Re-pin this repository in the parity corpus on template release v0.7.0

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

`tests/parity/corpus.toml` pins this repository at 0c07796, on template release v0.6.1, after the
port of `check-traceability.sh`. The repository has carried v0.7.0 since its take-up (150ee63), and
the spec of the hooks' port named the re-pin out of its scope; no ticket carries it, so it would
be lost with the spec. The corpus compares v0.7.0 only on the template's own tree, `nuc-v0-7-0`;
this repository on v0.7.0 is the one derived tree on that release the corpus could hold, its
`CONTRIBUTING.md` and its own `docs/plan/` included.

Move the pin to a commit of `main` on v0.7.0, after this revision merges, and its comment with it.
The parity harness then compares `nuc` with the bash of v0.7.0 on that tree; a difference it shows
is a finding for a ticket of its own, not a reason to keep the old pin.

## Acceptance criteria

- [ ] Given the new pin, when `scripts/check-parity.sh` runs, then it compares this repository as
  carrying v0.7.0, and reports no difference, or each difference is filed.
- [ ] Given `tests/parity/corpus.toml`, when it is read, then the comment of the entry names the
  release and the commit's place on `main`.

## Read first

- Touches: `tests/parity/corpus.toml`
- ADR-0015; ADR-0023, the trees the corpus pins per release.

## Blocked by

- None (can start immediately)

## Mode

with the human — ADR-0007 allows no unattended ticket; a pull request moves a pin (ADR-0015).
