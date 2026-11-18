# ARCH-rules-outcome-hooks-answer: One library crate whose rules return an outcome and whose hooks return an answer, its effects in one shell module, and a thin binary

- **Status:** 🟢 accepted
- **Date:** 2026-11-06
- **Deciders:** Markus Brigl
- **Applies to:** the source tree of the crate, `src/main.rs`, `src/shell/`, every rule module, `src/hook.rs`, the type a rule returns and the type a hook returns, and the structural test that holds them

## Context

`nuc` reads a repository and decides. Its checks, hooks, merge driver, and migrations are
computations over files and git data (G-1 to G-4). Its effects are few: it spawns git, reads and
writes files, reads the environment, and prints findings. The threats aim at exactly these effects:
acting on another repository through an inherited environment (Q-9), writing outside the repository
(Q-8), reading outside it (Q-12), disclosing a token (Q-10), reaching the network (Q-11), and
executing repository content (Q-7). The harness keeps the code that does them with the maintainer
([DEV-one-agent-one-ticket](DEV-one-agent-one-ticket.md)). Parity (Q-2) asks for many fixtures,
and fixtures are cheapest when a rule is a function of values.

The catalogue entry *Functional core, imperative shell* fits when the logic is rich and the effects
are few and simple, a CLI or a rules engine among its cases, and when a quality goal asks for
exhaustive tests of the logic. It does not fit logic that is mostly effects in sequence, which
`nuc` is not. It does not fit several technologies behind one behaviour either, which the
constraints exclude: git is reached only through its CLI.

A rule that returns findings alone is not enough. The rules ported from `check-docs.sh` and
`check-traceability.sh` say two more things. Some trees one rule cannot decide as its bash check
would: a wide line holding a byte the bash counts otherwise, for `docs/prose-width`, or a draft
whose name holds a newline, for `docs/no-draft-on-main`. Q-1 asks the run to end not decided there.
And `traceability/coverage` names the criteria no test cites, which are pending work and no finding
([NUC-decisions-cite-tests §5](NUC-decisions-cite-tests.md)). With findings the only return, the
shell would call these rules by name for both, and each port would add a case to it.

The hooks have a shape of their own, which the design revision of 2026-11-06 found in the port of
the hooks ([API-hook-contract-keyed-shims](API-hook-contract-keyed-shims.md)):
- the seven `hook/` rules are constants in one module, `src/hook.rs`, each an ID and the reason
  `nuc explain` gives, with no module of its own;
- the decision is one function per hook, not per rule: `pre_commit` decides `hook/commit-on-main`
  and `hook/merge-checks` together, as the bash hook decides both in one pass over the same state;
- a hook function returns an answer: pass, refuse with one finding, or run the check (exit 0, 1,
  or 3), and `pre_push` returns its notices beside it; it has no notes for a summary, since a hook
  prints none;
- what a hook cannot decide is refused before its function runs: by the shell, where it cannot read
  the state, and by the pure readers of git's standard input, `pushed_refs` and `ref_updates`,
  which return the lines they read or the reason a line is not in git's shape;
- the merge driver has no hook function: two pure functions decide it, `prose` for the prose of a
  side and `takes_union` for whether the union stands, and the shell has git merge the sides,
  writes the result, and exits by that decision.

These functions name no effect, as the rules name none. The structural test reads them with the
rest of the library outside `src/shell/`. `shell::hook` gathers the git state, calls them, and
carries out what they return.

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
    [API-hook-contract-keyed-shims](API-hook-contract-keyed-shims.md) decides.
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

**Out of scope:** how git is spawned and its environment built, which
[SEC-git-cleared-environment](SEC-git-cleared-environment.md) decides; the module layout inside the
core beyond one module per rule and one module for the hooks; the output formats; what each hook
decides, which [API-hook-contract-keyed-shims](API-hook-contract-keyed-shims.md) decides.

## Alternatives considered

- **One kind of decision: every rule a module that returns an outcome** — the record's first
  decision of this shape, written for the checks before the hooks were ported. It did not stand
  once the hooks were ported. An outcome has no "run the check", so the type would widen for every
  rule to serve seven. A hook's one decision over one state would be split into rules that each
  read the branch and `MERGE_HEAD` again. And `nuc check` would have to skip them among its own
  rules. Everything else that decision held stands here: the outcome of a rule, the shell, the
  binary, and the structural test.
- **The hooks' shape left unrecorded** — nothing to write, but the code would depart from the
  record, which [`AGENTS.md` §3](../../AGENTS.md#3-adr-rules), rule 5, forbids, and the next
  session would "fix" it by the alternative above.
- **A refusal field in the answer, as in an outcome** — one type closer to the rules', but no hook
  function has a case of its own it cannot decide: what it cannot decide is the reading of its
  input, which refuses before it runs.
- **Findings only, with the special cases in the shell as a convention** — the same shape with a
  rule that returns findings alone. The shell names each rule that refuses or notes, and each port
  adds a case, the growth a design revision found.
- **Two optional functions beside the check** — a rule keeps returning findings and may name a
  function that refuses and one that notes. That makes three ways out of a rule where one value
  carries all, and a rule that forgets to wire its refusal passes on what it cannot decide.
- **A refusal as a finding of its own kind** — keeps one list, but every reader of findings must
  then tell the two apart, and a refusal printed as a finding breaks the line contract of
  [API-command-line-with-measure](API-command-line-with-measure.md).
- **Library and thin binary, nothing more prescribed** — the effects could spread into the rule
  modules, and the parts the threats aim at would no longer be one module the maintainer reviews.
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
- The design revision of 2026-10-31, tickets *Decide where a rule's own refusals live* and
  *Decide how a rule reports what is not a finding*.
- The design revision of 2026-11-06, which found the shape of the hooks; `src/rules/mod.rs`, whose
  `Outcome` a rule returns; `src/hook.rs` and `src/shell/hook.rs`; the template's `.githooks/`,
  each of whose hooks decides its refusals in one pass.

## Consequences

- Positive: the rules and the hooks are tested without mocks, on values built in the test, which
  keeps the parity fixtures cheap (Q-2). Every effect the threats aim at sits in one module, small
  enough to read whole (Q-7 to Q-12). A rule's refusal and its notes are tested with the rule, on
  values, and the shell treats every rule alike, so a port adds no case to it (Q-1). The record
  says the shape the hooks have, so a session does not move them into `src/rules/`, and a hook
  decides its rules in one pass over one state, as the bash does (G-2).
- Negative / trade-offs: the shell gathers everything a rule might need before calling it, so a
  rule that would read lazily gets its input up front. Every rule runs before a refusal is
  reported, while the shell's own refusals end the run before any rule runs. The notes go beyond
  the bash, which names the pending criteria only on a pass; they stand on the standard error,
  which the harness does not compare. The structural test reads source text, so a forbidden path
  written in an unusual form can slip past it until the list grows. There are two kinds of
  decision, rules and hooks, each with its own type and list, and the shell has a case per hook,
  since each hook's state is its own. A `hook/` rule has no module whose name a reader can find it
  by, and the structural test's rule of one module per rule ID does not reach it.
- Follow-ups: whether the shell is split further once it grows; whether the two kinds of decision
  fold into one at a later normalization, if a later command returns a third.

## Enforcement

A structural test, `tests/functional_core.rs`, cited with this ADR's identifier, reads the sources
of the library outside `src/shell/`, `src/hook.rs` among them. The test fails on a use of `std::fs`,
`std::process`, `std::env`, `std::net`, the standard streams, or the printing macros, and on any
crate the list beside the test names as doing I/O. It fails as well on a rule module without exactly
one rule ID and on a rule ID used twice. It fails on logic in `src/main.rs` beyond the call into the
shell. It fails on a source of `src/shell/` that calls what a rule decides: a rule's `check`, or a
function of a rule module that the list beside the test names as part of an outcome, as the pending
criteria. The shell may still name a rule module for the types of its inputs and for the reading of
them. An example is the Template release line a rule checks by. The lists are the project's, kept
beside the test. A fixture of `tests/check.rs` shows that a rule's refusal ends the run with exit 2
and no finding printed.

`tests/contract.rs` fails on an ID that `rules::ALL` and `hook::ALL` together hold twice. It also
holds that every `hook/` rule keeps its ID and is explained (G-6). The unit tests of `src/hook.rs`
show that each reader of git's input refuses a line not in git's shape. That the shell stays thin is
review.
