# TEST-copies-keep-old-identifiers: An ADR identifier inside a copy of an older template release is not a citation of this record, and no map moves it

- **Status:** 🟢 accepted
- **Date:** 2026-11-14
- **Deciders:** Markus Brigl
- **Applies to:** the messages of `nuc` that copy a bash message of a template release, `tests/parity/`, `src/testing.rs`, `tests/support/`, and every map a normalization or a migration applies to this repository
- **Narrows:** [NUC-adrs-are-named-by-category §6](NUC-adrs-are-named-by-category.md)

## Context

`nuc` reproduces the findings of the bash checks of each template release it places (G-1, Q-2).
Its messages copy the bash messages word for word, and those name ADRs as the release named them:
"(ADR-0003)" for the release that carried it. The parity corpus pins trees of other repositories
whose ADR files carry the numbers of their own release. These identifiers name ADRs of another
record: the template at a given release, or a repository of the corpus.

The inherited ADR-0005 held the project's answer for numbered records. A normalization kept the
numbers 0001 to 0006, so no map moved a copied number. A test writes a foreign identifier through
`src/testing.rs` or `tests/support/`, out of the map's reach. Template release v0.9.0 ends the
numbers. The inherited ADRs become the fixed base of `NUC-` ADRs, and the migration renames every
ADR of this record. The template's base has no clause for a foreign identifier: in v0.9.1 its clause
on citations, [NUC-adrs-are-named-by-category §6](NUC-adrs-are-named-by-category.md), requires every
citation to name a file in `docs/adr/`. Section 8 of the `normalize` skill of v0.9.1 reverts such a
move by hand after the run. That clause carries the project's addition only if a decision of the
project says which identifiers are foreign.

## Decision

We will keep an ADR identifier inside a copy of another record as the copied record wrote it.

1. A copy of another record is one of these:
   - a message of `nuc` that reproduces a message of a template release;
   - a tree, a variant, or a fixture under `tests/parity/`;
   - a fixture text that `src/testing.rs` or `tests/support/` writes.
2. An identifier inside such a copy names an ADR of the copied record. It is not a citation of
   this record, and it must not be changed to an identifier of this record.
3. A map that a normalization or a migration applies must leave each such identifier as it was.
   The author of the change must revert each move of one by hand after the map script ran.
4. A comment or a document of this repository that cites a decision is a citation of this record.
   It names an ADR of this record, and the map moves it.
5. A test must write a foreign identifier through `src/testing.rs` or `tests/support/`, out of the
   reach of the map.

**Out of scope:** which messages `nuc` copies, which TEST-parity-harness and
TEST-hook-scenario-parity decide; the form of a citation of this record, which the template's base
decides.

## Alternatives considered

- **Keep the inherited numbers 0001 to 0006, as the inherited ADR-0005 did** — template release
  v0.9.0 deletes the numbered inherited ADRs and replaces them by the fixed base. A project must not
  edit that base, so the numbers cannot stay.
- **Rewrite the copied messages to the new identifiers** — `nuc` would then differ from the bash of
  every release before v0.9.0, and the parity harness would fail on each such message.
- **No decision, the reverts left to the skill** — the skill names the revert but not which
  identifiers are foreign. A reviewer without this conversation would take a kept "ADR-0003" for a
  missed reference.

## Sources / Prior art

- Template release v0.9.0, release notes, "Taking it up", item 5: a file that keeps an old number
  on purpose is reverted by hand.
- Template release v0.9.1, `.agents/skills/normalize/SKILL.md`, section 8, step 8: a reference to
  something other than this record's ADR keeps its old number.

## Consequences

- Positive: the migration and every later normalization keep parity with the releases `nuc`
  places. A reviewer can tell a kept old identifier from a missed one.
- Negative / trade-offs: each map run needs a hand revert of the foreign identifiers it moved,
  until the map script can exclude paths.
- Follow-ups: an exclusion option of the map script, suggested to the template.

## Enforcement

**Not mechanically decidable:** whether an identifier names an ADR of this record or of a copied
one depends on what the text copies. No test can read that. The review of a normalization or a
migration checks that the diff under `tests/parity/` touches no line outside a comment. Two checks
catch a moved copy as a side effect, without citing this ADR:

- the parity harness (`scripts/check-parity.sh`) fails when a message of `nuc` no longer matches the
  bash message of its release;
- the tests of `nuc` that expect a copied message fail the same way.
