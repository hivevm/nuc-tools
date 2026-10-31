# ADR-0012: A parity harness compares `nuc` with the bash scripts of a pinned corpus on the same trees, as a required check that fails on any difference

- **Status:** ⚪ superseded by ADR-draft-parity-harness-with-variants
- **Date:** 2026-10-31
- **Deciders:** Markus Brigl
- **Applies to:** `tests/parity/`, its manifest `tests/parity/corpus.toml`, `scripts/check-parity.sh`, its job in `.github/workflows/checks.yml`, and the map from bash messages to rule IDs

## Context

The strategy ports the bash scripts one by one, each proved against the bash of the matching
template release, on the template and on derived repositories (*Strategy*). Q-2 asks for zero
differences in findings and exit status on every repository of the corpus, for every ported
check, hook, and script; G-1 and G-2 state the same for `nuc check` and the hooks. A ticket of
the port is done only when the harness shows no difference. The bash scripts carry no rule IDs,
so their output has to be mapped onto the IDs of `nuc` before it can be compared (G-6). The
assumptions keep the bash runnable on Linux and the corpus readable, and leave its members open.
[ADR-0002](0002-decisions-verified-by-tests.md) keeps the checks, `scripts/check-all.sh`, and the
required checks one list.

## Decision

We will decide parity by a harness that runs the bash scripts and `nuc` on the same trees of a
corpus pinned by commit, compares their normalized findings and exit status, and fails on any
difference, run by `scripts/check-parity.sh` as a required job on Linux and from
`scripts/check-all.sh`.

- **Corpus:** `tests/parity/corpus.toml` lists each repository by URL and commit SHA: the
  template at a release tag's commit, and derived repositories at fixed commits. A pull request
  moves a pin.
- **Cache:** the harness fetches the corpus into an ignored cache under `.scratch/`; when it
  cannot fill the cache, it fails (Q-1).
- **Normalization:** the bash output is mapped to (rule ID, file, line) by a table beside the
  harness; a bash message the table does not know is a difference.
- **Comparison:** findings as sets, and exit status, per check, hook, and script ported; any
  difference fails.

**Out of scope:** which derived repositories form the corpus, decided when the harness is
planned; when the harness is retired, once the bash leaves the template.

## Alternatives considered

- **Parity in CI per pull request and weekly, not in the pre-push hook** — faster pushes, but a
  job outside `scripts/check-all.sh` breaks the lists ADR-0002 holds together, and a ticket could
  be handed off unproved.
- **Corpus at moving branches** — a commit elsewhere would turn this repository red; pinned
  commits keep a red run a change of this repository.
- **Comparing raw output** — the bash and `nuc` word their findings differently by design (G-6);
  compared raw, every finding would differ.

## Sources / Prior art

- The template's checks, sensors, and hooks at its release tags, the oracle.

## Consequences

- Positive: parity is a fact a check decides, not a claim of a ticket (Q-2); the corpus pins make
  a red run reproducible.
- Negative / trade-offs: the pre-push hook needs the network until the cache is filled, and is
  slower by the corpus; the harness runs on Linux only, where the bash runs; the normalization
  table is code to maintain, and a mistake in it can hide a difference both sides share.
- Follow-ups: the members of the corpus; moving the pins with each template release; retiring
  the harness after the switch-over.

## Enforcement

The harness's own self-test, cited with `Verifies: ADR-0012`, feeds it a pair of
outputs that differ in one finding, one exit status, and one unknown bash message, and checks
that each fails, and a pair that agrees, and checks that it passes. Check 9 of
[`scripts/check-docs.sh`](../../scripts/check-docs.sh) holds the `parity` job, its `run` line, and
the README's required checks together.
