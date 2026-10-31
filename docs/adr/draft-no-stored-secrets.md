# ADR-draft-no-stored-secrets: The workflows use no stored secret, only `GITHUB_TOKEN` and OIDC with permissions set per job

- **Status:** 🟢 accepted
- **Date:** 2026-10-31
- **Deciders:** Markus Brigl
- **Applies to:** every workflow under `.github/workflows/`, its `permissions`, and every reference to `secrets.` in them

## Context

The specification names tokens as an asset (*Threats*). The release publishes binaries,
packages on `ghcr.io`, and attestations ([ADR-draft-distribution](draft-distribution.md)), all of
which the short-lived `GITHUB_TOKEN` and the workflow's OIDC token can do; the project has no
credential of its own shape. The catalogue entry *Secrets* fits the baseline to one developer
with few credentials, all in GitHub's scanning patterns; secret scanning with push protection is
among the repository settings.

## Decision

We will store no secret for this repository and have its workflows authenticate only with
`GITHUB_TOKEN` and OIDC, with `permissions` set per job to what that job needs.

**Out of scope:** the tokens of the environments `nuc` runs in, which Q-10 and
[ADR-draft-git-cli-cleared-environment](draft-git-cli-cleared-environment.md) protect; a workflow
that needs a stored secret, which is a decision of its own.

## Alternatives considered

- **A scanner in CI (gitleaks, TruffleHog)** — for credentials of the project's own shape; there
  are none, and push protection covers GitHub's.
- **A personal access token for the release** — long-lived and broader than the job needs; the
  `GITHUB_TOKEN` and OIDC suffice.

## Sources / Prior art

- The catalogue entry [*Secrets*](../../.agents/skills/propose-adr/concepts/secrets.md) and its
  sources.

## Consequences

- Positive: no long-lived credential exists to leak; each job holds only the permissions it names.
- Negative / trade-offs: a channel that needs a stored token, such as crates.io, needs a new
  decision first.
- Follow-ups: none.

## Enforcement

A check over `.github/workflows/`, cited with `Verifies: ADR-draft-no-stored-secrets`, fails on a
reference to `secrets.` other than `secrets.GITHUB_TOKEN` and on a job without a `permissions`
key. That no secret is stored in the repository's settings is outside the tree; the check of the
workflows is what holds the decision there.
