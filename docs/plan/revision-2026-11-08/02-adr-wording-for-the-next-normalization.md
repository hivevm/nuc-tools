# Ticket: Carry six wording defects of accepted ADRs into the next normalization

<!-- Copy to docs/plan/<spec>/<NN>-<slug>.md, next to the spec it slices, if it has one. A
     vertical slice: a narrow but complete path through every layer the change touches, demoable
     or verifiable on its own, sized to one fresh agent session. Claim it by creating the branch
     ticket/<spec>/<NN>-<slug> BEFORE any work, so parallel sessions sharing the repository skip
     it. The branch that lands it deletes this file. Decided in
     docs/adr/NUC-large-work-is-planned.md. -->

## What to build

The work since the revision of 2026-11-07 left four places, and the take-ups of v0.9.1 and v0.10.0
two more, where an accepted ADR's text no longer says what the system does. Each was named in a pull
request's Doubts or its Documents and decisions as waiting for the next normalization or found in
the review of the take-up of v0.8.1, and nothing in the tree keeps them until then. Accepted ADRs
are not edited ([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules), rule 5), so this ticket only keeps
the list; the normalization restates them. It runs after the migration to template release v0.9.0,
which replaces ADR-0001 to ADR-0006 by the template's fixed base and gives every ADR an identifier
of a category and a slug; it restates the ADRs below under those identifiers.

- **OPS-feature-in-hivevm-features, *Enforcement*:** it says the release workflow checks the
  attestation "after it publishes"; `release.yml` verifies it before the release is created as a
  draft, as under OPS-ships-binaries-feature-action, whose wording OPS-feature-in-hivevm-features
  carried (#71).
- **SEC-git-cleared-environment, *Enforcement*:** it repeats the first half of Q-9 only; since #72
  Q-9 also says that a hook handed another git directory does not decide, that a reference
  transaction that touches no `main` passes, and that the merge driver merges in the repository git
  runs it in, which `tests/git_cli.rs` checks.
- **Short lines left by `scripts/number-adrs.sh`:** where it replaced an `ADR-draft-<slug>` by the
  shorter number, it left the line short without filling the paragraph again:
  API-command-line-with-hook-signal and TEST-fixtures-with-own-home in their *Enforcement*,
  API-output-formats-short-description in its *Enforcement* (`run \`nuc` / `check\``).
- **API-hook-contract *Sources*, TEST-hook-scenario-parity *Context* and *Sources*:** they name the
  template's hooks and `scripts/test-git-hooks.sh` at v0.5.0 through v0.7.0; the corpus also pins
  v0.8.1, whose hooks are the same as v0.7.0's.

- **Check numbers of `scripts/check-docs.sh` in the present tense:** since template release
  v0.10.0 its checks carry rule keys and no numbers. OPS-rust-stable-behind-scripts (lines 32,
  129, 135, 142) and TEST-parity-harness (line 117) name check 9, now `docs/check-lists`;
  TEST-fixtures-with-own-home (line 107) and TEST-fixtures-through-real-git (line 62) name
  check 10, now `docs/project-layout`; TEST-parity-harness names checks 12 and 13 (line 26), now
  `docs/base-blobs` and `docs/template-identity`, and check 16 (line 58), which v0.9.1 removed.
  The maintainer kept the text at the take-up (pull request #84).
- **SEC-gates-never-run-build, *Decision*:** it says the switch to a released `nuc` "supersedes the
  bash clause of NUC-checks-are-bash §1". Since v0.9.0 a project ADR narrows a `NUC-` clause and
  never supersedes it (NUC-base-is-fixed §1, NUC-checks-are-bash §8). The maintainer kept the text
  at the migration (pull request #84, Doubts).

## Acceptance criteria

- [ ] Given a normalization the human has instructed, when it restates SEC-git-cleared-environment,
      API-command-line-with-hook-signal, API-hook-contract, TEST-hook-scenario-parity,
      OPS-feature-in-hivevm-features, TEST-fixtures-with-own-home,
      API-output-formats-short-description, OPS-rust-stable-behind-scripts, TEST-parity-harness,
      TEST-fixtures-through-real-git, and SEC-gates-never-run-build, then each restatement says
      what the list above says the system does, with its paragraphs filled, and changes no
      decision.

## Read first

- Touches: `docs/adr/` (in the normalization only)
- The `normalize` skill; [`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules), rules 3 and 5.
- `.github/workflows/release.yml` (the publish job), Q-9 of `docs/SPECIFICATION.md`.

## Blocked by

- docs/plan/template-v0-11-0/02-take-up-v0-9-1.md

## Mode

with the human — the list is worked only inside a normalization, which runs on the human's
instruction alone ([`AGENTS.md` §3](../../../AGENTS.md#3-adr-rules), rule 5); until then the
ticket waits for the migration of the record to template release v0.9.0, section 8 of that
release's `normalize` skill. DEV-one-agent-one-ticket allows no unattended ticket.
