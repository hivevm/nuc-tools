# Ticket: `nuc measure` prints `tokens <n>` for the current branch of Claude Code

## What to build

The tracer bullet. On a branch, `nuc measure` reads the Claude Code transcripts of this
repository's working tree and their subagents, and prints `tokens <n>`. The figure is the input,
cache read, cache written, and output tokens of every request made on that branch, each request
counted once.

- The default directory is the harness's, and an option names another.
- A line that does not parse, or a request without usage, stops it with exit 2 and its file and
  line. No transcript for the branch stops it too.
- It prints nothing of a transcript's text.
- The JSON crate joins `Cargo.toml` and passes `cargo deny` and `cargo audit`.
- The README's command reference and `docs/ARCHITECTURE.md` name the command.
- The maintainer accepted API-measure-reads-harness-transcripts and API-command-line-with-measure
  on 2026-11-16. This ticket flips both to accepted and API-command-line-keyed-findings to
  superseded, in their header lines and the index, and moves the `Verifies:` markers of
  API-command-line-keyed-findings to API-command-line-with-measure.

## Acceptance criteria

- [ ] Given fixture transcripts with one request over three lines on branch `b`, when `nuc measure`
  runs on `b` with their directory named, then it prints `tokens <n>`, where `n` counts that request
  once, and exits 0.
- [ ] Given a session on `b` whose subagent made requests, when it runs, then the subagent's tokens
  are in the figure.
- [ ] Given requests on branches `a` and `b`, when it runs on `b`, then only those of `b` count.
- [ ] Given a line that does not parse, or an assistant line without usage, when it runs, then it
  exits 2 and names the file and line.
- [ ] Given no transcript of the branch, when it runs, then it exits 2 with a message, and prints no
  `tokens 0`.
- [ ] Given a canary string in every text field of the fixtures, when it runs, then the canary is in
  no output.
- [ ] Given `tests/contract.rs`, when it runs, then it holds the command and its exit codes.

## Read first

- Touches: `src/` (a pure module for the counting, an adapter for Claude Code, the shell's reading
  of the directory), `src/main.rs` help, `Cargo.toml`, `Cargo.lock`, `deny.toml`, `tests/` (a new
  test file with its fixtures), `tests/contract.rs`, `README.md`, `docs/ARCHITECTURE.md`
- The ADR of ticket 01; ARCH-rules-outcome-hooks-answer, OPS-crates-pass-deny-and-audit,
  TEST-fixtures-with-own-home
- The test pattern in `docs/ARCHITECTURE.md` (`tests/check.rs`), and `tests/functional_core.*` for
  the reads of the shell

## Blocked by

- None (can start immediately)

## Mode

with the human — a parser of untrusted input on the fail-closed path, and `Cargo.lock` with
`deny.toml` (DEV-one-agent-one-ticket).
