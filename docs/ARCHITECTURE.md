# Architecture — nuc-tools

> The system **as it stands today**: the parts it is built from, what each is responsible for, and
> how they fit together. It holds no rule and no decision. It names the ADR behind each structural
> choice; where it disagrees with an accepted ADR, the ADR is right and this document is stale. It
> is updated in the same change as the structure it describes
> ([ADR-0001](adr/0001-agent-governance-model.md)).
>
> **Last design revision:** 2026-10-31, due after 20 changes. The revision that ran moves the
> date; the number is this project's to set; a sensor counts the changes outside `docs/` since
> the commit that set the date and says when the next is due
> ([ADR-0006](adr/0006-feature-layer.md)).

## Context

What sits outside the system and what crosses its boundary. A diagram earns its place here more
than anywhere else, and none beats one that has stopped being true.

`nuc` runs in a working tree of a derived project, from a human's shell, a hook, or CI. What
crosses its boundary:

- **In:** the command line; the files of the working tree, each resolved inside it; what git
  answers about the tree, git run as a process; and the variables CI sets for a pull request,
  which the check of drafts on `main` reads. The tree is untrusted input.
- **Out:** the findings on the standard output, the summary and every refusal on the standard
  error, and the exit code. `nuc check` writes no file and keeps no state.
- **Beside it:** the template's bash scripts, the oracle the parity harness compares `nuc` with,
  on a corpus of repositories fetched from GitHub; and the release workflow, which publishes the
  binaries the Action and the Feature install.

## Building blocks

The parts the system is made of, each with one responsibility, named in the vocabulary of
[`GLOSSARY.md`](GLOSSARY.md). One level deep, deeper only where the size of a part earns it. The
golden path and the test pattern an agent copies from are named here.

- **Rules** — one module each under `src/rules/`, a pure function from the inputs the shell read
  to an outcome: the findings of `src/finding.rs`, a refusal of its own where it cannot decide the
  tree as its bash check would, and notes for the summary. Each has one rule ID, the reason
  `nuc explain` gives, and the inputs it reads; the list of every rule is `rules::ALL`. A rule
  reads no file and runs no process
  ([ADR-draft-rules-return-an-outcome](adr/draft-rules-return-an-outcome.md)).
  The rules of the decision record share one reading of it in `src/adr.rs`, and the rules of
  Markdown one reading of its links in `src/markdown.rs`, both outside `src/rules/`, the way the
  bash's `scripts/lib/adr.sh` and awk passes read them. The byte-level readings of the bash's tools,
  lines as awk, `read`, and `sed` split them, POSIX space and the bytes without it, and the first
  word `read` splits off, are in `src/bash.rs` and nowhere else, which the structural test holds.
  The rules of the area `traceability` share one reading of the criteria of the specification and of
  the identifiers the `Verifies:` markers cite in `src/traceability.rs`, beside `src/adr.rs`, so
  that no rule module calls another. The shell matches path patterns as bash's pathname expansion
  does through `src/glob.rs`. A rule whose bash check changed between template releases, today
  `docs/adr-supersessions` and `docs/check-lists`, reads the release the tree carries and checks as
  that release does (ADR-0021).
- **Configuration** — `config` reads `nuc.toml` and selects the rules it turns on, or says why
  `nuc` refuses: missing, unreadable, a minimum above the running version, an unknown rule ID
  ([ADR-0013](adr/0013-version-coupling.md)).
- **Shell** — `src/shell/` reads the command line and the repository, runs git under a cleared
  environment in `shell::git`, calls the rules, prints the findings, and sets the exit code
  ([ADR-0010](adr/0010-git-cli-cleared-environment.md)). Its `Tree`, in `shell::tree`, reads
  every file and lists every directory the rules need, each resolved through one function that
  refuses a path outside the repository, and refuses as well, as not decided, one the bash would
  read differently from its bytes alone.
- **Binary** — `src/main.rs` calls the shell and nothing else.
- **Parity harness** — `scripts/check-parity.sh` runs the bash checks of each repository of the
  corpus, `check-docs.sh` and `check-traceability.sh`, all on template release v0.5.0 or later,
  and `nuc` on the same tree and on each variant of `tests/parity/variants.tsv` that is for it,
  both as a pull request into `main` marked ready, and compares their findings by
  `tests/parity/normalize.tsv`. A variant edits, deletes, or
  writes one path inside a copy of the tree, never under its `.git`
  ([ADR-0022](adr/0022-parity-variants-name-their-trees.md)).
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
  2. what it needs to read, an `Input` the rule names in its `reads`, a field of the inputs, its
     type in the rule's module unless several rules read it, and the shell's reading of it;
  3. its rows in the normalization table of the parity harness, one per bash message;
  4. a variant in `tests/parity/variants.tsv` that makes it fire.
- **The test pattern** — `tests/check.rs`: a fixture repository built through git by
  `tests/support/`, `nuc` run on it as a process under a cleared environment, and its exit code
  and output asserted. The structural test `tests/functional_core.rs` holds the shape, and
  inside the shell lets only the functions `tests/functional_core.guarded` names read, list, or
  resolve a path ([ADR-0011](adr/0011-fixture-repositories.md)).

## How it runs

The few paths worth following end to end, such as a request, a job, or a build, and where state
lives between them. Only what a newcomer would otherwise reconstruct from code.

`nuc check` finds the root of the working tree with `git rev-parse --show-toplevel`, reads
`nuc.toml` there and selects the rules it turns on, and reads the inputs those rules name and no
others: `README.md` and the release it carries, the record under `docs/adr/`, the text files git
lists, the targets of relative links, `AGENTS.md`, the workflows and scripts, the Project
Layout's paths, the skills and their pointers, the identity files, the attributes that mark a
file generated, the variables CI sets, the specification, and the files that carry a `Verifies:`
marker. It runs each selected rule on what it read and carries out every outcome alike: it
prints the findings on the standard output, and exits 0 without a finding and 1 with one; the
summary on the standard error carries the rules' notes, as the criteria no marker cites, which
change no exit code. Every cause it can name that keeps it from deciding ends the run with exit 2
and a message on the standard error: before any rule runs for what it reads, an input that
resolves outside the repository, one the bash would read otherwise, a release a rule that checks
by it cannot place; after the rules run, before any finding is printed, for a rule's own refusal
and for a finding that names a path of no UTF-8.
A panic ends it with exit 101, non-zero as well (Q-1). Nothing is written and no state is kept
between runs.

A release starts when a human pushes a tag `vX.Y.Z` that names the version of `Cargo.toml`.
`release.yml` builds the four targets, writes `SHA256SUMS`, attests each binary and verifies the
attestations, and creates the release as a draft. It writes the release commit on top of the tag,
which carries the checksums and the version for the Action and the Feature, tags it
`pinned/vX.Y.Z`, moves the major tag `vN` to it unless `vN` names a later release, and only then
makes the release public and publishes the Feature from the release commit. A project on `@vN`
gets the new release through the moved tag; its Dependabot raises the next major.
