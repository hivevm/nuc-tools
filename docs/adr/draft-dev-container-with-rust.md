# ADR-draft-dev-container-with-rust: The Dev Container keeps the template's Debian base image, adds the Rust Feature, and reaches no containers

- **Status:** 🟢 accepted
- **Date:** 2026-10-31
- **Deciders:** Markus Brigl
- **Applies to:** `.devcontainer/devcontainer.json`, `.devcontainer/devcontainer-lock.json`, and the tools the container installs

## Context

The work runs in the Dev Container the template ships, under
[ADR-0004](0004-dev-container-runtime.md). The toolchain is Rust, pinned in `rust-toolchain.toml`
([ADR-draft-toolchain](draft-toolchain.md)); the parity harness needs bash 4.4 or later and the
GNU tools, which the Debian base image carries
([ADR-draft-parity-harness](draft-parity-harness.md)); the tests need git and no other
container ([ADR-draft-fixture-repositories](draft-fixture-repositories.md)).

## Decision

We will keep the template's Debian base image and add the Feature
`ghcr.io/devcontainers/features/rust:1`, locked in `.devcontainer/devcontainer-lock.json`, with no
container engine and no Compose services.

**Out of scope:** the Rust version, which `rust-toolchain.toml` sets and rustup installs; the
cargo tools the checks run, installed with the toolchain's job.

## Alternatives considered

- **A `Dockerfile` with Rust preinstalled** — one more file to maintain and update, where a
  Feature pinned by the lock does the same.
- **Compose or an engine inside** — nothing the tests run needs a container.

## Sources / Prior art

- The catalogue entry
  [*Container access*](../../.agents/skills/propose-adr/concepts/container-access.md).
- The Rust Feature — <https://github.com/devcontainers/features/tree/main/src/rust>.

## Consequences

- Positive: the container carries what the toolchain and the parity oracle need, and nothing
  else.
- Negative / trade-offs: the container is Linux; macOS and Windows are tested in CI only (Q-4).
- Follow-ups: none.

## Enforcement

`scripts/check-devcontainer.sh` holds the pin and the
lock of every Feature and refuses a host socket (ADR-0004). A test cited with
`Verifies: ADR-draft-dev-container-with-rust` reads `.devcontainer/devcontainer.json` and fails
unless it names the Debian base image and the Rust Feature, and on a `dockerComposeFile` or a
Feature that brings a container engine.
