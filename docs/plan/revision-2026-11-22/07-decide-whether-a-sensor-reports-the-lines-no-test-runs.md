# Ticket: Decide whether a sensor reports the lines a change adds that no test runs

## What to build

Each of the five tickets of `nuc-upgrade` took two rounds of review. In four of them, the first
round's must-fix or lesser findings were branches no test ran:

- ticket 02: three refusals no run of the binary decided;
- ticket 03: the plan deleting `README.md`, and the line moved into the new release's bytes,
  untested (left open);
- ticket 04: no test held a copy of another record listed and left, and the unit test did not
  decide the de-duplication;
- ticket 05: no self-test of the replay's failure branches, the setup branch untested (left open).

A round of review costs a run of `scripts/check-all.sh`, 923 s in the last run, the parity
harness 919 s of it. A report of the lines a change adds that no test runs would show these
branches before the review. The cause is in the project's own checks, not the template's.

The maintainer weighs, through `propose-adr`:

- a sensor over `cargo llvm-cov` or another tool for the Rust sources, reporting the lines a
  change adds that no test runs, run on instruction or in `scripts/check-all.sh`;
- what it costs: a tool in the supply chain (Q-3) and in the Dev Container, and the time of an
  instrumented test run;
- the bash under `scripts/lib/`, for which no such tool is in place;
- leaving it to review.

## Acceptance criteria

- [ ] Given the maintainer's choice, when the session lands it, then a `proposed` ADR names the
  tool, where the sensor runs, and what it reports, or the ticket is deleted with the reason.

## Read first

- Touches: `docs/adr/`, `scripts/`, `.devcontainer/`
- NUC-checks-are-bash (a sensor's form); OPS-rust-stable-behind-scripts, the ADR in the role
  `toolchain`; the ADRs `scripts/sensor-binding-adrs.sh --sections --path scripts/` lists; Q-3 in
  `docs/SPECIFICATION.md`; the skill `propose-adr`

## Blocked by

- None (can start immediately)

## Mode

with the human: a new tool and a sensor are decisions the human takes.
