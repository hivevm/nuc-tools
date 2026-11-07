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
   outside the container? Run the four commands listed there yourself, once per clone.
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

Every release of `nuc` carries one binary per platform, named `nuc-<target>`, a `SHA256SUMS`
file, and a build-provenance attestation per binary
([ADR-0025](docs/adr/0025-feature-in-hivevm-features.md)).

### Install

**In a Dev Container**, the Feature `nuc` is to come from
[`hivevm/features`](https://github.com/hivevm/features), which takes each release from here and
publishes it under `ghcr.io/hivevm/features`; how it versions and verifies the Feature, that
repository says. This repository publishes no Feature
([ADR-0025](docs/adr/0025-feature-in-hivevm-features.md)).

**In a GitHub workflow**, run `nuc` through the Action. It installs the `nuc` of its release,
checks it against the checksums its release commit carries, and runs `nuc` with `args`, in the
repository a checkout step put on the runner. Reference it by the major tag alone: `v0` names the
release commit of the newest 0.x release, while a tag `vX.Y.Z` names the source commit, which
carries no checksums and installs nothing. It runs on the runners a release has a binary for,
not on macOS on x64.

```yaml
- uses: actions/checkout@v7
- uses: hivevm/nuc-tools@v0
  with:
    args: check
```

**By hand**, download the binary for the platform and `SHA256SUMS` from the
[release](https://github.com/hivevm/nuc-tools/releases), check its checksum (on macOS
`shasum -a 256 -c` in place of `sha256sum -c`) and that the release workflow built it, then make
it executable and move it, as `nuc`, into a directory on the `PATH`, such as `~/.local/bin`:

```sh
grep ' nuc-x86_64-unknown-linux-musl$' SHA256SUMS | sha256sum -c
gh attestation verify nuc-x86_64-unknown-linux-musl --repo hivevm/nuc-tools \
  --signer-workflow hivevm/nuc-tools/.github/workflows/release.yml
chmod +x nuc-x86_64-unknown-linux-musl && mv nuc-x86_64-unknown-linux-musl ~/.local/bin/nuc
```

The targets are `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`,
`aarch64-apple-darwin`, and `x86_64-pc-windows-msvc` (`nuc-x86_64-pc-windows-msvc.exe`).

### Configure

`nuc.toml`, at the root of the repository, names the minimum version of `nuc` and the rules it
turns on: the rules `nuc check` runs, and the `hook/` rules `nuc hook` decides
([ADR-0023](docs/adr/0023-version-coupling-with-hook-defaults.md)):

```toml
minimum = "0.1.0"
rules = ["docs/template-release", "docs/prose-width"]
```

A `nuc` older than the minimum refuses to decide, and so does an unknown rule ID, so that a file
`nuc` does not understand never passes. Without a `nuc.toml` every `hook/` rule is on; with one,
each `hook/` rule its list leaves out is off, as the example above turns off all seven:
`hook/commit-on-main`, `hook/merge-checks`, `hook/merge-on-main`, `hook/merge-docs`,
`hook/push-to-main`, `hook/push-checks`, and `hook/main-moves`; `nuc explain <rule ID>` gives each
one's reason. A `nuc` built from source carries the version of its `Cargo.toml`, which the
minimum holds like a release's.

### Commands

- `nuc check [--repo <path>] [--format text|json|sarif]` runs the rules `nuc.toml` turns on over the
  repository that contains the current directory or `<path>`. Each finding is a line
  `<file>[:<line>]: <rule ID>: <message>`; with `--format json` the findings and notes are one
  JSON document, `{"format": "nuc-findings", "version": 1, "findings": [...], "notes": [...]}`,
  each finding with `rule`, `file`, `line` (or `null`), and `message`; with `--format sarif` they
  are a SARIF 2.1.0 log, which GitHub's code scanning reads. The exit code is 0 without
  a finding, 1 with one, and 2 where `nuc` could not decide, as without a readable `nuc.toml`;
  then nothing is written on the standard output in any format.
- `nuc explain <rule ID>` gives the reason of a rule.
- `nuc hook <name>` decides a git hook; the shims below call it.
- `nuc --version` prints the version.

The rules port every check of the template's `check-docs.sh`, 1 to 16, each as the release a tree
carries checks it, v0.5.0 or later: `docs/adr-index`, `docs/adr-numbering`, `docs/links`,
`docs/section-refs`, `docs/adr-refs`, `docs/adr-link-targets`, `docs/adr-header`,
`docs/adr-supersessions`, `docs/check-lists`, `docs/project-layout`, `docs/skill-pointers`,
`docs/inherited-adrs`, `docs/template-identity`, `docs/prose-width`, `docs/no-draft-on-main`, and
`docs/template-release`. They port every part of the template's `check-traceability.sh` too:
`traceability/criteria`, `traceability/citations`, and `traceability/coverage`, which names the
criteria no test cites yet in the summary on standard error.

### Git hooks

The shims under [`shims/`](shims/), in POSIX sh, are the reference hooks the template copies
when it switches over from its bash hooks; until it does, a project keeps the hooks its template
release carries ([ADR-0021](docs/adr/0021-hook-contract.md)). Each shim hands its hook to
`nuc hook <name>`: `nuc` decides, and where a hook's check is due the shim runs the project's
`scripts/check-all.sh` or `scripts/check-docs.sh` itself. Without `nuc` on the `PATH`, or with one
older than `nuc.toml` requires, every shim refuses and names where to install it. They rely on the
same git settings as the template's hooks, listed under
[Git hooks](CONTRIBUTING.md#git-hooks).

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
