# Spec: Port the checks and hooks of template releases v0.9.0 to v0.11.0

## Problem

`nuc` ports the bash checks and hooks of template releases v0.5.0 to v0.8.1. The template has
since released v0.9.0, v0.9.1, v0.10.0, and v0.11.0, and a derived project on one of them gets
from `nuc` neither the findings nor the decisions its bash reports:

- `nuc` checks a tree of a later release as v0.8.1, the newest it knows. From v0.9.0 on, ADRs are
  named by category and slug, not by number, so `nuc check` on such a tree reports what the bash
  of its release does not, and misses what it does (G-1, Q-2).
- The rules those releases add, among them the fixed base held by its blob list, the sensor
  tests, the issue-opening ban of the template, and the mode of the scripts, are not ported.
- The hooks of v0.9.0 run the documentation check on every commit off `main`, and those of v0.11.0
  refuse a merge on a `feat/` branch. `nuc hook` does neither (G-2).
- From v0.9.1 every finding of the bash names a key of rule and reason, and the template's fixture
  tables expect keys and data, never wording. `nuc` prints no key, so it cannot be held to those
  tables.
- The parity corpus pins no tree on any of these releases, although this repository itself
  carries v0.11.0.

The template's tag message of v0.11.0 expects a binary to replace its bash checks. Without this
port, no project on a current release can switch to `nuc`.

## Solution

A derived project on any release from v0.5.0 to v0.11.0 runs `nuc check` and `nuc hook` and gets
the findings, the exit status, and the hook decisions the bash of its release gives, each finding
with the key the bash names. The template can list in its `nuc.toml` every rule of `check-docs.sh`
and `check-traceability.sh` and the rule `shell/mode`, and switch those checks over.

## Existing behaviour

- Each rule checks a tree as the latest release at or before the one it carries, among the
  releases at which a ported bash check changed. A tree of a later release is checked as the
  newest of them. That stays: a tree on v0.5.0 to v0.8.1 is checked exactly as today, numbered
  ADRs, drafts, and inherited ADRs included. Every rule ID stays known, so a `nuc.toml` that lists
  `docs/adr-numbering`, `docs/no-draft-on-main`, or `docs/inherited-adrs` stays valid. On a tree of
  v0.9.0 or later those rules report nothing, as the bash of that release has no such check.
- Pinned today:
  - the parity harness, on the corpus trees of v0.5.0 to v0.8.1 and their variants;
  - the hook scenarios, on the template's trees;
  - the unit tests that copy the cases of the bash self-tests of v0.8.1, held to the case list;
  - the table of the releases at which each rule's bash changed;
  - the rule list of the contract test;
  - the output-format tests.
  All stay green throughout; a test that changes says why in its hand-off.
- The text output keeps its shape `<file>[:<line>]: <rule ID>: <message>`; the key is added as
  API-command-line-keyed-findings decides, and the parity harness and the output-format tests move
  with it.
- Hooks: a tree whose README names no release is decided as the newest release `nuc` knows. With
  this port that becomes v0.11.0, so a hook on such a tree runs the documentation check on every
  commit off `main` and refuses a merge on `feat/`.

## Goals served

- G-1: `nuc check` reports what the bash of v0.9.0 to v0.11.0 reports.
- G-2: `nuc hook` decides as the bash hooks of those releases.
- G-6: each finding names its stable key, rule and reason, as the bash does.
- G-7: the key is in the JSON and SARIF output too.
- Q-1: a tree of those releases that a rule cannot read is refused, not passed.
- Q-2: the parity harness runs on the template's trees of those releases and on this repository.
- Q-6: every rule added ships off; only a template release's `nuc.toml` turns it on.

## User stories

1. As a maintainer of a project on v0.9.0 or later, I want `nuc check` to report what my bash
   checks report, so that I can switch my checks to `nuc`.
2. As a maintainer of a project on v0.8.1 or earlier, I want `nuc check` to report exactly what it
   reports today, so that a `nuc` update turns nothing red (Q-6).
3. As a maintainer of the template, I want every rule of `check-docs.sh` and
   `check-traceability.sh` of v0.11.0 ported, so that the template can ship a `nuc.toml` listing
   them.
4. As a user reading a finding, I want the key the bash names, `<rule>:<reason>`, in every output
   format, so that a finding is told apart from its wording and the same in bash and `nuc`.
5. As a maintainer of `nuc`, I want each rule tested against the cases of the template's fixture
   tables, so that a port reports what the bash reports on inputs the corpus lacks.
6. As a maintainer of `nuc`, I want the copied tables pinned to the release they came from, so
   that a table changed by hand or left behind a release is caught.
7. As a contributor on a branch, I want the pre-commit hook to refuse a commit whose documentation
   check is red, as the bash hook of v0.9.0 does.
8. As a contributor integrating a `feat/` branch, I want a merge on it refused, as the bash hook of
   v0.11.0 does, so that the branch stays linear.
9. As a maintainer of a project whose scripts must be executable, I want `nuc check` to report a
   script without mode 100755, as the rule `shell/mode` of the bash does.
10. As the maintainer of `nuc`, I want the parity corpus to pin the template at v0.9.0, v0.9.1,
    v0.10.0, and v0.11.0 and this repository on v0.11.0, so that parity is measured on each.

## Implementation decisions

- **Releases.** v0.9.0, v0.9.1, and v0.10.0 join the releases at which a ported check changed;
  v0.11.0 changes no check, only the hooks. A rule whose bash changed at one of them dispatches by
  the release the tree carries, as the rules do today. The table of the releases per rule gains
  their rows, and the rules new from v0.9.0 report nothing on an earlier tree.
- **The ADR identifier.** From v0.9.0 the identifier of an ADR is its file name without
  extension, `<CAT>-<slug>`, the categories fixed or listed in the index. The model of the record
  reads both forms by release; every rule that reads ADR identifiers takes them from it, so the
  form is decided once.
- **Retired rules.** `docs/no-draft-on-main`, `docs/inherited-adrs`, and `docs/adr-numbering` stay
  known and report nothing from v0.9.0 on. `docs/adr-identifiers` reports from v0.9.0 on and
  accepts the key `docs/adr-numbering` as an older name, as the bash of v0.10.0 does.
- **The key.** Each finding carries the reason the bash names, and the output shows the key
  `<rule>:<reason>` in every format, as API-command-line-keyed-findings and
  API-output-formats-carry-reason decide. On a tree before v0.9.1, whose bash names no key, the key
  is shown all the same: it is the reason of `nuc`, not of that release's wording.
- **The fixture tables.** The tables `check-docs-record.txt`, `check-traceability.txt`, and
  `check-shell.txt` of v0.11.0 are copied into the tests with the blob of each at that tag, as
  TEST-cases-from-template-tables decides. A test fails where a copy differs from its blob. A runner
  builds each case's tree from the table's steps, runs the rules, and compares the keys and
  subjects. A case not yet ported is listed as pending with its rule; each ticket takes out the
  cases it ports, and no case is pending when the spec closes. The cases of `shell/bash-floor` and
  `shell/root` stay listed, with the reason that `nuc` does not port them. The list of copied
  self-test cases of v0.8.1 stays for the earlier releases.
- **Git history.** `docs/adr-identifiers` reads the identifiers the first-parent history deleted,
  and `docs/template-release` the merge base with `origin/main` or `main`. The shell gathers both
  through the git it spawns, under its cleared environment, before a rule runs.
- **`shell/mode`** is the one rule of `check-shell.sh` ported: it reads the mode of each script in
  git's index. ShellCheck, the in-place `sed` and piped `grep -q` rules, `shell/bash-floor`, and
  `shell/root` stay with the bash: the last two run the project's scripts, which Q-7 forbids `nuc`.
  A `nuc.toml` that lists one of them is refused as an unknown rule.
- **Hooks.** Two hook rules are added, on without a `nuc.toml` and governed by its list where
  there is one, as every hook rule is:
  - from v0.9.0, the pre-commit hook runs the documentation check on a commit off `main` that
    concludes no merge, and refuses when it fails;
  - from v0.11.0, the pre-merge-commit hook refuses a merge on a `feat/` branch, and the
    pre-commit hook refuses the conclusion of one.
  The pre-push hook gives its notice on drafts only for a tree before v0.9.0. Today the shim of
  `pre-commit` runs the whole check run on the answer 3; from v0.9.0 it runs the documentation
  check on an ordinary commit and the whole check run on a concluded merge, so the shim has to
  learn from `nuc` which one, with the answer 4 of API-hook-contract-names-check.
- **Corpus.** The template is pinned at the commits of v0.9.0, v0.9.1, v0.10.0, and v0.11.0, and
  this repository on v0.11.0. Each is pinned in the ticket that completes its release, since a
  pinned tree is compared on every rule. The bash of v0.10.0 and later reads its root only from
  `--root`; the harness runs each tree's scripts as that release reads them.
- **Mode.** Every ticket touches a parser of repository content, the fail-closed path, or the
  hooks, the parts DEV-one-agent-one-ticket keeps with the maintainer, so every ticket is worked
  with the human, one at a time.

## Testing decisions

- **The table runner** is the seam for each rule: one case per table row, run through the rules on
  a tree the runner builds through the real git CLI, as the fixtures of TEST-fixtures-with-own-home
  are. It copies the shape of the test that holds the unit tests to the bash self-test cases.
- **The parity harness** is the seam for a release as a whole: each release is done when its
  template tree is pinned and agrees. Variants are added where a rule never fires on a pinned tree.
- **The hook scenarios** are the seam for the hooks, on the template's trees of v0.9.0 and v0.11.0.
- **The contract test** gains each new rule ID, and **the output-format tests** the key in each
  format.
- Each rule's unit tests stay beside the rule, for what the tables do not reach.

## Out of scope

- ShellCheck, the in-place `sed` and piped `grep -q` rules, `shell/bash-floor`, and `shell/root`
  of `check-shell.sh`.
- `check-devcontainer.sh`, the sensors, the plain-rules sensor and its table, and `check-all.sh`.
- `nuc upgrade` (G-4) and moving any derived project to a later release.
- The minimum git version, though `docs/adr-identifiers` needs a git that reads the history as the
  bash's does.
- Template releases after v0.11.0.
- A `nuc.toml` in the template: the template ships it, not this repository.

## ADR impact

- Bind this work: API-version-coupling-hook-defaults (release dispatch, rules off by default, the
  newest release for hooks), API-command-line-keyed-findings (the text output, its key, and the
  hook's answer), API-output-formats-carry-reason (JSON and SARIF), API-hook-contract (what a shim
  runs on the answer), ARCH-rules-outcome-hooks-answer (rules pure, git in the shell),
  SEC-git-cleared-environment (the history reads), TEST-parity-harness, TEST-hook-scenario-parity,
  TEST-fixtures-with-own-home, TEST-copies-keep-old-identifiers, DEV-one-agent-one-ticket.
- Proposed in ticket 01 and accepted by the human on 2026-11-15, each flipped in the ticket that
  implements it:
  - API-command-line-keyed-findings and API-output-formats-carry-reason: every finding ends in its
    key `[<rule>:<reason>]`, and JSON and SARIF carry the reason (ticket 02);
  - TEST-cases-from-template-tables: the copies of the tables, pinned by blob (ticket 12, cut
    from ticket 02);
  - ARCH-rules-need-only-git: no rule that needs a process other than git (ticket 10);
  - API-hook-contract-names-check: `nuc hook pre-commit` answers 4 where the shim runs the
    documentation check, with Q-4 of the specification changed to "answers 3 or 4" (ticket 11).
