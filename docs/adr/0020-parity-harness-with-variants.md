# ADR-0020: A parity harness compares `nuc` with the bash scripts on the pinned trees of a corpus of releases v0.5.0 and later and on variants derived from them, as a required check that fails on any difference

- **Status:** 🟢 accepted
- **Date:** 2026-10-31
- **Deciders:** Markus Brigl
- **Applies to:** `tests/parity/`, its manifest `tests/parity/corpus.toml`, the variants kept beside it, `scripts/check-parity.sh`, its job in `.github/workflows/checks.yml`, and the map from bash messages to rule IDs
- **Supersedes:** [ADR-0012](0012-parity-harness.md)

## Context

The strategy ports the bash scripts one by one, each proved against the bash of the matching
template release, on the template and on derived repositories (*Strategy*). Q-2 asks for zero
differences in findings and exit status on every repository of the corpus, for every ported
check, hook, and script; G-1 and G-2 state the same for `nuc check` and the hooks. A ticket of
the port is done only when the harness shows no difference. The bash scripts carry no rule IDs,
so their output has to be mapped onto the IDs of `nuc` before it can be compared (G-6).
[ADR-0002](0002-decisions-verified-by-tests.md) keeps the checks, `scripts/check-all.sh`, and the
required checks one list.

[ADR-0012](0012-parity-harness.md) compares on the trees of the corpus as pinned. Those trees pass
every check, so agreement there shows only that both pass a clean tree: a ported rule that never
fires would agree as well. The port of `check-docs.sh` needs each check compared where it fires.
The maintainer also limits the corpus to repositories on template release v0.5.0 or later, the
release the rules are ported from; earlier releases are out of the port's scope.

## Decision

We will decide parity by a harness that runs the bash scripts and `nuc` on the pinned trees of a
corpus of template releases v0.5.0 and later and on variants derived from those trees by
committed edits, compares their normalized findings and exit status, and fails on any
difference, run by `scripts/check-parity.sh` as a required job on Linux and from
`scripts/check-all.sh`.

- **Corpus:** `tests/parity/corpus.toml` lists each repository by URL and commit SHA: the
  template at a release tag's commit, and derived repositories at fixed commits. A pull request
  moves a pin. Each pinned tree carries template release v0.5.0 or later; the harness refuses an
  entry whose tree names an earlier release or none, and never compares it.
- **Variants:** beside the harness, each ported rule brings one or more edits that make it fire,
  such as a deleted line or a broken link; the harness applies each to every corpus tree and
  compares on the result as on the tree itself. An edit that does not apply to a tree, a ported
  rule without an edit, and an edit on which the bash reports no finding of its rule beyond the
  tree's own each fail the harness, so that a moved pin cannot silently drop a comparison (Q-1).
  An edit stays inside the copy it is applied to: it names a path within the tree, and runs no
  command and writes no other file.
- **Cache:** the harness fetches the corpus into an ignored cache under `.scratch/`; when it
  cannot fill the cache, it fails (Q-1). It derives the variants in a directory of its own and
  never writes to the cached trees.
- **Normalization:** the bash output is mapped to (rule ID, file, line) by a table beside the
  harness; a bash message the table does not know is a difference.
- **Comparison:** findings as sets, and exit status, per check, hook, and script ported; any
  difference fails.

**Out of scope:** the format of the variants; which derived repositories of v0.5.0 or later form
the corpus, moved by pull requests; when the harness is retired, once the bash leaves the
template.

## Alternatives considered

- **Keeping [ADR-0012](0012-parity-harness.md) and reading variants into "the same trees of a
  corpus"** — the decision names what is compared; a new kind of input with failure modes of its
  own is a decision, not a reading.
- **No variants, the pinned trees only** — the cheapest, but a rule that never fires agrees with
  the bash on a clean tree.
- **Committed failing repositories in the corpus** — real trees, but each needs a repository kept
  red on purpose; [ADR-0011](0011-fixture-repositories.md) keeps repositories out of this one.
- **Corpus repositories on any release** — wider, but each earlier release's checks would need
  rule IDs of their own for behaviour the maintainer leaves out of scope.
- **Parity in CI per pull request and weekly, not in the pre-push hook** — faster pushes, but a
  job outside `scripts/check-all.sh` breaks the lists ADR-0002 holds together, and a ticket could
  be handed off unproved.
- **Corpus at moving branches** — a commit elsewhere would turn this repository red; pinned
  commits keep a red run a change of this repository.
- **Comparing raw output** — the bash and `nuc` word their findings differently by design (G-6);
  compared raw, every finding would differ.

## Sources / Prior art

- The template's checks, sensors, and hooks at its release tags, the oracle.
- Mutation testing, which changes code to show a test fails on it; the variants change the input
  to show a check fires on it.

## Consequences

- Positive: parity is a fact a check decides, on trees where each rule fires as well as where it
  passes (Q-2); the pins and the committed edits make a red run reproducible.
- Negative / trade-offs: the pre-push hook needs the network until the cache is filled, and is
  slower by the corpus and its variants; the harness runs on Linux only, where the bash runs; the
  normalization table and the edits are code to maintain, and a mistake in either can hide a
  difference both sides share.
- Follow-ups: the format of the variants, set by the first ticket that uses them; moving the pins
  with each template release; retiring the harness after the switch-over.

## Enforcement

The harness's own self-test, cited with `Verifies: ADR-0020`, feeds
it a pair of outputs that differ in one finding, one exit status, and one unknown bash message,
and checks that each fails, and a pair that agrees, and checks that it passes; it runs it on a
corpus entry of a release before v0.5.0 and checks that it fails, and on a variant that applies,
one that does not apply to one of the trees, one that does not make its rule fire, a ported rule
without a variant, and a variant that reaches outside its copy, and checks that the first is
compared and the others fail.
Check 9 of [`scripts/check-docs.sh`](../../scripts/check-docs.sh) holds the `parity` job, its
`run` line, and the README's required checks together.
