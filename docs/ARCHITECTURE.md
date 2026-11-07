# Architecture — nuc-tools

> The system **as it stands today**: the parts it is built from, what each is responsible for, and
> how they fit together. It holds no rule and no decision. It names the ADR behind each structural
> choice; where it disagrees with an accepted ADR, the ADR is right and this document is stale. It
> is updated in the same change as the structure it describes
> ([ADR-0001](adr/0001-agent-governance-model.md)).
>
> **Last design revision:** 2026-11-07, due after 20 changes. The revision that took the count
> moves the date; the number is this project's to set; a sensor counts the changes outside
> `docs/` since the commit that set the date and says when the next is due
> ([ADR-0006](adr/0006-feature-layer.md)).

## Context

What sits outside the system and what crosses its boundary. A diagram earns its place here more
than anywhere else, and none beats one that has stopped being true.

`nuc` runs in a working tree of a derived project, from a human's shell, a reference shim git
runs as a hook, or CI. What crosses its boundary:

- **In:** the command line; the files of the working tree, each resolved inside it; what git
  answers about the tree, git run as a process; the variables CI sets for a pull request, which
  the check of drafts on `main` reads; and for `nuc hook`, the arguments and the standard input
  git hands a hook, the `GIT_DIR` it sets, the run `NUC_HOOK_RUN` the `pre-push` shim names, and
  the three sides git hands the merge driver. The tree is untrusted input.
- **Out:** the findings on the standard output, the summary, the notices of a hook, and every
  refusal on the standard error, and the exit code. `nuc check` writes no file and keeps no
  state; `nuc hook pre-push` keeps the record of green trees in the git directory, and the merge
  driver writes the merged index over ours, after git merged the prose in a scratch directory it
  makes fresh in the git directory and removes.
- **Beside it:** the template's bash scripts and hooks, the oracle the parity harness compares
  `nuc` with, on a corpus of repositories fetched from GitHub; the reference shims, which run the
  project's checks where `nuc hook` answers that they run; the release workflow, which
  publishes the binaries the Action installs; and `hivevm/features`, which is to take each release
  into a Dev Container Feature under `ghcr.io/hivevm/features`, by its own record, and to which
  nothing here writes.

## Building blocks

The parts the system is made of, each with one responsibility, named in the vocabulary of
[`GLOSSARY.md`](GLOSSARY.md). One level deep, deeper only where the size of a part earns it. The
golden path and the test pattern an agent copies from are named here.

- **Rules** — one module each under `src/rules/`, a pure function from the inputs the shell read
  to an outcome: the findings of `src/finding.rs`, a refusal of its own where it cannot decide the
  tree as its bash check would, and notes for the summary. Each has one rule ID, the reason
  `nuc explain` gives, and the inputs it reads; the list of every rule is `rules::ALL`. A rule
  reads no file and runs no process
  ([ADR-0024](adr/0024-rules-return-an-outcome-and-hooks-an-answer.md)).
  The rules of the decision record share one reading of it in `src/adr.rs`, and the rules of
  Markdown one reading of its links in `src/markdown.rs`, both outside `src/rules/`, the way the
  bash's `scripts/lib/adr.sh` and awk passes read them. The byte-level readings of the bash's tools,
  lines as awk, `read`, and `sed` split them, POSIX space and the bytes without it, and the first
  word `read` splits off, are in `src/bash.rs` and nowhere else, which the structural test holds.
  The rules of the area `traceability` share one reading of the criteria of the specification and of
  the identifiers the `Verifies:` markers cite in `src/traceability.rs`, beside `src/adr.rs`, so
  that no rule module calls another. The shell matches path patterns as bash's pathname expansion
  does through `src/glob.rs`. A rule whose bash check changed between template releases, today
  `docs/adr-supersessions`, `docs/check-lists`, `docs/project-layout`, and
  `docs/template-identity`, reads the release the tree carries and checks as that release does
  (ADR-0023). `tests/parity/releases.tsv` names the releases at which each rule's check changed,
  which the parity harness holds to the template's trees and `tests/releases.rs` holds the rules
  to; `docs/template-release` gives its reason there for reading none.
- **Configuration** — `config` reads `nuc.toml` and selects the rules it turns on, or says why
  `nuc` refuses: missing, unreadable, a minimum above the running version, an unknown rule ID
  ([ADR-0023](adr/0023-version-coupling-with-hook-defaults.md)). It holds the one answer to which
  IDs `nuc.toml` may name, those of `rules::ALL` and of `hook::ALL`, and the reason `nuc explain`
  gives for each, so `nuc check`, `nuc hook`, and `nuc explain` ask it alike.
- **Formats** — `src/format.rs` turns what a run of `nuc check` decided into its standard
  output: the lines of ADR-0020, one JSON document, or one SARIF 2.1.0 log, without a crate and
  byte for byte the same for the same findings
  ([ADR-0027](adr/0027-output-formats.md)).
- **Hooks** — `src/hook.rs` decides what each git hook of the template decides, from the git
  state the shell reads in `shell::hook`, which finds each hook in its one table `HOOKS`, and
  holds the `hook/` rules, `hook::ALL`, which `nuc check` knows and runs never. The reference
  shims under `shims/`, in POSIX sh, hand each hook to `nuc hook <name>` and run the project's
  check where it answers 3 ([ADR-0021](adr/0021-hook-contract.md)). A hook's function returns
  an answer, not a rule's outcome
  ([ADR-0024](adr/0024-rules-return-an-outcome-and-hooks-an-answer.md)).
  `pre-commit`, `pre-merge-commit`, `pre-push`, `reference-transaction`, and the merge driver of
  the ADR index are ported;
  `tests/bench/hook-latency.sh` measures `reference-transaction` against Q-5 in `benchmark.yml`,
  on a schedule, never as a gate.
- **Shell** — `src/shell/` reads the command line and the repository, runs git under a cleared
  environment in `shell::git`, calls the rules, prints the findings, and sets the exit code
  ([ADR-0011](adr/0011-git-cli-cleared-environment.md)). Its `Tree`, in `shell::tree`, reads
  every file and lists every directory the rules need, each resolved through one function that
  refuses a path outside the repository, and refuses as well, as not decided, one the bash would
  read differently from its bytes alone. `shell::hook` reads `nuc.toml` and the README through
  the `Tree` as well; the two files of the green-tree record and the sides of a merge, at the
  paths git gives, it reads in the two functions `tests/functional_core.guarded` names for them.
  It writes, renames, creates, and removes only in the functions `tests/functional_core.writers`
  names, each inside the repository or its git directory (Q-8); `git merge-file` writes ours for
  the merge driver.
- **Binary** — `src/main.rs` calls the shell and nothing else.
- **Parity harness** — `scripts/check-parity.sh` runs the bash checks of each repository of the
  corpus, `check-docs.sh` and `check-traceability.sh`, all on template release v0.5.0 or later,
  and `nuc` on the same tree and on each variant of `tests/parity/variants.tsv` that is for it,
  both as a pull request into `main` marked ready, and compares their findings, normalized by
  `tests/parity/normalize.tsv`, and their exit status. A variant edits, deletes, or
  writes one path inside a copy of the tree, never under its `.git`
  ([ADR-0015](adr/0015-parity-harness.md)). Across the template's trees of the
  corpus it compares each rule's bash function with `tests/parity/releases.tsv`; a change to a
  helper the function calls shows in no row. Then `scripts/lib/hook-parity.sh` runs the
  scenarios of `tests/parity/hook-scenarios.sh` through git, with the bash hook of each of the
  template's trees and with the shim and `nuc`, maps the hooks' lines by `tests/parity/hooks.tsv`,
  and holds the template's hook self-test to the scenarios and `tests/parity/hook-cases.tsv`
  ([ADR-0022](adr/0022-hook-scenario-parity.md)).
- **Release tooling** — `release/` builds a binary per target, writes `SHA256SUMS`, checks the
  tag against the crate's version, writes the release commit, and moves the major tag;
  `release/install-nuc.sh` is the installer the Action runs, of which `hivevm/features` carries a
  copy, and refuses a binary whose checksum differs. The tag, the asset names, `SHA256SUMS`, and
  the attestations are the interface `hivevm/features` takes a release by, which changes only with
  a major release ([ADR-0025](adr/0025-feature-in-hivevm-features.md)).
- **Action** — `action.yml` installs the `nuc` of the release its release commit carries; at any
  other commit it installs nothing.
- **The golden path** — the rule `docs/template-release`, check 14 of the template's
  `check-docs.sh`, and the pieces it takes through every layer. A new rule copies it:
  1. a module under `src/rules/` with its `RULE_ID`, an `area/name`, and its `RULE`, listed in
     `rules::ALL` and its ID in `tests/contract.rule-ids`, which keeps it from then on, its unit
     tests the cases of the bash self-test of the check it ports, each named in a `// bash:`
     comment and given its rule in `tests/bash_cases.tsv`, which `tests/bash_cases.rs` holds to the
     comments;
  2. what it needs to read, an `Input` the rule names in its `reads`, a field of the inputs, its
     type in the rule's module unless several rules read it, and the shell's reading of it;
  3. its rows in the normalization table of the parity harness, one per bash message;
  4. a variant in `tests/parity/variants.tsv` that makes it fire;
  5. its row in `tests/parity/releases.tsv`, the function it ports and the releases it changed at.
- **The test pattern** — `tests/check.rs`: a fixture repository built through git by
  `tests/support/`, whose `Fixture::new` turns `core.autocrlf`, background maintenance, and the
  file-system monitor off in the repository's own configuration, `nuc` run on it as a process under
  a cleared environment whose `HOME` is an empty directory of the test process's own
  ([ADR-0026](adr/0026-fixture-repositories-with-a-home-of-their-own.md)), and its exit code and
  output asserted; an ADR identifier, a marker, or a section reference a test writes but does not
  mean comes from `src/testing.rs` or `tests/support/`, since the gate reads the tests as text. The
  structural test `tests/functional_core.rs` holds the shape, and inside the shell lets only the
  functions `tests/functional_core.guarded` names read, list, or resolve a path and only those
  `tests/functional_core.writers` names write. `tests/isolation.rs` runs `nuc check` on a tree of
  bait and without the network (Q-7, Q-11), and `tests/contract.rs` holds the command line's
  contract and the rule IDs that stay (G-6). `tests/hooks.rs` runs the shims through git with the
  built `nuc` or a stand-in, and calls `nuc hook` directly for every shim, on every platform of Q-4
  (G-2, G-3, G-5, Q-1, Q-7 to Q-11). `tests/adr_citations.rs` fails on a citation of a superseded or
  rejected ADR outside `docs/adr/` and `docs/plan/`, but where its list
  `tests/adr_citations.template` names it.

## How it runs

The few paths worth following end to end, such as a request, a job, or a build, and where state
lives between them. Only what a newcomer would otherwise reconstruct from code.

`nuc check` finds the root of the working tree with `git rev-parse --show-toplevel`, reads
`nuc.toml` there and selects the rules it turns on, and reads the inputs those rules name and no
others: `README.md` and the release it carries, `CONTRIBUTING.md`, the record under `docs/adr/`, the
text files git lists, the targets of relative links, `AGENTS.md`, the workflows and scripts, the
Project Layout's paths, the skills and their pointers, the identity files, the attributes that mark
a file generated, the variables CI sets, the specification, and the files that carry a `Verifies:`
marker. It runs each selected rule on what it read and carries out every outcome alike: it prints
the findings on the standard output, as lines, or as the JSON document or the SARIF log
`src/format.rs` builds from them for `--format json` or `sarif`, and exits 0 without a finding and 1
with one; the summary on the standard error carries the rules' notes, as the criteria no marker
cites, which change no exit code. Every cause it can name that keeps it from deciding ends the run
with exit 2 and a message on the standard error: before any rule runs for what it reads, a missing
README, which names no release, an input that resolves outside the repository, one the bash would
read otherwise, a release a rule that checks by it cannot place; after the rules run, before any
finding is printed, for a rule's own refusal and for a finding that names a path of no UTF-8. A
panic ends it with exit 101, non-zero as well (Q-1). Nothing is written and no state is kept between
runs.

`nuc hook pre-commit` runs in the directory git runs the hook in. Without `nuc.toml` every `hook/`
rule is on; with one, it is read as `nuc check` reads it and its list governs. Where git hands it
a git directory other than the one it finds from there, it does not decide. It asks git for the
branch and `MERGE_HEAD`, tests whether `scripts/check-all.sh` exists, and exits 1 with a finding
at `refs/heads/main` on `main`, 3 on the commit that concludes a merge where the check runs, and
0 otherwise. The shim runs the check on 3, with git's local variables unset, and refuses on red.
`nuc hook pre-merge-commit` refuses a merge on `main` and answers 3 on any other merge whose work
tree has `scripts/check-docs.sh`, where its rule is on; its shim runs that check.
`nuc hook pre-push` reads git's refs from the standard input, refuses a push to the remote's
`main`, passes one that only deletes, writes the draft notice in the wording of the README's
release, and passes a clean tree that `checks-green-trees` of the common git directory holds.
Otherwise it writes the tree of a clean `HEAD` to `checks-pending-tree` of the worktree's git
directory, with the run its shim names in `NUC_HOOK_RUN`, and answers 3; its shim runs the checks
and, on green, calls `nuc hook pre-push --record`, which moves that tree into the record and keeps
the last 50, and records nothing where another run's push replaced it. These
two files are the only state `nuc` keeps between runs.
`nuc hook reference-transaction prepared` reads the updates of a transaction from the standard
input and, where one touches `refs/heads/main`, asks git once for `main` and `origin/main` and
refuses a move of `main` to a commit `origin/main` lacks, as the transaction leaves it. It reads
`nuc.toml` in the directory git runs the hook in, the root of the work tree, without asking git,
and checks the git directory git hands it only for a transaction that touches `main`: the hook
runs on every update of a ref, and each git call costs a process (Q-5).
`nuc hook merge-adr-index` is the merge driver `.gitattributes` names for `docs/adr/README.md`,
through `merge.adr-index.driver`: it has `git merge-file` merge the prose of the three sides, every
line but a table row, and their union, writes the union into ours where its prose is the prose
merged on its own, and otherwise the usual conflict markers with git's labels. Its git runs under
the cleared environment of the shell, which passes `PATH` and `HOME`, and `SystemRoot` on Windows,
and nothing else. So it reads the configuration of the repository, of the system, and of the user
at its default places under `HOME`, and none git hands it through the environment, as
`git -c merge.conflictStyle=diff3`, `GIT_CONFIG_GLOBAL`, or `XDG_CONFIG_HOME`.

A release starts when a human pushes a tag `vX.Y.Z` that names the version of `Cargo.toml`.
`release.yml` builds the four targets, writes `SHA256SUMS`, attests each binary and verifies the
attestations, and creates the release as a draft. It writes the release commit on top of the tag,
which carries the checksums and the version for the Action, tags it `pinned/vX.Y.Z`, moves the
major tag `vN` to it unless `vN` names a later release, and only then makes the release public. A
project on `@vN` gets the new release through the moved tag; its Dependabot raises the next major.
`hivevm/features` takes the release from there, by its own record.
