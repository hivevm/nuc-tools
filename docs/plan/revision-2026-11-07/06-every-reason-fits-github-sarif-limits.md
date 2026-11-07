# Ticket: Every rule's reason fits what GitHub's code scanning takes as a SARIF description

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

ADR-0027 writes the reason `nuc explain` gives for each rule as the `shortDescription`, the
`fullDescription`, and the `help` of the rule in the SARIF log. GitHub's *SARIF support for code
scanning* marks all three required, takes no empty string for a required property, and limits
`shortDescription.text` and `fullDescription.text` to 1024 characters each. Today the longest
reason, that of `docs/check-lists`, has about 680 characters, so every upload holds; but nothing
keeps a reason under the limit, and a reason that grows past it would be found only when GitHub
rejects or truncates a log in a derived project's CI.

Add a test, cited with `Verifies: ADR-0027`, that holds every rule of `rules::ALL` to a reason
that is not empty and has at most 1024 characters, the limit and its source named beside it.

## Acceptance criteria

- [ ] Given `rules::ALL`, when the test runs, then it passes for every rule today and fails,
  naming the rule and the length, for a reason that is empty or longer than 1024 characters.
- [ ] Given the test, when its source is read, then it cites GitHub's page for the limit.

## Read first

- Touches: `src/format.rs` (its unit tests) or `tests/formats.rs`
- ADR-0027 (*SARIF*, *Sources*).
- The table of `reportingDescriptor` properties on GitHub's *SARIF support for code scanning*:
  <https://docs.github.com/en/code-security/code-scanning/integrating-with-code-scanning/sarif-support-for-code-scanning>

## Blocked by

- None (can start immediately)

## Mode

with the human — ADR-0007 allows no unattended ticket.
