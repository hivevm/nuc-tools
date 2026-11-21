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
- Outside the Dev Container: bash 4.4 or later for the checks
  ([NUC-checks-are-bash §1](docs/adr/NUC-checks-are-bash.md)), which macOS does not ship, and
  [`nuc`](https://github.com/hivevm/nuc-tools/releases) for the git hooks
  ([NUC-hooks-run-nuc](docs/adr/NUC-hooks-run-nuc.md))

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
  ([SEC-gates-run-released-nuc](docs/adr/SEC-gates-run-released-nuc.md))

[`scripts/check-build.sh`](scripts/check-build.sh) runs the lint, the build, and the tests;
[`scripts/check-deps.sh`](scripts/check-deps.sh) audits the dependencies.

## Usage

Every release of `nuc` carries one binary per platform, named `nuc-<target>`, a `SHA256SUMS`
file, and a build-provenance attestation per binary
([OPS-feature-in-hivevm-features](docs/adr/OPS-feature-in-hivevm-features.md)).

### Install

**In a Dev Container**, the Feature `nuc` is to come from
[`hivevm/features`](https://github.com/hivevm/features), which takes each release from here and
publishes it under `ghcr.io/hivevm/features`; how it versions and verifies the Feature, that
repository says. This repository publishes no Feature
([OPS-feature-in-hivevm-features](docs/adr/OPS-feature-in-hivevm-features.md)).

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
([API-version-coupling-hook-defaults](docs/adr/API-version-coupling-hook-defaults.md)):

```toml
minimum = "0.2.2"
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
  `<file>[:<line>]: <rule ID>: <message> [<rule ID>:<reason>]`, ending in its key; with
  `--format json` the findings and notes are one JSON document,
  `{"format": "nuc-findings", "version": 1, "findings": [...], "notes": [...]}`, each finding with
  `rule`, `reason`, `file`, `line` (or `null`), and `message`; with `--format sarif` the findings
  are a SARIF 2.1.0 log, which GitHub's code scanning reads, each with its reason, and the notes
  stay on the standard error. The exit code is 0 without a finding, 1 with one, and 2 where `nuc`
  could not decide, as without a readable `nuc.toml`; then nothing is written on the standard
  output in any format. A refusal of `nuc.toml` ends in its key, `[config/toml:missing]`,
  `[config/toml:unreadable]`, `[config/toml:old]`, or `[config/toml:unknown]`
  ([API-config-refusals-keyed](docs/adr/API-config-refusals-keyed.md)).
- `nuc explain <rule ID>` gives the reason of a rule.
- `nuc hook <name>` decides a git hook; the shims below call it.
- `nuc measure [--harness claude-code] [--transcripts <dir>] [--branch <name>]... [--since <time>]
  [--until <time>] [--breakdown] [--finding]` prints `tokens <n>`, the part of a hand-off's Measures
  line the harness knows: the input, cache read, cache written, and output tokens of every request
  the harness's sessions made on the current branch and on each branch `--branch` adds, subagents
  included, each request once. The current branch always counts: to measure other branches alone, as
  the tickets a `feat/` branch gathered, run it on a branch without sessions, such as `main`, or on
  a detached HEAD. `--since` and `--until` take a date `YYYY-MM-DD` or a UTC time
  `YYYY-MM-DDTHH:MM[:SS]Z` and keep the requests from the first and before the second. `--breakdown`
  prints before that line each session's and subagent's tokens in those four parts, each tool's
  calls and output bytes, and the three costliest subagents and largest tool outputs. `--finding`
  prints before it a draft of a template finding in the fields of the template's issue form: the
  README's Template release, the tokens and their four parts, and the tools and sizes of the largest
  outputs, an MCP tool as `mcp`, with no session, subagent, branch, or path; the human completes it
  and decides whether to send it. It reads Claude Code's transcripts of this working tree, in the
  directory under `CLAUDE_CONFIG_DIR` or `~/.claude` that Claude Code names for the working tree's
  path, or `CLAUDE_CODE_PROJECT_DIR_NAME` under a set `CLAUDE_CONFIG_DIR`, or those in `<dir>`; a
  session started in a subdirectory or in another worktree has a directory of its own, which
  `--transcripts` names. It prints nothing of their text but a subagent's type and description and
  the names of tools. It exits 0, or 2 where a transcript does not read as it expects, no request
  was selected, or, for `--finding`, the README cannot be read, with nothing on the standard output
  ([API-measure-reads-harness-transcripts](docs/adr/API-measure-reads-harness-transcripts.md)).
- `nuc --version` prints the version.

The rules port every check of the template's `check-docs.sh`, 1 to 16, each as the release a tree
carries checks it, v0.5.0 or later: `docs/adr-index`, `docs/adr-numbering`, `docs/links`,
`docs/section-refs`, `docs/adr-refs`, `docs/adr-link-targets`, `docs/adr-header`,
`docs/adr-supersessions`, `docs/check-lists`, `docs/project-layout`, `docs/skill-pointers`,
`docs/inherited-adrs`, `docs/template-identity`, `docs/prose-width`, `docs/no-draft-on-main`, and
`docs/template-release`. They port every part of the template's `check-traceability.sh` too:
`traceability/criteria`, `traceability/citations`, `traceability/coverage`, which names the
criteria no test cites yet in the summary on standard error, and, from v0.9.0 on,
`traceability/sensor-tests`. From v0.10.0 on, `shell/mode` ports the one rule of `check-shell.sh`
that needs only git. From v0.12.0 on, `hooks/pin` and `hooks/toml` port `check-hooks.sh`: the
blob list `.githooks/nuc.blobs` pins each hook unchanged, and `nuc.toml` names a minimum no older
than the hooks' release and every `hook/` rule the list names. The rules of the area
`devcontainer` port every rule of `check-devcontainer.sh`, each as the release a tree carries
checks it: `devcontainer/definition`, `devcontainer/socket`, `devcontainer/lock`,
`devcontainer/settings`, `devcontainer/git-settings`, from v0.7.0 on `devcontainer/mount-path`,
and from v0.12.0 on `devcontainer/nuc`. From v0.9.0 on, a tree without
`.devcontainer/` has no Dev Container and is held to nothing, its settings included, and a tree with
it is held to its settings only with `.vscode/`.

### Git hooks

The shims under [`shims/`](shims/), in POSIX sh, are the reference hooks the template copies
unchanged and pins by their blobs from release v0.12.0 on
([API-hook-contract-keyed-shims](docs/adr/API-hook-contract-keyed-shims.md),
[NUC-hooks-run-nuc](docs/adr/NUC-hooks-run-nuc.md)); this repository's `.githooks/` are those of
its template release.
Each shim hands its hook to `nuc hook <name>`: `nuc` decides, and where a hook's check is due
the shim runs the project's `scripts/check-all.sh` or `scripts/check-docs.sh` itself. Without `nuc`
on the `PATH`, or with one older than `nuc.toml` requires, every shim refuses and names where to
install it, ending in the key `[shim/<hook>:missing]` or `[config/toml:old]`. They rely on the same
git settings as the template's hooks, listed under [Git hooks](CONTRIBUTING.md#git-hooks).

## Template

- **Template release:** v0.12.1 — the release of [NUC](https://github.com/hivevm/nuc) this
  repository carries; a project moves the line when it takes up a later one
  ([NUC-releases-are-semver-tags §5](docs/adr/NUC-releases-are-semver-tags.md)).

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the workflow (specification- and ADR-driven, small
reviewable changes), the project layout, the Dev Container, the coding agents, the git hooks, the
integration branches, and the repository settings, and [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md)
for the community standards we expect of everyone taking part. Security issues: please follow
[`SECURITY.md`](SECURITY.md) instead of opening a public issue.

## License

Released under the MIT License. See [`LICENSE`](LICENSE).
