# ADR-draft-feature-in-hivevm-features: `nuc` ships as attested GitHub Release binaries and a GitHub Action, and its Dev Container Feature is published from `hivevm/features`, which takes each release from here

- **Status:** 🟡 proposed
- **Date:** 2026-11-07
- **Deciders:** Markus Brigl
- **Applies to:** the release workflow, the release tags of this repository, the release artifacts and their names, checksums, and attestations, and the GitHub Action
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
several tools under one namespace, `ghcr.io/hivevm/features`, with `nuc` one of them. A
namespace cannot be shared by several repositories that each publish into it: the publishing
action writes the namespace's collection metadata, `devcontainer-collection.json`, with the
Features of its own repository only, so each repository would drop the others from it. One
repository, `hivevm/features`, holds and publishes every Feature of the namespace, as
`devcontainers/features` does for its own. This repository then publishes what only it can
build: the binaries, their checksums and attestations, and the Action that runs them.

A release here cannot push into `hivevm/features`: that needs a token or an app with write access
to another repository, which [ADR-0019](0019-no-stored-secrets.md) rules out. So `hivevm/features`
pulls: it learns of a release, verifies its attestations, writes its checksums into the Feature,
and publishes with its own workflow token. What it relies on is the release as this repository
publishes it, which becomes an interface: the names of the binaries, `SHA256SUMS`, and an
attestation per binary signed by this repository's release workflow.

Three choices the maintainer left to defaults, each open to change at review: a Renovate pull
request in `hivevm/features` for each release of `nuc`, which a human merges; the Feature's
version equal to the version of `nuc` it installs; and the interface named above, without the
installer as a release asset.

## Decision

We will release `nuc` as binaries with checksums and build-provenance attestations on GitHub
Releases of this repository, and as a GitHub Action that installs the binary of its own release
after verifying its checksum, and leave its Dev Container Feature to `hivevm/features`, which
takes each release from here.

- **Tags and builds:** a human pushes a tag `vX.Y.Z`; the release workflow builds from it.
- **Targets:** `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl`, statically linked;
  `aarch64-apple-darwin`; `x86_64-pc-windows-msvc`.
- **Artifacts:** one binary per target, named `nuc-<target>` with `.exe` on Windows, a
  `SHA256SUMS` file, and a build-provenance attestation per binary through
  `actions/attest-build-provenance`.
- **The release as an interface:** the asset names of the binaries and the name and format of
  `SHA256SUMS` change only with a major release, and every binary carries an attestation that
  `gh attestation verify --repo hivevm/nuc-tools --signer-workflow
  hivevm/nuc-tools/.github/workflows/release.yml` accepts. `hivevm/features` relies on nothing
  else.
- **GitHub Action:** `hivevm/nuc-tools@vN`; it installs the binary of its release, verifies the
  checksum, and runs `nuc` with the arguments it is given.
- **Release commit:** the commit a tag names cannot hold the checksums of the binaries built from
  it. After publishing, the release workflow writes `SHA256SUMS` and the version into the Action
  in a commit of its own on top of the tag, tags it `pinned/vX.Y.Z` so that it stays reachable,
  and moves the major tag `vN` to it unless `vN` already names a later release. The tag `vX.Y.Z`
  keeps naming the source commit the attestation names. Projects reference the Action by `vN`
  alone: a finer pin that Dependabot moves lands on a `vX.Y.Z` source commit, which, like `main`,
  carries no checksums, installs nothing, and says so.
- **Dev Container Feature:** none is published from this repository. `hivevm/features` carries the
  Feature `nuc`, published as `ghcr.io/hivevm/features/nuc`, each version installing the `nuc` of
  the same version and verifying it against the checksum that version carries. It takes a release
  by a pull request of its own, verifies the attestations before it writes the checksums, and
  publishes with its own workflow token; no workflow here writes to it.
- **In the projects:** the Feature and the Action are referenced by major tag; the minimum in
  `nuc.toml` holds the floor ([ADR-0023](0023-version-coupling-with-hook-defaults.md)).

**Out of scope:** how `hivevm/features` is built, tested, and released, which its own record
decides; crates.io, Homebrew, winget, and other channels; macOS on x64; signing beyond the
attestation; how this repository references the actions it runs, a decision of its own.

## Alternatives considered

- **The Feature stays here, as ADR-0017 has it** — one release publishes all three channels, but
  under `ghcr.io/hivevm/nuc-tools`, not the namespace of all Features the maintainer wants.
- **This repository publishes into `ghcr.io/hivevm/features` itself** — through the action's
  namespace option, with no second repository; but each repository publishing there rewrites the
  namespace's collection metadata with its own Features alone, so the Features of other tools
  would drop out of it.
- **The release workflow pushes into `hivevm/features`** — no second workflow to keep, but it
  needs a stored token or an app with write access to another repository, against ADR-0019, and
  a compromised release would publish its Feature unreviewed.
- **`hivevm/features` polls on a schedule and publishes itself** — no human step, but a release
  reaches every project's Feature without a review in the repository that publishes it.
- **`hivevm/features` started by hand per release** — the simplest pull, and the one most easily
  forgotten.
- **The Feature with versions of its own, naming the version of `nuc` as an option** — lets the
  installer change without a release of `nuc`, but a Feature version no longer says which `nuc`
  it installs.
- **The installer attached to each release as an asset** — `hivevm/features` keeps no copy, but
  it becomes one more artifact to attest, and the Feature would download code to run where today
  it carries its own.
- **The Feature or the Action downloading the latest release** — the version would move without a
  pull request in the project, and a compromised release would reach every project at once.
- **The installers fetching `SHA256SUMS` from the release** — the checksum would come from the
  same place as the binary, so a replaced release asset would pass with its own checksum.
- **macOS on x64 as well; Linux linked against glibc; crates.io** — as ADR-0017 weighed them: a
  target each costs a reproducible build and a CI leg, glibc ties the binary to the build host,
  and `cargo install` builds from source with no attestation of what ran.

## Sources / Prior art

- GitHub Docs, *Use artifact attestations* —
  <https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations>.
- Dev Container Features specification, *Distribution*, the collection metadata of a namespace —
  <https://containers.dev/implementors/features-distribution/>.
- `devcontainers/features`, one repository that publishes many Features under one namespace —
  <https://github.com/devcontainers/features>.
- `devcontainers/action`, its `features-namespace` input —
  <https://github.com/devcontainers/action>.
- Renovate, the `github-releases` datasource, which tracks the releases of another repository —
  <https://docs.renovatebot.com/modules/datasource/github-releases/>.

## Consequences

- Positive: every artifact a project runs is still checked against a checksum fixed in a version
  it pinned, and traced to the workflow that built it (Q-3); the Features of several tools live in
  one namespace with one collection; this repository's release publishes two channels instead of
  three, and its workflow writes nowhere else.
- Negative / trade-offs: a release of `nuc` is not in the Feature until a pull request in
  `hivevm/features` merges and publishes; that repository has a workflow and a record of its own
  to keep, and does not exist yet; the names of the release assets and `SHA256SUMS` become an
  interface that changes only with a major release.
- Follow-ups: the record of `hivevm/features`; whether the Action also verifies the attestation;
  the removal of `feature/` and the `feature` job from this repository, and of the Feature from the
  overview, the README, the project layout, and the repository settings; the wording of the
  constraint on distribution formats, which still holds, the Feature now published elsewhere.

## Enforcement

The release workflow's own test, cited with `Verifies:` and this ADR's identifier, runs on every
pull request that changes it: it builds each target, writes `SHA256SUMS`, checks that the Action's
installer rejects a binary whose checksum differs, and checks that the asset names are the ones
the interface names. That the published release carries an attestation `gh attestation verify`
accepts is checked by the release workflow after it publishes, and fails the release otherwise. A
test fails while `feature/` or a step publishing a Feature remains in this repository.
