# Ticket: Carry four wording defects of accepted ADRs into the next normalization

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/0006-feature-layer.md. -->

## What to build

The work since the revision of 2026-11-07 left four places where an accepted ADR's text no
longer says what the system does. Each was named in a pull request's Doubts as waiting for the
next normalization or found in the review of the take-up of v0.8.1, and nothing in the tree
keeps them until then. Accepted ADRs are not edited
([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules), rule 5), so this ticket only keeps the list;
the normalization restates them. It runs after the migration to template release v0.9.0, which
replaces ADR-0001 to ADR-0006 by the template's fixed base and gives every ADR an identifier of a
category and a slug; it restates the ADRs below under those identifiers.

- **ADR-0025, *Enforcement*:** it says the release workflow checks the attestation "after it
  publishes"; `release.yml` verifies it before the release is created as a draft, as under
  ADR-0017, whose wording ADR-0025 carried (#71).
- **ADR-0011, *Enforcement*:** it repeats the first half of Q-9 only; since #72 Q-9 also says
  that a hook handed another git directory does not decide, that a reference transaction that
  touches no `main` passes, and that the merge driver merges in the repository git runs it in,
  which `tests/git_cli.rs` checks.
- **Short lines left by `scripts/number-adrs.sh`:** where it replaced an `ADR-draft-<slug>` by
  the shorter number, it left the line short without filling the paragraph again: ADR-0020 and
  ADR-0026 in their *Enforcement*, ADR-0028 in its *Enforcement* (`run \`nuc` / `check\``).
- **ADR-0021 *Sources*, ADR-0022 *Context* and *Sources*:** they name the template's hooks and
  `scripts/test-git-hooks.sh` at v0.5.0 through v0.7.0; the corpus also pins v0.8.1, whose hooks
  are the same as v0.7.0's.

## Acceptance criteria

- [ ] Given a normalization the human has instructed, when it restates ADR-0011, ADR-0020,
  ADR-0021, ADR-0022, ADR-0025, ADR-0026, and ADR-0028, then each restatement says what the list
  above says the system does, with its paragraphs filled, and changes no decision.

## Read first

- Touches: `docs/adr/` (in the normalization only)
- The `normalize` skill; [`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules), rules 3 and 5.
- `.github/workflows/release.yml` (the publish job), Q-9 of `docs/SPECIFICATION.md`.

## Blocked by

- None among the tickets; the migration to template release v0.9.0 gates it, see Mode.

## Mode

with the human — the list is worked only inside a normalization, which runs on the human's
instruction alone ([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules), rule 5); until then the
ticket waits for the migration of the record to template release v0.9.0, section 8 of that
release's `normalize` skill. ADR-0007 allows no unattended ticket.
