# Ticket: Write the findings as SARIF 2.1.0

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

`nuc check --format sarif` writes the findings of the run as one SARIF 2.1.0 log, as the ADR of
ticket 01 decides, through the JSON writer of ticket 01: one run, its tool listing every rule the
run turned on with its ID and the reason of `nuc explain`, and one result per finding at level
error, located by a URI relative to `%SRCROOT%` and, where the finding names one, a start line.
A test of G-7 then shows that the JSON and the SARIF carry the findings of the text output.

## Acceptance criteria

- [ ] Given a fixture with findings, one of them without a line, when `nuc check --format sarif`
  runs, then it exits 1 and the standard output is a SARIF 2.1.0 log holding `$schema`, `version`
  `2.1.0`, and one run whose results name, in the order of the text output, each finding's rule
  ID, message, URI under `%SRCROOT%`, and start line, with no region where the finding has no line.
- [ ] Given the same run, when the log's `tool.driver.rules` is read, then it lists each rule the
  run turned on, once, with its ID and the reason `nuc explain` gives, and each result's
  `ruleIndex` points at its rule.
- [ ] Given a fixture without findings, when it runs, then it exits 0 and writes the log with an
  empty `results` array; given an unreadable `nuc.toml`, it exits 2 and writes nothing on the
  standard output.
- [ ] Given fixtures with findings, without a line, with a message of quotes, backslashes, control
  characters, and non-ASCII text, and without findings, when `nuc check` runs with each format,
  then the JSON and the SARIF carry the findings of the text output field for field and in order;
  the test cites G-7.
- [ ] Given the log, when the fields the SARIF 2.1.0 schema requires and GitHub's upload of SARIF
  documents are checked against it, then none is missing.

## Read first

- Touches: the writer module of ticket 01, `src/shell/mod.rs`, the test of the formats,
  `docs/ARCHITECTURE.md`, `README.md` (usage)
- The ADR of ticket 01 and the spec next to this ticket.
- OASIS SARIF 2.1.0, sections 3.13 (sarifLog), 3.14 (run), 3.19 (toolComponent), 3.27 (result),
  3.28 (location), 3.30 (region); GitHub's documentation of SARIF support for code scanning.

## Blocked by

- docs/plan/output-formats/01-decide-the-formats-and-write-json.md

## Mode

with the human — ADR-0007 allows no unattended ticket.
