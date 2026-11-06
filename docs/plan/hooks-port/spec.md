# Spec: Port the git hooks and the ADR index merge driver to `nuc`

## Problem

A derived project's gates still run as bash: five scripts, four hooks and the
merge driver of the ADR index. They need bash 4.4 or later and the GNU tools, so a project on
Windows or macOS depends on what its machine ships, and the template cannot switch its hooks over
to `nuc` until `nuc` decides what each hook decides, as the bash of the release does. Today `nuc`
ports only the checks of `check-docs.sh` and `check-traceability.sh` (G-1); `nuc hook` does not
exist, and nothing shows that a hook run through `nuc` refuses and passes exactly where the bash
does (G-2, Q-2). A project with a missing or too old `nuc` would have no hook at all instead of
one that stops (G-5).

## Solution

A project that takes up the reference shims gets its hooks from `nuc`. Git runs the shim, the shim
hands over to `nuc hook <name>`, and `nuc` refuses or passes as the bash hook of the template
release the project carries. Where a hook runs the project's checks, `nuc` decides whether they
run and the shim runs them, so `nuc` never executes repository content (Q-7). When `nuc` is
missing or older than `nuc.toml` requires, every shim stops with a message naming how to install
it. When git merges the ADR index, `nuc` writes the index both sides imply, or stops with a
conflict when both changed its prose (G-3). A scenario harness shows, for every hook and every
template release of the corpus, that `nuc` and the bash agree.

## Existing behaviour

- The bash hooks and the merge driver are the template's, identical from v0.5.0 to v0.6.1; v0.7.0
  changes one message of `pre-push`, which names `CONTRIBUTING.md` instead of the README. This
  repository carries the v0.7.0 hooks and keeps them as they are: its gates never run the
  `nuc` being built (ADR-0016).
- What each hook refuses and passes is pinned by the template's hook self-test, one
  case per row of the table under "Git hooks" in `CONTRIBUTING.md`. Those cases are the oracle of
  the scenarios; nothing in `nuc` pins hook behaviour yet.
- `nuc check` and its exit codes 0, 1, 2 (ADR-0013), the fail-closed reading of `nuc.toml`
  (ADR-0014), and the parity harness over trees (ADR-0015) stay as they are; the scenario harness
  sits beside the tree harness and does not change what it compares.
- Nothing migrates: a project keeps its bash hooks until the template ships the shims.

## Goals served

- G-2, G-3, G-5
- Q-1, Q-2, Q-4, Q-5, Q-7, Q-8, Q-9, Q-10, Q-11

## User stories

1. As a maintainer of a derived project, I want git's hooks to run through `nuc`, so that they
   need no bash 4.4 and no GNU tools on my machine.
2. As a maintainer, I want a hook through `nuc` to refuse and pass exactly as the bash hook of my
   template release does, so that switching over changes nothing I rely on.
3. As a maintainer without `nuc` installed, or with one older than my `nuc.toml` asks for, I want
   every hook to stop and tell me how to install it, so that no commit, merge, or push passes a
   gate that did not run.
4. As an agent committing on `main` by mistake, I want the commit refused with the command that
   creates a branch, as today.
5. As an agent concluding a conflicted merge, I want the shim to run the project's checks on the
   resolution, as today, without `nuc` running any script of the repository.
6. As a maintainer pushing a tree the checks already passed on, I want the push to skip the checks,
   and to have the tree recorded once they pass.
7. As a maintainer moving the local `main` by a fast-forward, a reset, or `git branch -f`, I want
   the move refused unless `origin/main` has the commit, at a cost per reference update I do not
   notice.
8. As two branches each adding an ADR, I want their index rows merged without a conflict, and a
   conflict where both reworded the index's prose.
9. As the template maintainer, I want shims I can copy as they are, tested here on every platform
   of Q-4.
10. As the maintainer of this repository, I want a scenario harness that fails on any difference
    between `nuc` and the bash hooks of each pinned template release, so that parity is a fact a
    check decides.

## Implementation decisions

- **Split between shim and `nuc`:** `nuc hook <name>` decides what git state decides: the branch,
  `MERGE_HEAD`, the refs a push sends, the updates of a reference transaction, the record of green
  trees. The shim runs the project's own scripts, the full and the documentation check, when `nuc`
  says they run, and nothing else. The bash's clearing of git's local variables before the checks
  stays with the shim, where the checks run.
- **pre-push:** `nuc hook pre-push` refuses a push to the remote's `main`, passes a push that only
  deletes, writes the draft notice in the wording of the carried release, and looks the tree up
  in the record of green trees. It tells the shim whether the checks run or are skipped; after
  they pass, the shim has `nuc` record the tree. How it tells the shim is the hook contract ADR.
- **Rule IDs:** each refusal of a hook has a stable rule ID in a `hook/` area, mapped from the
  bash message by the scenario harness's table, as the tree harness maps the checks'.
- **Reference shims:** POSIX sh, so that git for Windows runs them with the sh it bundles; one
  per hook and one for the merge driver, kept in a directory of this repository apart from its
  own hooks. They find `nuc` on the `PATH`; a missing binary is reported by the shim, a too old
  one by `nuc` from `nuc.toml` (ADR-0014).
- **Merge driver:** `nuc` is the driver command `.gitattributes` names for the ADR index, called
  through its shim with git's arguments, and writes the merged file as the bash does: the union of
  the rows where the prose merges on its own, the usual markers otherwise.
- **Scenario harness:** each scenario builds a fixture repository and runs one git command once
  with the bash hooks of a pinned template release and once with the reference shims and the
  built `nuc`, and compares exit status and mapped refusals. The scenarios are those of the
  template's hook self-test; the releases are the template trees the corpus pins. It runs from
  the local gate and as part of the parity job, on Linux.
- **Q-5:** a benchmark of `nuc hook reference-transaction` against a no-op hook over 1,000 calls,
  failing above 2 ms median, runs in the Linux x64 job of CI only, not in the pre-push hook.

## Testing decisions

- The scenario harness is the seam for G-2 and G-3: it decides parity end to end through git.
  Unit tests in each hook's module cover what a scenario cannot reach cheaply, citing the bash
  case they mirror, as the rules' tests do.
- The shims are tested through git as well, in fixture repositories built as the tests of `nuc
  check` build them (ADR-0012), on all four platforms of Q-4, with no bash on the `PATH`.
- G-5 is shown per shim with `nuc` absent from the `PATH` and with a `nuc.toml` whose minimum is
  above the running version.
- The isolation tests of Q-7 to Q-11 extend to `nuc hook`: a canary script the shim must not run
  unasked, a decoy repository through git's variables, a canary token, and no network.
- Prior art: the tree harness and its normalization table, the fixture helpers of the tests,
  and the template's hook self-test.

## Out of scope

- Switching this repository's hooks or the template's over to `nuc`; that is the template's
  switch-over and, here, the switch of ADR-0016.
- Porting the full check itself, or any check it runs beyond those already ported.
- `nuc upgrade` (G-4) and the installation of `nuc`; the shims only name how to install it.
- Hooks the template does not ship.
- Re-pinning this repository in the parity corpus at a commit on template release v0.7.0.

## ADR impact

Binding: ADR-0010 (rules return an outcome), ADR-0011 (git CLI with a cleared environment),
ADR-0012 (fixture repositories), ADR-0013 (command line contract), ADR-0014 (version coupling),
ADR-0015 (parity harness), ADR-0016 (gates never run the build), ADR-0008 (toolchain and CI jobs).

Two new ADRs, both proposed in the tracer ticket before its implementation:

- **Hook-scenario parity:** a harness of scenarios through git, run with the bash hooks of each
  pinned template release and with the shims and `nuc`, failing on any difference. ADR-0015
  compares findings on trees and does not cover git state.
- **Hook contract:** what `nuc hook` decides and what the shim runs, how `nuc` tells the shim to
  run or skip the checks and how a passed run is recorded, the reference shims in POSIX sh. A
  signal beyond 0, 1, 2 widens the exit codes of ADR-0013, which would then be superseded in
  that clause.
