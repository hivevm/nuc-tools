# API-upgrade-takes-up-by-blob: `nuc upgrade <tag>` replaces the base, takes up each other file the project left as its release had it, moves the release line, writes only the working tree, and lists every other change as a finding

- **Status:** 🟡 proposed
- **Date:** 2026-11-22
- **Deciders:** Markus Brigl
- **Applies to:** the `nuc upgrade` command, its source in the shell and in the core, the template tags it reads, the files it writes and deletes, its `upgrade/` rules and keys, and its refusals

## Context

G-4 asks that a project taking up a template release run `nuc upgrade`, which applies the
mechanical migrations and lists what needs judgment. No test cites G-4 yet.

Today a maintainer takes up a release by hand, in the steps of
[NUC-releases-are-semver-tags §6](NUC-releases-are-semver-tags.md) and section 7 of the normalize
skill. The maintainer fetches the tag into `refs/template/<tag>`. The base is replaced: the `NUC-`
files, `AGENTS.md`, and the blob list ([NUC-base-is-fixed §2](NUC-base-is-fixed.md)). Then come
"the other files the release notes name", the release line, and the citations a re-cut moves. The
notes name the other files in prose, written by the maintainer of the template for each release. A
project that skips releases reads the notes of each. The map of a re-cut stands in prose too
([NUC-releases-are-semver-tags §7](NUC-releases-are-semver-tags.md)).

Most of a take-up needs no prose. A project never edits its base
([NUC-base-is-fixed §1](NUC-base-is-fixed.md)), so the new base replaces it outright. Any other
file the project left as its release had it is the template's alone. The project takes the new
release's copy, adds a file the release adds, and drops one it removes. The two template tags and
the project's tree decide that per path, as git decides the trivial cases of a three-way merge.
Only a path both sides changed, a citation whose clause the new release lacks, and the notes of a
skipped release need a human.

Forces from the record:

- [API-command-line-with-sense](API-command-line-with-sense.md) holds every command of `nuc` to one
  output and to the exit codes 0, 1, and 2. A judgment is a finding: a file, often a line, and a
  reason. It fits that contract as it stands, unlike the report of `nuc measure` or `nuc sense`.
- [ARCH-rules-outcome-hooks-answer](ARCH-rules-outcome-hooks-answer.md) keeps every decision in a
  pure function and every effect in `src/shell/`.
- [API-version-coupling-hook-defaults](API-version-coupling-hook-defaults.md) reads the release a
  project carries from its `Template release` line. It leaves open how `nuc upgrade` moves it.
- [SEC-git-cleared-without-lazy-fetch](SEC-git-cleared-without-lazy-fetch.md) and
  [SEC-repository-commands-neutralized-or-refused](SEC-repository-commands-neutralized-or-refused.md)
  bind every git `nuc` spawns. `nuc` runs no command the repository configures (Q-7).
- `nuc` reaches no network in checks and hooks (Q-11). It writes only inside the repository (Q-8)
  and reads nothing outside it (Q-12). It acts on no other repository (Q-9).
- [DEV-one-agent-one-ticket](DEV-one-agent-one-ticket.md) keeps the code with effects with the
  maintainer. `nuc upgrade` is the first command that writes many files of a project's tree.

The base has the same shape from template release v0.9.0 on: `docs/adr/NUC-*.md`,
`docs/adr/NUC.blobs`, and `AGENTS.md`. Every release from v0.9.0 on carries a `nuc.toml`. A
project on an earlier release still has numbered ADRs. It migrates them once with the normalize
skill of a v0.9 release.

## Decision

We will have `nuc upgrade <tag>` decide a take-up purely from three trees and write it into the
working tree once no refusal stands. The trees are the old release's, the new release's, and the
project's `HEAD`. Each change it cannot decide is a finding.

- **Inputs:**
  - The old release is the tag the first `**Template release:**` line of `README.md` names. The new
    release is the argument. Each is a tag `vX.Y.Z` as
    [NUC-releases-are-semver-tags §1](NUC-releases-are-semver-tags.md) forms it.
  - Each is read from `refs/template/<tag>`, which must be an annotated tag object whose tag name
    is `<tag>`. The maintainer fetches it as section 7 of the normalize skill says. `nuc` fetches
    nothing. The refusal of a missing ref names the fetch command to run.
  - The project's files are read from `HEAD`. The run refuses a working tree or an index that
    differs from `HEAD`, an untracked file that is not ignored included. It reads them under the
    neutralized filters as `nuc sense` does, with optional locks off.
  - `nuc upgrade` reads the project's `nuc.toml` as every command does. It refuses when the new
    release's `nuc.toml` names a `minimum` above the running version (G-5).
  - Every blob the plan writes is read before the first write.
- **The base:** `AGENTS.md`, `docs/adr/NUC.blobs`, and each `docs/adr/NUC-*.md` take the new
  release's entry, whatever the project's regular file holds. A `NUC-` file the new release lacks
  is deleted.
- **Every other path** is decided by its entry, mode and blob, in the three trees. A missing entry
  equals a missing one. The first case that holds decides:
  - the new release's entry equals the old's: nothing;
  - the project's entry equals the new release's: nothing;
  - the project's entry equals the old release's: the path takes the new release's entry, or is
    deleted where the new release lacks it;
  - any other case is a judgment.
- **Links:** where a tree has a symlink or a submodule, a path the plan would write or delete is a
  judgment, in the base too. A path with the same link in every tree is left
  alone.
- **Modes:** where the working tree cannot carry a mode, as an executable bit under
  `core.fileMode=false`, `nuc` writes the content and lists the mode as a judgment.
- **The working tree:** a path the plan adds that the working tree has, such as an ignored file, is
  a judgment. So is a file the plan writes where the working tree keeps a directory, or below a
  file it keeps, after the plan's deletions.
- **The release line:** the line is edited after the file is written. In the first
  `**Template release:**` line, the old tag is replaced by the new one; a line that names the new
  tag already stays. No other byte of `README.md` is written by this clause.
- **Citations:** each line that names a `NUC-` identifier the new release lacks is a judgment at its
  `file:line`. The lines are read outside the base, in the tree as the plan leaves it. A citation
  is read as the rule `docs/adr-refs` reads one, and a code span is none
  ([NUC-adrs-are-named-by-category §6](NUC-adrs-are-named-by-category.md)). Copies of another
  record are listed as well, and
  [TEST-copies-keep-old-identifiers](TEST-copies-keep-old-identifiers.md) has them left. No
  citation is moved.
- **Skipped releases:** the release after `vX.Y.Z` is `vX.Y.(Z+1)`, `vX.(Y+1).0`, or
  `v(X+1).0.0`. When the new release is none of these, one judgment says that the notes of the
  releases between the two were not read.
- **Writing:**
  - A file gets the blob's bytes as git stores them. No filter, `textconv`, or end-of-line
    conversion runs, and no git command writes the working tree.
  - Deletions come first, then a directory they leave empty is removed, then the writes, each
    creating its missing parent directories.
  - It writes a regular file with mode 644 or 755, deletes a regular file or an empty directory,
    and nothing else.
  - The guards hold each path the plan writes or deletes, the directories it removes included.
  - Before the first write, the run refuses each path git's `verify_path` refuses with
    `core.protectNTFS` and `core.protectHFS` on, whatever the platform. Among them are an empty,
    `.`, or `..` component, and a `.git` component in any case or alias. It also refuses an
    absolute path and a backslash. It refuses a path to write that differs only in letter case
    from another the plan writes or from one the working tree keeps.
  - It refuses a path with an ancestor in the working tree that is a symlink. Each ancestor is
    read without following a link.
  - `nuc upgrade` writes no index, no ref, and no commit.
- **Output:**
  - Each judgment is a finding line of
    [API-command-line-with-sense](API-command-line-with-sense.md) under one of three rules.
    `upgrade/citation` has the reason `removed`, and `upgrade/notes` the reason `skipped`.
  - `upgrade/file` has six reasons. `both`: both sides changed the path. `deleted`: the project
    lacks a path the template changed. `added`: the working tree keeps a file or a directory at a
    path the plan writes, or a file above it. `kept`: the template removes a path the project
    changed. `link`: a symlink or a submodule. `mode`: a mode the working tree cannot carry.
  - A path where several reasons hold carries the first in the order `link`, `both`, `deleted`,
    `added`, `kept`, `mode`.
  - The rules are explained by `nuc explain`. Their keys join the list kept beside the contract
    test. `nuc.toml` turns none of them on or off.
  - The summary on the standard error names both tags and the notes of the new release to read.
    It counts the paths written, deleted, and listed.
- **Exit codes:** 0 when the plan is written and holds no judgment, 1 when it holds one, 2 when it
  cannot decide. A refusal before the first write writes nothing. Each refusal of its own ends in
  `[upgrade/run:<reason>]`:
  - `usage`: no tag, or more than one argument;
  - `main`: `main` is checked out;
  - `changed`: the working tree or the index differs from `HEAD`;
  - `release`: the release line is `unreleased`, forked, or unreadable;
  - `old`: the old release is before v0.9.0;
  - `order`: the new release is not above the old;
  - `ref`: a ref is missing or not a tag of its name;
  - `version`: the new release needs a newer `nuc`;
  - `path`: a path to write fails the guards above;
  - `write`: a write failed after the first; the message names the paths written so far.

  A cause the reading of git refuses, as a missing object or a filter not decided, exits 2 under
  the key that reading gives.

**Out of scope:** what a human does: fetching a tag, moving citations by a map, and merging a file
both sides changed. Also the content of the notes, a take-up before v0.9.0 or by a fork, and
`nuc.toml` beyond the rule for other paths. The normalize skill calling `nuc upgrade` is the
template's.

## Latitude

None.

## Alternatives considered

- **A manifest in each template release** — a file that lists the paths to take up and the map of
  a re-cut, for `nuc` to read. It would let `nuc` move citations. It needs a template release first,
  and each release a list kept by hand beside the prose. Every path it lists, the three trees decide
  already.
- **Migrations coded in `nuc` per pair of releases** — a table in the tool of what each release
  changes. It puts the template's rules into the tool, against the non-goals, and every template
  release would wait on a release of `nuc`.
- **`nuc` fetches the tag** — one step less for the maintainer. It would be the first network access
  of `nuc` outside a sensor that names it, and a remote to trust. `git fetch` does the same in one
  line the skill already gives.
- **A dry run by default, writing only with a flag** — safer to try. The tree must be clean anyway,
  so `git restore` and `git clean` undo every write, and the diff is the clearest preview.
- **`nuc` commits the take-up** — one step less. The take-up is incomplete until a human settles
  the judgments. A commit before that would be red and need amending.
- **Conflict markers in a file both sides changed**, as `git merge-file` and copier write them — the
  maintainer would merge in place. Writing them runs the merge in `nuc`, a file with markers fails
  every later check, and a binary file has no markers. A finding naming both tags lets the
  maintainer merge with any tool.
- **Checking out the files with git** — `git checkout refs/template/<tag> -- <path>` as the skill
  does. It runs the repository's filters (Q-7). `nuc` writes the blob's bytes itself.
- **A successor of API-command-line-with-sense with a clause for `nuc upgrade`** — as measure and
  sense got. A judgment is a finding, so the contract holds as it stands, and its citations need not
  move.

## Sources / Prior art

- [git-read-tree, "3-Way Merge"](https://git-scm.com/docs/git-read-tree): a path the same in all
  three trees, or changed on one side only, collapses to the changed side. Any other difference is
  left to "porcelain policy". The rule for every other path is that table, with a judgment as the
  policy.
- [Copier, "Updating a project"](https://copier.readthedocs.io/en/stable/updating/): regenerates
  the project from the old template version, takes the diff to the project, applies the new
  version, and reapplies the diff, writing conflict markers or `.rej` files where a hunk does not
  apply.
- [NUC-releases-are-semver-tags §6](NUC-releases-are-semver-tags.md) and section 7 of the normalize
  skill: the take-up by hand this command does.
- The take-up of v0.12.1 in this repository, commit 2110947: 23 files changed. The base was
  unchanged, and every file the notes named was taken up from the tag.

## Consequences

- Positive: a take-up is one command and a list (G-4). A project several releases behind takes up
  the newest in one run. The notes of a release no longer need to name each file. A file the
  project changed is never overwritten.
- Positive: a judgment is a finding, so a hand-off can paste the list and a reviewer can check each
  line was settled.
- Negative: a template file the project never touched but should not carry, such as one only the
  template needs, is still taken up. The project deletes it once; from then on it is a judgment of
  reason `deleted` at each release that changes it.
- Negative: a re-cut still has its citations moved by hand with `remap.sh`. A repository that keeps
  copies of an older record, as this one does, gets their lines listed at each re-cut and exits 1.
- Negative: the rule for skipped releases reads only the two versions. A patch cut on an older line
  after the next minor, as a v0.12.2 after v0.13.0, goes unlisted.
- Negative: the shell gains a writer of many paths, which the structural test and the security
  tests have to hold to Q-7 and Q-8.
- Follow-ups: a machine-readable map of a re-cut in the template, which would let `nuc` move
  citations; the template's normalize skill calling `nuc upgrade` in section 7; raising
  `minimum` in a `nuc.toml` the project changed.

## Enforcement

`tests/contract.rs` must cite this ADR with `Verifies: API-upgrade-takes-up-by-blob` and `G-4`. On
fixtures with two template tags under `refs/template/` it must check the following:

- The base takes the new release's bytes and modes, and a `NUC-` file the new release lacks is
  gone.
- The release line names the new tag, and no other line of `README.md` changed by that clause.
- Each case of the rule for every other path gives its write, deletion, or finding. Each reason of
  `upgrade/file` has a case, the ignored file and the directory of `added` among them.
- A file that becomes a directory and a directory that becomes a file are taken up.
- A symlink that stays the same in every tree gives no finding.
- A citation of a `NUC-` identifier the new release lacks is a finding. One in a code span is none,
  and so is one the file the plan writes no longer has.
- A take-up across a skipped release gives the finding of `upgrade/notes`, and one to the next
  release none.
- Each refusal exits 2 with its key, and `git status --porcelain` is as before.
- A judgment exits 1, and a plan without one exits 0.

The core of `nuc upgrade` must have unit tests on three trees as values. `tests/functional_core.rs`
must cite this ADR and hold the core to no effect. It lists the shell function that writes in
`tests/functional_core.writers`. `tests/isolation.rs` must cite it and check the following:

- No filter or `textconv` driver of the repository runs (Q-7).
- Each path the guards refuse ends the run before any write (Q-8). Among them are a symlinked
  ancestor in the working tree and a tree entry with a `.git` alias, a `..`, or a backslash.

The replay of the take-ups of the corpus must cite this ADR. Each path `nuc upgrade` writes or
deletes there matches the take-up a human committed.
