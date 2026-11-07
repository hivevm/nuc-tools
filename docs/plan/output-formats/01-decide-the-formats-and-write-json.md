# Ticket: Decide the formats of G-7 and write the findings as JSON

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

First a draft ADR through `propose-adr` that decides the formats of G-7 as the spec's
*Implementation decisions* record them: `--format text|json|sarif` for `nuc check` only, the JSON
document and its versioning, the SARIF 2.1.0 shape, no crate for either, and nothing on the
standard output when the run cannot decide. It is handed to the human in a commit of ADRs only
([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules), rules 3 and 4); no code is written before the
human has reviewed it.

Then the tracer: `nuc check --format json` writes the findings of the run as the one document the
ADR decides, through a JSON writer in the library, and `--format text` and no option write the
lines of today. The command line accepts the option before or after `--repo`, once; an unknown
format or a second `--format` is a usage error, exit 2.

## Acceptance criteria

- [ ] Given the draft ADR, when the human has reviewed it, then the implementation follows it on
  the same branch, or the ticket is recorded as declined with the reason.
- [ ] Given a fixture with findings, one of them without a line and one whose message holds a
  quote, a backslash, a tab, and a non-ASCII letter, when `nuc check --format json` runs, then it
  exits 1, the standard output is exactly the document the ADR decides with those findings in the
  order of the text output, and the standard error holds the same summary as with the text.
- [ ] Given a fixture without findings, when `nuc check --format json` runs, then it exits 0 and
  writes the document with no findings.
- [ ] Given a fixture whose `nuc.toml` is unreadable, when `nuc check --format json` runs, then it
  exits 2, the standard output is empty, and the standard error names the refusal as with the text.
- [ ] Given any fixture, when `nuc check`, `nuc check --format text`, and
  `nuc check --repo <path> --format text` run, then their standard output and exit code are those
  of today; `tests/contract.rs` and the parity check pass unchanged.
- [ ] Given `nuc check --format yaml` or `nuc check --format json --format json`, when it runs,
  then it exits 2 with the usage message.
- [ ] Given the writer, when its unit tests run, then it escapes `"`, `\`, and every control
  character below U+0020 as RFC 8259 asks and leaves other characters as they are.

## Read first

- Touches: `docs/adr/`, `src/finding.rs`, a new module of the library for the writer,
  `src/shell/mod.rs`, `tests/check.rs` or a new test of the formats, `tests/contract.rs`,
  `docs/ARCHITECTURE.md`, `README.md` (usage)
- The spec next to this ticket, *Implementation decisions* and *Testing decisions*.
- ADR-0020 (*Output*, *Exit codes*), ADR-0024 (*Shell*), ADR-0008 and Q-3 on dependencies.
- The test pattern in `docs/ARCHITECTURE.md`; `tests/check.rs` as the golden path.
- RFC 8259, section 7 (strings).

## Blocked by

- None (can start immediately)

## Mode

with the human — it starts with an ADR the human decides, and ADR-0007 allows no unattended
ticket.
