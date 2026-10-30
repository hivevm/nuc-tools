# Specification — nuc-tools

> The **specification** of this project: the problem it solves, where it is going, and what has to
> be true for it to have succeeded, written in the vocabulary of [`GLOSSARY.md`](GLOSSARY.md). It
> is the **constitution**: every Architecture Decision Record in [`adr/`](adr/) derives from it and
> must not contradict it ([ADR-0001](adr/0001-agent-governance-model.md)).

## Problem

Every project derived from the NUC template carries its own copy of the template's mechanics:
about 8,000 lines of bash in checks, sensors, git hooks, the ADR index merge driver, and the
scripts run by hand. Their maintainers and the agents working in them live with four costs
today:

- **Fixes do not propagate.** A defect fixed in one copy stays in every other: the leak of
  `GIT_DIR` from a git hook into the self-tests is fixed in one derived repository and still
  present in the others.
- **Taking up a template release is a manual checklist.** For v0.5.0 it was ten steps, worked
  by hand in each project.
- **The checks are not portable.** They need bash 4.4 or later and GNU tools, which macOS and
  Windows lack outside the Dev Container.
- **Every project pays for the template's own tests.** The template's self-tests run in each
  project's `check-all` and CI, although they test the template, not the project.

The cost grows with every derived project and every template release, and the template's
ADRs keep moving logic into these scripts, so it is worth solving now, before the copies
diverge further.

## Mission

The core purpose of this project, in one sentence.

## Vision

The desired future state, in a paragraph or two.

## Strategy

The high-level approach to get there.

## Goals / Success Criteria

Acceptance criteria a test can decide, where it helps in the form *When <trigger>, the system
shall <response>*. Each starts with a bold `G-n`, which a test cites with `Verifies: G-n`. Until
that test exists the criterion is pending work, and the traceability check lists it
([ADR-0002](adr/0002-decisions-verified-by-tests.md)). Identifiers are never reused.
Only a normalization renumbers them ([`AGENTS.md` §3](../AGENTS.md#3-adr-rules), rule 5).

1. **G-1** — …
2. **G-2** — …

## Quality Goals

The few properties that drive decisions, in priority order. Every ADR weighs its *Consequences*
against them. Each has a measure concrete enough to settle a disagreement and starts with a bold
`Q-n`, cited like a goal. Security goals, what the system must never do, belong here, derived
from the threats below.

- **Q-1** — <Quality, e.g. Performance>: why it matters, and how it is measured (e.g. "p95 response
  under 200 ms at 100 requests per second").
- **Q-2** — <Quality, e.g. Security, Availability, Portability, Accessibility>: …

## Non-Goals

What this project explicitly does not try to do.

- …

## Constraints

What the solution has to live with regardless of the better choice: platforms, standards,
systems to interoperate with, limits of time, budget, and team. A constraint not written here is
rediscovered as a rejected alternative in an ADR.

- …

## Threats & Forbidden Actions

The assets, the actors who threaten them, and the trust boundaries between the parts. Then what
the system must never do, regardless of what a task asks. A prohibition not written here does not
exist for an agent. One a test can decide is restated as a `Q-n`, so that it is verified.

- **Assets:** …
- **Actors and trust boundaries:** …
- The system shall never …

## Assumptions

What is believed true today and would invalidate the decisions above if it turned out false:
expected scale, the shape of the data, who operates the result, which dependency stays
maintained.

- …
