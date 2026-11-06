# nuc-tools

[![Checks](https://github.com/hivevm/nuc-tools/actions/workflows/checks.yml/badge.svg)](https://github.com/hivevm/nuc-tools/actions/workflows/checks.yml)
[![Release](https://img.shields.io/github/v/release/hivevm/nuc-tools)](https://github.com/hivevm/nuc-tools/releases/latest)
[![License](https://img.shields.io/github/license/hivevm/nuc-tools)](LICENSE)

**nuc-tools** builds `nuc`, a Rust CLI that takes over the mechanical logic of the
[NUC](https://github.com/hivevm/nuc) template's bash scripts — checks, sensors, git hooks, the
ADR index merge driver, and the scripts run by hand — and ships it as one pinned binary for every
project derived from the template. The template keeps the rules; `nuc` decides only what is
mechanical. The work is driven by a written **specification**
([`docs/SPECIFICATION.md`](docs/SPECIFICATION.md)) and **Architecture Decision Records**
([`docs/adr/`](docs/adr/)).

> For agent instructions, see [`AGENTS.md`](AGENTS.md), the single source of truth for all coding
> agents.

## Overview

`nuc` takes over the mechanics of the NUC template — its checks, sensors, git hooks, ADR index
merge driver, and the scripts run by hand — so that a project derived from the template carries
its rules and a thin shim per hook instead of a copy of ~8,000 lines of bash. It is for the
maintainers of those projects and the agents working in them. The main goals: the same findings
as the bash checks of the template release a project carries, gates that fail closed, and one
pinned binary on Linux, macOS, and Windows without bash. The full problem statement and goals
live in [`docs/SPECIFICATION.md`](docs/SPECIFICATION.md), the vocabulary in
[`docs/GLOSSARY.md`](docs/GLOSSARY.md).

## Prerequisites

- [VS Code](https://code.visualstudio.com/) with the
  [Dev Containers](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)
  extension, or any DevContainer-compatible IDE
- Docker / Podman (rootless) available on the host
- Outside the Dev Container: bash 4.4 or later for the checks, 3.2 for the git hooks
  ([ADR-0002](docs/adr/0002-decisions-verified-by-tests.md)); macOS ships 3.2

## Getting Started

1. Open the repository in VS Code and choose **Reopen in Container**. The Dev Container and the
   preconfigured agent extensions build automatically, and the container enables the repository's
   git hooks with the merge settings they rely on ([Git hooks](CONTRIBUTING.md#git-hooks)). Working
   outside the container? Run the three commands listed there yourself, once per clone.
2. Authenticate your coding agent inside the container (for Claude Code: `claude login`).
3. Start working with the agent. Drive the work from the specification and the ADRs.

## Build, Test & Run

- **Build:** `cargo build --locked`
- **Test:** `cargo test --locked`
- **Lint:** `cargo fmt --check` and `cargo clippy --all-targets --locked -- -D warnings`
- **Run:** `cargo run --locked -- <command>`, such as `cargo run --locked -- check`; for
  development only, since the gates never run the build
  ([ADR-0016](docs/adr/0016-gates-never-run-the-build.md))

[`scripts/check-build.sh`](scripts/check-build.sh) runs the lint, the build, and the tests;
[`scripts/check-deps.sh`](scripts/check-deps.sh) audits the dependencies.

## Usage

No release of `nuc` exists yet; built from source, it checks a repository:

- `nuc check [--repo <path>]` runs the rules the `nuc.toml` of the repository turns on, over the
  repository that contains the current directory or `<path>`. Each finding is a line
  `<file>[:<line>]: <rule ID>: <message>`. The exit code is 0 without a finding, 1 with one, and 2
  when `nuc` could not decide, as without a readable `nuc.toml` or below its minimum version.
- `nuc explain <rule ID>` gives the reason of a rule.

`nuc.toml`, at the root of the repository, names the minimum version of `nuc` and the rules
turned on
([ADR-0023](docs/adr/0023-version-coupling-with-hook-defaults.md)):

```toml
minimum = "0.0.0"
rules = ["docs/template-release"]
```

A `nuc` built from source is version 0.0.0 until the first release, so a higher minimum refuses
it.

The rules port every check of the template's `check-docs.sh`, 1 to 16, each as the release a tree
carries checks it, v0.5.0 or later: `docs/adr-index`, `docs/adr-numbering`, `docs/links`,
`docs/section-refs`, `docs/adr-refs`, `docs/adr-link-targets`, `docs/adr-header`,
`docs/adr-supersessions`, `docs/check-lists`, `docs/project-layout`, `docs/skill-pointers`,
`docs/inherited-adrs`, `docs/template-identity`, `docs/prose-width`, `docs/no-draft-on-main`, and
`docs/template-release`. They port every part of the template's `check-traceability.sh` too:
the criteria of the specification, `traceability/criteria`; the citations of the `Verifies`
markers, `traceability/citations`; and the accepted ADRs each cited or exempt,
`traceability/coverage`, which names the criteria no test cites yet in the summary on standard
error. `nuc explain <rule ID>` gives each rule's reason.
`nuc hook <name>` follows with the port of the hooks.

## Template

- **Template release:** v0.7.0 — the release of [NUC](https://github.com/hivevm/nuc) this
  repository carries; a project moves the line when it takes up a later one
  ([ADR-0005](docs/adr/0005-template-releases.md)).

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the workflow (specification- and ADR-driven, small
reviewable changes), the project layout, the Dev Container, the coding agents, the git hooks, the
integration branches, and the repository settings, and [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md)
for the community standards we expect of everyone taking part. Security issues: please follow
[`SECURITY.md`](SECURITY.md) instead of opening a public issue.

## License

Released under the MIT License. See [`LICENSE`](LICENSE).
