# Ticket: The telemetry stack publishes each port on a named address

## What to build

Every port `.devcontainer/compose.yaml` publishes names the host address it binds to, so that the
development telemetry stack never listens on every interface of the host. The first review of
pull request #82 found the stack published on every interface with an anonymous Admin in
Grafana; b4d0f64 bound each port to `${OBSERVABILITY_BIND:-127.0.0.1}`, and nothing holds that
since. A check of the project, run by `scripts/check-all.sh` and CI, fails on a `ports:` entry of
a Compose file under `.devcontainer/` that has no host address, or whose address can be empty or
`0.0.0.0`.

## Acceptance criteria

- [ ] Given `.devcontainer/compose.yaml` as it stands, when the check runs, then it passes.
- [ ] Given a port entry `"4318:4318"` or `"0.0.0.0:4318:4318"`, when the check runs, then it
      fails and names the file, the line, and the entry.
- [ ] Given a port entry `"${OBSERVABILITY_BIND}:4318:4318"` without a default, when the check
      runs, then it fails: an unset variable publishes on every interface.
- [ ] Given the check, when `scripts/check-all.sh` and `checks.yml` are read, then both run it
      and its self-test, and the rule `docs/check-lists` passes.

## Read first

- Touches: `scripts/check-dashboards.sh` or a new `scripts/check-compose.sh` with its self-test,
  `scripts/check-all.sh`, `.github/workflows/checks.yml`, `CONTRIBUTING.md` (required checks)
- `scripts/check-workflows.sh` and `scripts/test-check-workflows.sh` as the golden path of a
  project check over files under one directory, `--root` through `scripts/lib/root.sh`.
- `.devcontainer/README.md`, "Start", for why the stack binds the loopback.

## Blocked by

- None (can start immediately)

## Mode

with the human — DEV-one-agent-one-ticket allows no unattended ticket, and a new check changes
the required checks of the ruleset on `main`.
