# Architecture — nuc-tools

> The system **as it stands today**: the parts it is built from, what each is responsible for, and
> how they fit together. It holds no rule and no decision. It names the ADR behind each structural
> choice; where it disagrees with an accepted ADR, the ADR is right and this document is stale. It
> is updated in the same change as the structure it describes
> ([ADR-0001](adr/0001-agent-governance-model.md)).
>
> **Last design revision:** 2026-10-25, due after 20 changes. The revision that ran moves the
> date; the number is this project's to set; a sensor counts the changes outside `docs/` since
> the commit that set the date and says when the next is due
> ([ADR-0006](adr/0006-feature-layer.md)).

## Context

What sits outside the system and what crosses its boundary. A diagram earns its place here more
than anywhere else, and none beats one that has stopped being true.

TODO — fill in once the system has a boundary worth drawing.

## Building blocks

The parts the system is made of, each with one responsibility, named in the vocabulary of
[`GLOSSARY.md`](GLOSSARY.md). One level deep, deeper only where the size of a part earns it. The
golden path and the test pattern an agent copies from are named here.

- **Rules** — one module each under `src/rules/`, a pure function from the inputs the shell read
  to the findings, with one rule ID and the reason `nuc explain` gives; the list of every rule is
  `rules::ALL`. A rule reads no file and runs no process
  ([ADR-0009](adr/0009-functional-core.md)).
- **Configuration** — `config` reads `nuc.toml` and selects the rules it turns on, or says why
  `nuc` refuses: missing, unreadable, a minimum above the running version, an unknown rule ID
  ([ADR-0013](adr/0013-version-coupling.md)).
- **Shell** — `src/shell/` reads the command line and the repository, runs git under a cleared
  environment in `shell::git`, calls the rules, prints the findings, and sets the exit code
  ([ADR-0010](adr/0010-git-cli-cleared-environment.md)).
- **Binary** — `src/main.rs` calls the shell and nothing else.
- **Parity harness** — `scripts/check-parity.sh` runs the bash checks of each repository of the
  corpus, all on template release v0.5.0 or later, and `nuc` on the same tree and on each variant
  of `tests/parity/variants.tsv` that fits it, and compares their findings
  ([ADR-draft-parity-harness-with-variants](adr/draft-parity-harness-with-variants.md)).
- **Release tooling** — `release/` builds a binary per target, writes `SHA256SUMS`, checks the
  tag against the crate's version, writes the release commit, and moves the major tag;
  `release/install-nuc.sh` is the one installer, run by the Action and the Feature, and refuses a
  binary whose checksum differs ([ADR-0014](adr/0014-distribution.md)).
- **Action and Feature** — `action.yml` and `feature/src/nuc/` install the `nuc` of the release
  their release commit carries; at any other commit they install nothing.
- **The golden path** — the rule `docs/template-release`, check 14 of the template's
  `check-docs.sh`, and the pieces it takes through every layer. A new rule copies it:
  1. a module under `src/rules/` with its `RULE_ID`, an `area/name`, and its `RULE`, listed in
     `rules::ALL`, its unit tests the cases of the bash self-test of the check it ports;
  2. what it needs to read, added to the inputs and read by the shell;
  3. its rows in the normalization table of the parity harness, one per bash message.
- **The test pattern** — `tests/check.rs`: a fixture repository built through git by
  `tests/support/`, `nuc` run on it as a process under a cleared environment, and its exit code
  and output asserted. The structural test `tests/functional_core.rs` holds the shape
  ([ADR-0011](adr/0011-fixture-repositories.md)).

## How it runs

The few paths worth following end to end, such as a request, a job, or a build, and where state
lives between them. Only what a newcomer would otherwise reconstruct from code.

`nuc check` finds the root of the working tree with `git rev-parse --show-toplevel`, reads
`nuc.toml` there and selects the rules it turns on, reads `README.md`, runs each selected rule on
what it read, prints the findings on the standard output, and exits 0 without a finding and 1
with one. Every cause it can name that keeps it from deciding ends the run with exit 2 and a
message on the standard error, before any rule runs; a panic ends it with exit 101, non-zero as
well (Q-1). Nothing is written and no state is kept between runs.

A release starts when a human pushes a tag `vX.Y.Z` that names the version of `Cargo.toml`.
`release.yml` builds the four targets, writes `SHA256SUMS`, attests each binary and verifies the
attestations, and creates the release as a draft. It writes the release commit on top of the tag,
which carries the checksums and the version for the Action and the Feature, tags it
`pinned/vX.Y.Z`, moves the major tag `vN` to it unless `vN` names a later release, and only then
makes the release public and publishes the Feature from the release commit. A project on `@vN`
gets the new release through the moved tag; its Dependabot raises the next major.
