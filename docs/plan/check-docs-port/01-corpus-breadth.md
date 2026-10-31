# Ticket: Compare bash and nuc on the template and opamp-fleet, and on variants

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

The parity harness compares the bash checks and `nuc` on the template at v0.5.0 and on
`mbrigl/opamp-fleet`, a derived repository of v0.5.0, and refuses an entry whose pinned tree
carries a release before v0.5.0. For the one rule ported so far, `docs/template-release`, it also
derives variants of each tree that make the rule fire, and compares bash and `nuc` on them. The
tracer bullet of the spec: every later ticket adds rules and variants to what this one builds.

## Acceptance criteria

- [ ] Given the corpus of the spec, when the harness runs, then it fetches each repository at its
  pinned commit and reports no difference for `docs/template-release` on any of them.
- [ ] Given a corpus entry whose pinned tree's README names a release before v0.5.0, or none, when
  the harness runs, then it fails and names the entry, never compares it (Q-1); a variant of an
  accepted tree is compared whatever its edit does to that line.
- [ ] Given a variant that deletes the Template release line of a tree, and one that writes a
  release that is no `vX.Y.Z`, when the harness runs, then bash and `nuc` each report the finding
  on every tree and the harness reports no difference.
- [ ] Given a variant whose edit no longer applies to a tree, when the harness runs, then it fails
  and names the variant and the tree, never skips it (Q-1).
- [ ] Given the harness's self-test, when it runs, then it has cases for a tree of a release before
  v0.5.0 and for a variant that applies, one that does not, and one on which bash and `nuc` differ.

## Read first

- ADR-draft-parity-harness-with-variants (parity), ADR-0013 (version coupling), ADR-0018 (gates), ADR-0019 (contract).
- Glossary: parity, parity harness, corpus, rule ID.
- The golden path and the test pattern in `docs/ARCHITECTURE.md`; the harness and its self-test.

## Blocked by

- None (can start immediately); ADR-draft-parity-harness-with-variants is accepted before it lands.

## Mode

with the human — ADR-0007 keeps every ticket with the human, one at a time.
