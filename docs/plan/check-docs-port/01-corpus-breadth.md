# Ticket: Compare bash and nuc on six repositories, each with its own release, and on variants

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

The parity harness compares the bash checks and `nuc` on the template at v0.5.0 and on the five
derived repositories the spec names, each against the `check-docs.sh` of its own template release
and with a `nuc.toml` that turns on the rules that release has. For the one rule ported so far,
`docs/template-release`, it also derives variants of each tree that make the rule fire, and
compares bash and `nuc` on them. The tracer bullet of the spec: every later ticket adds rules and
variants to what this one builds.

## Acceptance criteria

- [ ] Given the corpus of the spec, when the harness runs, then it fetches each repository at its
  pinned commit and reports no difference for `docs/template-release` on any of them.
- [ ] Given a corpus entry that names its template release and its rule IDs, when the harness
  writes `nuc.toml` for that tree, then it turns on exactly those IDs.
- [ ] Given a variant that deletes the Template release line of a tree, and one that writes a
  release that is no `vX.Y.Z`, when the harness runs, then bash and `nuc` each report the finding
  on every tree and the harness reports no difference.
- [ ] Given a variant whose edit no longer applies to a tree, when the harness runs, then it fails
  and names the variant and the tree, never skips it (Q-1).
- [ ] Given a release whose `check-docs.sh` lacks a check, when the harness compares a tree of that
  release, then a rule of that check is not turned on for it.
- [ ] Given the harness's self-test, when it runs, then it has cases for the per-release rule lists
  and for a variant that applies, one that does not, and one on which bash and `nuc` differ.

## Read first

- ADR-0012 (parity), ADR-0013 (version coupling), ADR-0018 (gates), ADR-0019 (contract).
- The spec's ADR impact on variants: the maintainer's answer decides whether this ticket opens
  with a superseding ADR.
- Glossary: parity, parity harness, corpus, rule ID.
- The golden path and the test pattern in `docs/ARCHITECTURE.md`; the harness and its self-test.

## Blocked by

- None (can start immediately), once the maintainer has answered the spec's question on variants.

## Mode

with the human — ADR-0007 keeps every ticket with the human, one at a time.
