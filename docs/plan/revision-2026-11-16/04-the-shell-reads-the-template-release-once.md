# Ticket: The shell reads the README's Template release in one function

## What to build

`shell/hook.rs` (`release`, for the hooks' release) and `shell/measure.rs` (`release`, for the
draft of a template finding) each read `README.md` through the tree and parse its Template
release line with `rules::docs_template_release::carried`. One returns a version, the other its
text. The evidence is duplication only, not a defect: the human weighs whether it is worth a
session.

Put one function in the shell that returns what both need, and have both call it.

## Acceptance criteria

- [ ] Given the hooks and `nuc measure --finding`, when they read a README with and without a
  release line, then they decide and print as today, through one function.

## Read first

- Touches: `src/shell/hook.rs`, `src/shell/measure.rs`, `src/shell/mod.rs`
- ARCH-rules-outcome-hooks-answer; `tests/functional_core.guarded`

## Blocked by

- None (can start immediately)

## Mode

with the human: it touches the hooks' reading of the release, on their fail-closed path.
