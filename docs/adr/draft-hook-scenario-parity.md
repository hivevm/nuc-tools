# ADR-draft-hook-scenario-parity: A scenario harness runs each hook through git, once with the bash hooks of each pinned template release and once with the shims and `nuc`, and fails on any difference or on a self-test case no scenario covers

- **Status:** 🟡 proposed
- **Date:** 2026-11-06
- **Deciders:** Markus Brigl
- **Applies to:** the scenarios of the hooks and the merge driver under `tests/parity/`, the map from bash hook messages to `hook/` rule IDs and notices, their run from `scripts/check-parity.sh`, and the coverage of the template's `scripts/test-git-hooks.sh`

## Context

Q-2 asks for zero differences between `nuc` and the bash in findings and exit status, for every
ported check, hook, and script, and G-2 and G-3 ask it for the hooks and the merge driver.
[ADR-0015](0015-parity-harness.md) decides parity for checks: findings on the pinned trees of a
corpus and on edits of them. A hook decides from the git state, not from a tree:
- the branch;
- `MERGE_HEAD`;
- the refs a push sends;
- the updates of a reference transaction;
- `origin/main`;
- the record of green trees.

None of these is a tree's content, so ADR-0015 cannot compare a hook.

The template pins what each hook refuses and passes in its hook self-test,
`scripts/test-git-hooks.sh`:
- 58 cases;
- the same names at v0.5.0 through v0.7.0;
- each building a fixture repository, running one git command, and asserting the exit code and a
  substring of the message.

The substrings are the bash wording. ADR-0013 leaves the wording of `nuc` free, and G-6 asks for
rule IDs, so a comparison has to map messages, as the tree harness does. Some `expect` cases
assert no hook decision but its aftermath, such as that a refused fast-forward leaves `main` where
it was. A hook's standard error mixes its own lines with git's and with the output of the check it
ran, which the self-test asserts as well.

How a hook through `nuc` behaves is decided in [ADR-draft-hook-contract](draft-hook-contract.md):
the shim, the exit codes, the green tree, and the defaults without `nuc.toml`.

## Decision

We will decide the parity of the hooks and the merge driver by scenarios kept in this repository.
Each scenario builds a fixture repository and runs one git command, once with the `.githooks/` of
each template tree the corpus pins and once with the reference shims and the built `nuc`. The
harness compares the exit status, the refusals mapped to rule IDs, and the notices, and fails on
any difference and on any case of a pinned release's hook self-test that no scenario covers.

- **Scenarios:**
  - each names the cases of the hook self-test it covers, by the name its `expect` line gives;
  - it builds its fixture through the real git CLI, as
    [ADR-0012](0012-fixture-repositories.md) has the tests build theirs;
  - stand-in check scripts that exit as the scenario says decide what a hook finds, as the
    self-test's do.
- **The release:**
  - each fixture's README carries the `Template release` line of the tree it is compared on, for
    both sides, and the bash side ignores it;
  - one scenario without README and without `nuc.toml` compares the defaults of the hook
    contract against the bash of the newest pinned release;
  - each scenario runs on the nuc side twice, without `nuc.toml` and with one that lists every
    `hook/` rule, as a project's template release will ship it; both must agree with the bash.
- **Comparison:**
  - per scenario and release: the exit status of the git command;
  - the hook's lines, those that start with the hook's name and a colon, as every bash hook
    writes them: each is mapped by a table beside the harness to a `hook/` rule ID or a notice ID
    with the fields it carries, such as the ref and the count of the draft notice, and compared as
    sets;
  - the lines of the stand-in checks, verbatim, since both sides run the same stand-ins;
  - for the merge driver, the bytes of the merged file and the exit status.

  Git's own lines are not compared. A hook's line no row of the table maps is a difference.
- **Coverage:**
  - each pinned release's `scripts/test-git-hooks.sh` is read for its `expect` names;
  - each must be covered by a scenario or listed beside the harness as no case of a hook, with
    its reason;
  - an uncovered name, a listed name the self-test lacks, and a scenario that names a case no
    release has each fail the harness.
- **Run:**
  - from `scripts/check-parity.sh`, after the tree comparison, on Linux, in its job and from
    `scripts/check-all.sh`;
  - with the built `nuc`, which ADR-0016 allows the parity harness;
  - its fixtures live under a temporary directory of its own and never touch the cache.

**Out of scope:**
- the format of the scenarios and of the table, set by the first ticket that uses them;
- the shims' tests on the other platforms of Q-4, which run in the crate's test suite;
- the latency of Q-5.

## Alternatives considered

- **The template's self-test reused, with the shims in place of `.githooks/`.** The cheapest, and
  the cases are the oracle verbatim. But `nuc` would have to match the bash's substrings word for
  word, which ADR-0013 leaves free. The rule IDs would never be compared. A message the self-test
  does not assert could differ unseen.
- **Rust tests only, no bash side.** No harness to maintain, but nothing compares, and Q-2 asks
  for a comparison of every ported hook.
- **Scenarios on every tree of the corpus.** A hook reads the git state, not the tree, and the
  derived trees carry the template's hooks of their release. The template's trees cover every
  release a hook could differ at.
- **Hooks folded into ADR-0015's variants.** One harness, but a variant edits a tree and has no
  way to express a push, a merge in progress, or a reference transaction.

## Sources / Prior art

- [ADR-0015](0015-parity-harness.md): the tree harness, its normalization table, and its rule
  that a pin cannot silently drop a comparison.
- The template's `scripts/test-git-hooks.sh` at v0.5.0 through v0.7.0: 58 `expect` cases, the same
  names in each, and the oracle the scenarios are named after.
- [githooks(5)](https://git-scm.com/docs/githooks): which git commands run which hook.

## Consequences

- Positive:
  - every hook decision and the merge driver are compared through git itself, per release (G-2,
    G-3, Q-2);
  - a moved pin or a new self-test case cannot drop a comparison unseen (Q-1).
- Negative / trade-offs:
  - the scenarios restate the self-test's setups in this repository, and both have to be kept
    together;
  - each scenario runs git twice per pinned release, which makes the parity job slower;
  - the harness runs on Linux only, where the bash runs.
- Follow-ups:
  - retiring the harness after the switch-over;
  - folding the two harnesses into one at a normalization, if they grow alike.

## Enforcement

The harness's own self-test, `scripts/test-check-parity.sh`, cited with
`Verifies: ADR-draft-hook-scenario-parity` and extended by the tracer of `docs/plan/hooks-port/`,
checks that each of the following fails:
- a stand-in `nuc` whose exit status differs from the bash in one scenario;
- one whose refusal maps to another rule ID;
- one that agrees without `nuc.toml` and differs with it;
- an unknown line of a bash hook;
- an uncovered self-test case;
- a listed case the self-test lacks;
- a scenario that names a case no release has.

It also checks that an agreeing run passes.
