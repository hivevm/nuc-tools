# TEST-hook-scenarios-on-bash-hooks: A scenario harness runs each hook through git, with the bash hooks of each pinned template tree up to the bound of the hooks `nuc` ports and with the shims and `nuc`, leaves out a tree whose blob list pins its hooks, and fails on any difference, on a self-test case no scenario covers, or when no tree is left

- **Status:** 🟢 accepted
- **Date:** 2026-11-18
- **Deciders:** Markus Brigl
- **Applies to:** the scenarios of the hooks and the merge driver under `tests/parity/`, the map from bash hook messages to `hook/` rule IDs and notices, the trees they run on and the bound `HOOKS_PORTED` in `scripts/lib/hook-parity.sh`, their run from `scripts/check-parity.sh`, and the coverage of the template's `scripts/test-git-hooks.sh`
- **Supersedes:** [TEST-hook-scenario-parity](TEST-hook-scenario-parity.md)

## Context

Q-2 asks for zero differences between `nuc` and the bash in findings and exit status, for every
ported check, hook, and script, and G-2 and G-3 ask it for the hooks and the merge driver.
[TEST-parity-harness](TEST-parity-harness.md) decides parity for checks: findings on the pinned
trees of a corpus and on edits of them. A hook decides from the git state, not from a tree:
- the branch;
- `MERGE_HEAD`;
- the refs a push sends;
- the updates of a reference transaction;
- `origin/main`;
- the record of green trees.

None of these is a tree's content, so TEST-parity-harness cannot compare a hook.
[TEST-hook-scenario-parity](TEST-hook-scenario-parity.md) decided a scenario harness for them,
run with the `.githooks/` of each template tree the corpus pins. This ADR restates that decision
in full and changes it in three points.

The template pins what each hook refuses and passes in its hook self-test,
`scripts/test-git-hooks.sh`:
- 58 cases with the same names at every release from v0.5.0 through v0.8.1, the hooks of v0.8.1
  being those of v0.7.0;
- the hooks of v0.11.1 are those of v0.11.0, and its self-test names the same cases;
- each case builds a fixture repository, runs one git command, and asserts the exit code and a
  substring of the message.

The substrings are the bash wording. G-6 asks for rule IDs, and the wording of `nuc` is left free by
[API-command-line-with-measure](API-command-line-with-measure.md), so a comparison maps
messages, as the tree harness does. Some `expect` cases assert no hook decision but its aftermath,
such as that a refused fast-forward leaves `main` where it was. A hook's standard error mixes its
own lines with git's and with the output of the check it ran, which the self-test asserts as well.

How a hook through `nuc` behaves is decided in
[API-hook-contract-keyed-shims](API-hook-contract-keyed-shims.md): the shim, the exit codes, and
the green tree. The defaults without `nuc.toml` are decided in
[API-version-coupling-hook-defaults](API-version-coupling-hook-defaults.md).

What changed since TEST-hook-scenario-parity was accepted:
- **Shims as the template's hooks.** From template release v0.12.0 on, `.githooks/` holds the
  reference shims of a `nuc` release, pinned by `.githooks/nuc.blobs`. Such a tree has no bash
  hook. A shim hands the decision to `nuc`, so a comparison would hold `nuc` against itself, and
  the tree's self-test tests `nuc`. The corpus pins two such trees: the template at v0.12.0 and
  this repository on v0.12.0.
- **An unrecorded bound.** The harness already left out a tree of a release later than the hooks
  `nuc` ports, by the bound `HOOKS_PORTED`. No ADR recorded the bound. The work that added v0.11.1
  to the corpus moved it from v0.11.0 to v0.11.1.
- **The newest tree.** The scenario of the defaults ran against the bash of the newest pinned
  release. The newest pinned release now carries shims, so that scenario has no bash to run there.

The human decided the behaviour. `scripts/lib/hook-parity.sh` and `scripts/check-parity.sh`
implement it, `scripts/test-check-parity.sh` tests it, and `docs/ARCHITECTURE.md` describes it.

## Decision

We will compare each hook through scenarios, with the bash hooks of the pinned template trees
up to the bound and with the shims and `nuc`. The merge driver is compared the same way. The
harness fails on any difference, on an uncovered self-test case, and when no tree is left.

1. **Scenarios.**
   - Each scenario builds a fixture repository and runs one git command.
   - Each scenario names the cases of the hook self-test it covers, by the name its `expect` line
     gives.
   - Each scenario builds its fixture through the real git CLI, as
     [TEST-fixtures-with-own-home](TEST-fixtures-with-own-home.md) has the tests build theirs.
   - Stand-in check scripts that exit as the scenario says decide what a hook finds, as the
     self-test's do.
2. **Trees.** The harness runs the scenarios on the template's trees of the corpus. It must leave
   a tree out in two cases:
   - **Pinned hooks.** The tree's `.githooks/nuc.blobs` pins a hook, as `pinned_files` of
     `scripts/lib/pin.sh` reads it. Such a tree carries the shims of `nuc` and no bash hook. The
     harness leaves it out whatever release its README names.
   - **The bound.** The tree's README names a release later than `HOOKS_PORTED` in
     `scripts/lib/hook-parity.sh`. The bound is v0.11.1, the last template release whose hooks
     are bash. Only the port of a later release's bash hooks may lift it.

   For each tree it leaves out, the harness must:
   - name the tree and the reason on standard output;
   - leave the tree's self-test cases out of the coverage;
   - still compare the tree's checks as any tree's, under TEST-parity-harness.

   A tree whose README names no release and pins no hook is compared. It fails where its hooks are
   not ported.
3. **The release.**
   - Each fixture's README carries the `Template release` line of the tree it is compared on, for
     both sides, and the bash side ignores it.
   - One scenario without README and without `nuc.toml` compares the defaults of the hook contract
     against the bash of the newest tree left in. Today that is the template at v0.11.1.
   - Each scenario runs on the `nuc` side twice: without `nuc.toml`, and with one that lists every
     `hook/` rule, as a project's template release ships it. Both runs must agree with the bash.
4. **Comparison.** Per scenario and tree, the harness compares:
   - the exit status of the git command;
   - the hook's lines, as sets. A hook's line starts with the hook's name and a colon, as every
     bash hook writes it. A table beside the harness maps each line to a `hook/` rule ID or a
     notice ID. The ID carries its fields, such as the ref and the count of the draft notice;
   - the lines of the stand-in checks, verbatim, since both sides run the same stand-ins;
   - for the merge driver, the bytes of the merged file and the exit status.

   The harness does not compare git's own lines. A hook's line no row of the table maps is a
   difference.
5. **Coverage.**
   - The harness reads each compared tree's `scripts/test-git-hooks.sh` for its `expect` names.
   - A scenario must cover each name, or a list beside the harness must name it as no case of a
     hook, with its reason.
   - Each of the following fails the harness:
     - an uncovered name;
     - a listed name no compared tree's self-test has;
     - a scenario that names a case no compared tree has.
6. **No tree left.** The harness must fail when the corpus pins no template tree, and when it
   leaves out every template tree it is given. No hook is compared then.
7. **Run.**
   - `scripts/check-parity.sh` runs the harness after the tree comparison, on Linux, in its job
     and from `scripts/check-all.sh`.
   - The harness runs the built `nuc`, which
     [SEC-gates-run-released-nuc](SEC-gates-run-released-nuc.md) leaves to the tests of the build,
     the parity harness among them, under its clause 5.
   - Its fixtures live under a temporary directory of its own and never touch the cache.

**Out of scope:**
- the format of the scenarios and of the table;
- the shims' tests on the other platforms of Q-4, which run in the crate's test suite;
- the latency of Q-5;
- how the hook self-test of a shim tree is held, which tests `nuc` and not the bash.

## Latitude

None.

## Alternatives considered

- **Compare a shim tree like any other.** Its `.githooks/` runs `nuc` on both sides, so every
  scenario agrees by construction and compares nothing. Its self-test cases would also enter the
  coverage, though they test `nuc`, not the bash.
- **Leave out by release, v0.12.0 and later, instead of by the blob list.** One rule instead of
  two. But the release line says what a project took up, and the blob list says what its
  `.githooks/` holds. A tree without a release line that pins the shims would be compared against
  itself.
- **Drop the bound now that the blob list decides.** One rule less. But a later tree that pins no
  hook would then be compared with hooks `nuc` has not ported, or pass on a self-test whose new
  cases nobody covers. The bound also names the last bash hooks in one place.
- **Pass when no tree is left.** A moved pin could then drop every hook comparison unseen, which
  Q-1 and TEST-parity-harness's rule against silent drops refuse.
- **Run the scenario of the defaults against the newest pinned tree, shims included.** That tree
  has no bash, so the scenario would hold `nuc`'s defaults against `nuc`.
- **Retire the harness.** The trees from v0.5.0 through v0.11.1 still carry bash hooks that
  projects run, and Q-2 asks for their comparison.
- **The template's self-test reused, with the shims in place of `.githooks/`.** The cheapest, and
  the cases are the oracle verbatim. But `nuc` would have to match the bash's substrings word for
  word, which API-command-line-with-measure leaves free. The rule IDs would never be compared. A
  message the self-test does not assert could differ unseen.
- **Rust tests only, no bash side.** No harness to maintain, but nothing compares, and Q-2 asks
  for a comparison of every ported hook.
- **Scenarios on every tree of the corpus.** A hook reads the git state, not the tree, and the
  derived trees carry the template's hooks of their release. The template's trees cover every
  release a hook could differ at.
- **Hooks folded into TEST-parity-harness's variants.** One harness, but a variant edits a tree and
  has no way to express a push, a merge in progress, or a reference transaction.
- **Revise TEST-hook-scenario-parity in place.** The narrowest form, but an accepted project ADR
  changes only by supersession
  ([NUC-decisions-are-proposed-then-accepted §6](NUC-decisions-are-proposed-then-accepted.md)).

## Sources / Prior art

- [TEST-hook-scenario-parity](TEST-hook-scenario-parity.md), the decision this ADR restates, and
  [TEST-parity-harness](TEST-parity-harness.md), its rule that a pin cannot silently drop a
  comparison.
- The template's `scripts/test-git-hooks.sh` at v0.5.0 through v0.8.1: 58 `expect` cases, the same
  names in each. The template's `.githooks/` at v0.8.1, the same as at v0.7.0. The template's
  `.githooks/` and self-test at v0.11.1, the same as at v0.11.0. The template's `.githooks/` and
  `.githooks/nuc.blobs` at v0.12.0, the shims of `nuc`.
- `scripts/lib/hook-parity.sh`, its header and `hook_parity`, which leave the trees out and hold
  the bound; `scripts/lib/pin.sh`, whose `pinned_files` reads the blob list;
  `tests/parity/corpus.toml`, the trees pinned.
- `tests/parity/hook-scenarios.sh`, the scenarios; `tests/parity/hooks.tsv`, the map of the hooks'
  lines; `tests/parity/hook-cases.tsv`, the cases that are no case of a hook, each with its reason.
- [githooks(5)](https://git-scm.com/docs/githooks): which git commands run which hook.

## Consequences

- Positive:
  - every bash hook decision and the merge driver are compared through git itself, per release
    up to v0.11.1 (G-2, G-3, Q-2);
  - a tree of shims no longer holds `nuc` against itself, and its self-test no longer counts as
    bash coverage;
  - a moved pin, a new self-test case, or a corpus of shim trees alone cannot drop a comparison
    unseen (Q-1);
  - the bound is recorded with its reason, so a reader does not take it for a stale value.
- Negative / trade-offs:
  - the scenarios restate the self-test's setups in this repository, and both have to be kept
    together;
  - each scenario runs git three times per compared tree, once with the bash and twice with
    `nuc`, which makes the parity job slower;
  - the harness runs on Linux only, where the bash runs;
  - the scenario of the defaults runs against v0.11.1, not the newest release, whose defaults
    `nuc` decides alone.
- Follow-ups:
  - retiring the harness once no pinned tree carries bash hooks;
  - folding the two harnesses into one at a later normalization, if they grow alike.

## Enforcement

The harness's own self-test, `scripts/test-check-parity.sh`, holds this decision. The change that
accepts this ADR moves every citation of TEST-hook-scenario-parity outside `docs/adr/` to it, as
`tests/adr_citations.rs` asks:
- the marker `Verifies: TEST-hook-scenario-parity` of `scripts/test-check-parity.sh`, and the
  citation in its header;
- `docs/ARCHITECTURE.md`, in the paragraph on the parity harness;
- `scripts/check-parity.sh`, in two comments;
- the headers of `scripts/lib/hook-parity.sh`, `tests/parity/hook-scenarios.sh`,
  `tests/parity/hooks.tsv`, and `tests/parity/hook-cases.tsv`.

The row of TEST-hook-scenario-parity in `docs/adr/README.md` moves under **Superseded and
rejected**. The accepted ADRs that cite it, API-hook-contract-keyed-shims and
TEST-copies-keep-old-identifiers, keep their text.

The self-test's cases check that each of the following fails:
- a stand-in `nuc` whose exit status differs from the bash in one scenario;
- one whose refusal maps to another rule ID;
- one that agrees without `nuc.toml` and differs with it;
- an unknown line of a bash hook;
- an uncovered self-test case;
- a listed case the self-test lacks;
- a scenario that names a case no release has;
- hook scenarios without a tree of the template (clause 6).

Its cases also check that:
- an agreeing run passes;
- a tree later than the bound is left out, and its difference is not compared (clause 2);
- a tree at the bound is compared (clause 2);
- without the bound set, a tree at v0.11.1 is compared and one at v0.11.2 left out (clause 2);
- the scenario of the defaults runs on the newest tree left in, not on a newer one left out
  (clause 3);
- a tree whose blob list pins its hooks is left out and named on standard output (clause 2);
- that tree's unknown hook line and uncovered case fail nothing (clauses 2 and 5);
- a run with that tree alone fails (clause 6).
