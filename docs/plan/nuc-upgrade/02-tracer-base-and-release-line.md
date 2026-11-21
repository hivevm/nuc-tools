# Ticket: `nuc upgrade <tag>` replaces the base and moves the release line

## What to build

The tracer bullet. On a clean `feat/` branch whose README names release A, with A and B fetched
into `refs/template/`, `nuc upgrade B` replaces the `NUC-` ADRs, `AGENTS.md`, and the blob list
with B's, deletes each `NUC-` ADR B lacks, moves the `Template release` line to B, and exits 0.
Every refusal of the spec is in place and leaves the tree untouched. No other path is touched yet.

The decision is a pure function from the trees of A, B, and `HEAD` to a plan. The shell reads the
trees through git under the cleared environment, checks the refusals, carries out the plan, and
prints the summary. The command is listed with its usage.

## Acceptance criteria

- [ ] Given a project on A whose base matches A, when `nuc upgrade B` runs, then the base matches
      B byte for byte with B's modes, a `NUC-` file only A has is gone, the README's release line
      names B and no other line of it changed, and the exit is 0.
- [ ] Given each refusal of the spec (no tag, `main`, a change in the tree or the index, an
      `unreleased`, forked, or unreadable release line, A before v0.9.0, B not newer than A, a
      missing ref or one that is no tag of that name, B's `nuc.toml` above the running version),
      when `nuc upgrade B` runs, then it exits 2 with its message and key on the standard error,
      and `git status --porcelain` is as before.
- [ ] Given a template tree whose base carries a `.gitattributes` filter and a `.git` path, when
      `nuc upgrade` runs, then no filter command ran (the canary of Q-7) and nothing outside the
      working tree was written.
- [ ] Given the plan function, when it is called on three trees as values, then its tests need no
      git and no file system, and the structural test of the functional core passes.
- [ ] A test cites `Verifies: G-4` and the ADR of ticket 01.

## Read first

- Touches: `src/upgrade.rs` (new), `src/shell/upgrade.rs` (new), `src/shell/mod.rs`,
  `src/shell/git.rs`, `src/shell/tree.rs`, `tests/contract.rs`, `tests/isolation.rs`,
  `tests/functional_core.*`
- The ADR of ticket 01; the spec, Implementation decisions.
- Glossary: migration, release commit; `docs/CONVENTIONS.md`.
- Golden path: `nuc sense` in `src/sense.rs` and `src/shell/sense.rs`: the release line read by
  `template_release`, the pure core called by the shell, `neutralized_filters` for a clean tree.
- Seam: the binary, as `tests/contract.rs` drives it, on fixtures built as
  TEST-fixtures-with-own-home says.

## Blocked by

- docs/plan/nuc-upgrade/01-upgrade-adr.md

## Mode

with the human: it adds a shell module that writes into the working tree, an effect
DEV-one-agent-one-ticket keeps with the maintainer.
