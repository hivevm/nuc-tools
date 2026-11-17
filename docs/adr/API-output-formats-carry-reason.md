# API-output-formats-carry-reason: `nuc check --format json` writes one versioned JSON document and `--format sarif` a SARIF 2.1.0 log, each finding with the reason of its key and each rule with the first sentence of its reason as a short description, both without a crate, and nothing when the run cannot decide

- **Status:** 🟢 accepted
- **Date:** 2026-11-15
- **Deciders:** Markus Brigl
- **Applies to:** the `--format` option of `nuc check`, the JSON and SARIF it writes and the code that writes them, and the tests that compare them with the text output

## Context

G-7 asks that a check run with `--format json` or `--format sarif` report the same findings as its
text output. [API-command-line-with-measure](API-command-line-with-measure.md) decides the text:
one finding per line on the standard output, ending in its key `[<rule ID>:<reason>]`, the summary
and every refusal on the standard error, and exit 0, 1, or 2; it leaves the formats of G-7 out of
scope. The text is made for a terminal, an editor, and the parity harness
([TEST-parity-harness](TEST-parity-harness.md)), which reads it back by its shape; a program that
wants to count, filter, or annotate findings has to parse those lines, and a message may hold a
colon. GitHub code scanning and other services read SARIF 2.1.0, the OASIS standard. Q-1 asks that
nothing pass that could not decide; Q-3 counts every dependency against the supply chain, and a
crate joins `nuc` only to widen what it reads or how fast it reads it
([OPS-crates-pass-deny-and-audit](OPS-crates-pass-deny-and-audit.md)), which writing JSON and SARIF
does not. A finding never names a path that is not UTF-8, since such a path is refused
where a finding would name it, so every finding can be written as a JSON string. The rules return
their findings to a functional core, and every effect is in the shell
([ARCH-rules-outcome-hooks-answer](ARCH-rules-outcome-hooks-answer.md)). GitHub shows a rule's short
description in its list of alerts, where a reader scans for a concise line, and groups the alerts by
the rule ID of a result. A property bag, `properties`, carries what SARIF has no field for.

## Decision

We will have `nuc check` take `--format text|json|sarif`, write for `json` one JSON document that
names its format and version and for `sarif` one SARIF 2.1.0 log, both built in the library
without a crate from the same findings as the text, and write nothing on the standard output in
any format when the run cannot decide.

- **Option:** `--format <name>` is given at most once, before or after `--repo <path>`; `text` is
  the default and writes what API-command-line-with-measure decides, byte for byte. An unknown
  name or a second `--format` is a usage error, exit 2. Only `nuc check` takes it.
- **The same run:** the format changes only what the standard output holds. The rules run, the
  exit code, the summary and its notes, and every refusal on the standard error are those of the
  text. A run that cannot decide exits 2 and writes nothing on the standard output, so no reader
  takes a partial document for a result.
- **JSON:** one object, its fields in this order: `"format": "nuc-findings"`, `"version": 1`,
  `"findings"`, an array of objects `{"rule", "reason", "file", "line", "message"}` with `reason`
  the reason of the finding's key and `line` a number or `null`, and `"notes"`, an array of strings.
  The findings stand in the order of the text output. The version grows when a field changes its
  meaning or goes, not when one is added, so the `reason` added beside the rule keeps it at 1. The
  document stands on one line, without spaces between its tokens, ended by a newline.
- **SARIF:** a log with `"$schema": "https://json.schemastore.org/sarif-2.1.0.json"`, `"version":
  "2.1.0"`, and one run. Its `tool.driver` is named `nuc`, gives the running version as
  `semanticVersion`, and lists each rule the run turned on once, in the order `nuc check` runs them,
  with its ID; as its short description, the first sentence of the reason of `nuc explain`, up to
  and including the period of the first `. ` that a capital letter follows (the whole reason where
  no such period stands); and the whole reason as its full description and its help, the three
  GitHub's code scanning asks for. Each finding is one result at level `error`, with its rule ID,
  the index of its rule in that list, its message as text, the reason of its key in its `properties`
  as `"reason"`, and one location: its file as a URI reference relative to the root of the working
  tree, with no `uriBaseId`, which GitHub reads against the root of the repository, and a region
  whose start line is the finding's line, or 1 where the finding names none, since GitHub asks for a
  start line. The document stands on one line, ended by a newline, as the JSON. The results stand in
  the order of the text output. The notes are no findings and stay on the standard error only.
- **URIs:** a file becomes a URI reference as RFC 3986 asks: its `/` and its unreserved characters
  (letters and digits of ASCII, `-`, `.`, `_`, `~`) stay, and every other byte of its UTF-8 is
  percent-encoded in upper-case hex, `:`, space, `#`, `?`, and `%` among them, so that no path
  reads as a scheme, a fragment, or a query.
- **Without a crate:** a writer in the library escapes strings as RFC 8259 asks: `"` and `\` with a
  backslash, U+0008, U+000C, U+000A, U+000D, and U+0009 as `\b`, `\f`, `\n`, `\r`, and `\t`, every
  other control character below U+0020 as `\u00XX` in lower-case hex; every other character as it
  is. It writes the fields in a fixed order, so that the same findings give the same bytes. The
  shell chooses the format and writes what the library returns.

**Out of scope:** `--format` for `nuc hook` and `nuc explain`; a SARIF upload in the GitHub Action;
JSON Lines and a published schema for the JSON; columns, fixes, and code flows in SARIF, which no
rule produces; `partialFingerprints`, which an upload computes; the wording of the messages, which
API-command-line-with-measure leaves free; a write to the standard output that fails, which ends
the run as any failed write does.

## Alternatives considered

- **Findings without a reason** — what the second ADR on the formats decided, accepted on
  2026-11-08 with every other clause of this one. Once the text line ends in the key, a document
  without the reason would carry less than the text, against G-7, so this ADR replaced that clause.
- **The key as the SARIF rule ID, `docs/adr-index:unlisted`** — the cause in the field GitHub
  groups by, but every reason would be a rule of its own in the list of rules, each with no
  description of its own, and an alert would move to a new rule where a release adds a reason.
- **The JSON version raised to 2** — a reader would see the new field by the number, but the
  version grows only when a field changes its meaning or goes, and a reader of version 1 that
  ignores an unknown field reads the document as before.
- **The key as one JSON field, `"key": "<rule>:<reason>"`** — as the text writes it, but it
  repeats the rule beside `"rule"`, and a reader splits it to branch on the cause.
- **serde and serde_json** — the usual way; it brings the first crates into `Cargo.lock` and into
  every release, against Q-3, for two small documents of fixed shape.
- **JSON Lines, one finding per line** — streams well, but carries no version and no place for the
  notes, so a reader cannot refuse a shape it does not know.
- **A document on exit 2, with the refusal in it** — convenient for an upload, in SARIF as an
  unsuccessful invocation; but it is a second way of reporting a refusal beside the standard error,
  and a reader that skips the flag takes the empty document for a pass.
- **The summary left out in the machine formats** — a quieter log, but the formats would differ in
  more than the standard output.
- **SARIF without the rules** — smaller, but a reader shows no description of what a rule asks.
- **URIs under the base `%SRCROOT%`** — the form of CodeQL's output; GitHub asks for
  `originalUriBaseIds` beside it to make the log valid, which would name the absolute path of the
  working tree, so that the log differs from machine to machine and shows a local path.
- **No region where a finding names no line** — what SARIF allows, but GitHub's code scanning
  lists the start line as required and may not show such a result.
- **The whole reason as the short description** — what the first ADR on the formats decided,
  accepted on 2026-11-08 with every other clause of this one. GitHub shows the short description
  where a reader scans the list of alerts and calls it a concise description, and the whole reason
  there repeats the full description word for word, so the second ADR on the formats replaced that
  clause.
- **`--format` for every command** — `nuc hook` writes for its shim and `nuc explain` prose; no
  reader needs either as a document.

## Sources / Prior art

- RFC 8259, *The JavaScript Object Notation (JSON) Data Interchange Format*, section 7
  (strings):
  <https://www.rfc-editor.org/rfc/rfc8259>.
- RFC 3986, *Uniform Resource Identifier (URI): Generic Syntax*, sections 2.1 (percent-encoding),
  2.3 (unreserved characters), and 4.2 (relative references):
  <https://www.rfc-editor.org/rfc/rfc3986>.
- OASIS, *Static Analysis Results Interchange Format (SARIF) Version 2.1.0*: sarifLog, run,
  toolComponent, reportingDescriptor, result, location, artifactLocation, region:
  <https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html>.
- GitHub, *SARIF support for code scanning*, the 2.1.0 subset GitHub reads:
  <https://docs.github.com/en/code-security/code-scanning/integrating-with-code-scanning/sarif-support-for-code-scanning>.
- ESLint's and Clippy's machine formats, which keep the text as the default and add documents
  beside it.

## Consequences

- Positive: a CI job annotates or uploads findings, and branches on their cause, without parsing
  lines; the text and every reader of it stay as they are; no dependency is added.
- Negative / trade-offs: the writer and the URI encoding are code of this project to keep
  correct, so both are tested against their RFCs; SARIF readers that want more than the subset
  above get no more. Without `partialFingerprints`, an upload that does not compute them, unlike
  GitHub's `upload-sarif` action, may show a finding again as a new alert.
- Follow-ups: a SARIF upload in the GitHub Action, should a project ask for it.

## Enforcement

A test cited with `Verifies: API-output-formats-carry-reason` runs `nuc check` with each format on
fixtures with findings, without a line, with messages of quotes, backslashes, control characters,
and non-ASCII text, on a path with a space, a colon where the platform allows one, `#`, and `%`, and
on one with a letter beyond ASCII, and without findings. It checks that the JSON and the SARIF carry
the findings of the text output field for field, the reason of each key included, and in order, that
the exit code and the standard error are those of the text, and that an unreadable `nuc.toml` leaves
the standard output empty. For the SARIF it checks `$schema`, `version`, the URI with no base, each
rule the run turned on listed once with the first sentence of its reason as its short description
and the whole reason as its full description and help, each `ruleIndex` pointing at its rule, the
reason in each result's `properties`, the percent-encoded URI, and start line 1 where a finding
names no line. Tests cited the same way run `--format` before and after `--repo`, with an unknown
name, and twice. Unit tests of the writer in `src/format.rs` check its escaping against RFC 8259,
the first sentence of a reason with and without a second one and with a `. ` that a lower-case
letter follows, and the URI encoding against RFC 3986. `tests/contract.rs` and the parity check hold
the text output unchanged.
