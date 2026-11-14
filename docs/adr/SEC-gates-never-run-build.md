# SEC-gates-never-run-build: This repository's gates never run the `nuc` being built, only the template's bash or a released `nuc` pinned like any dependency

- **Status:** 🟢 accepted
- **Date:** 2026-10-31
- **Deciders:** Markus Brigl
- **Applies to:** `scripts/check-all.sh`, the hooks under `.githooks/`, the jobs of `.github/workflows/checks.yml`, and every place they name the tool that runs a check

## Context

This repository builds the tool that derived projects run as their gates, and it has gates of its
own: the template's checks and hooks ([NUC-checks-are-bash](NUC-checks-are-bash.md)). A gate that
ran the `nuc` built from the change under review would judge that change with itself: a broken or
tampered build could pass itself, the silent pass Q-1 forbids, and a compromised change would reach
`main` through a gate it wrote (*Threats*). The bash scripts stay in the template until `nuc`
reaches parity (*Constraints*), and NUC-checks-are-bash §1 has the checks be bash scripts under
`scripts/`.

## Decision

We will run this repository's gates only with the template's bash scripts or with a released
`nuc` pinned as a dependency, never with a `nuc` built from the working tree or the change under
review.

- **Now:** the gates are the template's bash scripts, as NUC-checks-are-bash §1 decides.
- **Later:** a released `nuc` may replace them, installed in CI by the local
  `release/install-nuc.sh` from the `SHA256SUMS` and `VERSION` of a release committed in this
  repository and moved by a pull request, and in the Dev Container by its Feature pinned by the lock
  ([OPS-dev-container-adds-rust](OPS-dev-container-adds-rust.md)). Not through this repository's own
  Action: the SHA pin [SEC-actions-pinned-by-sha](SEC-actions-pinned-by-sha.md) asks for would be
  moved by Dependabot to a source commit `vX.Y.Z`, which carries no checksums and installs nothing
  ([OPS-ships-binaries-feature-action](OPS-ships-binaries-feature-action.md)). The switch is a
  decision of its own, an ADR that narrows the bash clause as NUC-checks-are-bash §1 and §8 say,
  and is taken only once the released `nuc` reaches parity for every check these gates run.
- **Never:** `cargo run`, a binary under `target/`, or any other build of the working tree as a
  gate. The tests of the build run the build, `scripts/check-build.sh` and the parity harness
  `scripts/check-parity.sh`; they are not the gate's judge of it.

**Out of scope:** when the switch comes and which release it pins; the gates of derived projects.

## Alternatives considered

- **The switch fixed at v0.1 now** — simpler, but it contradicts NUC-checks-are-bash §1 while that
  clause is in force, and a v0.1 that does not cover every check would leave part of the gates
  unchecked.
- **The gates run the binary being built** — the change judges itself; a broken build passes, and
  a malicious one waves itself through.

## Sources / Prior art

- [NUC-checks-are-bash](NUC-checks-are-bash.md): the checks, the hooks, and the local gate.
- Compiler bootstrapping, in which a compiler is built by its previous release rather than by
  itself.

## Consequences

- Positive: a change is always judged by code that was released and reviewed before it (Q-1, Q-3).
- Negative / trade-offs: a fix to `nuc` protects this repository only after it is released; the
  bash scripts and their dependency on bash 4.4 stay here until the switch.
- Follow-ups: the switch-over, once a released `nuc` reaches parity for every check of these gates.

## Enforcement

A check cited with `Verifies: SEC-gates-never-run-build` reads `scripts/check-all.sh`,
the hooks under `.githooks/`, and `.github/workflows/checks.yml`, and fails on `cargo run`, a
path under `target/`, or a `nuc` invoked without a released, pinned source, outside
`scripts/check-build.sh` and `scripts/check-parity.sh`, which run the build's own tests.
