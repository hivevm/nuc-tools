# API-git-only-sensors-under-sense: `nuc sense <name>` ports the template's sensors that need only git, prints their report byte for byte as the bash does, and exits as every `nuc` command does

- **Status:** 🟢 accepted
- **Date:** 2026-11-19
- **Deciders:** Markus Brigl
- **Applies to:** which sensors of the template `nuc` ports, the `nuc sense` command, its source in the shell and in the core, the names of the sensors it knows, its exit codes against the bash's, and the comparison of the sensors in the parity harness

## Context

The problem names the sensors among the bash `nuc` is to replace (*Problem*), and the strategy
ports the scripts one by one, each proved by the parity harness (*Strategy*, Q-2). Q-2 already
counts every ported script. G-1 to G-8 name no sensor, and
[TEST-parity-harness](TEST-parity-harness.md) compares findings mapped to rule IDs. A sensor
prints a report, not findings. The template ships twelve sensors:

- **Only files and git:** `sensor-action-refs.sh`, `sensor-backlog.sh`, `sensor-binding-adrs.sh`,
  `sensor-plain-rules.sh`, `sensor-plan-frontier.sh`, `sensor-review-binding.sh`,
  `sensor-revision-due.sh`, and `sensor-tests-kept.sh`.
- **A process of the project:** `sensor-red-commits.sh` runs `scripts/check-all.sh` of each commit
  it reads.
- **The network:** `sensor-release-notes.sh`, `sensor-repo-settings.sh`, and
  `sensor-work-measures.sh` call `gh` and `jq`.

Two sensors also run as checks. `scripts/check-all.sh` runs `sensor-plain-rules.sh --require` and
`sensor-action-refs.sh --require sha`, and fails on their exit 1. A sensor's report is prose a
session reads, and its first line is what `check-all.sh` shows. Some sensors exit 1 where they
cannot read their input, as `sensor-revision-due.sh` does without a readable *Last design
revision* line. The contract of the command line gives every cause that keeps `nuc` from deciding
the exit 2, and keeps the standard output for findings and measures; the report of a sensor needs
a clause of its own, which
[API-command-line-with-sense](API-command-line-with-sense.md) adds. Every bash sensor takes
`--root <dir>`, where every `nuc` command takes `--repo <path>`. Some sensors exist only from a
later release: `sensor-backlog.sh` and `sensor-plain-rules.sh` from v0.8.0.

[ARCH-rules-outcome-hooks-answer](ARCH-rules-outcome-hooks-answer.md) decides the rules, the hooks,
and the shell, and leaves the module layout inside the core beyond them open. `nuc measure` is the
prior art of a command whose core returns neither an outcome nor an answer: `src/measure.rs`
counts in the core, and the shell reads and prints.

[ARCH-rules-need-only-git](ARCH-rules-need-only-git.md) keeps every rule that needs a process
other than git in the bash, for Q-4 and Q-7. The constraints make git the only runtime dependency,
and the non-goals allow the network only in a sensor that names it. Without a decision, a session
would put the sensors under `nuc check`, where their reports do not fit the format of a finding,
or would port `sensor-red-commits.sh` with a run of the project's scripts.

## Decision

We will port the template's eight sensors that need only git as `nuc sense <name>`, with the
bash's report and the exit codes of
[API-command-line-with-sense](API-command-line-with-sense.md).

- **Names:** `<name>` is the part of the script's name after `sensor-` and before `.sh`, such as
  `revision-due`. `nuc sense` knows `action-refs`, `backlog`, `binding-adrs`, `plain-rules`,
  `plan-frontier`, `review-binding`, `revision-due`, and `tests-kept`.
- **Not ported:** `red-commits`, `release-notes`, `repo-settings`, and `work-measures`. `nuc sense`
  must refuse each of them as an unknown sensor, with exit 2. The template keeps them in its
  bash.
- **Arguments:** `nuc sense` must take the arguments the bash sensor takes, `--require` among
  them, with the same meaning. It must take `--repo <path>` in place of `--root <dir>`, as every
  `nuc` command does, and must refuse `--root`.
- **Standard output:** `nuc sense` must write the report on the standard output, byte for byte as
  the bash sensor writes it there.
- **Core:** each sensor must be a pure function of the library outside `src/shell/`. It takes the
  values the shell gathered and returns its report or a refusal. The shell prints the report or
  the refusal and exits. A sensor reads no file and runs no process, as a rule does.
- **Release:** `nuc sense` must report as the sensor of the release the tree carries, as the rules
  do ([API-version-coupling-hook-defaults](API-version-coupling-hook-defaults.md)). Where the
  README names no release, it must report as the newest release `nuc` knows.
  - A tree whose release has no such sensor is not decided. `nuc sense` must exit 2 there, with
    the key `[sense/<name>:release]`.
  - `tests/parity/releases.tsv` must name the releases at which each sensor changed, in a row
    `sense/<name>`, as it does for each rule. A change of the sensor's script or of a library it
    sources counts.
- **Configuration:** `nuc sense` must read `nuc.toml` as `nuc check` does and refuse where
  `nuc check` refuses, so that G-5 holds for it. It must not read the `rules` key for a choice of
  its own.
- **Exit codes and keys:** [API-command-line-with-sense](API-command-line-with-sense.md) decides
  them for `nuc sense`, and this ADR repeats none of them.
- **Parity:** the parity harness must run each ported sensor and `nuc sense` on every tree and
  variant of the corpus whose release has the sensor.
  - A table beside the harness must name the arguments and the git state of each run.
  - The harness must compare the standard output byte for byte and the exit status, and fail on
    any difference.
  - A second table must list each bash exit that differs from `nuc`'s by this ADR, with the
    release and the cause. The harness must fail on a difference no row lists.
  - The corpus must pin a tree of each release at which a ported sensor changed.

**Out of scope:**

- the wording of a refusal on the standard error;
- a sensor the template adds later, which joins the ported or the not ported by the rule above;
- when the template switches its sensors to `nuc sense`;
- an output format other than the bash's.

## Latitude

None.

## Alternatives considered

- **The sensors as rules of `nuc check`** — one command, but a report is not a finding with a rule
  ID and a line, and `nuc.toml` would have to turn on what a session runs by hand.
- **Only the two `--require` modes as rules, the reports under `nuc sense`** — the checks in the
  finding format, but one sensor then has two outputs to keep at parity, and two places decide
  the same text.
- **Also the sensors that call `gh`** — the whole set, but a second process beside git breaks the
  constraint on runtime dependencies, and the network breaks Q-11 for a command a gate runs.
- **`red-commits` with a run of `check-all.sh`** — the bash's behaviour, but `nuc` would execute
  repository content, against Q-7, as [ARCH-rules-need-only-git](ARCH-rules-need-only-git.md)
  rejects for the rules.
- **Exit codes as the bash's** — strict parity, but `nuc sense` would answer 1 for "cannot
  decide" where every other command answers 2, and a gate could no longer tell a failed
  `--require` from an unreadable tree.
- **`--root` as the bash names it** — a call copied from the bash runs unchanged, but `nuc` would
  name the repository two ways, and `--repo` is what every command of `nuc` takes.
- **A superseding ADR for the shape of the core** — a third kind of decision, a report, beside the
  outcome and the answer of
  [ARCH-rules-outcome-hooks-answer](ARCH-rules-outcome-hooks-answer.md). That ADR leaves the
  layout of the core beyond the rules and the hooks open, and `nuc measure` already returns a
  third kind under it. Its structural test holds the sensors to no effect outside the shell as it
  stands.
- **The bash sensor's report on a tree of a release without it** — the newest sensor would report
  on an old tree, which no bash of that release does, and parity has nothing to compare.
- **Parity of the counts only, not the wording** — looser to keep, but `check-all.sh` shows the
  first line, and a session reads the report as the bash wrote it.
- **Parity on the template's self-test fixtures only** — no corpus run, but the corpus is where
  the rules showed differences the fixtures missed.
- **One pilot sensor, the pattern decided per sensor** — less up front, but each later port would
  reopen the command, the exit codes, and the parity.

## Sources / Prior art

- The template's sensors under `scripts/` and their self-tests `scripts/test-sensor-*.sh`, at the
  release this repository carries.
- `scripts/check-all.sh`, which tells a sensor from a check by `--require`.
- The specification: *Problem*, *Strategy*, *Constraints*, *Non-Goals*, G-5, Q-2, Q-4, Q-7, and
  Q-11.
- [ARCH-rules-need-only-git](ARCH-rules-need-only-git.md),
  [API-command-line-with-measure](API-command-line-with-measure.md),
  [ARCH-rules-outcome-hooks-answer](ARCH-rules-outcome-hooks-answer.md), `src/measure.rs`, and
  [TEST-parity-harness](TEST-parity-harness.md), whose patterns this ADR carries over to the
  sensors.

## Consequences

- Positive: a project runs eight sensors without bash (Q-4), and the two sensors that act as
  checks can move to `nuc` with the rest of the gates. Parity is decided byte for byte, so the
  report a session reads does not drift (Q-2).
- Negative / trade-offs: `nuc` copies the bash's prose, and every change of a sensor's wording in
  a template release is a change of `nuc`. The parity harness needs scenarios with arguments and
  git state, since most sensors read a range of commits, and a pinned tree alone has none. The
  exit table is one more table to keep. Four sensors stay in the bash.
- Follow-ups: the specification's criterion for the sensors, proposed beside this ADR; the
  scenarios of the sensors in the parity harness; the port of each sensor; a later decision on
  the sensors that call `gh`, should the constraint on runtime dependencies move.

## Enforcement

`tests/contract.rs` must cite this ADR with `Verifies: API-git-only-sensors-under-sense`. It must
check the following:

- `nuc sense` with each of the four not ported names exits 2, with the key `[sense/unknown:name]`.
- With a `nuc.toml` above the running version, `nuc sense` exits 2 as too old before it judges the
  name.
- `nuc sense` with `--root` exits 2.
- On a tree whose release has no such sensor, `nuc sense` exits 2, with the key
  `[sense/<name>:release]`.
- A sensor that cannot read its input exits 2, and a failed `--require` exits 1.

`tests/releases.rs` must cite it the same way and hold each row `sense/<name>` to the template's
trees, as it holds the rows of the rules. `tests/functional_core.rs` must cite it and hold the
sensors' modules to no effect outside the shell.

The self-test of the parity harness, `scripts/test-check-parity.sh`, must cite it the same way. It
must check that each of the following fails:

- a pair of sensor outputs that differ in one byte;
- a pair that differs in an exit status no row of the exit table lists.
