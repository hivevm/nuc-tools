# Ticket: Bring the revision's findings on the two proposed ADRs to the human

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

Three findings on the proposed ADRs on `feat/check-docs-port`, for the human to take or send
back before accepting either. Each that is taken lands as a revision, a commit of ADRs only
([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules), rule 4); no status changes here.

1. **ADR-draft-rules-follow-the-carried-release, Context.** It says the template and this
   repository carry v0.6.0 while the other derived projects of the corpus still carry v0.5.x.
   The corpus as pinned is the reverse: the trees of v0.5.0 are the template at that tag and this
   repository at its setup; `hivevm/nectar` and `hivevm/ocelli` carry v0.6.0, and the one derived
   project on v0.5.x left the corpus. Proposed wording: "The projects do not move at once: the
   corpus pins trees of v0.5.0, the template at that tag and this repository at its setup, beside
   trees of v0.6.0, the template at that tag and two derived projects."
2. **ADR-draft-parity-variants-name-their-trees, Decision.** The harness runs bash and `nuc`
   with the environment of a pull request into `main` marked ready, every tree and every variant,
   so the check of drafts on `main` is compared in that one environment only. The draft does not
   say so; a reader would take a gap in the comparison for a mistake. Proposed: one sentence under
   **Comparison** naming the environment both sides run in, or under **Out of scope** that other
   environments are not compared.
3. **Consolidation.** ADR-0013 has `nuc` read the release from the README;
   ADR-draft-rules-follow-the-carried-release decides what a rule does with it. Said in one
   sentence: a project's `nuc.toml` turns rules on and each checks the tree as the release its
   README names. The human may prefer one ADR that supersedes ADR-0013 and carries both, instead
   of a second ADR beside it.

## Acceptance criteria

- [ ] Given each finding, when the human has decided it, then it is either a revision commit of
  ADRs only on the branch of the draft, or recorded in the hand-off as sent back with the reason.

## Read first

- ADR-0013, ADR-0020, and both drafts in full; `tests/parity/corpus.toml`;
  `scripts/check-parity.sh`, its `CI_ENV`.

## Blocked by

- None (can start immediately)

## Mode

with the human — ADR text and its status are the human's
([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules), rules 3 and 4).