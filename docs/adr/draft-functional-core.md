# ADR-draft-functional-core: One library crate with the rules as a functional core, its effects in one shell module, and a thin binary

- **Status:** 🟡 proposed
- **Date:** 2026-10-30
- **Deciders:** Markus Brigl
- **Applies to:** the source tree of the crate, `src/main.rs`, `src/shell/`, every rule module, and the structural test that holds them

## Context

`nuc` reads a repository and decides: its checks, hooks, merge driver, and migrations are
computations over files and git data (G-1 to G-4). Its effects are few: spawning git, reading and
writing files, reading the environment, printing findings. Those effects are exactly what the
threats aim at: acting on another repository through an inherited environment (Q-9), writing
outside the repository (Q-8), disclosing a token (Q-10), reaching the network (Q-11), executing
repository content (Q-7). The harness keeps the code that does them with the maintainer
([ADR-draft-harness](draft-harness.md)). Parity (Q-2) asks for many fixtures, which are cheapest
when a rule is a function of values.

The catalogue entry *Functional core, imperative shell* fits when the logic is rich and the
effects are few and simple, a CLI or a rules engine among its cases, and when a quality goal asks
for exhaustive tests of the logic. It does not fit logic that is mostly effects in sequence, which
`nuc` is not, or several technologies behind one behaviour, which the constraints exclude: git is
reached only through its CLI.

## Decision

We will build `nuc` as one library crate whose rules are pure functions over values, with every
effect in one shell module, `src/shell/`, and a binary, `src/main.rs`, that only calls into it.

- **Rules:** each rule is a module of the library with one stable rule ID, unique across the
  crate (G-6). It takes values, such as file contents, paths, and git data the shell gathered,
  and returns findings.
- **Shell:** `src/shell/` is the only place that spawns processes, touches the file system, reads
  the environment, prints, or reaches the network. It gathers the values the rules need, calls
  them, and carries out what they return.
- **Binary:** `src/main.rs` holds no logic; it calls the shell's entry point and exits with the
  code it returns.

**Out of scope:** how git is spawned and its environment built, a decision of its own; the
module layout inside the core beyond one module per rule; the output formats.

## Alternatives considered

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

## Consequences

- Positive: the rules are tested without mocks, on values built in the test, which keeps the
  parity fixtures cheap (Q-2); every effect the threats aim at sits in one module, small enough to
  read whole (Q-7 to Q-11).
- Negative / trade-offs: the shell has to gather everything a rule might need before calling it,
  so a rule that would read lazily gets its input up front; the structural test reads source
  text, so a forbidden path written in an unusual form can slip past it until the list grows.
- Follow-ups: how the shell spawns git with a cleared environment; whether the shell is split
  further once it grows.

## Enforcement

A structural test, cited with `Verifies: ADR-draft-functional-core`, reads the sources of the
library outside `src/shell/` and fails on a use of `std::fs`, `std::process`, `std::env`,
`std::net`, the standard streams, or the printing macros, and on any crate the list beside the
test names as doing I/O. It fails as well on a rule module without exactly one rule ID, on a rule
ID used twice, and on logic in `src/main.rs` beyond the call into the shell. The list of forbidden
paths is the project's, kept beside the test. That the shell stays thin is review.
