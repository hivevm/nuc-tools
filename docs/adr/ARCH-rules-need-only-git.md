# ARCH-rules-need-only-git: `nuc` ports no bash rule whose decision needs a process other than git, such as a script of the project or ShellCheck, and leaves such a rule to the bash

- **Status:** 🟡 proposed
- **Date:** 2026-11-15
- **Deciders:** Markus Brigl
- **Applies to:** which rules of the template's checks `nuc` ports, the rule IDs it knows, the processes `src/shell/` spawns, and the cases `tests/tables.cases` lists as not ported

## Context

The strategy ports the template's bash scripts one by one, each proved by the parity harness
(*Strategy*, Q-2). The constraints make the git CLI the only runtime dependency (*Constraints*).
Q-7 forbids `nuc` to execute repository content, and `nuc` spawns no process but git
([SEC-git-cleared-environment](SEC-git-cleared-environment.md)). Q-4 asks the suite to pass with no
bash on the `PATH` of the process under test.

From template release v0.10.0, `scripts/check-shell.sh` keys three rules. `shell/mode` reads the
mode of each script in git's index. `shell/bash-floor` runs each script that sources the bash floor
under a stand-in version of 3.2 and expects it to stop with its name. `shell/root` runs each script
with a stand-in root and options and reads what it prints. The same script runs ShellCheck, an
external tool, over every script, and refuses `sed -i` and a pipe into `grep -q`, without a key.
What the two run-based rules decide is how a script behaves when bash runs it, which no reading of
its text gives in every case: a script may source the floor through another file, or set its root
in a function of its own. A reader of `nuc`'s rule list who finds `shell/mode` and not its two
siblings would take the gap for unfinished work.

## Decision

We will port no bash rule whose decision needs a process other than git, such as a script of the
project or ShellCheck, and leave each such rule to the bash.

- **Not ported:** `shell/bash-floor` and `shell/root`, which run the project's scripts, and
  ShellCheck, which runs a tool beside git. A rule of a later release that needs such a process
  joins them.
- **Unknown to `nuc`:** a rule left to the bash has no rule ID in `nuc`, so a `nuc.toml` that lists
  one is refused as naming an unknown rule
  ([API-version-coupling-hook-defaults](API-version-coupling-hook-defaults.md)). The template keeps
  such rules in its bash, beside the checks it switches to `nuc`.
- **No approximation:** `nuc` does not read a script's text in place of running it, under the bash
  rule's name or another, since that can decide otherwise than the bash (Q-2).
- **The tables:** each case of a rule left to the bash stands in `tests/tables.cases` as not
  ported, its reason naming this ADR
  ([TEST-template-table-cases](TEST-template-table-cases.md)).

**Out of scope:** the rules of `check-shell.sh` that read text only, `sed -i` and the pipe into
`grep -q`, which carry no key and wait on one; which checks the template switches to `nuc`; the
checks of the project's own, which run in its bash.

## Alternatives considered

- **Running the scripts from `nuc`, as the bash does** — parity by construction, but `nuc` would
  execute repository content, against Q-7, and need bash on the `PATH`, against Q-4.
- **A static approximation** — read a script for the line that sources the floor and for a call of
  `enter_root`, run nothing. It would decide otherwise than the bash where a script sources the
  floor through another file or sets its root by its own means, against Q-2, and a finding of
  `nuc` would then be one the bash does not report.
- **ShellCheck spawned from `nuc`** — one tool for the whole check, but a second process beside git,
  against the constraints, and its findings move with its version, which `nuc` does not pin.
- **Leaving the gap unrecorded** — nothing to write, but a later session would port the rules,
  taking the gap for unfinished work.

## Sources / Prior art

- The template's `scripts/check-shell.sh` at v0.10.0 and v0.11.0, and its
  `scripts/fixtures/check-shell.txt`, whose header says which cases run a script.
- The specification: *Strategy*, *Constraints*, Q-2, Q-4, and Q-7.

## Consequences

- Positive: `nuc` keeps to git as its one process (Q-7, Q-4), and what it ports decides as the bash
  does (Q-2). A reader sees why the two rules are missing.
- Negative / trade-offs: the template cannot drop its `check-shell.sh`, and keeps bash for it on
  every platform where those rules run. A project that wants every check from `nuc` gets the shell
  rules only from the bash.
- Follow-ups: the text rules of `check-shell.sh`, once the template keys them.

## Enforcement

`tests/contract.rs` cites this ADR with `Verifies: ARCH-rules-need-only-git`. It checks that a
`nuc.toml` listing `shell/bash-floor` or `shell/root` is refused as naming an unknown rule. The
structural test `tests/functional_core.rs`, cited the same way, checks that the one place
`src/` starts a process is the spawn of git in `src/shell/git.rs`. `tests/tables.rs` checks that
each case of `shell/bash-floor` and `shell/root` stands in `tests/tables.cases` as not ported.
