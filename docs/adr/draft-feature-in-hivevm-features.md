# ADR-draft-feature-in-hivevm-features: `nuc` ships as attested GitHub Release binaries and a GitHub Action, pinned by major tag and updated by Dependabot, and its Dev Container Feature is published from `hivevm/features`

- **Status:** 🟡 proposed
- **Date:** 2026-11-07
- **Deciders:** Markus Brigl
- **Applies to:** the release workflow, the release tags of this repository, the release artifacts and their names, checksums, and attestations, the GitHub Action, and the absence of `feature/` and of any Feature publication from this repository
- **Supersedes:** [ADR-0017](0017-distribution.md)

## Context

`nuc` runs in the hooks and CI of every derived project, so a compromised artifact reaches all of
them; the published artifact is the trust boundary of the supply chain (*Threats*). Q-3 asks for
a SHA-256 checksum and a build-provenance attestation per artifact, and for a rebuild of the tag
to yield the published checksum per target. Q-4 names Linux x64 and arm64, macOS, and Windows x64.
The constraints name the distribution formats: GitHub Releases, the Dev Container Feature
specification, and GitHub Actions. The vision has a fix reach a project with a Dependabot update.

[ADR-0017](0017-distribution.md) decided the three channels and published the Feature from this
repository as `ghcr.io/hivevm/nuc-tools/nuc`. The maintainer wants the Dev Container Features of
several tools under one namespace, `ghcr.io/hivevm/features`, with `nuc` one of them. Several
repositories can push Features into one namespace, and each Feature stays installable by its ID,
but not its listing: each publish also pushes the namespace's collection metadata,
`devcontainer-collection.json`, tagged `latest`, built from the Features of the publishing
repository alone, so each repository would drop the others from the listing tools read to
discover them. One repository, `hivevm/features`, holds and publishes every Feature of the
namespace, as `devcontainers/features` does for its own. This repository then publishes what only
it can build: the binaries, their checksums and attestations, and the Action that runs them.

A release here cannot push into `hivevm/features`: that needs a token or an app with write access
to another repository, which [ADR-0019](0019-no-stored-secrets.md) rules out. So `hivevm/features`
pulls each release. What it relies on is the release as this repository publishes it, which
becomes an interface that changes only with a major release, so that a minor release of `nuc`
never breaks the Feature. How `hivevm/features` takes a release is its own record's to decide; the
maintainer's defaults for it, each open to change there, are a Renovate pull request per release
of `nuc`, which a human merges after its workflow has verified the attestations and written the
checksums, a Feature version equal to the version of `nuc` it installs, and an installer of its
own rather than one attached to each release.

## Decision

We will release `nuc` as binaries with checksums and build-provenance attestations on GitHub
Releases of this repository, and as a GitHub Action that installs the binary of its own release
after verifying its checksum, and leave its Dev Container Feature to `hivevm/features`, which
takes each release from here.

- **Tags and builds:** a human pushes a tag `vX.Y.Z`; the release workflow builds from it.
- **Targets:** `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl`, statically linked;
  `aarch64-apple-darwin`; `x86_64-pc-windows-msvc`.
- **Artifacts:** one binary per target, a `SHA256SUMS` file, and a build-provenance attestation
  per binary through `actions/attest-build-provenance`.
- **The release as an interface:** what another repository may rely on, and what changes only
  with a major release:
  - the release tag `vX.Y.Z` and the download location
    `https://github.com/hivevm/nuc-tools/releases/download/vX.Y.Z/<asset>`;
  - the asset name of each target's binary, `nuc-<target>`, with `.exe` on Windows;
  - `SHA256SUMS`, one line `<sha256>  <asset>` per binary;
  - an attestation per binary that `gh attestation verify --repo hivevm/nuc-tools
    --signer-workflow hivevm/nuc-tools/.github/workflows/release.yml` accepts.
- **GitHub Action:** `hivevm/nuc-tools@vN`; it installs the binary of its release, verifies the
  checksum, and runs `nuc` with the arguments it is given.
- **Release commit:** the commit a tag names cannot hold the checksums of the binaries built from
  it. After publishing, the release workflow writes `SHA256SUMS` and the version into the Action
  in a commit of its own on top of the tag, tags it `pinned/vX.Y.Z` so that it stays reachable,
  and moves the major tag `vN` to it unless `vN` already names a later release. The tag `vX.Y.Z`
  keeps naming the source commit the attestation names. Projects reference the Action by `vN`
  alone: a finer pin that Dependabot moves lands on a `vX.Y.Z` source commit, which, like `main`,
  carries no checksums, installs nothing, and says so.
- **Dev Container Feature:** this repository publishes none and no workflow here writes to
  `hivevm/features`; the Feature `nuc` is published there as `ghcr.io/hivevm/features/nuc`.
- **In the projects:** the Action and the Feature are referenced by major tag and updated by
  Dependabot; the minimum in `nuc.toml` holds the floor
  ([ADR-0023](0023-version-coupling-with-hook-defaults.md)).

**Out of scope:** how `hivevm/features` takes a release, verifies it, versions and tests its
Feature, and publishes, which its own record decides; crates.io, Homebrew, winget, and other
channels; macOS on x64; signing beyond the attestation; how this repository references the
actions it runs, a decision of its own.

## Alternatives considered

- **The Feature stays here, as ADR-0017 has it** — one release publishes all three channels, but
  under `ghcr.io/hivevm/nuc-tools`, not the namespace of all Features the maintainer wants.
- **This repository publishes into `ghcr.io/hivevm/features` itself** — through the action's
  namespace option, with no second repository; the Feature would install, but each repository
  publishing there rewrites the namespace's listing with its own Features alone, so the Features
  of other tools would drop out of it.
- **The release workflow pushes into `hivevm/features`** — no second workflow to keep, but it
  needs a stored token or an app with write access to another repository, against ADR-0019, and
  a compromised release would publish its Feature unreviewed.
- **`hivevm/features` polls on a schedule and publishes itself, or is started by hand per
  release** — for its record to weigh: the one publishes a release without a review there, the
  other is easily forgotten.
- **The installer attached to each release as an asset** — `hivevm/features` would keep no copy,
  but it becomes one more artifact to attest, and the Feature would download code to run where it
  carries its own.
- **The Feature or the Action downloading the latest release** — the version would move without a
  pull request in the project, and a compromised release would reach every project at once.
- **The installers fetching `SHA256SUMS` from the release** — no release commit and no moved tag,
  but the checksum would come from the same place as the binary, so a replaced release asset
  would pass with its own checksum.
- **macOS on x64 as well** — Intel Macs are going out of use, and each target costs a
  reproducible build (Q-3) and a CI leg (Q-4).
- **Linux linked against glibc** — ties the binary to the glibc of the build host, and makes the
  build harder to reproduce.
- **crates.io** — `cargo install` builds from source on the user's machine, with no attestation of
  what ran; a later channel if asked for.

## Sources / Prior art

- GitHub Docs, *Use artifact attestations* —
  <https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations>.
- Dev Container Features specification, *Distribution*, the collection metadata of a namespace,
  tagged `latest` —
  <https://containers.dev/implementors/features-distribution/>.
- `devcontainers/cli`, which packages the collection metadata from the publishing repository's
  source folder alone — <https://github.com/devcontainers/cli>.
- `devcontainers/features`, one repository that publishes many Features under one namespace —
  <https://github.com/devcontainers/features>.
- `devcontainers/action`, its `features-namespace` input —
  <https://github.com/devcontainers/action>.
- GitHub Docs, *Dependabot options reference*, the `devcontainers` and `github-actions`
  ecosystems —
  <https://docs.github.com/en/code-security/reference/supply-chain-security/dependabot-options-reference>.
- Renovate, the `github-releases` datasource, which tracks the releases of another repository —
  <https://docs.renovatebot.com/modules/datasource/github-releases/>.

## Consequences

- Positive: every artifact a project runs is still checked against a checksum fixed in a version
  it pinned, and traced to the workflow that built it (Q-3); the Features of several tools live in
  one namespace with one listing; this repository's release publishes two channels instead of
  three, and its workflow writes nowhere else; updates arrive as reviewable pull requests.
- Negative / trade-offs: a release of `nuc` is not in the Feature until `hivevm/features` takes
  it; that repository has a workflow and a record of its own to keep, and does not exist yet; the
  release becomes an interface that changes only with a major release; the Feature's installer
  lives in `hivevm/features` beside the Action's here, two copies that can drift, of which only
  the Action's is tested here; Intel Macs get no binary.
- Follow-ups: the record of `hivevm/features`; the release workflow and how it proves the rebuild
  of Q-3; whether the Action also verifies the attestation; further channels; the removal of
  `feature/` and the `feature` job from this repository, and of the Feature from the overview, the
  README, the project layout, and the repository settings; the wording of the constraint on
  distribution formats, which still holds, the Feature now published elsewhere.

## Enforcement

`scripts/test-release.sh`, cited with `Verifies:` and this ADR's identifier, runs in the local
gate and in the required checks: it checks the asset names `release/build.sh` and
`release/sums.sh` write against the literal list of the interface and each line of `SHA256SUMS`
against its form, checks that the Action's installer rejects a binary whose checksum differs, and
fails while `feature/` or a workflow step that publishes a Feature remains in this repository.
`release-test.yml` builds each target with `release/build.sh` on every pull request that changes
the release tooling. That the published release carries an attestation the command of the
interface accepts is checked by the release workflow after it publishes, and fails the release
otherwise.

**Not mechanically decidable:** that the interface changes only with a major release. The test
pins today's names and forms, so a change to them fails it, and the review of that change decides
whether it waits for a major release.
