# Architecture Decision Records

This directory contains all Architecture Decision Records (ADRs) for this project. Accepted ADRs are
**binding** for humans and coding agents alike (see [`AGENTS.md`](../../AGENTS.md) in the repository
root). ADRs derive from the specification in [`docs/SPECIFICATION.md`](../SPECIFICATION.md).

## Mechanics

The rules live in [`AGENTS.md` §3](../../AGENTS.md#3-adr-rules): when an ADR is written, how it
is developed, who flips its status, that an accepted one is immutable and changed only by
supersession. This section carries only the mechanics of the record;
[`scripts/check-docs.sh`](../../scripts/check-docs.sh) names in its header which of them it
verifies.

- **File.** Copy [`template.md`](template.md) to `draft-<slug>.md`, the slug in lower case letters
  and digits joined by hyphens. Until it reaches `main` the ADR is a draft: its heading and every
  reference name it `ADR-draft-<slug>`, its index row links it as `draft-<slug>`, and no number
  is taken ([ADR-0002](0002-decisions-verified-by-tests.md)). On the branch whose pull request
  brings it to `main`, once that branch has taken up the current `main`,
  [`scripts/number-adrs.sh`](../../scripts/number-adrs.sh) gives it the next free number. A
  numbered ADR is `NNNN-short-title.md`, its `# ADR-NNNN` heading matches the filename, and the
  numbers run `0001..N` without gaps.
- **Index.** The tables below change in the same commit as the ADR they describe, for
  additions, supersessions, and status flips alike. The status is shown via the legend's bullet
  and the ADR's `Applies to` header is mirrored verbatim in its own column. The index routes a
  reader from a change to the decisions that bind it, so a row says what its ADR governs without
  the file being opened. Two tables keep what a session reads apart from what it looks up: an
  accepted or proposed ADR sits under **Binding**, a superseded or rejected one under
  **Superseded and rejected**, and a status flip moves the row.
- **Numbers are permanent once on `main`, until a normalization.** Between two normalizations,
  never renumber, delete, or merge an ADR there: other ADRs, commits (`Implements ADR-NNNN`), and
  code may reference a number. No draft reaches `main`. A branch that already carries a numbered
  proposed ADR whose number another branch brought to `main` first gives it the next free number
  before it merges, in every reference its own branch makes to it. Until a normalization, a
  superseded ADR keeps its file and its body. Its `Status` line flips to `⚪ superseded by
  ADR-NNNN` in the change that lands the superseding ADR, whose `Supersedes` field names it back.
- **Between normalizations, sprawl is curbed by supersession, never by editing.** One ADR may
  supersede several whose decisions have grown into one. Its `Supersedes` field names each, and
  each flips its status in the same change, so the set an agent reads for a change shrinks
  while every number and every body stays.
- **A normalization restates the record** ([`AGENTS.md` §3](../../AGENTS.md#3-adr-rules), rule
  5).
  - **Start.** The tree has no proposed ADR, no feature spec or ticket under `docs/plan/`, and
    green checks.
  - **Steps.** Three, in this order on one branch, each handed to the human and confirmed
    before the next begins: the restated specification, whose `G-n` and `Q-n` run from 1; the
    supersessions it calls for; and the restated record. The record begins once the human has
    decided each supersession. One pull request lands them, merged by a human.
  - **Order.** The record numbers the ADRs in force `0001..N` by what they govern. The document
    set and the ADR process come first, then the founding decisions in the order of the setup,
    then the rest in groups. An inherited ADR takes its place like any other, and may be merged
    with the project's own.
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
    script of the `normalize` skill. The map stands in the hand-off and the
    commit body, and in no document. A commit older than the normalization cites the numbers it
    had then.
- **Cite only what exists.** Every `ADR-NNNN` reference names a file already in this directory.
  An anticipated follow-up is described by topic ("a follow-up ADR on session storage"), never by
  a number. In Markdown, cite an ADR as a link to its own file.

## Index

An ADR whose `Deciders` line names the **NUC maintainer** is inherited from the template. It
binds a derived project once its own maintainer adds their name to `Deciders` and its `Status`
line reads `🟢 accepted` ([`AGENTS.md` §3](../../AGENTS.md#3-adr-rules)). Change an inherited
decision by superseding it, never by editing. Once the template setup is done, check 12 of
[`scripts/check-docs.sh`](../../scripts/check-docs.sh) fails on an inherited ADR in force that
names no decider of the project, or that is still proposed.

**Status legend:** 🟢 accepted · 🟡 proposed · 🔴 rejected · ⚪ superseded

### Binding

The decisions in force: accepted, and proposed ones that bind the work implementing them. This
is the table a session reads.

| ADR | Title | Applies to | Status |
|-----|-------|------------|--------|
| [0001](0001-agent-governance-model.md) | The document set: one specification, one ADR record, one `AGENTS.md`, one overview, one glossary, one conventions file | every document that carries rules for humans or agents, `AGENTS.md` among them, the architecture overview, the glossary, the conventions, `scripts/check-docs.sh`, `scripts/sensor-binding-adrs.sh`, and the map script of the `normalize` skill, `.agents/skills/normalize/remap.sh` | 🟢 accepted |
| [0002](0002-decisions-verified-by-tests.md) | Every accepted decision and every success criterion is verified by a test that cites it, and the checks that hold the decisions are bash scripts under `scripts/` | every accepted ADR, the identifiers by which ADRs are cited, the Goals and Quality Goals of `docs/SPECIFICATION.md`, the tests that verify them, the checks and sensors under `scripts/`, and the git hooks under `.githooks/` | 🟢 accepted |
| [0003](0003-procedures-as-skills.md) | The procedures of the rule file are Agent Skills under `.agents/skills/`, each carrying a how and no rule of its own | `.agents/skills/`, the pointers under `.claude/skills/`, and every document that describes how a procedure of `AGENTS.md` is carried out | 🟢 accepted |
| [0004](0004-dev-container-runtime.md) | The Dev Container keeps the host daemon out of reach and its Features locked | `.devcontainer/`, `.vscode/settings.json`, `scripts/check-devcontainer.sh`, and everything the container pulls in | 🟢 accepted |
| [0005](0005-template-releases.md) | The template is released as SemVer tags on `main`, and every repository names the release it carries | the tags of the template repository, its own ADRs, the **Template** section of `README.md`, check 14 of `scripts/check-docs.sh`, and the pull request that lands a release | 🟢 accepted |
| [0006](0006-feature-layer.md) | Work larger than one session is planned in the tree as a feature spec cut into tracer-bullet tickets, files deleted when their work lands, and the design is revised when a spec closes or after a number of changes | every change larger than one agent session, `docs/plan/`, the spec and ticket templates of the `plan-feature` skill, the pull request template, the `Last design revision` line of `docs/ARCHITECTURE.md`, `scripts/sensor-revision-due.sh`, `scripts/sensor-plan-frontier.sh`, and the claim script `.agents/skills/work-tickets/claim.sh` | 🟢 accepted |
| [0007](0007-harness.md) | One agent implements and a fresh session reviews, one ticket at a time, and the maintainer reviews every change to the protected parts | every change made by a coding agent, the `review` skill, the agents and models that work here, and the parts of the system listed under Decision as never worked unattended | 🟢 accepted |
| [0008](0008-toolchain.md) | Rust stable pinned in `rust-toolchain.toml`, cargo's build, test, fmt, and clippy behind `scripts/check-build.sh`, and the dependency audit behind `scripts/check-deps.sh` | `rust-toolchain.toml`, `Cargo.toml`, `Cargo.lock`, `deny.toml`, `scripts/check-build.sh`, `scripts/check-deps.sh`, their jobs in `.github/workflows/checks.yml`, the benchmark workflow, and the Build, Test & Run section of `README.md` | 🟢 accepted |
| [0009](0009-functional-core.md) | One library crate with the rules as a functional core, its effects in one shell module, and a thin binary | the source tree of the crate, `src/main.rs`, `src/shell/`, every rule module, and the structural test that holds them | 🟢 accepted |
| [0010](0010-git-cli-cleared-environment.md) | The shell spawns the git CLI with a cleared environment, an allowlist passed through, the repository named by `-C`, and command-running configuration neutralized | every spawn of git in `src/shell/`, the environment and arguments it passes, and the list of neutralized configuration keys kept beside its test | 🟢 accepted |
| [0011](0011-fixture-repositories.md) | Tests build their repositories through the real git CLI in fresh temporary directories, under a cleared environment | every test that needs a git repository, the test helpers that build one, and the environment the tests run under | 🟢 accepted |
| [0013](0013-version-coupling.md) | A project's `nuc.toml` names the minimum `nuc` and the rules turned on, the template release ships both, and `nuc` reads the release from the README | the format of `nuc.toml`, how `nuc` loads it and the README's `Template release` line, and every rule's default state | 🟢 accepted |
| [0014](0014-distribution.md) | `nuc` ships as attested GitHub Release binaries, a Dev Container Feature, and a GitHub Action, pinned by major tag and updated by Dependabot | the release workflow, the release tags of this repository, the release artifacts and their checksums and attestations, the Dev Container Feature and its publication, and the GitHub Action | 🟢 accepted |
| [0015](0015-action-references-by-sha.md) | The workflows reference every action by full commit SHA with a version comment, kept current by Dependabot | every `uses:` line under `.github/workflows/`, `.github/dependabot.yml`, and the run of `scripts/sensor-action-refs.sh` in `scripts/check-all.sh` and the `sensors` job | 🟢 accepted |
| [0016](0016-no-stored-secrets.md) | The workflows use no stored secret, only `GITHUB_TOKEN` and OIDC with permissions set per job | every workflow under `.github/workflows/`, its `permissions`, and every reference to `secrets.` in them | 🟢 accepted |
| [0017](0017-dev-container-with-rust.md) | The Dev Container keeps the template's Debian base image, adds the Rust Feature, and reaches no containers | `.devcontainer/devcontainer.json`, `.devcontainer/devcontainer-lock.json`, and the tools the container installs | 🟢 accepted |
| [0018](0018-gates-never-run-the-build.md) | This repository's gates never run the `nuc` being built, only the template's bash or a released `nuc` pinned like any dependency | `scripts/check-all.sh`, the hooks under `.githooks/`, the jobs of `.github/workflows/checks.yml`, and every place they name the tool that runs a check | 🟢 accepted |
| [0019](0019-command-line-contract.md) | `nuc` reads `nuc.toml` as a TOML subset, writes one finding per line as `<file>[:<line>]: <rule ID>: <message>`, and exits 0, 1, or 2 | the keys and the syntax of `nuc.toml` and how `src/config.rs` reads them, the text output of every `nuc` command, and its exit codes | 🟢 accepted |
| [draft-parity-harness-with-variants](draft-parity-harness-with-variants.md) | A parity harness compares `nuc` with the bash scripts on the pinned trees of a corpus of releases v0.5.0 and later and on variants derived from them, as a required check that fails on any difference | `tests/parity/`, its manifest `tests/parity/corpus.toml`, the variants kept beside it, `scripts/check-parity.sh`, its job in `.github/workflows/checks.yml`, and the map from bash messages to rule IDs | 🟢 accepted |

### Superseded and rejected

The record of what was decided against or replaced: read one when a change touches what it
governed, to learn why the current decision stands. A row moves here in the change that
flips its status.

| ADR | Title | Applies to | Status |
|-----|-------|------------|--------|
| [0012](0012-parity-harness.md) | A parity harness compares `nuc` with the bash scripts of a pinned corpus on the same trees, as a required check that fails on any difference | `tests/parity/`, its manifest `tests/parity/corpus.toml`, `scripts/check-parity.sh`, its job in `.github/workflows/checks.yml`, and the map from bash messages to rule IDs | ⚪ superseded by ADR-draft-parity-harness-with-variants |
