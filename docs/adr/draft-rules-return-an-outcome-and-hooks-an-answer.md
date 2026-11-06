# ADR-draft-rules-return-an-outcome-and-hooks-an-answer: One library crate whose rules return an outcome and whose hooks return an answer, its effects in one shell module, and a thin binary

- **Status:** 🟡 proposed
- **Date:** 2026-11-06
- **Deciders:** Markus Brigl
- **Applies to:** the source tree of the crate, `src/main.rs`, `src/shell/`, every rule module, `src/hook.rs`, the type a rule returns and the type a hook returns, and the structural test that holds them
- **Supersedes:** [ADR-0010](0010-rules-return-an-outcome.md)

## Context

[ADR-0010](0010-rules-return-an-outcome.md) decides the shape of the crate: rules as pure
functions that return an outcome, every effect in `src/shell/`, a thin binary. Its context names
the hooks and the merge driver among the computations, but its *Rules* clause was written for
the checks: each rule is a module of its own with one rule ID, and returns its findings, a
refusal of its own, and notes for the summary, which the shell carries out alike for every rule.

The port of the hooks ([ADR-0021](0021-hook-contract.md)) grew another shape, which the design
revision of 2026-11-06 found and no ADR decides:
- the seven `hook/` rules are constants in one module, `src/hook.rs`, each an ID and the reason
  `nuc explain` gives, with no module of its own;
- the decision is one function per hook, not per rule: `pre_commit` decides
  `hook/commit-on-main` and `hook/merge-checks` together, as the bash hook decides both in one
  pass over the same state;
- a hook function returns an answer, pass, refuse with one finding, or run the check (exit 0, 1,
  or 3), and `pre-push` its notices beside it; it has no refusal of its own, since what the shell
  cannot read ends the run before it, and no notes for a summary, which a hook does not print;
- the merge driver is two pure functions, the prose of a side and whether the union stands, and
  the shell has git merge the sides.

The functions in `src/hook.rs` hold to everything else ADR-0010 decides: they name no effect, the
structural test reads them with the rest of the library outside `src/shell/`, and `shell::hook`
gathers the git state, calls them, and carries out the answer. The departure is the module per
rule and the type returned. A reader who takes ADR-0010 at its word would move each hook rule into
a module of its own under `src/rules/` returning an `Outcome`, and split a hook's one decision
into several.

## Decision

We will build `nuc` as one library crate whose rules are pure functions over values that return an
outcome and whose hooks are pure functions over the git state that return an answer, with every
effect in one shell module, `src/shell/`, and a binary, `src/main.rs`, that only calls into it.

- **Rules:** each rule is a module of the library with one stable rule ID, unique across the
  crate (G-6). It takes values, such as file contents, paths, and git data the shell gathered,
  and returns an outcome: its findings; a refusal of its own, the reason it cannot decide the
  tree as its bash check would; and notes for the summary of the run, which are no findings.
- **Hooks:** the decisions of `nuc hook` are in `src/hook.rs`: one function per hook, which takes
  the git state the shell gathered and the `hook/` rules turned on, and returns an answer, pass,
  refuse with one finding, or run the hook's check, with the notices it writes beside it. The
  `hook/` rules are listed there, each with its stable rule ID, unique across the crate, and its
  reason; a hook function decides every rule of its hook. Where the shell cannot read the state,
  the run ends not decided before the function is called.
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

- [ADR-0010](0010-rules-return-an-outcome.md), which this restates, and its sources: the catalogue
  entry
  [*Functional core, imperative shell*](../../.agents/skills/propose-adr/concepts/functional-core.md)
  and Gary Bernhardt, *Functional Core, Imperative Shell* —
  <https://www.destroyallsoftware.com/screencasts/catalog/functional-core-imperative-shell>.
- `src/hook.rs` and `src/shell/hook.rs` as the port of the hooks left them, and the template's
  `.githooks/`, whose each hook decides its refusals in one pass.

## Consequences

- Positive: the record says the shape the hooks have, so a session does not move them into
  `src/rules/`; the hooks keep what ADR-0010 bought, decisions tested on values and every effect
  in one module (Q-1, Q-7 to Q-12); a hook decides its rules in one pass over one state, as the
  bash does (G-2).
- Negative / trade-offs: two kinds of decision, rules and hooks, each with its own type and list;
  a `hook/` rule has no module whose name a reader can find it by; the structural test's rule of
  one module per rule ID does not reach the `hook/` rules, whose IDs `tests/contract.rs` holds.
- Follow-ups: whether the two kinds fold into one at a normalization, if a later command returns
  a third.

## Enforcement

The structural test `tests/functional_core.rs`, cited with
`Verifies: ADR-draft-rules-return-an-outcome-and-hooks-an-answer` once accepted, in place of its
citation of ADR-0010, reads the sources of the library outside `src/shell/`, `src/hook.rs`
among them, and fails on a use of `std::fs`, `std::process`, `std::env`, `std::net`, the standard
streams, or the printing macros, and on any crate the list beside the test names as doing I/O. It
fails as well on a rule module without exactly one rule ID, on a rule ID used twice, on logic in
`src/main.rs` beyond the call into the shell, and on a source of `src/shell/` that calls what a
rule decides: a rule's `check`, or a function of a rule module that the list beside the test
names as part of an outcome, as the pending criteria. The shell may still name a rule module for
the types of its inputs and for the reading of them, as the Template release line a rule checks
by. The lists are the project's, kept beside the test. A fixture shows that a rule's refusal ends
the run with exit 2 and no finding printed. `tests/contract.rs` holds that every `hook/` rule
keeps its ID and is explained (G-6). That the shell stays thin is review.
