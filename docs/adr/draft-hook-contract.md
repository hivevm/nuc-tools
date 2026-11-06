# ADR-draft-hook-contract: A POSIX sh shim hands each hook to `nuc hook <name>`, which decides from the git state and answers 0, 1, 2, or 3, and the shim runs the project's checks only on 3

- **Status:** 🟡 proposed
- **Date:** 2026-11-06
- **Deciders:** Markus Brigl
- **Applies to:** the `nuc hook` command and its source, the reference shims under `shims/`, the `hook/` rule IDs, the files `nuc hook` writes in the git directory, and the merge driver of the ADR index

## Context

G-2 asks that a hook through the shim refuse or pass exactly as the bash hook of the release does.
G-3 asks the same for the merge driver of the ADR index. G-5 asks that a hook fail closed when
`nuc` is missing or too old. Q-7 forbids `nuc` to run repository content. Three of the four bash
hooks do run repository content, and they must:
- `pre-commit` runs `scripts/check-all.sh --changed` on a commit that concludes a conflicted merge;
- `pre-merge-commit` runs `scripts/check-docs.sh`;
- `pre-push` runs `scripts/check-all.sh --changed` unless the tree is in the record of green
  trees, and records the tree once the checks pass.

So the work is split. `nuc` decides from the git state, and something else runs the scripts.

ADR-0013 gives every command the exit codes 0, 1, and 2 and keeps the standard output for
findings, so `nuc` has no way left to say "the checks run now". Git hands a hook its standard input
once, so a second call cannot read the refs again. The bash hook decides the green tree before it
clears git's variables and runs the checks, and records that same tree afterwards. A tree computed
after the checks could differ, because a check may touch the working tree, or could be missing,
because the variables of a linked worktree are already cleared.

The bash hooks never read `nuc.toml` or the release line, and the fixtures of the template's hook
self-test carry neither. The maintainer decided that a hook works without `nuc.toml`, on defaults.

[ADR-0016](0016-gates-never-run-the-build.md) keeps this repository's own `.githooks/` on the
bash, so the shims cannot live there. The template copies them at its switch-over. Git for Windows
runs hooks with the sh it bundles, and Q-4 tests with no bash on the `PATH`.

## Decision

We will have a reference shim in POSIX sh hand each hook to `nuc hook <name>` with git's arguments
and standard input. `nuc` decides everything the git state decides and answers with exit 0 (pass),
1 (refuse), 2 (could not decide), or 3 (pass so far, the checks run now). The shim runs the
project's check for that hook only on 3 and refuses on every code it does not name.

- **What `nuc hook` decides:**
  - the branch;
  - `MERGE_HEAD`;
  - the refs a push sends, and the deletions among them;
  - the updates of a reference transaction and `origin/main` as the transaction leaves it;
  - the draft notice in the wording of the carried release;
  - the record of green trees;
  - whether the hook's check script exists. Testing that it exists reads nothing (Q-12).

  It runs no script and no command but git ([ADR-0011](0011-git-cli-cleared-environment.md)).
- **What the shim runs:** on 3, the check of its hook:
  - `scripts/check-all.sh --changed` for `pre-commit` and `pre-push`;
  - `scripts/check-docs.sh` for `pre-merge-commit`.

  It runs the check with the variables `git rev-parse --local-env-vars` names unset, as the bash
  does. On red it refuses with the bash hook's message. A check it cannot run, because bash is
  missing or for any other reason, is red.
- **Exit codes of `nuc hook`:** 0, 1, and 2 as for every command; 3 only from a hook whose bash
  runs a check. A panic exits 101. The shim treats every code other than 0 and 3 as a refusal,
  101 and codes a later `nuc` might add included (Q-1).
- **The green tree:**
  - `nuc hook pre-push` decides the tree before it answers: the tree of `HEAD` when the working
    tree is clean, otherwise none.
  - A tree the record holds answers 0, with the skip notice. Otherwise it answers 3 and writes the
    tree it decided to `checks-pending-tree` in the git directory of the worktree, or removes that
    file when there is no tree.
  - Once the checks pass, the shim calls `nuc hook pre-push --record` with git's variables as git
    handed them. That call moves the pending tree into `checks-green-trees` of the common git
    directory, the file and the limit of 50 the bash keeps, and computes no tree of its own.
- **Findings:** a refusal is a finding in the line form of ADR-0013. Its location is the ref the
  hook guards, such as `refs/heads/main`, or `docs/adr/README.md` for the merge driver. Its rule ID
  is in the area `hook/`. Notices go to the standard error and decide nothing.
- **`nuc.toml`:** `nuc hook` works without one, with every `hook/` rule on and no minimum. With
  one, its list of rules governs the hooks as it governs the checks, and its minimum holds (G-5).
- **The release:** a hook reads the release line as `nuc check` does. Where the line or the README
  is missing or names no release, the hook takes the newest release `nuc` knows rather than
  refusing: check 14 gates the line, not the hooks.
- How these two clauses depart from ADR-0014 is that ADR's restatement,
  [ADR-draft-version-coupling-with-hook-defaults](draft-version-coupling-with-hook-defaults.md);
  the exit code 3 is that of ADR-0013,
  [ADR-draft-command-line-contract-with-hook-signal](draft-command-line-contract-with-hook-signal.md).
- **Merge driver:** `.gitattributes` names the driver's shim for `docs/adr/README.md`. Git calls
  it with the base, ours, theirs, and the three labels, and `nuc` writes the merged file into ours.
  It exits 0 on a merge and 1 on a conflict, and git takes any other code as a conflict as well.
- **The shims:** one per hook of the template and one for the merge driver, under `shims/`, in
  POSIX sh. Each finds `nuc` on the `PATH`. Without it, the shim exits 1 with a message naming how
  to install `nuc` from its releases ([ADR-0017](0017-distribution.md)). A too old `nuc` is
  reported by `nuc` itself from `nuc.toml`.

**Out of scope:**
- the names of the `hook/` rules, set by the ticket that ports each hook;
- the wording of the messages, beyond what the scenario harness compares;
- this repository's and the template's switch-over to the shims;
- the installation of `nuc`.

## Alternatives considered

- **Two calls, no new exit code.** `nuc hook pre-push` refuses or passes, and a second call,
  `--needs-checks`, answers 0 or 1. The shim would have to buffer standard input, which git hands
  over once, and 1 would mean "no" here but "finding" everywhere else.
- **A verdict on the standard output.** No new code would be needed, but ADR-0013 keeps the
  standard output for findings, and a shim would have to parse text.
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
- The template's `.githooks/` and `scripts/test-git-hooks.sh` at v0.5.0 through v0.7.0, the
  oracle: identical in behaviour, except the wording of the draft notice at v0.7.0.
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
  - `nuc hook` has one exit code more than every other command, so ADR-0013 is restated;
  - the hooks read `nuc.toml` differently from `nuc check`, so ADR-0014 is restated;
  - the checks a hook runs stay bash until they are ported;
  - a second file in the git directory carries state between two calls, and a push the user
    aborts leaves it behind, which only a later push overwrites.
- Follow-ups:
  - the switch-over of the template's hooks to the shims;
  - the wording of the glossary's *shim*, which places it in `.githooks/`.

## Enforcement

The scenario harness of the hook-scenario parity ADR runs every hook through its shim.

Tests cited with `Verifies: ADR-draft-hook-contract` check:
- that each shim exits non-zero on exit 1, 2, 101, and an unnamed code from a stand-in `nuc`, and
  on a missing `nuc`, with the install message;
- that a shim runs its check only on 3;
- that a canary script stays unrun when `nuc hook <name>` is called directly;
- that `--record` records the pending tree, not one computed after a check changed the working
  tree;
- that `nuc hook` without `nuc.toml` refuses a commit on `main`, and with a `nuc.toml` that lists
  no `hook/` rule passes it.
