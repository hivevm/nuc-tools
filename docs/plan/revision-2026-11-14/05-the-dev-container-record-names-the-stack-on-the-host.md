# Ticket: The record of the Dev Container names the telemetry stack on the host

## What to build

OPS-dev-container-adds-rust says that the Dev Container "reaches no containers" and has "no
container engine and no Compose services". Since pull request #82 the repository carries
`.devcontainer/compose.yaml`, a stack that runs on the host apart from the Dev Container, and the
Dev Container reaches its Collector through the host's network. The review of #82 found the
conflict, and the human chose to leave it as follow-up work. What still holds: the Dev Container
has no engine socket (NUC-container-hides-host-daemon) and no `dockerComposeFile`, and the ADR's
test passes. Propose, through `propose-adr`, an ADR that supersedes OPS-dev-container-adds-rust
and says what holds: the container starts no container, and a development tool may run on the
host beside it, reached over the host's network, with nothing shipped depending on it.

## Acceptance criteria

- [ ] Given the proposed ADR, when it is read, then it supersedes OPS-dev-container-adds-rust,
      keeps each of its decisions that still holds, and names the stack and how the container
      reaches it.
- [ ] Given the proposal, when it lands, then it stands in a commit of ADRs only, and the human
      decides it.

## Read first

- Touches: `docs/adr/`
- OPS-dev-container-adds-rust in full; NUC-container-hides-host-daemon; `.devcontainer/README.md`,
  "Reaching it from the Dev Container"; the Doubts of pull request #82.

## Blocked by

- None (can start immediately)

## Mode

with the human — an ADR is the human's to accept.
