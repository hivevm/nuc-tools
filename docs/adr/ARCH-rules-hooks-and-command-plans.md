# ARCH-rules-hooks-and-command-plans: One library crate whose rules return an outcome, whose hooks return an answer, and whose commands that change the tree return a plan, its effects in one shell module, and a thin binary

- **Status:** 🟢 accepted
- **Date:** 2026-11-22
- **Deciders:** Markus Brigl
- **Applies to:** the source tree of the crate, `src/main.rs`, `src/shell/`, every rule module, `src/hook.rs`, `src/upgrade.rs`, the type a rule returns, the type a hook returns, the plan a command returns, the lists of rule IDs and how `nuc explain` reads them, and the structural test that holds them
- **Supersedes:** [ARCH-rules-outcome-hooks-answer](ARCH-rules-outcome-hooks-answer.md)

## Context

`nuc` reads a repository and decides. Its checks, hooks, merge driver, and take-ups are
computations over files and git data (G-1 to G-4). Its effects are few: it spawns git, reads and
writes files, reads the environment, and prints findings. The threats aim at exactly these effects:
acting on another repository through an inherited environment (Q-9), writing outside the repository
(Q-8), reading outside it (Q-12), disclosing a token (Q-10), reaching the network (Q-11), and
executing repository content (Q-7). The harness keeps the code that does them with the maintainer
([DEV-one-agent-one-ticket](DEV-one-agent-one-ticket.md)). Parity (Q-2) asks for many fixtures,
and fixtures are cheapest when a decision is a function of values.

The catalogue entry *Functional core, imperative shell* fits when the logic is rich and the effects
are few and simple, a CLI or a rules engine among its cases, and when a quality goal asks for
exhaustive tests of the logic. It does not fit logic that is mostly effects in sequence, which
`nuc` is not. It does not fit several technologies behind one behaviour either, which the
constraints exclude: git is reached only through its CLI.

ARCH-rules-outcome-hooks-answer decided two kinds of decision. A rule returns an outcome: its
findings, a refusal of its own, and notes for the summary. A hook returns an answer: pass, refuse
with one finding, or run the check, with its notices. Its *Follow-ups* asked whether the kinds fold
into one if a later command returned a third.

A rule that returns findings alone is not enough. Some trees one rule cannot decide as its bash
check would: a wide line holding a byte the bash counts otherwise, for `docs/prose-width`, or a
draft whose name holds a newline, for `docs/no-draft-on-main`. Q-1 asks the run to end not decided
there. And `traceability/coverage` names the criteria no test cites, which are pending work and no
finding ([NUC-decisions-cite-tests §5](NUC-decisions-cite-tests.md)). With findings the only
return, the shell would call these rules by name for both.

`nuc upgrade` returns a third
([API-upgrade-by-blob-toml-optional](API-upgrade-by-blob-toml-optional.md)):
- its three rules, `upgrade/file`, `upgrade/citation`, and `upgrade/notes`, are constants in one
  module, `src/upgrade.rs`, listed in `upgrade::ALL` with their reasons, outside `src/rules/`;
- no `nuc.toml` turns them on or off: a take-up lists every judgment, and `config` would refuse a
  `nuc.toml` naming them as unknown, so `nuc explain` finds them after `config`'s;
- its decision is a set of pure functions. `plan` and `settle` return the plan, the paths to write
  and to delete with the findings. `readme` returns the README with its release line moved, and
  `summary` the counts of the summary. `releases`, `refuse_paths`, `settle`, and `readme` may
  refuse, keyed `upgrade/run:<reason>`;
- `shell::upgrade` reads the trees, the blobs, and the working tree, calls those functions, writes
  the plan, and prints the findings and the summary. It refuses on its own what it reads: no
  argument, `main` checked out, a tree not clean, a README it cannot read, a tag missing under
  `refs/template/`, a release that asks for a newer `nuc`, a tree git cannot list. Each of these is
  keyed `upgrade/run` as well, and ends the run before the first write. A cause the reading of git
  or of `nuc.toml` refuses keeps the key that reading gives, as a missing object does. A write that
  fails after the first is refused as `upgrade/run:write`, naming what was written. Which README the
  line moves in and what the summary counts are the core's, unit-tested on values.

An outcome has no paths to write, and an answer has no findings beyond one. Widening either for one
command would put a plan into every rule or every hook. The design revision of 2026-11-22 found the
shape undecided: the structural test's rule of one module per rule ID does not reach `upgrade/`,
and the record named no third kind.

The merge driver also writes a file, the one git names it, and stays a hook: git decides which
file and runs it, and the driver writes that file alone. The sensors of `nuc sense`, `nuc measure`,
and the reading of `nuc.toml` have shapes their own ADRs decide.

## Decision

We will build `nuc` as one library crate of pure decisions of three kinds, its effects in
`src/shell/`, and a binary, `src/main.rs`, that only calls into it.

- **Rules:** each rule is a module of the library with one stable rule ID, unique across the
  crate (G-6). It takes values the shell gathered, such as file contents, paths, and git data. It
  returns an outcome with three parts. The findings come first. A refusal of its own is the reason
  it cannot decide the tree as its bash check would. Notes for the summary of the run are no
  findings.
- **Hooks:** the decisions of `nuc hook` are in `src/hook.rs`.
  - The `hook/` rules are listed there, each with its stable rule ID, unique across the crate,
    and its reason.
  - Each hook whose bash decides on the git state has one function. It takes the state the shell
    gathered and the `hook/` rules turned on, and decides every rule of its hook. It returns an
    answer: pass, refuse with one finding, or run the hook's check. The notices it writes stand
    beside the answer.
  - Git hands a hook text on the standard input, as the refs of `pre-push` and the updates of
    `reference-transaction`. A pure function of its own reads it and returns what it read or the
    reason it cannot. On that reason the shell ends the run not decided, before the hook's function
    runs. Where the shell cannot read the state, it ends the run the same way.
  - Pure functions over the bytes of its sides decide the merge driver. One finds the prose git
    merges. The other decides whether the union stands over the prose merged on its own. The shell
    has git merge and writes the union or git's conflict into ours. It exits 0 or 1 by that
    decision, as [API-hook-contract-keyed-shims](API-hook-contract-keyed-shims.md) decides.
- **Commands that write a plan:** such a command decides, from trees it reads, which files of the
  working tree to write and to delete. It has one module of the library outside `src/rules/`, named
  for the command; `nuc upgrade` has `src/upgrade.rs`. The merge driver is no such command.
  - Its rules are listed there, each with its stable rule ID, unique across the crate, and its
    reason. No `nuc.toml` names them, and `nuc explain` explains them after the rules `config`
    knows.
  - Its decision is pure functions over the values the shell gathered. They return the changes to
    the tree with the findings, and the counts of its summary, or a refusal keyed
    `<command>/run:<reason>`.
- **Shell:** `src/shell/` is the only place that spawns processes, touches the file system, reads
  the environment, prints, or reaches the network. It gathers the values the rules, hooks, and
  commands need, calls them, and carries out what they return.
  - It treats every rule alike. A refusal of any rule ends the run not decided, exit 2, before any
    finding is printed. Otherwise it prints the findings, and the notes in the summary, whether the
    run passed or not. It names no rule.
  - It carries out every answer of a hook alike. It prints the finding of a refusal and the
    notices, and exits with the answer's code.
  - For a command it ends the run, exit 2, before the first write, on a refusal of the core. Its own
    refusals of what it reads end the run the same way, each keyed `<command>/run:<reason>`. A
    refusal of git's reading or of `nuc.toml` keeps the key that reading gives.
    Otherwise it writes the plan and prints the findings, in the order of their file and line, and
    the summary. It exits 0 without a finding and 1 with one. A write that fails after the first
    ends the run, exit 2, keyed `<command>/run:write`, and names what was written. The shell decides
    nothing of the plan.
  - A refusal of the reading the shell does for several rules stays with that reading, as a byte
    the record's awk splits fields on.
- **Binary:** `src/main.rs` holds no logic; it calls the shell's entry point and exits with the
  code it returns.

**Out of scope:**
- how git is spawned and its environment built, which
  [SEC-git-cleared-without-lazy-fetch](SEC-git-cleared-without-lazy-fetch.md) decides;
- the module layout inside the core beyond one module per rule, one module for the hooks, and one
  module per command that writes a plan;
- the sensors of `nuc sense`, whose cores may ask the shell, which
  [API-sensor-cores-ask-the-shell](API-sensor-cores-ask-the-shell.md) decides, and `nuc measure`,
  which [API-measure-reads-harness-transcripts](API-measure-reads-harness-transcripts.md) decides;
- the output formats;
- what each hook decides, which [API-hook-contract-keyed-shims](API-hook-contract-keyed-shims.md)
  decides;
- what `nuc upgrade` decides, which
  [API-upgrade-by-blob-toml-optional](API-upgrade-by-blob-toml-optional.md) decides.

## Latitude

None.

## Alternatives considered

- **Fold the kinds into one list** — one list of every rule ID with its reason and kind, which
  `config`, `nuc explain`, and the contract tests read alike, each kind keeping its own return
  type. It removes the chain of lists in `nuc explain` and the tests, but `config` and the tests of
  every kind would read a list beside the one each kind keeps, and the return types stay three
  either way. The maintainer chose to record the kinds as they stand.
- **Leave the third kind unrecorded** — nothing to write, but the code departs from the record,
  which [`AGENTS.md` §3](../../AGENTS.md#3-adr-rules), rule 5, forbids, and a session would move
  `upgrade/` into `src/rules/` or the plan into the shell.
- **The plan as an outcome** — an outcome widened by the changes to the tree. Every rule would
  carry an empty plan, and `nuc check` would have to refuse a rule that returned one.
- **The plan's decisions in the shell** — the shape before the design revision of 2026-11-22:
  `shell::upgrade` chose the README to move the line in and counted the summary. Those branches
  were tested only through the binary, and the reviews found them untested.
- **The take-up's rules as rules of `src/rules/`** — one module per rule ID, as the structural test
  reads them. They would be turned on by `nuc.toml` like any rule, while a take-up lists every
  judgment, and a rule module would hold one constant and no check.
- **One kind of decision: every rule a module that returns an outcome** — the record's first
  decision of this shape, written for the checks before the hooks were ported. An outcome has no
  "run the check", so the type would widen for every rule to serve seven hooks, and a hook's one
  decision over one state would be split into rules that each read the state again.
- **A refusal field in the answer, as in an outcome** — no hook function has a case of its own it
  cannot decide: what it cannot decide is the reading of its input, which refuses before it runs.
- **Findings only, with the special cases in the shell as a convention** — the shell names each
  rule that refuses or notes, and each port adds a case.
- **Two optional functions beside the check** — a rule keeps returning findings and may name a
  function that refuses and one that notes. That makes three ways out of a rule where one value
  carries all, and a rule that forgets to wire its refusal passes on what it cannot decide.
- **A refusal as a finding of its own kind** — keeps one list, but every reader of findings must
  then tell the two apart, and a refusal printed as a finding breaks the line contract of
  [API-command-line-with-sense](API-command-line-with-sense.md).
- **Library and thin binary, nothing more prescribed** — the effects could spread into the
  decision modules, and the parts the threats aim at would no longer be one module the maintainer
  reviews.
- **Ports and adapters** — draws the seam at technologies; there is one, git's CLI, and nothing to
  swap behind it.
- **Plugin** — rules loaded at run time would be code `nuc` executes from outside its own
  artifact, against Q-7 and the supply chain of Q-3.
- **A core crate and a shell crate in a workspace** — the crate boundary does not stop the core
  from using `std`. The structural test is needed either way, and one crate is simpler to release.

## Sources / Prior art

- The catalogue entry
  [*Functional core, imperative shell*](../../.agents/skills/propose-adr/concepts/functional-core.md).
- Gary Bernhardt, *Functional Core, Imperative Shell* —
  <https://www.destroyallsoftware.com/screencasts/catalog/functional-core-imperative-shell>.
- Gary Bernhardt, *Boundaries* — <https://www.destroyallsoftware.com/talks/boundaries>.
- ARCH-rules-outcome-hooks-answer, which this restates for the rules, the hooks, the shell, and the
  binary, and its *Follow-ups*.
- The design revision of 2026-11-22, ticket *Decide where the decisions of nuc upgrade stand*;
  `src/upgrade.rs` and `src/shell/upgrade.rs`; `nuc explain` in `src/shell/mod.rs`.

## Consequences

- Positive: the rules, the hooks, and the take-up are tested without mocks, on values built in the
  test, which keeps the parity fixtures cheap (Q-2). Every effect the threats aim at sits in one
  module, small enough to read whole (Q-7 to Q-12); the first command that writes many files
  writes them only there. A rule's refusal and its notes are tested with the rule, on values, and
  the shell treats every rule alike, so a port adds no case to it (Q-1). A hook decides its rules
  in one pass over one state, as the bash does (G-2). The record says the shape each kind has, so
  a session moves neither the hooks nor `upgrade/` into `src/rules/`, nor the plan into the shell.
- Negative / trade-offs: the shell gathers everything a decision might need before calling it, so a
  decision that would read lazily gets its input up front; a take-up reads every blob it might cite.
  Every rule runs before a refusal is reported, while the shell's own refusals end the run before
  any rule runs. The notes go beyond the bash, which names the pending criteria only on a pass; they
  stand on the standard error, which the harness does not compare. There are three kinds of
  decision, each with its own type and list, and `nuc explain` and the contract tests chain the
  lists. The shell has a case per hook, since each hook's state is its own. A take-up's write that
  fails midway leaves the files written so far, which its refusal names. A `hook/` or `upgrade/`
  rule has no module whose name a reader can find it by, and the structural test's rule of one
  module per rule ID does not reach it. The structural test reads source text, so a forbidden path
  written in an unusual form can slip past it until the list grows.
- Follow-ups: whether the shell is split further once it grows; whether the kinds fold into one
  list at a later normalization, if a second command that writes a plan comes.

## Enforcement

A structural test, `tests/functional_core.rs`, cited with this ADR's identifier, reads the sources
of the library outside `src/shell/`, `src/hook.rs` and `src/upgrade.rs` among them. The test fails
on a use of `std::fs`, `std::process`, `std::env`, `std::net`, the standard streams, or the printing
macros. It fails on any crate the list beside the test names as doing I/O. It fails as well on a
rule module without exactly one rule ID and on a rule ID used twice. It fails on logic in
`src/main.rs` beyond the call into the shell. It fails on a source of `src/shell/` that calls what a
rule decides. That is a rule's `check`, or a function of a rule module that the list beside the test
names as part of an outcome. The pending criteria are one. The shell may still name a rule module,
for the types of its inputs and their reading. The lists are the project's, kept beside the test. A
fixture of `tests/check.rs` shows that a rule's refusal ends the run with exit 2 and no finding
printed.

`tests/contract.rs` fails on an ID that `rules::ALL`, `hook::ALL`, and `upgrade::ALL` together hold
twice, in `no_rule_id_is_held_twice_by_the_rules_the_hooks_and_the_take_up`; this test, not the
structural one, reaches the `hook/` and `upgrade/` rules. It holds that every `hook/` and `upgrade/`
rule keeps its ID and is explained, and that no `nuc.toml` turns an `upgrade/` rule on (G-6). It
holds every key of the `upgrade/run` refusals.
`every_refusal_of_upgrade_writes_nothing_and_ends_in_its_key` holds that each refusal before the
first write writes nothing and ends in its key. The unit tests of `src/hook.rs` show that each
reader of git's input refuses a line not in git's shape. The unit tests of `src/upgrade.rs` show
which README a take-up writes and what its summary counts. That the shell stays thin, and that each
command that writes a plan has one module outside `src/rules/`, is review.
