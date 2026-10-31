# ADR-draft-rules-follow-the-carried-release: A rule checks a tree as the bash check of the template release the tree carries

- **Status:** 🟡 proposed
- **Date:** 2026-10-31
- **Deciders:** Markus Brigl
- **Applies to:** every rule whose bash check differs between template releases, how the shell reads the README's `Template release` line for the rules, and the trees `tests/parity/corpus.toml` pins for each such release

## Context

A derived project gets the findings of the bash checks of the template release it carries (G-1).
Template releases change those checks: v0.6.0 changes check 8 of `scripts/check-docs.sh` so that
an old ADR flips to superseded only when its successor is accepted, and changes check 9 to name
a job with a matrix by its check runs. The projects do not move at once. The template and this
repository carry v0.6.0, while other derived projects of the corpus still carry v0.5.x
([ADR-0020](0020-parity-harness-with-variants.md)). A rule that follows only one release reports
false findings on the trees of the other, or passes what that release fails.

[ADR-0013](0013-version-coupling.md) already has `nuc` read the release from the `Template
release` line of the README. It leaves open what a rule does with that release. A rule ID stays
stable (G-6), and nothing passes when it could not decide (Q-1).

## Decision

We will have each rule read the template release the tree carries from its README and check the
tree as the bash check of that release does, and have `nuc` refuse a tree whose release such a rule
cannot place.

- **The release:** the first `Template release` line of `README.md`, read as check 14 reads it.
  The value is a `vX.Y.Z` tag at its start, or `unreleased`.
- **The behaviour:** a rule knows the releases at which its bash check changed. It checks a tree
  as the latest of them at or before the release the tree carries. A rule whose bash check never
  changed does not read the release.
- **A later release:** a release newer than any this `nuc` knows is checked as the newest one it
  knows. The `nuc.toml` of a release that changes a check raises its minimum to a `nuc` that
  ports the change ([ADR-0013](0013-version-coupling.md)).
- **`unreleased`:** checked as the newest release this `nuc` knows.
- **Not decided:** when a rule turned on reads the release, `nuc` refuses a tree whose release
  is before v0.5.0, the earliest the rules port. It also refuses a tree without a `README.md`,
  and one whose README has no `Template release` line or a line that names no release, unless
  `docs/template-release` is turned on. That rule then fails the run with the finding the bash
  reports, and the tree is checked as the newest release.
- **The corpus:** for each release at which a ported check changed, the parity corpus pins a
  tree that carries it.

**Out of scope:** template releases before v0.5.0; how `nuc upgrade` moves a project between
releases (G-4).

## Alternatives considered

- **A new rule ID for each changed bash check** (`docs/adr-supersessions-2`). This fits the
  default of ADR-0013, since the `nuc.toml` of v0.6.0 would turn the new ID on, and no rule would
  read the release. It was not chosen because the ID of the same rule would change with every
  release that touches its check, against G-6, and `nuc explain` would have to tell the two apart.
- **Only the newest release.** The corpus floor would move to v0.6.0, and a project on v0.5.x
  would get the findings of a release it does not carry until it takes up v0.6.0. That
  contradicts G-1.
- **The release in `nuc.toml`.** ADR-0013 already rejected repeating the release there.

## Sources / Prior art

- The release notes of hivevm/nuc v0.6.0 and the diff of `scripts/check-docs.sh` between v0.5.0
  and v0.6.0.
- ESLint and Clippy, from memory and not checked here: both keep a rule's ID stable and gate
  changed behaviour behind configuration, ESLint through its config's `ecmaVersion`, Clippy
  through `msrv`. Here the carried release plays that part.

## Consequences

- Positive: one `nuc` checks every project of the corpus by the release it carries, and a rule
  keeps its ID across releases.
- Negative / trade-offs:
  - A rule that changed carries both behaviours, together with tests and corpus trees for each.
  - Without `docs/template-release` turned on, a tree with no readable release line is not
    decided by those rules, so it exits 2 rather than 1.
- Follow-ups: when no derived project carries an old release any more, the behaviour of that
  release can go, with its corpus tree.

## Enforcement

The tests of `src/rules/docs_adr_supersessions.rs` and `tests/check.rs` cite this ADR with a
`Verifies:` marker: a rule checks a tree on v0.5.x and one on v0.6.0 each as its bash check does,
and that `nuc` refuses a tree whose release it cannot place. The parity harness runs the template
at v0.5.0 and at v0.6.0 ([ADR-0020](0020-parity-harness-with-variants.md)). A variant edits one
file, while the behaviour that differs between the releases needs two ADRs, so only the tests
above tell the two behaviours apart.
