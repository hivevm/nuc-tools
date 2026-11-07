# ADR-0024: One library crate whose rules return an outcome and whose hooks return an answer, its effects in one shell module, and a thin binary

- **Status:** 🟢 accepted
- **Date:** 2026-11-06
- **Deciders:** Markus Brigl
- **Applies to:** the source tree of the crate, `src/main.rs`, `src/shell/`, every rule module, `src/hook.rs`, the type a rule returns and the type a hook returns, and the structural test that holds them
- **Supersedes:** [ADR-0010](0010-rules-return-an-outcome.md)

## Context

`nuc` reads a repository and decides: its checks, hooks, merge driver, and migrations are
computations over files and git data (G-1 to G-4). Its effects are few: spawning git, reading and
writing files, reading the environment, printing findings. Those effects are exactly what the
threats aim at: acting on another repository through an inherited environment (Q-9), writing
outside the repository (Q-8), reading outside it (Q-12), disclosing a token (Q-10), reaching the
network (Q-11), executing repository content (Q-7). The harness keeps the code that does them
with the maintainer ([ADR-0007](0007-harness.md)). Parity (Q-2) asks for many fixtures, which are
cheapest when a rule is a function of values.

The catalogue entry *Functional core, imperative shell* fits when the logic is rich and the
effects are few and simple, a CLI or a rules engine among its cases, and when a quality goal asks
for exhaustive tests of the logic. It does not fit logic that is mostly effects in sequence, which
`nuc` is not, or several technologies behind one behaviour, which the constraints exclude: git is
reached only through its CLI.

A rule that returns findings alone is not enough: the ports of `check-docs.sh` and
`check-traceability.sh` show two more things a single rule has to say. Some trees one rule cannot
decide as its bash check would, as a wide line holding a byte the bash counts otherwise for
`docs/prose-width`, or a draft whose name holds a newline for `docs/no-draft-on-main`; Q-1 asks the
run to end not decided there. And `traceability/coverage` names the criteria no test cites, which
are pending work and no finding ([ADR-0002](0002-decisions-verified-by-tests.md)). With findings
the only return, the shell would call these rules by name for both, and each port would add a
case to it.

[ADR-0010](0010-rules-return-an-outcome.md) decided all of the above, with a *Rules* clause
written for the checks: each rule a module of its own with one rule ID, returning its findings, a
refusal of its own, and notes for the summary, which the shell carries out alike for every rule.
The port of the hooks ([ADR-0021](0021-hook-contract.md)) grew another shape, which the design
revision of 2026-11-06 found and no ADR decides:
- the seven `hook/` rules are constants in one module, `src/hook.rs`, each an ID and the reason
  `nuc explain` gives, with no module of its own;
- the decision is one function per hook, not per rule: `pre_commit` decides
  `hook/commit-on-main` and `hook/merge-checks` together, as the bash hook decides both in one
  pass over the same state;
- a hook function returns an answer, pass, refuse with one finding, or run the check (exit 0, 1,
  or 3), and `pre-push` its notices beside it; it has no notes for a summary, which a hook does not
  print;
- what a hook cannot decide is refused before its function runs: by the shell, where it cannot
  read the state, and by the pure readers of git's standard input, `pushed_refs` and
  `ref_updates`, which return the lines they read or the reason a line is not in git's shape;
- the merge driver has no hook function: two pure functions decide it, the prose of a side and
  whether the union stands, and the shell has git merge the sides, writes the result, and exits
  by that decision.

Everything else ADR-0010 decides holds for these functions. They name no effect. The structural
test reads them with the rest of the library outside `src/shell/`. `shell::hook` gathers the git
state, calls them, and carries out what they return. The departure is the module per rule and the
type returned. A reader who takes ADR-0010 at its word would move each hook rule into a module of
its own under `src/rules/` returning an `Outcome`, and split a hook's one decision into several.

## Decision

We will build `nuc` as one library crate whose rules are pure functions over values that return an
outcome and whose hooks are pure functions over the git state that return an answer, with every
effect in one shell module, `src/shell/`, and a binary, `src/main.rs`, that only calls into it.

- **Rules:** each rule is a module of the library with one stable rule ID, unique across the
  crate (G-6). It takes values, such as file contents, paths, and git data the shell gathered,
  and returns an outcome: its findings; a refusal of its own, the reason it cannot decide the
  tree as its bash check would; and notes for the summary of the run, which are no findings.
- **Hooks:** the decisions of `nuc hook` are in `src/hook.rs`.
  - The `hook/` rules are listed there, each with its stable rule ID, unique across the crate,
    and its reason.
  - Each hook whose bash decides on the git state has one function, which takes the state the
    shell gathered and the `hook/` rules turned on, decides every rule of its hook, and returns an
    answer: pass, refuse with one finding, or run the hook's check, with the notices it writes
    beside it.
  - What git hands a hook as text, as the refs of `pre-push` and the updates of
    `reference-transaction` on the standard input, is read by a pure function of its own, which
    returns what it read or the reason it cannot, and the shell ends the run not decided on that
    reason before the hook's function runs. Where the shell cannot read the state, it ends the run
    the same way.
  - The merge driver is decided by pure functions over the bytes of its sides: the prose git
    merges, and whether the union stands over the prose merged on its own. The shell has git
    merge, writes the union or git's conflict into ours, and exits 0 or 1 by that decision, as
    [ADR-0021](0021-hook-contract.md) decides.
- **Shell:** `src/shell/` is the only place that spawns processes, touches the file system, reads
  the environment, prints, or reaches the network. It gathers the values the rules and hooks
  need, calls them, and carries out what they return. For the rules it does so the same for every
  rule: a refusal of any rule ends the run not decided, exit 2, before any finding is printed;
  otherwise it prints the findings, and the notes in the summary, whether the run passed or not.
  It names no rule. For the hooks it carries out every answer alike: it prints the finding of a
  refusal and the notices, and exits with the answer's code. A refusal of the reading the shell
  does for several rules, as a byte the record's awk splits fields on, stays with that reading.
- **Binary:** `src/main.rs` holds no logic; it calls the shell's entry point and exits with the
  code it returns.

**Out of scope:** how git is spawned and its environment built, a decision of its own; the
module layout inside the core beyond one module per rule and one module for the hooks; the output
formats; what each hook decides, which [ADR-0021](0021-hook-contract.md) decides.

## Alternatives considered

- **Each hook rule a module under `src/rules/` returning an outcome** — the letter of ADR-0010.
  An outcome has no "run the check", so the type would widen for every rule to serve seven; a
  hook's one decision over one state would be split into rules that each read the branch and
  `MERGE_HEAD` again, and `nuc check` would have to skip them among its own rules.
- **The hooks left as they are, ADR-0010 unchanged** — nothing to write, but the code departs
  from an accepted ADR, which [`AGENTS.md` §3](../../AGENTS.md#3-adr-rules), rule 5, forbids, and
  the next session "fixes" it by the alternative above.
- **The hooks recorded at the next normalization** — a normalization changes no decision, and
  this one is not in the record yet.
- **A refusal field in the answer, as in an outcome** — one type closer to the rules', but no hook
  function has a case of its own it cannot decide: what it cannot decide is the reading of its
  input, which refuses before it runs.
- **Findings only, with the special cases in the shell as a convention** — the same shape with a
  rule that returns findings alone; the shell names each rule that refuses or notes, and each
  port adds a case, the growth a design revision found.
- **Two optional functions beside the check** — a rule keeps returning findings and may name a
  function that refuses and one that notes; three ways out of a rule where one value carries all,
  and a rule that forgets to wire its refusal passes on what it cannot decide.
- **A refusal as a finding of its own kind** — keeps one list, but every reader of findings must
  then tell the two apart, and a refusal printed as a finding breaks the line contract of
  [ADR-0020](0020-command-line-contract-with-hook-signal.md).
- **Library and thin binary, nothing more prescribed** — the effects could spread into the rule
  modules, and the parts the threats aim at would no longer be one module the maintainer reviews.
- **Ports and adapters** — draws the seam at technologies; there is one, git's CLI, and nothing to
  swap behind it.
- **Plugin** — rules loaded at run time would be code `nuc` executes from outside its own
  artifact, against Q-7 and the supply chain of Q-3.
- **A core crate and a shell crate in a workspace** — the crate boundary does not stop the core
  from using `std`; the structural test is needed either way, and one crate is simpler to release.

## Sources / Prior art

- The catalogue entry
  [*Functional core, imperative shell*](../../.agents/skills/propose-adr/concepts/functional-core.md).
- Gary Bernhardt, *Functional Core, Imperative Shell* —
  <https://www.destroyallsoftware.com/screencasts/catalog/functional-core-imperative-shell>.
- Gary Bernhardt, *Boundaries* — <https://www.destroyallsoftware.com/talks/boundaries>.
- The design revision of 2026-10-31, tickets *Decide where a rule's own refusals live* and
  *Decide how a rule reports what is not a finding*.
- The design revision of 2026-11-06, which found the shape of the hooks; `src/hook.rs` and
  `src/shell/hook.rs` as the port of the hooks left them; the template's `.githooks/`, each of
  whose hooks decides its refusals in one pass.

## Consequences

- Positive: the rules and the hooks are tested without mocks, on values built in the test, which
  keeps the parity fixtures cheap (Q-2); every effect the threats aim at sits in one module, small
  enough to read whole (Q-7 to Q-12); a rule's refusal and its notes are tested with the rule, on
  values, and the shell treats every rule alike, so a port adds no case to it (Q-1); the record
  says the shape the hooks have, so a session does not move them into `src/rules/`, and a hook
  decides its rules in one pass over one state, as the bash does (G-2).
- Negative / trade-offs: the shell has to gather everything a rule might need before calling it,
  so a rule that would read lazily gets its input up front; every rule runs before a refusal is
  reported, where the shell's own refusals end the run before any; the notes go beyond the bash,
  which names the pending criteria only on a pass, on the standard error the harness does not
  compare; the structural test reads source text, so a forbidden path written in an unusual form
  can slip past it until the list grows; two kinds of decision, rules and hooks, each with its own
  type and list, and the shell's case per hook, which a hook's state makes its own; a `hook/` rule
  has no module whose name a reader can find it by, and the structural test's rule of one module
  per rule ID does not reach it.
- Follow-ups: how the shell spawns git with a cleared environment; whether the shell is split
  further once it grows; whether the two kinds of decision fold into one at a normalization, if a
  later command returns a third.

## Enforcement

A structural test, cited with this ADR's identifier, reads the sources of the library outside
`src/shell/`, `src/hook.rs` among them, and fails on a use of `std::fs`, `std::process`,
`std::env`, `std::net`, the standard streams, or the printing macros, and on any crate the list
beside the test names as doing I/O. It fails as well on a rule module without exactly one rule ID,
on a rule ID used twice, on logic in `src/main.rs` beyond the call into the shell, and on a source
of `src/shell/` that calls what a rule decides: a rule's `check`, or a function of a rule module
that the list beside the test names as part of an outcome, as the pending criteria. The shell may
still name a rule module for the types of its inputs and for the reading of them, as the Template
release line a rule checks by. The lists are the project's, kept beside the test. A fixture shows
that a rule's refusal ends the run with exit 2 and no finding printed.

`tests/contract.rs`, extended by the implementation of this ADR, fails on an ID that
`rules::ALL` and `hook::ALL` together hold twice, besides holding that every `hook/` rule keeps its
ID and is explained (G-6). The unit tests of `src/hook.rs` show that each reader of git's input
refuses a line not in git's shape. That the shell stays thin is review.
