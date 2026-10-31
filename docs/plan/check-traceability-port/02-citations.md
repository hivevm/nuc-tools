# Ticket: Port the citations of the markers

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

`nuc check` runs a rule `traceability/citations`. It reports what part 3 of
`check-traceability.sh` reports for every `Verifies:` marker in a file git does not ignore
that is not Markdown: a citation of an ADR that does not exist, of one that is superseded or
rejected, or of a criterion the specification does not hold, each at its `file:line`.

The rule reads files as the bash does:
- tracked and new files, ignored files left out;
- a file the bash's grep takes for binary is skipped;
- a file it cannot read as the bash does refuses, as not decided.

## Acceptance criteria

- [ ] Given each case of `test-check-traceability.sh` for the citations, when the rule runs, then
  it reports the same finding or none.
- [ ] Given a marker in a Markdown file, an ignored file, and a binary file, when the rule runs,
  then none of them is read as a citation, as the bash reads none.
- [ ] Given a marker in a new, untracked file, when the rule runs, then it is read.
- [ ] Given several identifiers on one marker, separated by spaces or commas, when the rule runs,
  then each is checked, as the bash's pattern splits them.
- [ ] Given a file with bytes the bash and the port would read differently, when the rule runs,
  then `nuc` exits 2, naming the file.
- [ ] Given the corpus and three variants (a marker citing a missing ADR, a superseded ADR, and
  an unknown criterion), when the harness runs, then it reports no difference.

## Read first

- **ADRs:** ADR-0009, ADR-0010 (git, for the file list), ADR-0011, ADR-0019, ADR-0022,
  ADR-0023.
- **The conventions** on reads inside the repository and on the byte classes a port is tried on.
- **The bash:** part 3 of the template's `scripts/check-traceability.sh`, its cases in
  `scripts/test-check-traceability.sh`, and the identifier pattern in `scripts/lib/adr.sh`.
- **Reading the file list:** how the docs rules read it through git.

## Blocked by

- docs/plan/check-traceability-port/01-criteria.md

## Mode

with the human. ADR-0007 keeps every ticket with the human, one at a time.
