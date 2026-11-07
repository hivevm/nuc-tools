# Ticket: Every finding names the rule that returned it, or the run does not decide

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

The SARIF log gives each result the index of its rule in the list of the rules the run turned
on, and `src/format.rs` finds that index with an `expect`: a finding whose rule ID is not that
of a rule the run turned on panics there, after the rules ran, and `nuc check --format sarif`
ends with exit 101 and a panic message instead of a result. Today every rule module builds its
findings with its own `RULE_ID`, but nothing holds that: a rule that one day reports under
another ID, a shared reader returning findings, or a copy-paste of another rule's constructor,
passes every check and breaks only the SARIF output, and only on a tree where that finding fires.
The text and JSON outputs would carry the foreign ID silently.

Make the invariant held rather than assumed, in the simplest of two shapes the session weighs:

- the shell, in `run_rules`, ends the run as not decided (exit 2, a message naming the rule and
  the ID it reported) where a rule's outcome holds a finding of another ID; this treats every
  outcome alike, as `tests/functional_core.rs` asks of the shell; or
- the structural test `tests/functional_core.rs`, which already holds that each rule module
  declares one `RULE_ID`, also holds that every `Finding` a rule module builds takes `rule:
  RULE_ID`.

Either way `src/format.rs` keeps no panic a tree can reach.

## Acceptance criteria

- [ ] Given a rule whose outcome holds a finding with an ID not its own (a unit test with a stand-in
  rule, or a source fixture of the structural test), when the check runs, then it fails with a
  message naming the rule, and no path of `nuc check` reaches the `expect` of `ruleIndex`.
- [ ] Given the fixtures of `tests/formats.rs`, when they run, then every format writes what it
  writes today.

## Read first

- Touches: `src/shell/mod.rs` (`run_rules`) or `tests/functional_core.rs`, `src/format.rs`
- ADR-0024 (*Shell*, the outcome a rule returns), ADR-0027 (*SARIF*, the index of a result's
  rule).
- The *Rules* and *Formats* blocks of `docs/ARCHITECTURE.md`.

## Blocked by

- None (can start immediately)

## Mode

with the human — ADR-0007 allows no unattended ticket.
