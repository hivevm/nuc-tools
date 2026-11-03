# Ticket: Check the template's badge as the carried release's check 13 does

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

Template v0.6.1 changed check 13 of `scripts/check-docs.sh`: the template's repository name in a
badge now ends at `/`, `)`, `?`, `#`, `&`, `.svg`, or the end of the line, where v0.5.x and v0.6.0
match `hivevm/nuc` as a prefix. `docs/template-identity` took the v0.6.1 pattern in 5e0da0b and
applies it to every tree. It reads no release, and `rules::RELEASES` stops at v0.6.0.

On a tree that carries v0.5.x or v0.6.0, whose setup is closed and whose README has a badge of a
repository named `hivevm/nuc-…`, the bash of that release reports a template badge and `nuc`
passes. That goes against G-1 ("the same findings as the bash checks of the template release the
project carries"), ADR-0021 ("a rule knows the releases at which its bash check changed"), and
the convention *A port copies the bash, quirks included*. The corpus cannot show it, since no
derived tree of the corpus carries a release before v0.6.1 any more, and the template's own trees
name `hivevm/nuc` itself.

Two ways out, for the human to choose:

- **Follow ADR-0021:** add v0.6.1 to `rules::RELEASES`, let `docs/template-identity` read the
  release, and keep the prefix match for trees before v0.6.1. This needs no ADR.
- **Keep the correction for every release:** this contradicts G-1 as it reads. It needs a drafted
  specification change and an ADR that supersedes ADR-0021 for a bash defect that a later release
  fixed. Then this ticket becomes that draft.

## Acceptance criteria

- [ ] Given a tree on v0.6.0 after setup whose README badge links `hivevm/nuc-tools`, when
  `nuc check` runs `docs/template-identity`, then it reports what that release's bash reports. If
  the human chose the second way, the drafted specification change and the proposed ADR are
  filed instead.
- [ ] Given the same tree on v0.6.1, when `nuc check` runs, then it reports no badge finding, as
  today.
- [ ] Given docs/ARCHITECTURE.md, when the change lands, then the Rules block names the rules that
  read the release as they stand.

## Read first

- ADR-0021 and ADR-0013.
- G-1 in docs/SPECIFICATION.md.
- The conventions on porting the bash with its quirks.
- `src/rules/docs_template_identity.rs` and `src/rules/docs_check_lists.rs`, which already
  checks by the release.
- The diff of `scripts/check-docs.sh` between the template's v0.6.0 and v0.6.1.

## Blocked by

- None (can start immediately)

## Mode

with the human — the choice between following ADR-0021 and changing G-1 is the human's, and
ADR-0007 keeps every ticket with the human, one at a time.
