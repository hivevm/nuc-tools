# ADR-draft-harness: One agent implements and a fresh session reviews, one ticket at a time, and the maintainer reviews every change to the protected parts

- **Status:** 🟡 proposed
- **Date:** 2026-10-30
- **Deciders:** Markus Brigl
- **Applies to:** every change made by a coding agent, the `review` skill, the agents and models that work here, and the parts of the system listed under Decision as never worked unattended

## Context

The specification names one maintainer, with agents working the tickets and the maintainer
reviewing and deciding (*Constraints*). It also names assets worth protecting: the gates of
every derived project, their repositories, and the tokens in the environments `nuc` runs in
(*Threats & Forbidden Actions*), and puts the integrity of the gates first (Q-1). The port is
planned as a feature spec of tickets ([ADR-0006](0006-feature-layer.md)), each done only when
the parity harness shows no difference (Q-2).

The catalogue entry *Harness* offers three forms. The smallest, one agent with the human
reviewing every change, fits a one-person tool but leaves the protected parts with a single
reader. Parallel tickets under a cap fit work that outruns one session at a time; with one
maintainer, the review is the bottleneck, and parallel agents would produce a backlog of
reviews, not software. The middle form fits: an implementer and a reviewer apart, the parts the
threats protect read by the maintainer as well.

## Decision

We will have one agent implement, a fresh session of the
[`review`](../../.agents/skills/review/SKILL.md) skill review each change before its
hand-off, and the maintainer review every change to the protected parts, with one ticket of agent
work open at a time, each session started by the maintainer.

- **Implementer:** Claude Code with the model `claude-opus-5-5`.
- **Reviewer:** a fresh session of the same agent and model through the `review` skill, never
  the session that wrote the change
  ([`AGENTS.md` §5](../../AGENTS.md#5-quality-bar--definition-of-done)).
- **Unattended work:** none. The maintainer starts every session; there is no command that starts
  one without a human, and no ticket's *Mode* allows unattended work.
- **Cap:** one ticket of agent work open at a time; no parallel sessions, so no worktrees are
  needed to keep sessions apart.
- **Protected parts,** never worked unattended and reviewed by the maintainer as well however
  small the ticket: every parser of repository content (ADR files, Markdown, YAML, TOML); the
  code that spawns git and builds its environment; every path through which `nuc` writes a file,
  such as the merge driver and `nuc upgrade`; the fail-closed path of hooks and checks; and the
  release and CI workflows, `Cargo.lock`, and the configuration of `cargo deny`.
- **Decisions:** the maintainer alone decides the ADRs and lands changes to the specification,
  as the code owner of both in [`.github/CODEOWNERS`](../../.github/CODEOWNERS).

**Out of scope:** the agent's permission mode and tool configuration, which no file of this
repository enforces ([`AGENTS.md` §6](../../AGENTS.md#6-project-rules)); a second maintainer and
who would decide which part; parallel tickets.

## Alternatives considered

- **One agent, the human reviews every change** — the smallest form; it leaves the review of
  every change to the one maintainer and gives the protected parts no second reader of another
  context.
- **Parallel tickets under a cap** — several agents at once in worktrees of their own; with one
  maintainer the review is the bottleneck, and parallel work would outrun it.
- **A reviewer of a different agent or model** — the catalogue suggests one for the parts the
  threats protect; not decided here, see *Follow-ups*.

## Sources / Prior art

- The catalogue entry [*Harness*](../../.agents/skills/propose-adr/concepts/harness.md).
- Birgitta Böckeler, *Harness engineering* —
  <https://martinfowler.com/articles/harness-engineering.html>.
- Addy Osmani, *Agentic autonomy levels* — <https://addyosmani.com/blog/agentic-autonomy-levels/>.

## Consequences

- Positive: every change has a second reader in a fresh context before the maintainer sees it;
  the protected parts, which carry Q-1 and Q-7 to Q-10, are always read by the maintainer.
- Negative / trade-offs: throughput is one ticket at a time, bounded by the maintainer's review;
  the reviewer shares the implementer's agent and model, and with them its blind spots.
- Follow-ups: whether the reviewer of the protected parts should be an agent or model of a
  different kind; the parallel form, with its start command, cap, and worktrees, once the review
  keeps up and the frontier holds several independent tickets.

## Enforcement

**Not mechanically decidable:** which agent and model work here, who reviews a change, and
whether a session was started by a human are facts outside the tree; no test of this repository
can observe them. What the harness relies on mechanically is checked elsewhere: the git hooks,
the required checks of the ruleset on `main`, and the review the pull request template asks for.
