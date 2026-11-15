# OPS-dev-container-carries-and-reaches: The Dev Container carries the Rust base and each check tool at the version CI runs, and reaches containers only on the host

- **Status:** 🟢 accepted
- **Date:** 2026-11-15
- **Deciders:** Markus Brigl
- **Applies to:** `.devcontainer/devcontainer.json`, `.devcontainer/devcontainer-lock.json`, `.devcontainer/install-shellcheck.sh`, `.devcontainer/compose.yaml`, every Compose file of a development tool run on the host, every tool the container installs, and the steps of `.github/workflows/checks.yml` that install the same tools
- **Supersedes:** [OPS-dev-container-adds-rust](OPS-dev-container-adds-rust.md)

## Context

The work runs in the Dev Container the template ships, under
[NUC-container-hides-host-daemon](NUC-container-hides-host-daemon.md), which keeps the host's
socket out of reach and leaves the base image, the Features, and the way to any container to the
project. The toolchain is Rust, pinned in `rust-toolchain.toml`
([OPS-rust-stable-behind-scripts](OPS-rust-stable-behind-scripts.md)); the parity harness needs
bash 4.4 or later and the GNU tools, which the Debian base image carries
([TEST-parity-harness](TEST-parity-harness.md)); the tests need git and no other container
([TEST-fixtures-through-real-git](TEST-fixtures-through-real-git.md)).

[OPS-dev-container-adds-rust](OPS-dev-container-adds-rust.md) decided the base image and the Rust
Feature, and "no container engine and no Compose services", and left the cargo tools out of scope.
Three things have moved since:

- **A stack on the host.** Since pull request #82 the repository carries
  `.devcontainer/compose.yaml`, a telemetry stack (an OpenTelemetry Collector, ClickHouse, and
  Grafana) that a developer starts on the host, apart from the container. The container reaches
  the Collector through the host's network, over a port the stack publishes. The container still
  has no engine, no socket, and no `dockerComposeFile`, and the old ADR's test passes; its title
  ("reaches no containers") no longer says what holds. The review of #82 found the conflict, and
  the maintainer left it as follow-up work.
- **ShellCheck.** The container installed ShellCheck from the base image's Debian archive and the
  shell job of CI used the runner image's, and the two reported a warning under different codes:
  the shell job failed on a script that was clean in the container (e81aead), which the pre-push
  hook cannot catch. Both now install one release, pinned by version and by the SHA-256 of its
  archives in `.devcontainer/install-shellcheck.sh`. That moves
  the source from a signed distribution archive to a release binary of the upstream project.
- **The cargo tools.** The container builds cargo-deny and cargo-audit with
  `cargo install --locked` at the versions the deps job of CI installs, so that
  `scripts/check-deps.sh` runs in the pre-push hook as it does in CI. No ADR decided it.

The force behind the last two is Q-1 of [`docs/SPECIFICATION.md`](../SPECIFICATION.md): the local
gate and CI decide the same way only where they run the same tools.

The three are one decision because they share one boundary: what the Dev Container carries
itself, and what it reaches outside itself. A tool the checks run is carried, at CI's version; a
tool that needs containers is reached, on the host. Each clause below places a tool on one side
of that boundary, and one *Applies to* routes every change of it here.

## Decision

We will have the Dev Container carry the Debian base, the Rust Feature, and each check tool at
CI's version, start no container, and reach a tool that needs containers only on the host.

1. **Base.** The definition names the base image `mcr.microsoft.com/devcontainers/base:debian`
   and the Feature `ghcr.io/devcontainers/features/rust:1`. The Rust version is the one
   `rust-toolchain.toml` sets, installed by rustup on the first cargo call.
2. **Pinned tools.** A tool the checks run that neither the base image nor a Feature brings is
   installed at a pinned version, the same in the container and in CI:
   - ShellCheck, from a release archive of koalaman/shellcheck. Its version and SHA-256 are
     written only in `.devcontainer/install-shellcheck.sh`. The container and the shell job both
     install through that script, and the shell job lints only after verifying the version on
     its PATH.
   - cargo-deny and cargo-audit, built with `cargo install --locked`. Their versions are written
     twice, in the deps job of `.github/workflows/checks.yml` and in the `postCreateCommand` of
     the definition, and a test holds the two equal.
3. **Updates.** A pinned version moves by hand, when the maintainer decides. One pull request
   changes it in the container and in CI together. No automated update sees these pins.
4. **No container from the container.** The definition adds no container engine, no Feature that
   brings one, and no `dockerComposeFile`; the Dev Container starts no container.
5. **Stacks on the host.** A development tool that needs containers runs as a Compose project of
   its own on the host, which a developer starts and stops there. The container reaches it only
   over the host's network, never through an engine. Each port the stack publishes is bound to a
   host address written in its Compose file, not to every interface of the host. Nothing the
   project ships, and no check or hook, depends on such a stack. The one stack today is the
   telemetry stack of `.devcontainer/compose.yaml`.

**Out of scope:**

- the host socket and the lock of the Features, which
  [NUC-container-hides-host-daemon](NUC-container-hides-host-daemon.md) decides;
- the GitHub CLI and its wrapper, a Feature and a file of the project that no check runs;
- what the telemetry stack stores and shows, which `.devcontainer/README.md` describes.

## Alternatives considered

- **A `Dockerfile` with Rust and the tools preinstalled** — one more file to maintain and update,
  where a Feature pinned by the lock and a pinned install on creation do the same.
- **ShellCheck from the Debian archive, as before** — a signed source, but its version follows the
  base image, and the runner image's follows GitHub: the two drifted apart once already
  (e81aead).
- **The runner's ShellCheck, failing CI when its version differs from the container's** — nothing
  to download, but CI goes red whenever GitHub updates the runner image, and someone must follow.
  The maintainer chose the pin.
- **Each tool at its latest release** — no pin to keep current, but the local gate and CI drift
  apart on every release between two container builds.
- **A sensor that reports a newer release of a pinned tool** — it would read GitHub and crates.io,
  so it could run only on an instruction; with two tools pinned, it does not pay for itself yet.
- **Compose services of the Dev Container (`dockerComposeFile`)** — the telemetry stack would
  start with every container, and the container would carry Compose services the tests do not
  need; a stack on the host starts only when someone wants to look.
- **An engine inside the container** — nothing the tests run needs a container, and the
  Docker-in-Docker Feature runs the container privileged.
- **Only the telemetry stack on the host, any other stack by an ADR of its own** — every stack on
  the host meets the same conditions of clause 5, so one clause covers them: a check decides the
  address of a stack under `.devcontainer/`, the review the rest.
- **One ADR for the carried tools and one for the stacks on the host** — each would cover half of
  the container's boundary, and a change that moves a tool from one side to the other would touch
  both.

## Sources / Prior art

- The catalogue entry
  [*Container access*](../../.agents/skills/propose-adr/concepts/container-access.md): no
  container access, Compose services, an engine inside.
- The Rust Feature — <https://github.com/devcontainers/features/tree/main/src/rust>.
- ShellCheck's releases — <https://github.com/koalaman/shellcheck/releases>: the archives carry
  no signature; the checksums in the install script are the release's asset digests, which agreed
  with the downloaded archives on 2026-11-15.
- `cargo install --locked` — <https://doc.rust-lang.org/cargo/commands/cargo-install.html>: builds
  with the crate's own `Cargo.lock`.
- Compose, published ports — <https://docs.docker.com/reference/compose-file/services/#ports>: a
  host address before the port binds it to that address alone.

## Consequences

- Positive: the pre-push hook and CI lint and audit with the same versions (Q-1); the container
  carries what the toolchain, the parity oracle, and the local gate need; the telemetry stack has a
  record that says where it runs and how the container reaches it.
- Negative / trade-offs: ShellCheck's source is trusted on first use, not signed; the pins age
  until someone moves them; a container built before a change of a pin keeps the old version until
  it is rebuilt; the container is Linux, so macOS and Windows are tested in CI only (Q-4).
- Follow-ups:
  - Once this ADR is accepted, the comment of `devcontainer.json` and the test of the definition
    move from OPS-dev-container-adds-rust to it; the header of that test also drops its stale
    numbered reference.
  - [SEC-gates-never-run-build](SEC-gates-never-run-build.md) is accepted and keeps its citation
    of OPS-dev-container-adds-rust. The superseded ADR's file stays, so the link holds.

## Enforcement

`scripts/check-devcontainer.sh` holds the pin and the lock of every Feature and refuses a host
socket (NUC-container-hides-host-daemon §2, §3). Three tests will cite
`Verifies: OPS-dev-container-carries-and-reaches`:

- The test of the definition in `tests/devcontainer.rs`, extended, reads
  `.devcontainer/devcontainer.json`. It fails unless the definition names the Debian base image
  and the Rust Feature (clause 1). It fails on a `dockerComposeFile` or a Feature that brings a
  container engine (clause 4). It fails unless each `cargo install` of the `postCreateCommand`
  carries `--locked` and the version the deps job of `.github/workflows/checks.yml` installs
  (clause 2).
- `scripts/test-install-shellcheck.sh` holds that the container and the shell job install
  ShellCheck through the one script, that the shell job verifies the version before it lints, and
  that the script refuses an archive with another checksum (clause 2).
- `scripts/test-check-compose.sh` holds `scripts/check-compose.sh` to its cases. The check reads
  every Compose file under `.devcontainer/`. It fails on a published port without a host address,
  on one whose address can be empty or names every interface, on a form it cannot read, and on
  `network_mode: host` (clause 5).

**Not mechanically decidable:** three parts, each a judgement no test can read from the tree:

- the address of a port a stack outside `.devcontainer/` publishes (clause 5), which the check
  does not read;
- that nothing shipped and no check depends on a stack on the host (clause 5);
- when a pin moves (clause 3).

The review reads the first two against the diff; the third is the maintainer's.
