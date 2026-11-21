# Spec: `nuc upgrade` takes up a template release

## Problem

A derived project takes up a template release by hand, in the steps of
NUC-releases-are-semver-tags §6 and section 7 of the normalize skill. The maintainer fetches the
tag, replaces the base, reads the prose of the release notes for "the other files" to take up,
checks each out, merges where the project changed it, moves the release line, and runs the checks.
Most of that is mechanical: a file the project never touched since the release it carries is
simply replaced by the new release's copy. Only a file both sides changed, and a citation whose
clause moved, needs judgment. The prose lists both alike, and a skipped release multiplies the
notes to read. The vision of the specification promises "one `nuc upgrade` plus the judgment calls
it lists"; G-4 has no test.

## Solution

On a clean `feat/` branch, with the old and the new tag fetched into `refs/template/`, the
maintainer runs `nuc upgrade <tag>`. `nuc` writes into the working tree every change it can decide
from the two tags and the project's tree, moves the release line, and lists, one finding per line,
each place that needs judgment. It exits 0 when nothing is left to judge, 1 when the list is not
empty, and 2 when it refuses, having written nothing. The maintainer reviews the diff, settles the
list, runs the checks, and commits; `nuc` commits nothing.

## Existing behaviour

Greenfield: `nuc` has no command that writes into a project's working tree beyond the merge
driver, which writes only the file git hands it. What must stay:

- the command line, the output, and the exit codes every command of `nuc` shares
  (API-command-line-with-sense), pinned by the contract tests;
- the shape of rules as pure functions and every effect in the shell
  (ARCH-rules-outcome-hooks-answer), pinned by the structural test of the functional core;
- the cleared environment of every git `nuc` spawns, without lazy fetch and with the repository's
  commands neutralized or refused (SEC-git-cleared-environment, SEC-git-cleared-without-lazy-fetch,
  SEC-repository-commands-neutralized-or-refused), pinned by the security tests of Q-7 to Q-12;
- the reading of the `Template release` line (API-version-coupling-hook-defaults), pinned by the
  tests of `docs/template-release`.

## Goals served

- G-4 — met: the first test that cites it.
- G-5 — a new release whose `nuc.toml` asks for a newer `nuc` than the running one is refused.
- G-6 — each judgment is a finding with a stable rule ID and key that `nuc explain` explains.
- Q-1 — anything `nuc` cannot decide is refused before a byte is written.
- Q-7, Q-8, Q-9, Q-11, Q-12 — the first command that writes many files must keep each of them.

## User stories

1. As a maintainer of a derived project, I want `nuc upgrade <tag>` to replace the base (the
   `NUC-` ADRs, `AGENTS.md`, and the blob list) with the new release's, so that I no longer check
   them out by hand.
2. As that maintainer, I want every other file I never changed since my release to be replaced,
   added, or deleted as the template changed it, so that I read only what needs me.
3. As that maintainer, I want a file both the template and my project changed listed as a
   judgment, with both tags named, so that I merge it myself and nothing of mine is overwritten.
4. As that maintainer, I want the `Template release` line moved in the same run, so that the
   checks read the new release at once.
5. As that maintainer, I want each citation of a `NUC-` ADR that the new release no longer has
   listed by file and line, so that I move it by the release's map.
6. As that maintainer, I want to skip releases in one run, so that a project several releases
   behind takes up the newest in one step.
7. As that maintainer, I want `nuc upgrade` to refuse, writing nothing, on `main`, on a tree with
   changes, on a release before v0.9.0, on a forked release line, on a tag that is not newer, on
   a tag missing under `refs/template/`, and on a release that needs a newer `nuc`, with a message
   that says what to do, so that a half-written take-up never happens.
8. As a reviewer of the take-up's pull request, I want the list of judgments in the hand-off, so
   that I check that each was settled.
9. As the maintainer of `nuc`, I want the take-ups the corpus already holds replayed by
   `nuc upgrade`, so that a release of `nuc` proves it does what a human did.

## Implementation decisions

- **Inputs.** The old release is the one the `Template release` line names; the new one is the
  argument. Both are read from `refs/template/<tag>`, which the maintainer fetched with
  `git fetch --no-tags` as the normalize skill says. `nuc` reaches no network; a missing ref is
  refused with the fetch command to run.
- **The decision is a pure function** over three trees, the old tag's, the new tag's, and the
  project's `HEAD`, each a list of paths with their mode and blob. It returns a plan: the writes,
  the deletions, the release line, and the judgments. The shell reads the trees, carries out the
  plan, and prints the judgments. This is the shape of the rules and hooks.
- **The base** (`docs/adr/NUC-*`, `AGENTS.md`, the blob list) is replaced by the new release's
  whatever the project's copy holds, and a `NUC-` file the new release lacks is deleted: the
  project never edits it (NUC-base-is-fixed §1).
- **Every other path** is decided by its blob in the three trees:
  - unchanged by the template: nothing;
  - changed by the template, the project's blob equal to the old tag's: replaced, mode included;
  - added by the template, the project without the path: added;
  - removed by the template, the project's blob equal to the old tag's: deleted;
  - every other case, including a project that deleted the file or has a path of its own there:
    a judgment.
  A symlink or a submodule on either side is a judgment, never written.
- **The release line** is the one line of the README `nuc` edits in place; the rest of the README
  follows the rule above.
- **Citations.** Each line of a project file outside the base that names a `NUC-` identifier the
  new release lacks is a judgment. `nuc` moves no citation: the map lives only in the prose of the
  release notes.
- **Release notes.** A take-up across several releases also gets a judgment that names the
  releases between the two tags, whose notes may ask for more than files.
- **Writing.** `nuc` writes the blob's bytes as git stores them, with no filter, no `textconv`,
  and no end-of-line conversion run (Q-7). It writes only regular files below the working tree. It
  refuses a path with a `.git` component, a path through a symlink of the working tree, and a
  path outside the tree (Q-8), all before the first write.
- **Output and exit codes** follow API-command-line-with-sense: each judgment a finding line
  `<file>[:<line>]: upgrade/<rule>: <message> [upgrade/<rule>:<reason>]`, a summary on the standard
  error, exit 0, 1, or 2. The rule IDs are explained by `nuc explain`.
- **Refusals** (exit 2, nothing written): no tag given; `main` checked out; changes in the
  working tree or the index, read under the neutralized filters; a release line that is
  `unreleased`, forked, or unreadable; an old release before v0.9.0; a new tag not newer than the
  old; either ref missing or not a tag of that name; a new release whose `nuc.toml` asks for a
  newer `nuc` than the running one.

## Testing decisions

- The seam is the binary, as the contract tests drive it: a fixture repository with two template
  tags under `refs/template/` and a project commit, built by the fixture helpers with their own
  home (TEST-fixtures-with-own-home), then `nuc upgrade` and an assertion on the working tree, the
  output, and the exit code.
- The decision function is tested on values, three trees in and a plan out, as the rules are.
- The security tests of Q-7 and Q-8 gain the upgrade: a template tree carrying a filter, a symlink,
  and a path outside the tree.
- The replay: for each take-up commit the corpus pins, `nuc upgrade` on its parent with the two
  template tags yields the take-up's tree on every path it does not list as a judgment.

## Out of scope

- Fetching a template tag; `nuc` keeps off the network.
- Moving citations by a release's map, and any machine-readable form of the map or of the release
  notes; that needs the template first.
- Merging a file both sides changed.
- Committing, pushing, or opening the pull request.
- A take-up from before v0.9.0, whose numbered ADRs need the migration of a v0.9 release's
  normalize skill, and a take-up by a fork.
- Rewriting `nuc.toml` beyond the three-way rule, such as raising `minimum` in a project that
  changed the file.
- Changing the template's normalize skill to call `nuc upgrade`; that is a template release.

## ADR impact

Binding: API-command-line-with-sense (the output and exit codes of a new command),
ARCH-rules-outcome-hooks-answer (the plan as a pure function, the shell carrying it out),
API-version-coupling-hook-defaults (the release line as the starting point, its follow-up "how
`nuc upgrade` moves the README line and `nuc.toml` together"), SEC-git-cleared-environment,
SEC-git-cleared-without-lazy-fetch, SEC-repository-commands-neutralized-or-refused,
DEV-one-agent-one-ticket (the shell stays with the maintainer), and of the base
NUC-releases-are-semver-tags §6, NUC-base-is-fixed §1 and §2, and NUC-main-only-via-pr.

A new `proposed` ADR is needed before the tracer bullet, by ticket 01: how `nuc upgrade` decides a
take-up (the inputs from `refs/template/`, the three-way rule by blob, the base replaced
outright, judgments as findings) and how it writes (raw blobs, the path guards, nothing before
every refusal is passed). It adds `nuc upgrade` to the command contract, which may supersede
API-command-line-with-sense.
