# Ticket: Decide whether the conventions keep the clauses their checks decide

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

The two porting conventions of docs/CONVENTIONS.md gained 13 lines since the last revision. Part
of that growth restates what a structural test now decides:

- *The shell reads, the rules decide*: "the structural test fails on a read, a listing, or a
  resolution in any other function of the shell". `tests/functional_core.rs` decides this against
  `tests/functional_core.guarded`.
- *A port copies the bash, quirks included*: "`tests/bash_cases.rs` holds them together, its list
  giving each case its rule, and fails on a case of a rule no test names", and "the structural
  test fails on a reader of `tests/functional_core.bash` defined elsewhere".

The file's own header says "A convention a check can decide is the check, not an entry". The
review of #20 asked for the opposite: a test should hold a rule that is stated somewhere, and
neither ADR-0009 nor a convention stated this one. With the human's choice, the conventions state
these rules and the checks hold them. So the header and the practice disagree.

Bring this to the human. One way: each clause shrinks to the rule in a sentence and names the
test that decides it, and the header gains that a check's rule is stated where the check cites
it. The other way: the clauses go, and the tests' module docs state the rules, as the header asks.
Either way, the conventions say no more than one sentence about each check.

## Acceptance criteria

- [ ] Given the human's choice, when this ticket closes, then docs/CONVENTIONS.md and its header
  agree on where a rule that a check decides is stated. Every module doc of the three tests cites
  the place of the rule it holds.
- [ ] Given `scripts/check-docs.sh`, when the change lands, then it passes, and no clause the
  tests cite was lost.

## Read first

- ADR-0001, on the conventions file, and ADR-0025, whose Enforcement names the structural test.
- docs/CONVENTIONS.md, with its diff since ff8f3e9.
- The reviews in the descriptions of #13, #20, and #29.

## Blocked by

- None (can start immediately)

## Mode

with the human — the wording of the conventions was approved by the human, and the choice of
where a rule lives is theirs.
