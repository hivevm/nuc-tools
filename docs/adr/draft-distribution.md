# ADR-draft-distribution: `nuc` ships as attested GitHub Release binaries, a Dev Container Feature, and a GitHub Action, pinned by major tag and updated by Dependabot

- **Status:** 🟡 proposed
- **Date:** 2026-10-30
- **Deciders:** Markus Brigl
- **Applies to:** the release workflow, the release tags of this repository, the release artifacts and their checksums and attestations, the Dev Container Feature and its publication, and the GitHub Action

## Context

`nuc` runs in the hooks and CI of every derived project, so a compromised artifact reaches all of
them; the published artifact is the trust boundary of the supply chain (*Threats*). Q-3 asks for
a SHA-256 checksum and a build-provenance attestation per artifact, and for a rebuild of the tag
to yield the published checksum per target. Q-4 names Linux x64 and arm64, macOS, and Windows x64.
The constraints name the distribution formats: GitHub Releases, the Dev Container Feature
specification, and GitHub Actions. The vision has a fix reach a project with a Dependabot update.

## Decision

We will release `nuc` as binaries with checksums and build-provenance attestations on GitHub
Releases of this repository, and as a Dev Container Feature on `ghcr.io` and a GitHub Action that
each install the binary of their own release after verifying its checksum.

- **Tags and builds:** a human pushes a tag `vX.Y.Z`; the release workflow builds from it.
- **Targets:** `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl`, statically linked;
  `aarch64-apple-darwin`; `x86_64-pc-windows-msvc`.
- **Artifacts:** one binary per target, a `SHA256SUMS` file, and a build-provenance attestation
  per binary through `actions/attest-build-provenance`.
- **Dev Container Feature:** `ghcr.io/hivevm/nuc-tools/nuc`; each version installs the `nuc` of
  the same version and verifies it against the checksum the Feature carries; a mismatch fails the
  installation.
- **GitHub Action:** `hivevm/nuc-tools@vN`; it installs the binary of its release, verifies the
  checksum, and runs `nuc` with the arguments it is given.
- **In the projects:** the Feature and the Action are referenced by major tag and updated by
  Dependabot; the minimum in `nuc.toml` holds the floor
  ([ADR-draft-version-coupling](draft-version-coupling.md)).

**Out of scope:** crates.io, Homebrew, winget, and other channels; macOS on x64; signing beyond
the attestation; how this repository references the actions it runs, a decision of its own.

## Alternatives considered

- **macOS on x64 as well** — Intel Macs are going out of use, and each target costs a
  reproducible build (Q-3) and a CI leg (Q-4).
- **Linux linked against glibc** — ties the binary to the glibc of the build host, and makes the
  build harder to reproduce.
- **The Feature or the Action downloading the latest release** — the version would move without a
  pull request in the project, and a compromised release would reach every project at once.
- **crates.io** — `cargo install` builds from source on the user's machine, with no attestation of
  what ran; a later channel if asked for.

## Sources / Prior art

- GitHub Docs, *Use artifact attestations* —
  <https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations>.
- Dev Container Features specification, *Distribution* —
  <https://containers.dev/implementors/features-distribution/>.
- GitHub Docs, *Dependabot options reference*, the `devcontainers` and `github-actions`
  ecosystems —
  <https://docs.github.com/en/code-security/reference/supply-chain-security/dependabot-options-reference>.

## Consequences

- Positive: every artifact a project runs is checked against a checksum fixed in a version it
  pinned, and can be traced to the workflow that built it (Q-3); updates arrive as reviewable
  pull requests.
- Negative / trade-offs: three channels to release in step; the Feature and the Action carry the
  checksums of their release, so a release is not done until both are published; Intel Macs get no
  binary.
- Follow-ups: the release workflow and how it proves the rebuild of Q-3; whether the Action also
  verifies the attestation; further channels.

## Enforcement

The release workflow's own test, cited with `Verifies: ADR-draft-distribution`, runs on every
pull request that changes it: it builds each target, writes `SHA256SUMS`, and checks that the
Feature's and the Action's installers reject a binary whose checksum differs. That the published
release carries an attestation `gh attestation verify` accepts is checked by the release workflow
after it publishes, and fails the release otherwise.
