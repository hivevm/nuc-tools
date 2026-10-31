# ADR-draft-action-references-by-sha: The workflows reference every action by full commit SHA with a version comment, kept current by Dependabot

- **Status:** 🟢 accepted
- **Date:** 2026-10-31
- **Deciders:** Markus Brigl
- **Applies to:** every `uses:` line under `.github/workflows/`, `.github/dependabot.yml`, and the run of `scripts/sensor-action-refs.sh` in `scripts/check-all.sh` and the `sensors` job

## Context

A compromised release pipeline is a threat to every derived project (*Threats*): the release
workflow of this repository holds a token that publishes releases, packages on `ghcr.io`, and
attestations ([ADR-draft-distribution](draft-distribution.md)), and every `uses:` line runs
third-party code with it. Q-3 puts the supply chain third among the quality goals. The template
ships its workflows with first-party actions by major version tag and Dependabot configured.

The catalogue entry *Action references* fits a major tag to first-party actions and few
maintainers, and a full commit SHA to a release pipeline or a token with write access; a tag can
be repointed and runs in CI before anyone sees a diff, as the tags of `tj-actions/changed-files`
were. SHA pins without a tool that updates them freeze security fixes out.

## Decision

We will reference every action the workflows run by its full commit SHA with the version as a
comment, kept current by Dependabot, and gate the form with `scripts/sensor-action-refs.sh
--require sha`.

- **Form:** `owner/action@<40-hex SHA> # vX.Y.Z`; a `docker://` image carries an explicit tag;
  local actions (`./…`) are exempt.
- **Updates:** `.github/dependabot.yml` keeps the `github-actions` ecosystem, which updates a pin
  with its comment, and adds the `cargo` ecosystem, so that the crates arrive as pull requests
  `scripts/check-deps.sh` audits ([ADR-draft-toolchain](draft-toolchain.md)).
- **Gate:** `scripts/sensor-action-refs.sh --require sha` runs in `scripts/check-all.sh` and in
  the `sensors` job.

**Out of scope:** how derived projects reference the Action and the Feature of this repository,
which [ADR-draft-distribution](draft-distribution.md) decides; workflow audits beyond the form of
the reference, such as zizmor's.

## Alternatives considered

- **Major version tag with Dependabot** — what the template ships; fixes arrive without a pull
  request, but a repointed tag runs with the release token before anyone sees it.
- **Tags for `actions/*`, SHAs for every other owner** — the check needs a list of trusted owners,
  and the release token is as exposed to a repointed first-party tag.
- **pinact instead of Dependabot** — a second tool where Dependabot, already configured, updates
  the pins with their comments.

## Sources / Prior art

- The catalogue entry
  [*Action references*](../../.agents/skills/propose-adr/concepts/action-references.md) and its
  sources, among them GitHub's *Security hardening for GitHub Actions* and advisory
  GHSA-mrrh-fwg8-r2c3 on `tj-actions/changed-files`.

## Consequences

- Positive: every change of the code that runs with the release token is a reviewed diff (Q-3).
- Negative / trade-offs: fixes within a major arrive as Dependabot pull requests to review, not by
  themselves; the workflows read less easily, which the version comment offsets.
- Follow-ups: a workflow audit such as zizmor, if a design revision finds a mistake it would have
  caught.

## Enforcement

`scripts/sensor-action-refs.sh --require sha` in `scripts/check-all.sh` and in the `sensors` job
fails on every reference in another form; the comment beside its `run` line cites this ADR with
`Verifies: ADR-draft-action-references-by-sha`.
