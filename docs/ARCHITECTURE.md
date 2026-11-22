# Architecture — nuc-tools

> The system **as it stands today**: the parts it is built from, what each is responsible for, and
> how they fit together. It holds no rule and no decision. It names the ADR behind each structural
> choice; where it disagrees with an accepted ADR, the ADR is right and this document is stale. It
> is updated in the same change as the structure it describes
> ([NUC-documents-are-fixed](adr/NUC-documents-are-fixed.md)).
>
> **Last design revision:** 2026-11-20, due after 20 changes. The revision that took the count
> moves the date; the number is this project's to set; a sensor counts the changes outside
> `docs/` since the commit that set the date and says when the next is due
> ([NUC-design-is-revised §4](adr/NUC-design-is-revised.md)).

## Context

What sits outside the system and what crosses its boundary. A diagram earns its place here more
than anywhere else, and none beats one that has stopped being true.

`nuc` runs in a working tree of a derived project, from a human's shell, a reference shim git
runs as a hook, or CI. What crosses its boundary:

- **In:** the command line; the files of the working tree, each resolved inside it; what git
  answers about the tree, git run as a process; the variables CI sets for a pull request, which
  the check of drafts on `main` reads; and for `nuc hook`, the arguments and the standard input
  git hands a hook, the `GIT_DIR` it sets, the run `NUC_HOOK_RUN` the `pre-push` shim names, and
  the three sides git hands the merge driver; for `nuc measure`, the transcripts of a harness
  in its directory outside the repository; and for `nuc upgrade`, the template's release tags a
  maintainer fetched into `refs/template/`. The tree, the tags, and the transcripts are untrusted
  input.
- **Out:** the findings on the standard output, or the measures of `nuc measure`; the summary,
  the notices of a hook, and every refusal on the standard error; and the exit code. `nuc check`
  and `nuc measure` write no file and keep no state; `nuc hook pre-push` keeps the record of green
  trees in the git directory, and the merge driver writes the merged index over ours, after git
  merged the prose in a scratch directory it makes fresh in the git directory and removes. `nuc
  upgrade` writes and deletes files of the working tree, and nothing else: no index, no ref, no
  commit.
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

- **Rules** — one module each under `src/rules/`, a pure function from the inputs the shell read to
  an outcome: the findings of `src/finding.rs`, a refusal of its own where it cannot decide the tree
  as its bash check would, and notes for the summary. Each has one rule ID, the reason `nuc explain`
  gives, and the inputs it reads; the list of every rule is `rules::ALL`. A rule reads no file and
  runs no process ([ARCH-rules-outcome-hooks-answer](adr/ARCH-rules-outcome-hooks-answer.md)). The
  rules of the decision record share one reading of it in `src/adr.rs` with the sensors
  binding-adrs, plain-rules, and review-binding. The rules of Markdown share one reading of its
  links in `src/markdown.rs`. Both stand outside `src/rules/`, the way the bash's
  `scripts/lib/adr.sh` and awk passes read them. The byte-level readings of the bash's tools, lines
  as awk, `read`, and `sed` split them, POSIX space and the bytes without it, the first word
  `read` splits off, awk's numbers, exact up to 2^53, and the new side of git's hunk headers, are
  in `src/bash.rs`, and the structural test holds that the names it lists stand nowhere else. The
  rules of the area `traceability` share one reading of the criteria of the specification and of the
  identifiers the `Verifies:` markers cite in `src/traceability.rs`, beside `src/adr.rs`, so that no
  rule module calls another. The shell matches path patterns as bash's pathname expansion does
  through `src/glob.rs`. A rule whose bash check changed between template releases reads the
  release the tree carries and checks as that release does (API-version-coupling-hook-defaults).
  Since v0.9.0 named the ADRs by category and slug, that is most rules of `docs/` and
  `traceability/`. The thresholds the rules share stand in `src/rules/mod.rs`, `rules::RELEASES`
  among them; a threshold only one rule reads stands in its module, as in `docs/adr-supersessions`,
  `docs/check-lists`, and `docs/template-identity`; those of the hooks stand in `src/hook.rs`.
  The rules of `scripts/check-hooks.sh`, `hooks/pin` and `hooks/toml`, share one reading of the
  blob list of the hooks in `src/pin.rs`, beside `pinned_files` of the template's `pin.sh`.
  `hooks/toml` takes `nuc.toml` as `src/config.rs` read it for the run, not as the bash's subset:
  no `nuc.toml`, a file `nuc` cannot read, one without a minimum, a minimum above the running
  `nuc`, or an unknown rule ID ends the run before it.
  The blob of a hook is git's, `git hash-object` as the bash runs it. A hook whose attributes name
  a filter is not decided, since git would run the filter's driver (Q-7); so is a file
  `docs/base-blobs` hashes under an active filter, the `NUC-` files of the record, tracked or not,
  and `AGENTS.md`
  ([SEC-repository-commands-neutralized-or-refused](adr/SEC-repository-commands-neutralized-or-refused.md)).
  The rules of `scripts/check-devcontainer.sh`, one per rule ID of its keys under `devcontainer/`,
  share one reading of the Dev Container's files in `src/devcontainer.rs`: text without its
  full-line `//` comments, the Features by the depth of the braces, and the value of a top-level
  key up to the next one, as the bash's awk reads them, never as JSON. A file of the Dev Container
  with a NUL byte is not decided. Before v0.8.0 the bash pipes `devcontainer.json` into `grep -q`
  for the socket, so a match in one longer than a pipe holds is not decided either.
  `src/adr.rs` reads the record in each of its two forms, numbered and named by category, through
  pairs of functions, and each rule, and the shell for the inputs, chooses one by the release; a
  sensor passes whether its release names the ADRs by category, by the release its bash changed.
  `tests/parity/releases.tsv` names the releases at which each
  rule's check changed, which the parity harness holds to the template's trees and
  `tests/releases.rs` holds the rules to; a rule that checks some or every release alike although
  its bash changed, as `docs/template-release` does, gives its reason there, after a field that
  names the releases the reason covers, or `-` for all.
- **Configuration** — `config` reads `nuc.toml` and selects the rules it turns on, or says why `nuc`
  refuses: missing, unreadable, a minimum above the running version, an unknown rule ID
  ([API-version-coupling-hook-defaults](adr/API-version-coupling-hook-defaults.md)), each refusal
  ending in its key `[config/toml:<reason>]`
  ([API-config-refusals-keyed](adr/API-config-refusals-keyed.md)). It holds the
  one answer to which IDs `nuc.toml` may name, those of `rules::ALL` and of `hook::ALL`, and the
  reason `nuc explain` gives for each, so `nuc check`, `nuc hook`, and `nuc explain` ask it alike.
- **Formats** — `src/format.rs` turns what a run of `nuc check` decided into its standard output:
  the lines of API-command-line-with-sense, one JSON document, or one SARIF 2.1.0 log, without
  a crate and byte for byte the same for the same findings
  ([API-output-formats-carry-reason](adr/API-output-formats-carry-reason.md)).
- **Hooks** — `src/hook.rs` decides what each git hook of the template decides, from the git state
  the shell reads in `shell::hook`, which finds each hook in its one table `HOOKS`, and holds the
  `hook/` rules, `hook::ALL`, which `nuc check` knows and runs never. The reference shims under
  `shims/`, in POSIX sh, hand each hook to `nuc hook <name>` and run the project's check where it
  answers 3, and the documentation check where `pre-commit` answers 4; each refusal of a shim's
  own ends in its key, `shim/<hook>` or the `hook/` rule of a red check
  ([API-hook-contract-keyed-shims](adr/API-hook-contract-keyed-shims.md)). Each decides as the
  release the tree carries, and as the newest release `nuc` knows where the README names none. A
  hook's function returns an answer, not a rule's outcome
  ([ARCH-rules-outcome-hooks-answer](adr/ARCH-rules-outcome-hooks-answer.md)).
  `pre-commit`, `pre-merge-commit`, `pre-push`, `reference-transaction`, and the merge driver of the
  ADR index are ported; `tests/bench/hook-latency.sh` measures `reference-transaction` against Q-5
  in `benchmark.yml`, on a schedule, never as a gate.
- **Shell** — `src/shell/` reads the command line and the repository, runs git under a cleared
  environment in `shell::git`, calls the rules, writes the findings in the format `src/format.rs`
  builds, and sets the exit code. Every call opens with `--no-lazy-fetch` where git knows it, from
  2.45.0 on, and an older git runs only on a repository that names no promisor remote;
  `git::blob_at` reads a blob at a revision, telling a path the commit lacks from an object a
  partial clone lacks, which is not decided
  ([SEC-git-cleared-without-lazy-fetch](adr/SEC-git-cleared-without-lazy-fetch.md)); and
  `git::diff` starts every diff, `diff-tree`, and `log -G` with `--no-ext-diff` and
  `--no-textconv`
  ([SEC-repository-commands-neutralized-or-refused](adr/SEC-repository-commands-neutralized-or-refused.md));
  and `git::environment_driver` reads, once a run, whether the caller's environment names a filter,
  a diff driver, or an external diff that the cleared git does not see: `nuc check` and every
  sensor are then not decided, and `nuc hook pre-push` runs the checks.
  Its `Tree`, in `shell::tree`, reads every file and lists every directory the rules need, each
  resolved through one function that refuses a path outside the repository, and refuses as well, as
  not decided, one the bash would read differently from its bytes alone. `shell::hook` reads
  `nuc.toml` and the README through the `Tree` as well; the two files of the green-tree record and
  the sides of a merge, at the paths git gives, it reads in the two functions
  `tests/functional_core.guarded` names for them. It writes, renames, creates, and removes only in
  the functions `tests/functional_core.writers` names, each inside the repository or its git
  directory (Q-8); `git merge-file` writes ours for the merge driver.
- **Measure** — `src/measure.rs` counts the tokens of `nuc measure` from the requests an adapter
  per harness reads out of its transcripts, `src/measure/claude_code.rs` the one for Claude Code,
  each request once at the largest usage its lines report, selected by branch and time, and
  breaks them down by session, subagent, and tool, or drafts a template finding that names no
  session, subagent, branch, path, or MCP server; its counting knows no harness, which `Harness`
  hands to the adapter, and it reads nothing.
  `shell::measure` finds the harness's directory and reads its transcripts in the two functions
  `tests/functional_core.guarded` names for it, the one read outside the repository, following no
  symbolic link
  ([API-measure-reads-harness-transcripts](adr/API-measure-reads-harness-transcripts.md)).
- **Sensors** — `src/sense.rs` lists the sensors of the template `nuc sense` ports, `sense::ALL`,
  each a module beside it, as `src/sense/revision_due.rs`: a function to the report the bash sensor
  of the tree's release writes on the standard output, byte for byte, or a refusal keyed
  `sense/<name>:<reason>`. Seven of them take only the values the shell gathered: binding-adrs
  takes the file and the diff of each path `binding_adrs::checked_paths` names, which the shell
  reads before the report. review-binding asks git through a trait, below, since each of its
  questions depends on the answer before; its core reads no file and runs no process. The *Core*
  of [API-sensor-cores-ask-the-shell](adr/API-sensor-cores-ask-the-shell.md) allows that where
  gathering up front would read more than the bash and no value of the core can tell the shell
  what to read; `tests/functional_core.rs` holds that only the shell implements a trait of the
  sensors' core. The sensors that read a change share one resolution of it, as the bash's share
  `resolve_change`: `src/sense/change.rs` reads the base and the head they
  were given, and `resolve_change` in `shell::sense` asks git for the commits; a change without a
  head ends in the working tree, its untracked files included. Their diffs against the working tree
  run with every filter driver of the repository neutralized and look into no submodule's working
  tree (Q-7,
  [SEC-repository-commands-neutralized-or-refused](adr/SEC-repository-commands-neutralized-or-refused.md)).
  Where that could change what they report, they refuse: plain-rules a file of rules under a filter
  and a rename a filter could pair otherwise, binding-adrs a path under a filter whose stat differs
  from the index or that is racily clean, its modification time no older than the index's in whole
  seconds, and any submodule checked out. A path whose hunks decide a report and that the
  bash's git reads through a textconv driver or a diff command the configuration sets, or through
  `diff.external`, is refused as well: in tests-kept every path its diff does not leave out, in
  plain-rules a file of rules and the path it was renamed from, in binding-adrs a path named with
  checks, and in revision-due the overview under a textconv driver, the one driver its `log -G`
  runs. A submodule the change moves where `diff.submodule` reads `diff` or `log` is refused in
  tests-kept and, where checks name it, in binding-adrs.
  review-binding asks git what each answer leads it to, as its bash does, so its core asks through
  the trait `review_binding::Repo`, which `shell::sense` answers with git; its diffs compare commits
  and read no working tree, and its hand-off is a file named from where `nuc` runs that must resolve
  inside the repository. `shell::sense` reads `nuc.toml` as `nuc check` does, the README's release
  line, the files of the tree, and git as the bash asks it, and writes the report. A tree whose
  release has no such sensor is not decided. The template's sensors that need a process other than
  git or the network are none of them
  ([API-sensor-cores-ask-the-shell](adr/API-sensor-cores-ask-the-shell.md)).
- **Upgrade** — `src/upgrade.rs` decides the take-up of a template release
  ([API-upgrade-takes-up-by-blob](adr/API-upgrade-takes-up-by-blob.md)): from the README the old and
  the new release, or a refusal keyed `upgrade/run:<reason>`; from the trees of both and of `HEAD`,
  each path with its kind and object, a plan of writes, deletions, and judgments, the findings of
  `upgrade::ALL`; the plan settled against what the working tree holds at each path and each
  directory above it, read by the shell without following a link, with each entry below a directory
  at a path to write, listed the same way; the guards of a path, as git's `verify_path` refuses it
  with `core.protectNTFS` and `core.protectHFS` on; the release line spliced into the README's
  bytes; each line of a file outside the base that names a `NUC-` ADR the new release lacks, read as
  the rule `docs/adr-refs` reads a reference; and whether the take-up skips a release.
  `shell::upgrade` reads the trees and the blobs through git, refuses before the first write, and
  writes the plan in `write_plan`, the one writer of the working tree.
- **Binary** — `src/main.rs` calls the shell and nothing else.
- **Parity harness** — `scripts/check-parity.sh` runs the bash checks of each repository of the
  corpus, `check-docs.sh`, `check-traceability.sh`, and `check-devcontainer.sh`, from v0.10.0 on
  `check-shell.sh`, and from v0.12.0 on `check-hooks.sh`, all on template release v0.5.0 or later,
  and `nuc` on the same tree and on each variant of `tests/parity/variants.tsv` that is for it, both
  as a pull request into `main` marked ready, and compares their findings, normalized by
  `tests/parity/normalize.tsv`, and their exit status. The `nuc.toml` it writes for `nuc` turns on
  every rule the table maps to, and keeps the minimum and the `hook/` rules of the tree's own, which
  `check-hooks.sh` reads as well. A variant edits, deletes, or writes one path inside a copy of the
  tree, never under its `.git` ([TEST-parity-harness](adr/TEST-parity-harness.md)). Across the
  template's trees of the corpus it holds to `tests/parity/releases.tsv` each rule's bash function,
  and each sensor's script together with the libraries it sources; a change to a helper the
  function calls shows in no row. Then
  `scripts/lib/hook-parity.sh` runs the scenarios of `tests/parity/hook-scenarios.sh` through git,
  with the bash hook of each of the template's trees and with the shim and `nuc`, maps the hooks'
  lines by `tests/parity/hooks.tsv`, and holds the template's hook self-test to the scenarios and
  `tests/parity/hook-cases.tsv`
  ([TEST-hook-scenarios-on-bash-hooks](adr/TEST-hook-scenarios-on-bash-hooks.md)).
  `scripts/lib/sensor-parity.sh` runs each sensor of `tests/parity/sensors.tsv` with the bash and
  with `nuc sense` on each tree and variant whose release has it, one whose README names no
  release with the bash sensor of the newest template tree, and on the git states of
  `tests/parity/sensor-scenarios.sh` built for each of the template's trees, both sides run in
  the tree or the fixture, and compares the standard output byte for byte and the exit status; an
  exit that differs stands in `tests/parity/sensor-exits.tsv` with its cause
  ([API-sensor-cores-ask-the-shell](adr/API-sensor-cores-ask-the-shell.md)). A
  tree whose blob list `.githooks/nuc.blobs` pins a hook, as the template's trees do from v0.12.0
  on, carries nuc's own shims and no bash hook. The scenarios and their coverage leave it out,
  whatever release it names, and its checks are compared as any tree's. They also leave out a tree
  later than the bound `HOOKS_PORTED`, v0.11.1, and the scenario of the defaults runs on the newest
  tree left in. Before the comparisons it holds each pin of `tests/tables.blobs` to the template's
  tag of its release. With `--shard <k>/<n>` a run makes a contiguous slice of the comparisons,
  tree by tree in the order of the corpus, each tree as pinned first and then its variants. The
  slices are of about equal cost by the `cost` of each tree in the corpus, the seconds one of its
  comparisons takes, measured with four jobs. The cost of the parts that read all the template's
  trees at once, which the last shard alone runs, is appended at the end of the sequence. A slice
  that starts inside a tree also compares that tree as pinned, since its variants are measured
  against it, but only the slice that holds that comparison reports it. The deal reads the corpus
  and the variants alone, so every shard deals the same, a refused or unfetched tree included.
  Every shard reports such a tree, and a variant that names a tree the corpus lacks, and fails on
  it. CI runs six shards, one required check run each, since the comparisons are bound by the
  processor.
- **Table runner** — `tests/tables.rs` builds the tree of each case of the template's fixture
  tables, copied under `tests/tables/` from the newest release `nuc` ports and pinned by their
  blobs in `tests/tables.blobs`, runs `nuc check` on it, or `nuc sense plain-rules` for the
  sensor's table, and compares the exit and the keys and subjects each case expects; a case that
  does not agree stands in `tests/tables.cases`, pending on the ticket that ports it, not ported,
  or decided by an ADR in force
  ([TEST-template-table-cases](adr/TEST-template-table-cases.md)).
- **Release tooling** — `release/` builds a binary per target, writes `SHA256SUMS`, checks the
  tag against the crate's version, writes the release commit, and moves the major tag;
  `release/install-nuc.sh` is the installer the Action runs, of which `hivevm/features` carries a
  copy, and refuses a binary whose checksum differs. The tag, the asset names, `SHA256SUMS`, and
  the attestations are the interface `hivevm/features` takes a release by, which changes only with
  a major release ([OPS-feature-in-hivevm-features](adr/OPS-feature-in-hivevm-features.md)).
- **Action** — `action.yml` installs the `nuc` of the release its release commit carries; at any
  other commit it installs nothing.
- **Gates** — this repository's own gates are the template's bash checks under `scripts/`, which
  `scripts/check-all.sh` runs, and the hooks under `.githooks/`. The hooks are the reference shims
  of the release that `.githooks/nuc.blobs` pins, and the template's `scripts/check-hooks.sh`
  fails on a hook that differs from its blob. The hooks run a released `nuc`, never the one being
  built ([SEC-gates-run-released-nuc](adr/SEC-gates-run-released-nuc.md)). In CI each job that
  runs a hook installs it by `release/install-nuc.sh` from the `SHA256SUMS` and `VERSION` of one
  release committed under `release/gate/`. The Dev Container installs it by the Feature
  `ghcr.io/hivevm/features/nuc`, which its lock pins. Two checks are this project's own:
  - `scripts/check-gates.sh` fails on each of these:
    - a line of a gate that would run a build of the working tree;
    - `nuc` run outside the hooks;
    - a job of `checks.yml` that runs a hook without first installing `nuc` from `release/gate/`;
    - a `minimum` of `nuc.toml` newer than `release/gate/VERSION`.
  - `scripts/check-adr-text.sh` fails on a commit that changes a project ADR, accepted at its
    parent or flipped to accepted, beyond its Status and Date lines, unless the commit message
    carries an `ADR map:` line. The `NUC-` ADRs are left to `docs/adr/NUC.blobs`.
- **The golden path** — the rule `docs/template-release` of the template's `check-docs.sh`, and
  the pieces it takes through every layer. A new rule copies it:
  1. a module under `src/rules/` with its `RULE_ID`, an `area/name`, and its `RULE`, listed in
     `rules::ALL`, its ID in `tests/contract.rule-ids` and its keys in `tests/contract.rule-keys`,
     which keep them from then on, its unit tests the cases of the bash self-test of the check it
     ports, each named in a `// bash:` comment and given its rule in `tests/bash_cases.tsv`, which
     `tests/bash_cases.rs` holds to the comments, and from template release v0.9.1 the cases of the
     fixture tables, which `tests/tables.rs` runs;
  2. what it needs to read, an `Input` the rule names in its `reads`, a field of the inputs, its
     type in the rule's module unless several rules read it, and the shell's reading of it;
  3. its rows in the normalization table of the parity harness, one per bash message;
  4. a variant in `tests/parity/variants.tsv` that makes it fire;
  5. its row in `tests/parity/releases.tsv`, the function it ports and the releases it changed at.
- **The test pattern** — `tests/check.rs`: a fixture repository built through git by
  `tests/support/`, whose `Fixture::new` turns `core.autocrlf`, background maintenance, and the
  file-system monitor off in the repository's own configuration, `nuc` run on it as a process under
  a cleared environment whose `HOME` is an empty directory of the test process's own
  ([TEST-fixtures-with-own-home](adr/TEST-fixtures-with-own-home.md)), and its exit code and
  output asserted; an ADR identifier, a marker, or a section reference a test writes but does not
  mean comes from `src/testing.rs` or `tests/support/`, since the gate reads the tests as text. A
  test that reads this repository's own tree takes its root from `tests/support/sources.rs`, which
  fails where the test binary was built from another checkout than the one it runs in. The
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
text files git lists, from v0.12.0 on without the hooks `.githooks/nuc.blobs` pins, the targets of
relative links, `AGENTS.md`, the workflows and scripts, the
Project Layout's paths, the skills and their pointers, the identity files, the attributes that mark
a file generated, the variables CI sets, the specification, the files that carry a `Verifies:`
marker, the sensors and their self-tests, the rule keys of `scripts/check-docs.sh`, the blobs of the
base, the modes git's index holds for the scripts and skills, from v0.12.0 on the blob list of the
hooks and each hook's blob, and through git the README at the
merge base with `main` and, from v0.9.0 on, the ADR files the first-parent history deleted. It runs
each selected rule on what it read and carries out every outcome alike: it prints the findings on
the standard output, as lines, or as the JSON document or the SARIF log `src/format.rs` builds from
them for `--format json` or `sarif`, and exits 0 without a finding and 1 with one; the summary on
the standard error carries the rules' notes, as the criteria no marker cites, which change no exit
code. Every cause it can name that keeps it from deciding ends the run with exit 2 and a message on
the standard error: before any rule runs for what it reads, a missing README, which names no
release, an input that resolves outside the repository, one the bash would read otherwise, a release
a rule that checks by it cannot place; after the rules run, before any finding is printed, for a
rule's own refusal, for a finding a rule reports under another rule's ID, and for a finding that
names a path of no UTF-8. A panic ends it with exit 101, non-zero as well (Q-1). Nothing is written
and no state is kept between runs.

`nuc hook pre-commit` runs in the directory git runs the hook in. Without `nuc.toml` every `hook/`
rule is on; with one, it is read as `nuc check` reads it and its list governs. Where git hands it a
git directory other than the one it finds from there, it does not decide. It asks git for the branch
and `MERGE_HEAD`, tests whether `scripts/check-all.sh` and `scripts/check-docs.sh` exist, reads the
release of the README, and exits 1 with a finding at `refs/heads/main` on `main`; from v0.11.0, 1 as
well on the commit that concludes a merge on a `feat/` branch; 3 on the commit that concludes any
other merge where the check runs; from v0.9.0, 4 on any other commit where `scripts/check-docs.sh`
exists; and 0 otherwise. The shim runs the check on 3 and the documentation check on 4, with git's
local variables unset, and refuses on red.
`nuc hook pre-merge-commit` refuses a merge on `main`, from v0.11.0 one on a `feat/` branch as well,
and answers 3 on any other merge whose work tree has `scripts/check-docs.sh`, where its rule is on;
its shim runs that check.
`nuc hook pre-push` reads git's refs from the standard input, refuses a push to the remote's `main`,
passes one that only deletes, writes the draft notice in the wording of the README's release on a
tree before v0.9.0, and passes a clean tree that `checks-green-trees` of the common git directory
holds. Otherwise it writes the tree of a clean `HEAD` to `checks-pending-tree` of the worktree's git
directory, with the run its shim names in `NUC_HOOK_RUN`, and answers 3; its shim runs the checks
and, on green, calls `nuc hook pre-push --record`, which moves that tree into the record and keeps
the last 50, and records nothing where another run's push replaced it. These two files are the only
state `nuc` keeps between runs. A tree is clean where git's status, every active filter
neutralized, lists nothing, and none of these holds: the caller's environment names a driver, a
submodule is checked out, asked before any git looks into it, an active filter's name cannot be
neutralized, or a tracked path under an active filter is stat-dirty or racily clean
([SEC-repository-commands-neutralized-or-refused](adr/SEC-repository-commands-neutralized-or-refused.md)).
`nuc hook reference-transaction prepared` reads the updates of a transaction from the standard input
and, where one touches `refs/heads/main`, asks git once for `main` and `origin/main` and refuses a
move of `main` to a commit `origin/main` lacks, as the transaction leaves it. It reads `nuc.toml` in
the directory git runs the hook in, the root of the work tree, without asking git, and checks the
git directory git hands it only for a transaction that touches `main`: the hook runs on every update
of a ref, and each git call costs a process (Q-5).
`nuc hook merge-adr-index` is the merge driver `.gitattributes` names for `docs/adr/README.md`,
through `merge.adr-index.driver`: it has `git merge-file` merge the prose of the three sides, every
line but a table row, and their union, writes the union into ours where its prose is the prose
merged on its own, and otherwise the usual conflict markers with git's labels. Its git runs under
the cleared environment of the shell, which passes `PATH` and `HOME`, and `SystemRoot` on Windows,
and nothing else. So it reads the configuration of the repository, of the system, and of the user at
its default places under `HOME`, and none git hands it through the environment, as `git -c
merge.conflictStyle=diff3`, `GIT_CONFIG_GLOBAL`, or `XDG_CONFIG_HOME`.

`nuc measure` finds the root of the working tree and its current branch through git, takes the
harness's directory for that tree, or the one `--transcripts` names, and lists it without following
a symbolic link: each `<session>.jsonl`, and each `agent-<id>.jsonl` with its `agent-<id>.meta.json`
under `<session>/subagents/`. The adapter of the harness turns each transcript's bytes into requests
and tool outputs, and the counting selects them by branch and time, counts each request once, and
builds the line `tokens <n>`, after the breakdown and the draft of a finding where they are asked
for; for the draft it reads the README through the `Tree`. The whole output is built before it is
printed, so that a refusal, exit 2 with its cause on the standard error, leaves the standard output
empty. It writes nothing and keeps no state.

`nuc upgrade <tag>` finds the root of the working tree, reads `nuc.toml` as `nuc check` does,
refuses on `main`, and refuses a tree that is not clean as `nuc hook pre-push` tells it, an
untracked file included. It reads the README's bytes and the release its line names, the old
release, and takes the argument as the new one. It checks that each is an annotated tag of its name
under `refs/template/`, and that the new release's `nuc.toml` asks for no newer `nuc`. It lists the
trees of both tags and of `HEAD` with `git ls-tree`, and has the core plan the take-up. Before
anything is written, it refuses a path the guards refuse, reads what the working tree holds at each
path of the plan and above it, has the core settle the plan against that, and reads, in one `git
cat-file --batch`, every blob the plan writes and every file outside the base as the plan leaves it,
whose citations of a `NUC-` ADR the new release lacks it lists. Then it deletes, removes the
directories the deletions leave empty, writes each file as a new one with the blob's bytes and mode,
and splices the release line into the README. It prints the judgments as findings and exits 1 where
there is one, and 0 otherwise; every refusal exits 2 with nothing written, and a write that fails
after the first names what was written. It keeps no state.

A release starts when a human pushes a tag `vX.Y.Z` that names the version of `Cargo.toml`.
`release.yml` builds the four targets, writes `SHA256SUMS`, attests each binary and verifies the
attestations, and creates the release as a draft. It writes the release commit on top of the tag,
which carries the checksums and the version for the Action, tags it `pinned/vX.Y.Z`, moves the
major tag `vN` to it unless `vN` names a later release, and only then makes the release public. A
project on `@vN` gets the new release through the moved tag; its Dependabot raises the next major.
`hivevm/features` takes the release from there, by its own record.
