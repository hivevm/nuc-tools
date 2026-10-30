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
  one it carries in its README ([ADR-0005](adr/0005-template-releases.md)).
- **check** — A mechanical test of a repository's tree or history that fails a commit, a push, or
  CI on a finding.
- **sensor** — A mechanical reading of a repository that reports what it counts and fails on
  nothing until a decision turns it into a check.
- **gate** — The checks and hooks that keep a change from landing while a check fails.
- **shim** — The thin script in `.githooks/` that git runs as a hook and that hands over to
  `nuc hook <name>`.
- **finding** — One violation a check reports, named by its rule ID and its `file:line`.
- **rule ID** — The stable identifier of one rule `nuc` checks; it never changes meaning once
  released, and `nuc explain <id>` gives its reason.
- **parity** — The state in which `nuc` and the bash scripts of the same template release report
  the same findings and exit status on the same tree.
- **parity harness** — The test that runs the bash scripts and `nuc` on the same tree and compares
  their findings.
- **corpus** — The template and the derived repositories the parity harness runs on.
- **migration** — One mechanical change `nuc upgrade` applies when a project takes up a later
  template release.
  _Avoid:_ upgrade step.
- **`nuc.toml`** — The file in a derived project that names the minimum version of `nuc` it
  requires.
