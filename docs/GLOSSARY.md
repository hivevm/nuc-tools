# Glossary — nuc-tools

> The **vocabulary** of this project: the terms everyone, humans and agents alike, uses in code,
> comments, documentation, and ADRs. It defines words and decides nothing. Whoever resolves a term
> adds it here in the same change ([ADR-0001](adr/0001-agent-governance-model.md)).

One term per concept, only terms specific to this domain. Say what a term *is* in one or two
sentences, and list under _Avoid:_ the synonyms it displaces, where there are any. A term the
specification, an ADR, or the code uses without an entry here is a gap to close, not a convention
to invent.

- **derived project** — A repository created from the NUC template, or one that took the template
  up, that carries a template release.
  _Avoid:_ downstream project, consumer repo.
- **template release** — A tagged version `vX.Y.Z` of the NUC template; a derived project names the
  one it carries in its README ([ADR-0024](adr/0024-template-releases.md)).
- **carried release** — The template release a tree names in the Template release line of its
  README; a rule whose bash check changed between releases checks the tree as that release does
  ([ADR-0021](adr/0021-rules-follow-the-carried-release.md)).
- **check** — A mechanical test of a repository's tree or history that fails a commit, a push, or
  CI on a finding.
- **sensor** — A mechanical reading of a repository that reports what it counts and fails on
  nothing until a decision turns it into a check.
- **gate** — The checks and hooks that keep a change from landing while a check fails.
- **shim** — The thin script in `.githooks/` that git runs as a hook and that hands over to
  `nuc hook <name>`.
- **finding** — One violation a check reports, named by its rule ID and its `file:line`.
- **not decided** — What `nuc` reports where it cannot check a tree as the bash would: it names
  the cause, exits 2, and never passes (Q-1).
- **rule ID** — The stable identifier of one rule `nuc` checks; it never changes meaning once
  released, and `nuc explain <id>` gives its reason.
- **area** — The first part of a rule ID, before the slash, as `docs` or `traceability`, named
  after the bash check its rules port.
- **parity** — The state in which `nuc` and the bash scripts of the same template release report
  the same findings and exit status on the same tree.
- **parity harness** — The test that runs the bash scripts and `nuc` on each tree of the corpus and
  on each variant for it, and compares their findings and exit status.
- **corpus** — The template and the derived repositories the parity harness runs on.
- **variant** — An edit to a copy of a corpus tree, listed in `tests/parity/variants.tsv` with the
  trees it is for, that makes a ported rule fire, so that the parity harness compares bash and
  `nuc` on a finding and not only on a clean tree
  ([ADR-0022](adr/0022-parity-variants-name-their-trees.md)).
- **migration** — One mechanical change `nuc upgrade` applies when a project takes up a later
  template release.
  _Avoid:_ upgrade step.
- **`nuc.toml`** — The file in a derived project that names the minimum version of `nuc` it
  requires.
- **release commit** — The commit the release workflow writes on top of a tag `vX.Y.Z`, carrying
  the checksums and the version the Action and the Feature install from; tagged `pinned/vX.Y.Z`.
- **major tag** — The tag `vN` that names the release commit of the latest release of major
  version N, and by which projects reference the Action.
- **marker** — A `Verifies:` line in a file git does not ignore that cites ADRs or criteria by
  their identifiers, naming the test that decides them
  ([ADR-0023](adr/0023-decisions-verified-by-tests.md)).
- **pending criterion** — A criterion of the specification no marker cites yet; the summary of
  `nuc check` names it, and it never fails the run.
