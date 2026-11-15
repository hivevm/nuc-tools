# SEC-no-stored-secrets: The workflows use no stored secret, only `GITHUB_TOKEN` and OIDC with permissions set per job

- **Status:** 🟢 accepted
- **Date:** 2026-10-31
- **Deciders:** Markus Brigl
- **Applies to:** every workflow under `.github/workflows/`, its `permissions`, and every reference to `secrets.` in them

## Context

The specification names tokens as an asset (*Threats*). The release publishes binaries, their
checksums, and attestations, and pushes a release commit and its tags
([OPS-feature-in-hivevm-features](OPS-feature-in-hivevm-features.md)). The short-lived
`GITHUB_TOKEN` and the workflow's OIDC token can do all of it. The project has no credential of
its own shape. The Dev Container Feature of `nuc` is published from `hivevm/features`, which pulls
each release, so no workflow here needs a token for another repository. The catalogue entry
*Secrets* fits the baseline to one developer with few credentials, all in GitHub's scanning
patterns. Secret scanning with push protection is among the repository settings `CONTRIBUTING.md`
lists.

## Decision

We will store no secret for this repository and have its workflows authenticate only with
`GITHUB_TOKEN` and OIDC, with `permissions` set per job to what that job needs.

**Out of scope:** the tokens of the environments `nuc` runs in, which Q-10 and
[SEC-git-cleared-environment](SEC-git-cleared-environment.md) protect; a workflow that needs a
stored secret, which is a decision of its own.

## Alternatives considered

- **A scanner in CI (gitleaks, TruffleHog)** — for credentials of the project's own shape. There
  are none, and push protection covers GitHub's.
- **A personal access token for the release** — long-lived and broader than the job needs. The
  `GITHUB_TOKEN` and OIDC suffice.

## Sources / Prior art

- The catalogue entry [*Secrets*](../../.agents/skills/propose-adr/concepts/secrets.md) and its
  sources.

## Consequences

- Positive: no long-lived credential exists to leak. Each job holds only the permissions it names.
- Negative / trade-offs: a channel that needs a stored token, such as crates.io, needs a new
  decision first. So does a push into another repository, which is why `hivevm/features` pulls
  the releases.
- Follow-ups: none.

## Enforcement

`scripts/check-workflows.sh` runs in `scripts/check-all.sh` and in the `gates` job of `checks.yml`.
Its self-test `scripts/test-check-workflows.sh` cites this ADR with
`Verifies: SEC-no-stored-secrets`. The check reads every workflow under `.github/workflows/`. It
fails on the word `secrets` other than as `secrets.GITHUB_TOKEN`, on a `secrets:` key, and on a job
without a `permissions` key of its own. That no secret is stored in the repository's settings is
outside the tree; the check of the workflows holds the decision where the tree can.
