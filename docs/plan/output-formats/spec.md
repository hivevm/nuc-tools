# Spec: `nuc check` reports its findings as JSON and as SARIF

<!-- Copy to docs/plan/<spec>/spec.md. The destination of a piece of work larger than one agent
     session, written from the user's side; its tickets lie next to it in the same directory. It
     is disposable: the design revision deletes it once its tickets have merged, and whatever has
     to last moves into docs/SPECIFICATION.md, an ADR, docs/ARCHITECTURE.md, docs/GLOSSARY.md, or
     docs/CONVENTIONS.md first. No file paths or code; they go stale. How it is written is the
     plan-feature skill under .agents/skills/. Decided in docs/adr/0006-feature-layer.md. -->

## Problem

`nuc check` writes its findings only as lines of text. A CI job that wants to annotate a pull
request, a dashboard that counts findings by rule, or a code-scanning service has to parse those
lines back, and the line form is made for a terminal and an editor, not for a program: a message
may hold a colon, and nothing in the line says which version of its shape a reader holds.

## Solution

`nuc check --format json` writes the findings as one JSON document, and `nuc check --format sarif`
as one SARIF 2.1.0 log, which GitHub code scanning and other SARIF readers take as they are.
`--format text`, the default, writes the lines it writes today. Whatever the format, the run finds
the same findings, exits with the same code, and writes the same summary and refusals on the
standard error.

## Existing behaviour

- `nuc check` writes one finding per line, `<file>[:<line>]: <rule ID>: <message>`, on the
  standard output and nothing else there; the summary and every refusal go to the standard error;
  it exits 0, 1, or 2 (ADR-0020). That stays the default, byte for byte. The parity harness
  compares exactly these lines with the bash checks (ADR-0015), and `tests/contract.rs`,
  `tests/check.rs`, and the parity corpus pin them.
- The command line accepts `check` and `check --repo <path>`; anything else is a usage error that
  exits 2. `--format` is new; an unknown format is a usage error as well.
- A finding never names a path that is not UTF-8: such a path is refused where a finding would name
  it, so every finding can be written as a JSON string.
- `nuc` depends on no crate; Q-3 counts every dependency against the supply chain.

## Goals served

- G-7 — the formats themselves, decided by a test that compares each with the text output.
- Q-1 — a run that cannot decide writes no document, so no reader takes a partial one for a
  result.
- Q-3 — no dependency is added for the formats.

## User stories

1. As a maintainer of a derived project, I want `nuc check --format sarif` in CI, so that its
   findings show in GitHub code scanning beside the lines they name.
2. As the author of a CI script, I want `nuc check --format json`, so that I can count, filter, or
   annotate findings without parsing the text lines.
3. As a reader of the JSON, I want it to name its format and version, so that my script can refuse
   a shape it does not know rather than misread it.
4. As a SARIF reader, I want each rule listed with the reason `nuc explain` gives, so that a
   finding shows what the rule asks for.
5. As anyone running `nuc check` in a terminal, I want the text output unchanged, so that editors,
   scripts, and the parity harness that read it keep working.
6. As a CI job, I want an undecided run to write nothing on the standard output and exit 2, so
   that I never upload a document that is not a result.

## Implementation decisions

The human's answers in the interrogation, taken into the ADR of ticket 01:

- Only `nuc check` takes `--format`; `nuc hook` writes for its shim and `nuc explain` prose.
- The formats are `text` (default), `json`, and `sarif`; the option is given once, before or after
  `--repo`.
- JSON and SARIF are written by the library, without a crate: a small writer that escapes strings
  as RFC 8259 asks and writes the fields in a fixed order, so that the same findings give the same
  bytes.
- The JSON is one document: its format name, a version number that grows only when a field
  changes meaning or goes, the findings with rule ID, file, line or null, and message, and the
  notes. The findings stand in the order of the text output.
- The SARIF is a 2.1.0 log with one run: the tool with every rule the run turned on, each with its
  ID and the reason of `nuc explain`; one result per finding, at level error, its file as a URI
  relative to the root of the working tree under the base `%SRCROOT%`, its line as the start line
  of a region, and no region where the finding names no line.
- A run that cannot decide writes nothing on the standard output in any format, exits 2, and
  says why on the standard error, as ADR-0020 says for the text.
- The summary and the notes as text stay on the standard error in every format.
- The formatting is a function of the findings in the functional core; the shell only chooses it
  and writes its result (ADR-0024).

## Testing decisions

- The seam is `nuc` run as a process on a fixture repository, as `tests/check.rs` does, with each
  format; the tests parse nothing with a crate either, but compare against the document built from
  the same findings, and check the text output unchanged.
- G-7 is decided by a test that runs `nuc check` on fixtures with findings, without a line, with a
  message holding quotes, backslashes, control characters, and non-ASCII text, and with none, and
  shows that each format carries the findings of the text output, field for field and in order.
- The writer's escaping has unit tests against RFC 8259's examples.
- The SARIF shape is checked against the fields the 2.1.0 schema requires and GitHub's upload
  documents, as a list in the test; the schema itself is not vendored.

## Out of scope

- A SARIF upload in the GitHub Action, which would need `security-events: write` and touches the
  protected release workflows; a later plan if a project asks.
- `--format` for `nuc hook` and `nuc explain`.
- JSON Lines, a schema file published for the JSON, and any field beyond those above.
- Column numbers, fixes, or code flows in SARIF, which no rule produces.

## ADR impact

- ADR-0020 leaves the formats of G-7 out of scope; the new ADR decides them without contradicting
  it, so it supersedes nothing.
- ADR-0024 (one library crate, effects in the shell) binds where the writer lives.
- ADR-0008 (toolchain) and Q-3 bind the choice of no crate.
- A new `proposed` ADR is needed before implementation: the command-line option, the JSON
  document and its versioning, the SARIF shape, and the behaviour on exit 2. Ticket 01 writes it
  through `propose-adr` and stops for the human's review before any code.
