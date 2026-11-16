# Ticket: `nuc measure` takes further branches and a time range, and shows where the tokens went

## What to build

On the tracer of ticket 02, this ticket adds the selection and the breakdown.

- **Selection:** options name further branches, one or several, and a time range. With a time
  range, only the requests inside it count. The harness's configuration directory variable is
  respected for the default directory.
- **Breakdown:** an option prints, per session and per subagent of the selection, its tokens read
  (input and cache read), written to the cache, and output. It prints per tool name the calls and
  the size of their outputs, and names the three costliest subagents and the three largest tool
  outputs, by size.
  - Subagents are named by type and description. Tool outputs are named by tool and size, never by
    their content or input.
  - The line `tokens <n>` stays the last line, so that it pastes as before.

## Acceptance criteria

- [ ] Given requests on branches `a`, `b`, and `c`, when `nuc measure` runs with `a` and `c` named,
  then the figure counts those of `a` and `c`.
- [ ] Given requests on either side of a time range, when it runs with that range, then only those
  inside count.
- [ ] Given the harness's configuration directory variable set to a fixture directory, when it runs
  without naming one, then it reads that directory.
- [ ] Given two sessions with subagents and tool calls, when it runs with the breakdown, then each
  session's and subagent's four figures sum to the line `tokens <n>`, and the costliest subagents
  and largest outputs are named.
- [ ] Given the canary fixtures, when it runs with the breakdown, then the canary is in no output.

## Read first

- Touches: the counting module and the Claude Code adapter of ticket 02, the shell's options, its
  test file, `tests/contract.rs`, `README.md`
- The ADR of ticket 01; the spec's decisions on selection and output

## Blocked by

- docs/plan/nuc-measure/02-tokens-of-the-current-branch.md

## Mode

with the human — it widens the parser of untrusted input and what the command prints
(DEV-one-agent-one-ticket).
