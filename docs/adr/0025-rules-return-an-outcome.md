# ADR-0025: One library crate whose rules return an outcome, its effects in one shell module, and a thin binary

- **Status:** 🟡 proposed
- **Date:** 2026-11-01
- **Deciders:** Markus Brigl
- **Applies to:** the source tree of the crate, `src/main.rs`, `src/shell/`, every rule module, the type a rule returns, and the structural test that holds them
- **Supersedes:** [ADR-0009](0009-functional-core.md)

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

ADR-0009 decided that shape with a rule that returns findings. The port of `check-docs.sh` and
`check-traceability.sh` found two more things a single rule has to say. Some trees one rule cannot
decide as its bash check would, as a wide line holding a byte the bash counts otherwise for
`docs/prose-width`, or a draft whose name holds a newline for `docs/no-draft-on-main`; Q-1 asks the
run to end not decided there. And `traceability/coverage` names the criteria no test cites, which
are pending work and no finding (ADR-0023). With findings the only return, the shell called these
rules by name for both, and each port added a case to it.

## Decision

We will build `nuc` as one library crate whose rules are pure functions over values that return an
outcome, with every effect in one shell module, `src/shell/`, and a binary, `src/main.rs`, that only
calls into it.

- **Rules:** each rule is a module of the library with one stable rule ID, unique across the
  crate (G-6). It takes values, such as file contents, paths, and git data the shell gathered,
  and returns an outcome: its findings; a refusal of its own, the reason it cannot decide the
  tree as its bash check would; and notes for the summary of the run, which are no findings.
- **Shell:** `src/shell/` is the only place that spawns processes, touches the file system, reads
  the environment, prints, or reaches the network. It gathers the values the rules need, calls
  them, and carries out what they return, the same for every rule: a refusal of any rule ends the
  run not decided, exit 2, before any finding is printed; otherwise it prints the findings, and
  the notes in the summary, whether the run passed or not. It names no rule. A refusal of the
  reading the shell does for several rules, as a byte the record's awk splits fields on, stays
  with that reading.
- **Binary:** `src/main.rs` holds no logic; it calls the shell's entry point and exits with the
  code it returns.

**Out of scope:** how git is spawned and its environment built, a decision of its own; the
module layout inside the core beyond one module per rule; the output formats.

## Alternatives considered

- **Findings only, with the special cases in the shell as a convention** — keeps ADR-0009 as it
  stands; the shell names each rule that refuses or notes, and each port adds a case, the growth
  the design revision found.
- **Two optional functions beside the check** — a rule keeps returning findings and may name a
  function that refuses and one that notes; three ways out of a rule where one value carries all,
  and a rule that forgets to wire its refusal passes on what it cannot decide.
- **A refusal as a finding of its own kind** — keeps one list, but every reader of findings must
  then tell the two apart, and a refusal printed as a finding breaks the line contract of
  ADR-0019.
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

## Consequences

- Positive: the rules are tested without mocks, on values built in the test, which keeps the
  parity fixtures cheap (Q-2); every effect the threats aim at sits in one module, small enough to
  read whole (Q-7 to Q-12); a rule's refusal and its notes are tested with the rule, on values,
  and the shell treats every rule alike, so a port adds no case to it (Q-1).
- Negative / trade-offs: the shell has to gather everything a rule might need before calling it,
  so a rule that would read lazily gets its input up front; every rule runs before a refusal is
  reported, where the shell's own refusals end the run before any; the notes go beyond the bash,
  which names the pending criteria only on a pass, on the standard error the harness does not
  compare; the structural test reads source text, so a forbidden path written in an unusual form
  can slip past it until the list grows.
- Follow-ups: how the shell spawns git with a cleared environment; whether the shell is split
  further once it grows.

## Enforcement

A structural test, cited with `Verifies: ADR-0025`, reads the sources of
the library outside `src/shell/` and fails on a use of `std::fs`, `std::process`, `std::env`,
`std::net`, the standard streams, or the printing macros, and on any crate the list beside the
test names as doing I/O. It fails as well on a rule module without exactly one rule ID, on a rule
ID used twice, on logic in `src/main.rs` beyond the call into the shell, and on a source of
`src/shell/` that calls what a rule decides: a rule's `check`, or a function of a rule module
that the list beside the test names as part of an outcome, as the pending criteria. The shell may
still name a rule module for the types of its inputs and for the reading of them, as the
Template release line a rule checks by. The lists are the project's, kept beside the test. A
fixture shows that a rule's refusal ends the run with exit 2 and no finding printed. That the
shell stays thin is review.
