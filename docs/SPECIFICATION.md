# Specification — nuc-tools

> The **specification** of this project: the problem it solves, where it is going, and what has to
> be true for it to have succeeded, written in the vocabulary of [`GLOSSARY.md`](GLOSSARY.md). It
> is the **constitution**: every Architecture Decision Record in [`adr/`](adr/) derives from it and
> must not contradict it ([ADR-0001](adr/0001-agent-governance-model.md)).

## Problem

Every project derived from the NUC template carries its own copy of the template's mechanics:
about 8,000 lines of bash in checks, sensors, git hooks, the ADR index merge driver, and the
scripts run by hand. Their maintainers and the agents working in them live with four costs
today:

- **Fixes do not propagate.** A defect fixed in one copy stays in every other: the leak of
  `GIT_DIR` from a git hook into the self-tests is fixed in one derived repository and still
  present in the others.
- **Taking up a template release is a manual checklist.** For v0.5.0 it was ten steps, worked
  by hand in each project.
- **The checks are not portable.** They need bash 4.4 or later and GNU tools, which macOS and
  Windows lack outside the Dev Container.
- **Every project pays for the template's own tests.** The template's self-tests run in each
  project's `check-all` and CI, although they test the template, not the project.

The cost grows with every derived project and every template release, and the template's
ADRs keep moving logic into these scripts, so it is worth solving now, before the copies
diverge further.

## Mission

Make the NUC template's checks, hooks, and migrations a dependency that is fixed once and updated
by version, not a copy maintained per project.

## Vision

A derived project carries only the template's rules and a thin shim per hook; a fix reaches it
with a Dependabot update, taking up a template release is one `nuc upgrade` plus the judgment
calls it lists, and the checks behave the same on Linux, macOS, and Windows without bash.

## Strategy

Port the bash scripts one by one, each proved by a parity harness against the bash of the
matching template release, on the template and on derived repositories; the bash stays in the
template until `nuc` reaches parity, then the template switches over.

## Goals / Success Criteria

Acceptance criteria a test can decide, where it helps in the form *When <trigger>, the system
shall <response>*. Each starts with a bold `G-n`, which a test cites with `Verifies: G-n`. Until
that test exists the criterion is pending work, and the traceability check lists it
([ADR-0023](adr/0023-decisions-verified-by-tests.md)). Identifiers are never reused.
Only a normalization renumbers them ([`AGENTS.md` §3](../AGENTS.md#3-adr-rules), rule 5).

1. **G-1** — When a derived project runs `nuc check`, it reports the same findings as the bash
   checks of the template release the project carries report on the same tree.
2. **G-2** — When git runs a hook, the shim in `.githooks/` hands over to `nuc hook <name>`,
   which refuses or passes exactly as the bash hook of that release does.
3. **G-3** — When git merges `docs/adr/README.md`, `nuc` writes the index both sides imply, or
   stops with a conflict when both sides changed its prose.
4. **G-4** — When a project takes up a template release, `nuc upgrade` applies its mechanical
   migrations and lists what needs judgment.
5. **G-5** — When the `nuc` binary is missing or older than the project's `nuc.toml` requires,
   every hook and check fails closed with a message naming how to install it.
6. **G-6** — When a check fails, its output names a stable rule ID and `file:line`, and
   `nuc explain <id>` gives the reason.
7. **G-7** — When a check runs with `--format json` or `--format sarif`, it reports the same
   findings as its text output.

## Quality Goals

The few properties that drive decisions, in priority order. Every ADR weighs its *Consequences*
against them. Each has a measure concrete enough to settle a disagreement and starts with a bold
`Q-n`, cited like a goal. Security goals, what the system must never do, belong here, derived
from the threats below.

- **Q-1** — Integrity of the gates: a check or hook never passes when it could not decide. For
  each class of failure — git missing or failing, an unreadable or malformed input, an unknown
  rule, an internal error or panic — a fixture shows a non-zero exit and a message naming the
  cause; no input makes `nuc` exit 0 with a finding dropped.
- **Q-2** — Parity with the bash checks during the migration: the parity harness reports zero
  differences in findings and exit status between `nuc` and the bash scripts of the template
  release each repository carries, on the template and on every repository of the corpus, for
  every ported check, hook, and script.
- **Q-3** — Supply chain: every release artifact carries a SHA-256 checksum and a
  build-provenance attestation that `gh attestation verify` accepts; a rebuild of the tag in CI
  yields the published checksum for each target; `cargo deny check` and `cargo audit` pass on
  every pull request, and every ignored advisory names its reason.
- **Q-4** — Portability: the full test suite passes in CI on Linux x64, Linux arm64, macOS, and
  Windows x64, with no bash on the `PATH` of the process under test.
- **Q-5** — Hook latency: `nuc hook reference-transaction` adds at most 2 ms median per call over
  a no-op hook on Linux x64, measured over 1,000 calls.
- **Q-6** — Compatibility: a tool update never turns a green project red. Every repository of the
  corpus that passes with the last release of `nuc` passes with the new one under an unchanged
  `nuc.toml` and template release; a new check ships off, and only a template release turns it
  on.
- **Q-7** — No execution of repository content: fixtures carrying content that would run if
  interpreted leave a canary file uncreated after every command.
- **Q-8** — Writes stay inside: across the test suite, no file outside the repository and its
  git directory is created or changed.
- **Q-9** — One repository: with `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, and
  `GIT_CONFIG_PARAMETERS` set to a decoy repository, every command reports on the repository it
  was pointed at and leaves the decoy untouched.
- **Q-10** — No token disclosure: a canary token in the environment and in the git configuration
  appears in no output and in no written file.
- **Q-11** — No network in checks and hooks: with network access blocked, every check and hook
  reports the same findings as with it.

## Non-Goals

What this project explicitly does not try to do.

- **No rules in the tool.** `AGENTS.md`, the ADRs, and the skills of the template stay the
  source of every rule; `nuc` decides only what is mechanical and makes no judgment — no
  review, no design revision, no ADR content.
- **No agent- or editor-specific mechanism.** Enforcement runs through git hooks and CI only,
  so it holds for every agent and every human alike. An LSP or MCP front end is a later
  option, never the enforcement path.
- **No release automation for projects.** How a derived project releases is the template's
  ADR on releases, not this tool's.
- **No writes beyond the repository without a human's instruction.** `nuc` pushes nothing,
  writes nothing to GitHub, and applies no repository settings unless a human explicitly
  instructs it; sensors only read.
- **No network access in checks and hooks,** except in a sensor that names it.

## Constraints

What the solution has to live with regardless of the better choice: platforms, standards,
systems to interoperate with, limits of time, budget, and team. A constraint not written here is
rediscovered as a rejected alternative in an ADR.

- **Language:** Rust stable, with the toolchain pinned.
- **Runtime dependency:** the git CLI only, spawned as a process rather than linked through
  libgit2, so that `nuc` behaves exactly as git does.
- **Distribution formats:** GitHub Releases, the Dev Container Feature specification, and
  GitHub Actions.
- **Consumer:** the NUC template (`hivevm/nuc`) and the projects derived from it, coupled to
  `nuc` by a minimum version in their `nuc.toml`.
- **Migration:** the bash scripts stay in the template until `nuc` reaches parity with them.
- **Team:** one maintainer; agents work the tickets, and the maintainer reviews and decides.

## Threats & Forbidden Actions

The assets, the actors who threaten them, and the trust boundaries between the parts. Then what
the system must never do, regardless of what a task asks. A prohibition not written here does not
exist for an agent. One a test can decide is restated as a `Q-n`, so that it is verified.

- **Assets:** the gates — checks and hooks — of every project derived from the template; those
  repositories, their trees, git directories, and refs; and the tokens present in the
  environments `nuc` runs in, such as CI's.
- **Actors and trust boundaries:**
  - *The supply chain:* a compromised dependency, or a compromised build or release pipeline,
    reaches every derived project through its hooks and CI. The boundary is the published
    artifact.
  - *Repository content:* ADR files, Markdown, and YAML are untrusted input, since they may come
    from a fork's pull request. The boundary is every parser of `nuc`.
  - *The inherited environment:* variables git sets for hooks, such as `GIT_DIR` and
    `GIT_CONFIG_PARAMETERS`, can point `nuc` and the git it spawns at another repository or
    another configuration. The boundary is the environment of every process `nuc` starts.
- The system shall never execute repository content (Q-7).
- The system shall never write outside the repository and its git directory (Q-8).
- The system shall never act on a repository other than the one it was pointed at (Q-9).
- The system shall never print or store a token (Q-10).
- The system shall never pass when it could not decide (Q-1).

## Assumptions

What is believed true today and would invalidate the decisions above if it turned out false:
expected scale, the shape of the data, who operates the result, which dependency stays
maintained.

- The bash scripts of each template release stay runnable as the parity oracle, on Linux with
  bash 4.4 or later and the GNU tools, for as long as parity is measured.
- The repositories of the parity corpus stay readable to the harness; which derived repositories
  form it is decided when the harness is planned.
- A git CLI is present wherever `nuc` runs, and its output is stable across the versions in use;
  the minimum git version is not yet fixed.
- `nuc` learns which template release a project carries either from the `Template release` line
  of its README or from its `nuc.toml`; the founding decision on version coupling settles which.
- The template switches its hooks and checks over to `nuc` once parity is reached, and keeps its
  rules in `AGENTS.md`, the ADRs, and the skills.
- The targets of Q-3 and Q-4 can be built reproducibly with the pinned Rust toolchain; which
  macOS architectures are released, arm64 or x64 as well, is still open.
