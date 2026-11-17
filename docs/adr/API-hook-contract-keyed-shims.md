# API-hook-contract-keyed-shims: A POSIX sh shim hands each hook to `nuc hook <name>`, which decides from the git state and answers 0, 1, 2, 3, or 4, the shim runs the hook's check on 3 and the documentation check on 4, and every refusal of a shim ends in its key

- **Status:** 🟢 accepted
- **Date:** 2026-11-17
- **Deciders:** Markus Brigl
- **Applies to:** the `nuc hook` command and its source, the reference shims under `shims/`, the `hook/` rule IDs, the files `nuc hook` writes in the git directory, and the merge driver of the ADR index, and the keys and wording of the shims' own refusals
- **Supersedes:** [API-hook-contract-names-check](API-hook-contract-names-check.md)
## Context

G-2 asks that a hook through the shim refuse or pass exactly as the bash hook of the release does.
G-3 asks the same for the merge driver of the ADR index. G-5 asks that a hook fail closed when
`nuc` is missing or too old. Q-7 forbids `nuc` to run repository content. Three of the four bash
hooks do run repository content, and they must:
- `pre-commit` runs `scripts/check-all.sh --changed` on a commit that concludes a conflicted merge,
  and from template release v0.9.0 `scripts/check-docs.sh` on every other commit off `main`;
- `pre-merge-commit` runs `scripts/check-docs.sh`;
- `pre-push` runs `scripts/check-all.sh --changed` unless the tree is in the record of green
  trees, and records the tree once the checks pass.

So the work is split. `nuc` decides from the git state, and something else runs the scripts. From
v0.9.0, one hook runs one of two checks by the git state, so the answer that the checks run now
has to say which.

The command line gives every command the exit codes 0, 1, and 2 and keeps the standard output for
findings, so "the checks run now" needs a code of its own, which
[API-command-line-with-measure](API-command-line-with-measure.md) gives `nuc hook` alone.
Git hands a hook its standard input once, so a second call cannot read the refs again. The bash
hook decides the green tree before it clears git's variables and runs the checks, and records that
same tree afterwards. A tree computed after the checks could differ, because a check may touch the
working tree, or could be missing, because the variables of a linked worktree are already cleared.

The bash hooks never read `nuc.toml` or the release line, and the fixtures of the template's hook
self-test carry neither. The maintainer decided that a hook works without `nuc.toml`, on defaults.

[SEC-gates-never-run-build](SEC-gates-never-run-build.md) keeps this repository's own `.githooks/`
on the bash, so the shims cannot live there. The template copies them at its switch-over. Git for
Windows runs hooks with the sh it bundles, and Q-4 tests with no bash on the `PATH`.

Every finding of `nuc hook` ends in its key
([API-command-line-with-measure](API-command-line-with-measure.md)), but a shim refuses on its own
in four cases: `nuc` is missing, it answers a code the shim does not know, git names no work tree,
and the check the shim runs is red. Those lines carry no key, and the clause they cite differs from
shim to shim: `NUC-checks-are-bash §6`, `AGENTS.md, section 6`, or `AGENTS.md, section 5`. The
template is to switch its hooks to the shims, and its self-test of the hooks is to assert keys and
exit codes, not wording, so that a normalization's map can run over the tree while the messages of
`nuc` are compiled. A refusal without a key cannot be asserted that way.

## Decision

We will have a reference shim in POSIX sh hand each hook to `nuc hook <name>` with git's arguments
and standard input. `nuc` decides everything the git state decides and answers with exit 0 (pass),
1 (refuse), 2 (cannot decide), 3 (pass so far, the hook's check runs now), or 4 (pass so far,
the documentation check runs now). The shim runs the hook's check on 3 and the documentation check
on 4, and refuses on every code it does not name, each refusal of its own ending in its key.

- **What `nuc hook` decides:**
  - the branch;
  - `MERGE_HEAD`;
  - the refs a push sends, and the deletions among them;
  - the updates of a reference transaction and `origin/main` as the transaction leaves it;
  - the draft notice in the wording of the carried release;
  - the record of green trees;
  - whether the hook runs a check, and which: 3 or 4 only where the bash would run it, that is with
    a work tree, the hook's check script present (a test of existence, which reads nothing, Q-12),
    the `hook/` rule that runs it turned on, and for `pre-push` a tree the record does not hold.
    Otherwise the hook answers 0 where it does not refuse. Only `pre-commit` answers 4, from v0.9.0,
    on a commit off `main` that concludes no merge; on a tree of an earlier release it answers 0
    there, as its bash runs no check.

  It runs no script and no command but git
  ([SEC-git-cleared-environment](SEC-git-cleared-environment.md)).
- **What the shim runs:** on 3, the check of its hook:
  - `scripts/check-all.sh --changed` for `pre-commit` and `pre-push`;
  - `scripts/check-docs.sh` for `pre-merge-commit`.

  On 4, `scripts/check-docs.sh`. A shim of a hook that never answers 4 refuses on it as on any
  code it does not name.

  It runs the check with the variables `git rev-parse --local-env-vars` names unset, as the bash
  does. On red it refuses with a line that cites
  [NUC-checks-are-bash §6](NUC-checks-are-bash.md), the clause that runs the checks before a
  commit, a merge, and a push, and ends in the key of the `hook/` rule that ran the check with the
  reason `red`: `[hook/merge-checks:red]` and `[hook/commit-docs:red]` from `pre-commit`,
  `[hook/merge-docs:red]` from `pre-merge-commit`, `[hook/push-checks:red]` from `pre-push`. A
  check it cannot run, because bash is missing or for any other reason, is red.
- **Exit codes of `nuc hook`:** 0, 1, and 2 as for every command; 3 only from a hook whose bash runs
  a check, and 4 only from `pre-commit`. A panic exits 101. The shim treats every code other than 0,
  3, and 4 as a refusal, 101 and codes a later `nuc` adds included (Q-1). A shim of an earlier
  contract, which knows no 4, refuses where a newer `nuc` answers it, so a commit is never let
  through unchecked.
- **The green tree:**
  - `nuc hook pre-push` decides the tree before it answers: the tree of `HEAD` when the working
    tree is clean, otherwise none.
  - A tree the record holds answers 0, with the skip notice. Where it answers 3, it writes the
    tree it decided to `checks-pending-tree` in the git directory of the worktree, or removes that
    file when there is no tree.
  - Once the checks pass, the shim calls `nuc hook pre-push --record` with git's variables as git
    handed them. That call moves the pending tree into `checks-green-trees` of the common git
    directory, the file and the limit of 50 the bash keeps, and computes no tree of its own.
  - `--record` exits 0 when it recorded and when there was no pending tree, and 2 when it cannot
    write. The shim ignores its exit status and lets the push pass, as the bash ignores a
    failed write of the record: a lost record costs only a second run of the checks.
- **Findings:** a refusal is a finding in the line form of API-command-line-with-measure, its
  key included,
  whose location for a hook is the ref it guards, such as `refs/heads/main`. Its rule ID is in
  the area `hook/`. The check a hook runs is a `hook/` rule as well, so a
  `nuc.toml` that turns it off keeps the shim from running the check. Notices go to the standard
  error and decide nothing; they are no rule and are written whatever `nuc.toml` lists.
- **`nuc.toml` and the release:** how `nuc hook` reads them, with and without `nuc.toml`, is the
  *Hooks* clause of [API-version-coupling-hook-defaults](API-version-coupling-hook-defaults.md),
  and stated there only. The exit codes 3 and 4 and the ref as location are stated in
  API-command-line-with-measure.
- **Merge driver:** `.gitattributes` names the driver `adr-index` for `docs/adr/README.md`, and
  the git configuration `merge.adr-index.driver` names its shim. Git calls the shim with the base,
  ours, theirs, and the three labels, and `nuc` writes the merged file into ours. It exits 0 on a
  merge, 1 on a conflict, and 2 when it cannot decide; the shim passes the code through, and
  git takes every non-zero code as a conflict.
- **The shims:** one per hook of the template and one for the merge driver, under `shims/`, in
  POSIX sh. Each finds `nuc` on the `PATH`. Without it, the shim exits 1 with a message naming how
  to install `nuc` from its releases
  ([OPS-feature-in-hivevm-features](OPS-feature-in-hivevm-features.md)). A too old `nuc` is
  reported by `nuc` itself from `nuc.toml`.
- **The keys of a shim's own refusals:** in the area `shim/`, under the name of the hook, so that
  no `nuc.toml` lists them: `[shim/<hook>:missing]` where `nuc` is missing,
  `[shim/<hook>:unknown]` on a code the shim does not know, and `[shim/<hook>:outside]` where git
  names no work tree. The list of keys beside the contract's test
  holds them with the keys of `nuc`.

**Out of scope:**
- the names of the `hook/` rules, set where each hook was ported;
- the wording of the messages, beyond what the scenario harness compares;
- this repository's and the template's switch-over to the shims;
- the installation of `nuc`.

## Alternatives considered

- **The shim's own refusals without a key** — what the contract decided before, accepted on
  2026-11-15 with every other clause of this one. A test of the hooks could assert those refusals
  only by their wording, and the template's self-test is to assert keys, so this ADR replaced that
  clause.
- **Every refusal of a shim under `hook/<hook>`** — one area for all, but `hook/pre-push` is no
  rule ID, and `nuc.toml` and `nuc explain` know only rule IDs.

- **One code for "the checks run now", the check fixed per hook** — what the first hook contract
  decided, accepted on 2026-11-06 with every other clause of this one. From v0.9.0 `pre-commit`
  runs one of two checks by the git state, and one code cannot say which, so this ADR replaced
  that clause.
- **The shim chooses by `MERGE_HEAD`** — no new code, but the shim would read the git state
  itself, which `nuc` decides and the scenario harness compares through `nuc`, and the choice
  would be split between the two.
- **The name of the check in a file of the git directory** — no new code, but one more write and
  one more file of state between two calls, for a choice an exit code carries.

- **Two calls, no new exit code.** `nuc hook pre-push` refuses or passes, and a second call,
  `--needs-checks`, answers 0 or 1. The shim would have to buffer standard input, which git hands
  over once, and 1 would mean "no" here but "finding" everywhere else.
- **A verdict on the standard output.** No new code would be needed, but the command line keeps
  the standard output for findings, and a shim would have to parse text.
- **`nuc` runs the checks itself.** The shim would be the simplest, but `nuc` would execute
  repository content, against Q-7.
- **The green-tree record left in the shim.** It is not ported and gets no parity, and the shim
  would carry logic that the scenario harness cannot reach through `nuc`.
- **The tree recomputed at `--record`.** No file is needed, but a check that touches the working
  tree, or a call from a linked worktree, records another tree or none.
- **Shims in bash.** Closer to the bash hooks, but they need bash on Windows and break Q-4's "no
  bash on the `PATH`" for every hook, the ones that run no check included.
- **The hooks refuse without `nuc.toml`, as `nuc check` does.** One rule for every command, but
  the template's fixtures and every project without `nuc.toml` would refuse every commit, and
  parity could not hold.

## Sources / Prior art

- [githooks(5)](https://git-scm.com/docs/githooks):
  - the arguments and the standard input of `pre-commit`, `pre-merge-commit`, `pre-push`, and
    `reference-transaction`;
  - that a non-zero exit refuses.
- gitattributes(5), *Defining a custom merge driver*, for the placeholders and that a non-zero exit
  is a conflict:
  <https://git-scm.com/docs/gitattributes#_defining_a_custom_merge_driver>.
- POSIX, *Shell Command Language*, the language of the shims:
  <https://pubs.opengroup.org/onlinepubs/9799919799/utilities/V3_chap02.html>.
- The template's `.githooks/` and `scripts/test-git-hooks.sh` at v0.5.0 through v0.7.0 and at
  v0.8.1, the releases the corpus pins, the oracle: identical in behaviour, except the wording of
  the draft notice from v0.7.0 on; the hooks of v0.8.1 are those of v0.7.0. At v0.9.0, whose
  `pre-commit` runs `scripts/check-docs.sh` on every commit off `main` that concludes no merge.
- pre-commit and lefthook, from memory and not checked here: both put a thin generated script in
  the hooks directory that hands over to an installed binary and fails when it is missing.

## Consequences

- Positive:
  - every decision a hook takes is ported and compared, the record of green trees included
    (G-2, Q-2);
  - `nuc` runs no repository content (Q-7);
  - an unknown or crashing answer stops the hook (Q-1);
  - a commit, merge, or move of `main` that runs no check needs no bash.
- Negative / trade-offs:
  - `nuc hook` has two exit codes more than every other command, as
    API-command-line-with-measure states;
  - the hooks read `nuc.toml` differently from `nuc check`, as API-version-coupling-hook-defaults
    states;
  - the checks a hook runs stay bash until they are ported;
  - a second file in the git directory carries state between two calls, and a push the user
    aborts leaves it behind, which only a later push overwrites.
- Follow-ups: the switch-over of the template's hooks to the shims.

## Enforcement

The scenario harness of [TEST-hook-scenario-parity](TEST-hook-scenario-parity.md) runs every hook
through its shim.

Tests in `tests/hooks.rs`, cited with `Verifies: API-hook-contract-keyed-shims`, check:
- that each shim exits non-zero on exit 1, 2, 101, and an unnamed code from a stand-in `nuc`, and
  on a missing `nuc`, with the install message;
- that each refusal of a shim's own ends in its key, `shim/` for a missing `nuc`, an unknown code,
  and no work tree, and the `hook/` rule of the check for a red one, and that the list of keys
  beside the contract's test holds every key a shim writes;
- that a shim runs its hook's check only on 3 and the documentation check only on 4, that the
  shims of the hooks other than `pre-commit` refuse on 4, and that a red or unrunnable check
  refuses;
- that `nuc hook pre-commit` answers 4 on a commit off `main` that concludes no merge on a tree of
  v0.9.0 or later, and 0 there on a tree of an earlier release;
- that a canary script stays unrun when `nuc hook <name>` is called directly;
- that `--record` records the pending tree, not one computed after a check changed the working
  tree, and that a failed `--record` lets the push pass;
- that `nuc hook pre-commit` without `nuc.toml` refuses a commit on `main` and exits 0 on a
  branch, and with a `nuc.toml` that lists no `hook/` rule exits 0 on `main` and on a conflicted
  merge, without running the check.
