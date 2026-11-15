# Ticket: The stack's README names the host's address under the bridge bind

<!-- A ticket of the design revision of 2026-11-14, in the shape of the plan-feature skill's
     ticket template, without a spec. Claim it by creating the branch
     ticket/revision-2026-11-14/<NN>-<slug> BEFORE any work, by the steps of the work-tickets
     skill. The branch that lands it deletes this file. Its sections are the parts a ticket
     carries, as docs/adr/NUC-large-work-is-planned.md decides (clause 3). -->

## What to build

The table of endpoints in `.devcontainer/README.md` sends a session or a browser on the host to
`127.0.0.1` and `localhost`. On Docker Engine on Linux, where the stack is started with
`OBSERVABILITY_BIND=172.17.0.1`, nothing listens on the loopback, and the host reaches the stack
at `172.17.0.1` as well. The second review of #84 found it and left it as out of the take-up's
scope. The README says which address the host uses under each bind, in its table and in
"Checking that data arrives".

## Acceptance criteria

- [ ] Given Docker Engine on Linux and the stack started with `OBSERVABILITY_BIND=172.17.0.1`,
      when the README is read, then its table names `172.17.0.1` for the host, for OTLP, Grafana,
      and ClickHouse.
- [ ] Given the default bind, when the README is read, then it still names `127.0.0.1` for the
      host.

## Read first

- Touches: `.devcontainer/README.md`
- `.devcontainer/compose.yaml` (the published ports); "Measuring a session" of `CONTRIBUTING.md`.

## Blocked by

- None (can start immediately)

## Mode

with the human — DEV-one-agent-one-ticket allows no unattended ticket.
