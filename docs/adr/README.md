# Architecture Decision Records

This directory contains all Architecture Decision Records (ADRs) for this project. Accepted ADRs are
**binding** for humans and coding agents alike (see [`AGENTS.md`](../../AGENTS.md) in the repository
root). ADRs derive from the specification in [`docs/SPECIFICATION.md`](../SPECIFICATION.md).

## Mechanics

The rules live in three `NUC-` ADRs.
[NUC-decisions-are-proposed-then-accepted](NUC-decisions-are-proposed-then-accepted.md) decides
when an ADR is written, how it is developed, who flips its status, and that an accepted one is
immutable. [NUC-adrs-are-named-by-category](NUC-adrs-are-named-by-category.md) decides how it is
named, cited, superseded, and indexed. [NUC-base-is-fixed](NUC-base-is-fixed.md) decides the fixed
base. This section carries only the mechanics of the record;
[`scripts/check-docs.sh`](../../scripts/check-docs.sh) names in its header which of them it
verifies.

- **File.** Copy [`template.md`](template.md) to `<CAT>-<slug>.md` and head it
  `# <CAT>-<slug>: <title>`, the category from the [list](#categories). The author picks the
  identifier when creating the file, and it never changes or returns
  ([NUC-adrs-are-named-by-category §1](NUC-adrs-are-named-by-category.md)).
- **Wording.** The *Decision* and the *Enforcement* follow the writing rule of
  [`AGENTS.md` §4](../../AGENTS.md#4-working-style).
- **Index.** The tables below list the project's ADRs, grouped by category, and no `NUC-` ADR
  ([NUC-adrs-are-named-by-category §5](NUC-adrs-are-named-by-category.md)). They change in the same
  commit as the ADR they describe, for additions, supersessions, and status flips alike. The status
  is shown via the legend's bullet and the ADR's `Applies to` header is mirrored verbatim in its own
  column. The index routes a reader from a change to the decisions that bind it, so a row says what
  its ADR governs without the file being opened. Two tables keep what a session reads apart from
  what it looks up: an accepted or proposed ADR sits under **Binding**, a superseded or rejected one
  under **Superseded and rejected**, and a status flip moves the row.
- **Identifiers are permanent once on `main`.** A normalization keeps them, and only a merged or
  split ADR takes a new one
  ([NUC-decisions-are-proposed-then-accepted §8](NUC-decisions-are-proposed-then-accepted.md)).
  Between two normalizations, never rename, delete, or merge an ADR there: other ADRs, commits
  (`Implements <CAT>-<slug>`), and code cite it. A superseded ADR keeps its file and its body. The
  superseding ADR's `Supersedes` field names it from the proposal on; its `Status` line flips to
  `⚪ superseded by <CAT>-<slug>` in the change where a human accepts the successor
  ([NUC-adrs-are-named-by-category §4](NUC-adrs-are-named-by-category.md)), its index row moving
  with it.
- **Between normalizations, sprawl is curbed by supersession, never by editing.** One ADR may
  supersede several whose decisions have grown into one. Its `Supersedes` field names each, and
  each flips its status when the superseding ADR is accepted, so the set an agent reads shrinks
  while every identifier and every body stays.
- **A normalization restates the record**
  ([NUC-decisions-are-proposed-then-accepted §8](NUC-decisions-are-proposed-then-accepted.md)).
  - **Start.** The tree has no proposed ADR, no feature spec or ticket under `docs/plan/`, and
    green checks.
  - **Steps.** Three, in this order on one branch, each handed to the human and confirmed
    before the next begins: the restated specification, whose `G-n` and `Q-n` run from 1; the
    supersessions it calls for; and the restated record. The record begins once the human has
    decided each supersession. One pull request lands them, merged by a human.
  - **Identifiers.** An ADR restated alone keeps its identifier; a merged or split one takes a
    new one. The `NUC-` files stay as the release brought them.
  - **Clauses.** Every clause of an old ADR lives on in one new ADR. The hand-off of the
    record lists each clause with the new ADR that carries it.
  - **Header.** A new ADR keeps the status of the ADRs it restates. Its `Deciders` name all of
    theirs, its `Date` is the latest of theirs, and it has no `Supersedes` field. In an
    adoption, an ADR the project had accepted stays accepted. A decision found outside any
    record enters as proposed.
  - **Dead ADRs.** Superseded and rejected ADRs are deleted. What one of them chose, and why it
    did not stand, becomes an entry under *Alternatives considered* in the new ADR that governs
    its topic.
  - **References.** Each step moves its references with a map, old to new, applied by the map
    script of the `normalize` skill. The map stands in the hand-off, and in the commit body under a
    line that reads `ADR map:` alone, and in no document. A commit older than the normalization
    cites the identifiers it had then.
- **Cite only what exists.** Every citation, `<CAT>-<slug>` or a clause `<CAT>-<slug> §n`, names a
  file already in this directory. An anticipated follow-up is described by topic ("a follow-up ADR
  on session storage"). What counts as a citation, and its form in Markdown, is
  [NUC-adrs-are-named-by-category §6](NUC-adrs-are-named-by-category.md).

## Index

**The base:** the template's `NUC-` ADRs in this directory, each named with its blob in
[`NUC.blobs`](NUC.blobs), as `AGENTS.md` is. They bind like any ADR in force, and a project
extends or narrows a clause in an ADR of its own, never edits one
([NUC-base-is-fixed §1](NUC-base-is-fixed.md)).

### Categories

The category of an identifier says what its decision is about
([NUC-adrs-are-named-by-category §2](NUC-adrs-are-named-by-category.md)). The fixed ones:

- **NUC** — the template's own ADRs, reserved to the template repository.
- **SEC** — security.
- **OPS** — build, release, deployment, and operation.
- **API** — interfaces the system offers and consumes, adapters among them.

The project's own follow here, each in the same form with one line on what it holds:

- **DEV** — how the project is worked: agents, sessions, review.
- **ARCH** — the structure of the code: crates, modules, and what each returns.
- **TEST** — how the system is tested: fixture repositories and the parity harnesses.

**Status legend:** 🟢 accepted · 🟡 proposed · 🔴 rejected · ⚪ superseded

### Binding

The decisions in force: accepted, and proposed ones that bind the work implementing them. This
is the table a session reads.

| ADR | Title | Applies to | Status |
|-----|-------|------------|--------|
| [SEC-git-cleared-environment](SEC-git-cleared-environment.md) | The shell spawns the git CLI with a cleared environment, an allowlist passed through, the repository named by `-C`, and command-running configuration neutralized | every spawn of git in `src/shell/`, the environment and arguments it passes, and the list of neutralized configuration keys kept beside its test | 🟢 accepted |
| [SEC-git-cleared-without-lazy-fetch](SEC-git-cleared-without-lazy-fetch.md) | The shell spawns the git CLI with a cleared environment, an allowlist passed through, no lazy fetch, the repository named by `-C`, and command-running configuration neutralized | every spawn of git in `src/shell/`, the environment and arguments it passes, the list of neutralized configuration keys kept beside its test, the git versions that block the lazy fetch, and every read of an object a partial clone may lack | 🟡 proposed |
| [SEC-gates-run-released-nuc](SEC-gates-run-released-nuc.md) | This repository's gates never run the `nuc` being built: the checks stay the template's bash, and the hooks run a released `nuc` that CI installs from checksums committed here | `scripts/check-all.sh`, the hooks under `.githooks/`, the jobs of `.github/workflows/checks.yml`, the checksums and the version under `release/gate/`, `release/install-nuc.sh` as a gate runs it, the `minimum` of `nuc.toml`, and every place they name the tool that runs a check | 🟢 accepted |
| [SEC-actions-pinned-by-sha](SEC-actions-pinned-by-sha.md) | The workflows reference every action by full commit SHA with a version comment, kept current by Dependabot | every `uses:` line under `.github/workflows/`, `.github/dependabot.yml`, and the run of `scripts/sensor-action-refs.sh` in `scripts/check-all.sh` and the `sensors` job | 🟢 accepted |
| [SEC-no-stored-secrets](SEC-no-stored-secrets.md) | The workflows use no stored secret, only `GITHUB_TOKEN` and OIDC with permissions set per job | every workflow under `.github/workflows/`, its `permissions`, and every reference to `secrets.` in them | 🟢 accepted |
| [OPS-rust-stable-behind-scripts](OPS-rust-stable-behind-scripts.md) | Rust stable pinned in `rust-toolchain.toml`, cargo's build, test, fmt, and clippy behind `scripts/check-build.sh` on every platform of Q-4 as required checks, and the dependency audit behind `scripts/check-deps.sh` | `rust-toolchain.toml`, `Cargo.toml`, `Cargo.lock`, `deny.toml`, `scripts/check-build.sh`, `scripts/check-deps.sh`, their jobs in `.github/workflows/checks.yml`, the `portability` job among them, the required checks of `CONTRIBUTING.md`, the ruleset on `main`, the benchmark workflow, and the Build, Test & Run section of `README.md` | 🟢 accepted |
| [OPS-dev-container-carries-and-reaches](OPS-dev-container-carries-and-reaches.md) | The Dev Container carries the Rust base and each check tool at the version CI runs, and reaches containers only on the host | `.devcontainer/devcontainer.json`, `.devcontainer/devcontainer-lock.json`, `.devcontainer/install-shellcheck.sh`, `.devcontainer/compose.yaml`, every Compose file of a development tool run on the host, every tool the container installs, and the steps of `.github/workflows/checks.yml` that install the same tools | 🟢 accepted |
| [OPS-feature-in-hivevm-features](OPS-feature-in-hivevm-features.md) | `nuc` ships as attested GitHub Release binaries and a GitHub Action, pinned by major tag and updated by Dependabot, and its Dev Container Feature is published from `hivevm/features` | the release workflow, the release tags of this repository, the release artifacts and their names, checksums, and attestations, the GitHub Action, and the absence of `feature/` and of any Feature publication from this repository | 🟢 accepted |
| [OPS-crates-pass-deny-and-audit](OPS-crates-pass-deny-and-audit.md) | `nuc` may depend on a crate from crates.io that widens what it reads or how fast it reads it, under MIT where it can choose and under a permissive license compatible with MIT otherwise, and that passes `cargo deny` and `cargo audit` | the `[dependencies]` of `Cargo.toml`, `Cargo.lock`, `deny.toml`, and `scripts/check-deps.sh` | 🟢 accepted |
| [OPS-nuc-version-by-change](OPS-nuc-version-by-change.md) | A release of `nuc` takes the part of its version by what the change does to a project | the version in `Cargo.toml` and `Cargo.lock` that a release of `nuc` moves, the release tag `vX.Y.Z` it names, and the pull request that prepares a release | 🟢 accepted |
| [API-hook-contract-keyed-shims](API-hook-contract-keyed-shims.md) | A POSIX sh shim hands each hook to `nuc hook <name>`, which decides from the git state and answers 0, 1, 2, 3, or 4, the shim runs the hook's check on 3 and the documentation check on 4, and every refusal of a shim ends in its key | the `nuc hook` command and its source, the reference shims under `shims/`, the `hook/` rule IDs, the files `nuc hook` writes in the git directory, and the merge driver of the ADR index, and the keys and wording of the shims' own refusals | 🟢 accepted |
| [API-version-coupling-hook-defaults](API-version-coupling-hook-defaults.md) | A project's `nuc.toml` names the minimum `nuc` and the rules turned on, the template release ships both, each rule checks the tree as the bash check of the release its README names, and the hooks run on defaults without it | the format of `nuc.toml`, how `nuc` loads it and the README's `Template release` line, every rule's default state, every rule whose bash check differs between template releases, and the trees `tests/parity/corpus.toml` pins for each such release, and how `nuc hook` reads both | 🟢 accepted |
| [API-command-line-with-sense](API-command-line-with-sense.md) | `nuc` reads `nuc.toml` as a TOML subset, writes one finding per line as `<file>[:<line>]: <rule ID>: <message> [<rule ID>:<reason>]`, `nuc measure` its measures ending in `tokens <n>`, `nuc sense` a sensor's report, and exits 0, 1, or 2, and `nuc hook` also 3 or 4 | the keys and the syntax of `nuc.toml` and how `src/config.rs` reads them, the text output of every `nuc` command, `nuc measure`'s and `nuc sense`'s among them, the reason every finding carries and the list of keys kept beside its test, and the exit codes | 🟢 accepted |
| [API-config-refusals-keyed](API-config-refusals-keyed.md) | Every refusal `nuc` reports while it reads `nuc.toml` ends in a key `[config/toml:<reason>]` | the refusals of `src/config.rs` and of a `nuc.toml` that cannot be read at all, their message on the standard error of `nuc check` and `nuc hook`, the keys `config/toml:<reason>` in the list of keys kept beside the contract's test, and the tests that assert them | 🟢 accepted |
| [API-measure-reads-harness-transcripts](API-measure-reads-harness-transcripts.md) | `nuc measure` reads a harness's transcripts from one directory outside the repository, read-only, through a pure adapter per harness, and prints numbers and names only | the `nuc measure` command and its source, the adapter of each harness it reads, the two functions of `src/shell/` that read outside the repository, the structural test that holds them, and the tests of `nuc measure` | 🟢 accepted |
| [API-output-formats-carry-reason](API-output-formats-carry-reason.md) | `nuc check --format json` writes one versioned JSON document and `--format sarif` a SARIF 2.1.0 log, each finding with the reason of its key and each rule with the first sentence of its reason as a short description, both without a crate, and nothing when the run cannot decide | the `--format` option of `nuc check`, the JSON and SARIF it writes and the code that writes them, and the tests that compare them with the text output | 🟢 accepted |
| [API-git-only-sensors-under-sense](API-git-only-sensors-under-sense.md) | `nuc sense <name>` ports the template's sensors that need only git, prints their report byte for byte as the bash does, and exits as every `nuc` command does | which sensors of the template `nuc` ports, the `nuc sense` command, its source in the shell and in the core, the names of the sensors it knows, its exit codes against the bash's, and the comparison of the sensors in the parity harness | 🟢 accepted |
| [DEV-one-agent-one-ticket](DEV-one-agent-one-ticket.md) | One agent implements and a fresh session reviews, one ticket at a time, and the maintainer reviews every change to the protected parts | every change made by a coding agent, the `review` skill, the agents and models that work here, and the parts of the system listed under Decision as never worked unattended | 🟢 accepted |
| [ARCH-rules-outcome-hooks-answer](ARCH-rules-outcome-hooks-answer.md) | One library crate whose rules return an outcome and whose hooks return an answer, its effects in one shell module, and a thin binary | the source tree of the crate, `src/main.rs`, `src/shell/`, every rule module, `src/hook.rs`, the type a rule returns and the type a hook returns, and the structural test that holds them | 🟢 accepted |
| [ARCH-rules-need-only-git](ARCH-rules-need-only-git.md) | `nuc` ports no bash rule whose decision needs a process other than git, such as a script of the project or ShellCheck, and leaves such a rule to the bash | which rules of the template's checks `nuc` ports, the rule IDs it knows, the processes `src/shell/` spawns, and the cases `tests/tables.cases` lists as not ported | 🟢 accepted |
| [TEST-parity-harness](TEST-parity-harness.md) | A parity harness compares `nuc` with the bash scripts on the pinned trees of a corpus of releases v0.5.0 and later and on variants derived from those trees, each on the trees it is for, as a required check that fails on any difference | `tests/parity/`, its manifest `tests/parity/corpus.toml`, the variants kept beside it, `scripts/check-parity.sh`, its job in `.github/workflows/checks.yml`, and the map from bash messages to rule IDs | 🟢 accepted |
| [TEST-hook-scenarios-on-bash-hooks](TEST-hook-scenarios-on-bash-hooks.md) | A scenario harness runs each hook through git, with the bash hooks of each pinned template tree up to the bound of the hooks `nuc` ports and with the shims and `nuc`, leaves out a tree whose blob list pins its hooks, and fails on any difference, on a self-test case no scenario covers, or when no tree is left | the scenarios of the hooks and the merge driver under `tests/parity/`, the map from bash hook messages to `hook/` rule IDs and notices, the trees they run on and the bound `HOOKS_PORTED` in `scripts/lib/hook-parity.sh`, their run from `scripts/check-parity.sh`, and the coverage of the template's `scripts/test-git-hooks.sh` | 🟢 accepted |
| [TEST-fixtures-with-own-home](TEST-fixtures-with-own-home.md) | Tests build their repositories through the real git CLI in fresh temporary directories, under a cleared environment with a `HOME` of the test run's own | every test that needs a git repository, the test helpers that build one, the environment the tests run under, and the `HOME` they pass to git and to `nuc` | 🟢 accepted |
| [TEST-copies-keep-old-identifiers](TEST-copies-keep-old-identifiers.md) | An ADR identifier inside a copy of an older template release is not a citation of this record, and no map moves it | the messages of `nuc` that copy a bash message of a template release, `tests/parity/`, `src/testing.rs`, `tests/support/`, and every map a normalization or a migration applies to this repository | 🟢 accepted |
| [TEST-template-table-cases](TEST-template-table-cases.md) | The rules are tested against copies of the template's fixture tables of the newest release `nuc` ports, each pinned by its blob, with every case that does not agree listed as pending on a ticket, as not ported, or as decided by an ADR in force | the copies of the template's fixture tables under `tests/tables/`, their blob list `tests/tables.blobs`, the list of cases that do not agree `tests/tables.cases`, the runner `tests/tables.rs`, and the check of the blobs in `scripts/check-parity.sh` | 🟢 accepted |

### Superseded and rejected

The record of what was decided against or replaced: read one when a change touches what it
governed, to learn why the current decision stands. A row moves here in the change that
flips its status.

| ADR | Title | Applies to | Status |
|-----|-------|------------|--------|
| [API-command-line-with-measure](API-command-line-with-measure.md) | `nuc` reads `nuc.toml` as a TOML subset, writes one finding per line as `<file>[:<line>]: <rule ID>: <message> [<rule ID>:<reason>]`, `nuc measure` its measures ending in `tokens <n>`, and exits 0, 1, or 2, and `nuc hook` also 3 or 4 | the keys and the syntax of `nuc.toml` and how `src/config.rs` reads them, the text output of every `nuc` command, `nuc measure`'s among them, the reason every finding carries and the list of keys kept beside its test, and the exit codes | ⚪ superseded by API-command-line-with-sense |
| [TEST-hook-scenario-parity](TEST-hook-scenario-parity.md) | A scenario harness runs each hook through git, once with the bash hooks of each pinned template release and once with the shims and `nuc`, and fails on any difference or on a self-test case no scenario covers | the scenarios of the hooks and the merge driver under `tests/parity/`, the map from bash hook messages to `hook/` rule IDs and notices, their run from `scripts/check-parity.sh`, and the coverage of the template's `scripts/test-git-hooks.sh` | ⚪ superseded by TEST-hook-scenarios-on-bash-hooks |
