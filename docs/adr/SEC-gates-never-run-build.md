# SEC-gates-never-run-build: This repository's gates never run the `nuc` being built, only the template's bash or a released `nuc` pinned like any dependency

- **Status:** ⚪ superseded by SEC-gates-run-released-nuc
- **Date:** 2026-11-18
- **Deciders:** Markus Brigl
- **Applies to:** `scripts/check-all.sh`, the hooks under `.githooks/`, the jobs of `.github/workflows/checks.yml`, and every place they name the tool that runs a check

## Context

This repository builds the tool that derived projects run as their gates. It has gates of its own:
the template's checks and hooks ([NUC-checks-are-bash](NUC-checks-are-bash.md)). A gate that ran
the `nuc` built from the change under review would judge that change with itself. A broken or
tampered build could pass itself, the silent pass Q-1 forbids. A compromised change would reach
`main` through a gate it wrote (*Threats*). The bash scripts stay in the template until `nuc`
reaches parity (*Constraints*). NUC-checks-are-bash §1 has the checks be bash scripts under
`scripts/`. Under NUC-checks-are-bash §8 a project runs a binary in place of the scripts only
through an ADR that narrows NUC-checks-are-bash §1, and the binary passes the fixture tables under
`scripts/fixtures/`. No project ADR supersedes a `NUC-` clause
([NUC-base-is-fixed §1](NUC-base-is-fixed.md)).

A released `nuc` reaches a machine in three ways
([OPS-feature-in-hivevm-features](OPS-feature-in-hivevm-features.md)). The release commit carries
`SHA256SUMS` and `VERSION`, and `release/install-nuc.sh` installs the binary they name after
verifying its checksum. The Action `hivevm/nuc-tools@vN` runs that installer. The Feature
`ghcr.io/hivevm/features/nuc` comes from `hivevm/features`.

## Decision

We will run this repository's gates only with the template's bash scripts or with a released
`nuc` pinned as a dependency, never with a `nuc` built from the working tree or the change under
review.

- **Now:** the gates are the template's bash scripts, as NUC-checks-are-bash §1 decides.
- **Later:** a released `nuc` may replace them. CI installs it with the local
  `release/install-nuc.sh`, from the `SHA256SUMS` and `VERSION` of a release committed in this
  repository and moved by a pull request. The Dev Container installs it by the Feature
  `ghcr.io/hivevm/features/nuc`
  ([OPS-feature-in-hivevm-features](OPS-feature-in-hivevm-features.md)), pinned by the lock
  ([NUC-container-hides-host-daemon](NUC-container-hides-host-daemon.md)).
  Not through this repository's own Action: the SHA pin
  [SEC-actions-pinned-by-sha](SEC-actions-pinned-by-sha.md) asks for would be moved by Dependabot
  to a source commit `vX.Y.Z`, which carries no checksums and installs nothing
  ([OPS-feature-in-hivevm-features](OPS-feature-in-hivevm-features.md)). The switch is a decision
  of its own, an ADR that narrows NUC-checks-are-bash §1 under NUC-checks-are-bash §8. It is
  taken only once the released `nuc` reaches parity for every check these gates run.
- **Never:** `cargo run`, a binary under `target/`, or any other build of the working tree as a
  gate. The tests of the build run the build, `scripts/check-build.sh` and the parity harness
  `scripts/check-parity.sh`; they are not the gate's judge of it.

**Out of scope:** when the switch comes and which release it pins; the gates of derived projects.

## Alternatives considered

- **The switch fixed at v0.1 now** — simpler, but it narrows NUC-checks-are-bash §1 before a
  released `nuc` can stand in for every check. A v0.1 that does not cover every check would leave
  part of the gates unchecked.
- **The gates run the binary being built** — the change judges itself. A broken build passes, and
  a malicious one waves itself through.
- **The switch as a supersession of NUC-checks-are-bash §1** — the record's first wording of the
  later clause. No project ADR supersedes a `NUC-` clause; it narrows one
  ([NUC-base-is-fixed §1](NUC-base-is-fixed.md), NUC-checks-are-bash §8). The decision stays; the
  form of the switch follows the fixed base.

## Sources / Prior art

- [NUC-checks-are-bash](NUC-checks-are-bash.md): the checks, the hooks, the local gate, and the
  binary in place of the scripts.
- Compiler bootstrapping, in which a compiler is built by its previous release rather than by
  itself.

## Consequences

- Positive: a change is always judged by code that was released and reviewed before it (Q-1, Q-3).
- Negative / trade-offs: a fix to `nuc` protects this repository only after it is released. The
  bash scripts and their dependency on bash 4.4 stay here until the switch. The bash checks read
  every text file, the tests included, so a test assembles the references it writes, as
  `src/testing.rs` and `tests/support/` do.
- Follow-ups: the switch-over, once a released `nuc` reaches parity for every check of these gates.

## Enforcement

`scripts/check-gates.sh` runs in `scripts/check-all.sh` and in the `gates` job of `checks.yml`. Its
self-test `scripts/test-check-gates.sh` cites this ADR with `Verifies: SEC-gates-never-run-build`.
The check reads `scripts/check-all.sh`, the hooks under `.githooks/`,
`.github/workflows/checks.yml`, and every script under `scripts/`. It fails on `cargo run`, on
`cargo install --path`, on a path under `target/`, and on `nuc` run as a command. It does not read
`scripts/check-build.sh` and `scripts/check-parity.sh`, which run the build's own tests, nor itself
and its self-test. A command assembled from variables is left to review. When a released, pinned
`nuc` becomes a gate, the check changes with that decision.
