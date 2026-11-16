# Spec: `nuc measure` reads a harness's sessions and prints the tokens of the Measures line

## Problem

From template release v0.10.0, each hand-off states what the work cost in a fixed line,
`Measures: tokens <n> | checks <n> s | review rounds <n>`, and the design revision reads those lines
to find what cost much and gave little (NUC-design-is-revised, NUC-template-learns-from-projects).
The seconds of the checks come from `check-all.sh`, the rounds from the review. The tokens are known
only to the harness that ran the sessions, so every hand-off of this repository so far writes
`tokens -`. The design revision has no token figure to weigh, and a finding about the template
cannot name what a rule or a skill costs in tokens.

Reading the figure by hand is not practical. Claude Code keeps one transcript per session, and one
per subagent beside it. A transcript holds a usage record per request of the model, but one request
spans several lines, each reporting its usage as it streams. In this repository's sessions, 3749
lines carry 1858 requests, so a naive sum counts most requests more than once.

## Solution

A maintainer on a branch runs `nuc measure` before the hand-off. It reads the transcripts of the
harness for this repository and the current branch, subagents included, and prints the part of the
Measures line ready to paste: `tokens <n>`. On request it prints where those tokens went, by
session, by subagent, and by tool, and names the costliest subagents and the largest tool outputs.
It can also print a draft of a template finding in the fields of the template's issue form, which
carries only numbers and tool names, nothing of the project. It never sends anything: the human
decides what to post (NUC-template-learns-from-projects §4).

## Existing behaviour

Greenfield: `nuc` has no command that reads anything outside the repository, and no test pins one.

- What must stay:
  - `nuc check` and `nuc hook` read only inside the repository and its git directory, resolved
    through the one function that refuses a path outside it.
  - The command line keeps its contract: the existing commands, their options, and their exit codes
    (API-command-line-keyed-findings), which `tests/contract.rs` holds.
- What changes:
  - A new command joins the command line and the help text.
  - Its reading of the harness's directory is the one read outside the repository, decided by its
    own ADR.

## Goals served

The specification has no goal for this yet. Ticket 01 drafts one for the human:

- **G-8 (drafted)** — When a maintainer hands off a change, `nuc measure` prints the tokens the
  harness's sessions on that branch read and wrote, in the form of the Measures line.

It also bears on these criteria in force:

- **Q-1:** a transcript `nuc measure` cannot read is refused, with its file and line, never counted
  as zero.
- **Q-7:** `nuc measure` runs nothing it reads.
- **Q-8:** `nuc measure` writes no file.
- **Q-10** and **Q-11:** `nuc measure` prints no content of a transcript and uses no network.

## User stories

1. As a maintainer handing off a ticket, I want one command that prints `tokens <n>` for the
   current branch, so that the Measures line carries a figure instead of `-`.
2. As a maintainer, I want subagents counted with the session that started them, so that a review
   run as a subagent is part of the cost.
3. As a maintainer, I want each request counted once however many lines it spans, so that the
   figure is the harness's, not twice it.
4. As a maintainer of a `feat/` branch, I want to measure each ticket branch at its own hand-off and
   name further branches or a time range by option, so that the gathering pull request can sum the
   tickets' lines as its template asks.
5. As a maintainer, I want to see the tokens read, written to the cache, and output per session, per
   subagent, and per tool, with the costliest subagents and largest tool outputs named, so that I
   find what cost much.
6. As a maintainer, I want a transcript that does not read as `nuc measure` expects to stop the
   command with its file and line, so that a change of the harness's format does not report a low
   figure silently.
7. As a maintainer, I want nothing of a transcript's content printed, only numbers, identifiers,
   subagent types, and tool names, so that the output is safe to paste.
8. As a maintainer running a design revision, I want a draft of a template finding with the
   Measures field filled and nothing of the project in it, so that I can send it after reading it.
9. As a maintainer whose harness keeps its data elsewhere, I want to name the directory of the
   transcripts, so that `nuc measure` reads the right sessions.

## Implementation decisions

- **The figure.** `tokens <n>` is the sum, over every request counted once, of the input tokens,
  the tokens read from the cache, the tokens written to the cache, and the output tokens, as the
  harness reports each: what the sessions read and wrote, as the pull request template words it.
  The breakdown gives the four parts apart.
- **One request, once.** Lines that share a request's identifier report its usage as it streams, so
  the output grows from line to line; each request counts once, at the largest count each line
  reports.
- **Selection.**
  - By default, the requests made on the current branch, read from the branch each line records.
  - Options name further branches, one or several, and a time range. With a time range, only the
    requests inside it count.
  - A `feat/` branch's pull request sums the Measures lines of its tickets. `nuc measure` reads no
    git history to find them: a `feat/` branch integrates its tickets by rebase and fast-forward, so
    no ticket branch's name stays in git.
- **Subagents.** A subagent's transcript belongs to the session that started it and counts with it.
  The breakdown names each subagent by its type and description.
- **Where.**
  - By default, the transcripts of this repository's working tree in Claude Code's directory, under
    the configuration directory the harness's own variable names, or its default under the home
    directory.
  - An option names the directory instead. Tests always name it.
- **Adapters.** One adapter per harness turns its transcripts into requests with their usage, their
  branch, their session, their subagent, and their tool calls. Claude Code's is the first and only
  one in this spec. The counting, the selection, and the output are the harness's concern no
  further.
- **Fail closed.** A line that does not parse, or a request without the usage it should carry, stops
  the command with exit 2, its file, and its line. A field `nuc measure` does not read is ignored.
  No transcript for the selection is a message and exit 2, not `tokens 0`.
- **Output.**
  - Standard output carries the line `tokens <n>`.
  - The breakdown and the finding draft are separate options.
  - Nothing of a transcript's text is printed: no prompt, no tool input, no tool output, no path a
    tool read. A tool output is measured by its size.
- **The finding draft** fills the five fields of the template's issue form for template findings:
  Template release, Concerns, Finding, Measures, and Proposal.
  - Template release is read from the README.
  - Measures gets the figures.
  - Concerns, Finding, and Proposal stand as placeholders for the human to write.
  - It is printed, not written to a file, and carries no subagent description, branch, or path.
- **JSON.** The transcripts are read with a crate that parses JSON, under
  OPS-crates-pass-deny-and-audit: it widens what `nuc` reads.
- **Shape.** The counting, the selection, and the output are pure functions of the library. The
  shell lists and reads the transcripts and gives them over, as for every other command
  (ARCH-rules-outcome-hooks-answer).

## Testing decisions

- **The seam** is the command as a process: fixture transcripts written by the tests into a
  directory of their own, `nuc measure` run on them with that directory named, and its output and
  exit asserted. This is the test pattern of `tests/check.rs`, under a cleared environment with a
  `HOME` of the test's own (TEST-fixtures-with-own-home).
- **The fixtures** are synthetic lines in the shape Claude Code writes today. They include a request
  over several lines, a subagent, two branches, a line that does not parse, and a request without
  usage. No real transcript is copied into the tests.
- **A canary** string in every text field of the fixtures must appear in no output of any option.
- **Unit tests** beside the pure functions decide the counting and the selection. The contract test
  gains the command and its exit codes.

## Out of scope

- The summary across the projects on one machine, which issue #81 names; a later spec may take it
  up.
- Harnesses other than Claude Code, and its OpenTelemetry export.
- Reading git history to find the ticket branches of a `feat/` branch.
- Writing the Measures line into a pull request, or sending anything to any host.
- The seconds of the checks and the rounds of review in the Measures line.
- A JSON or SARIF format for `nuc measure`.
- Prices or money.

## ADR impact

- **Bind this work:**
  - API-command-line-with-measure, which supersedes API-command-line-keyed-findings: the command
    line, its help, and its exit codes;
  - ARCH-rules-outcome-hooks-answer: a pure library and one shell;
  - OPS-crates-pass-deny-and-audit: the JSON crate;
  - SEC-git-cleared-environment: if the current branch is asked of git;
  - TEST-fixtures-with-own-home;
  - DEV-one-agent-one-ticket;
  - NUC-template-learns-from-projects: the finding draft and that nothing is sent.
- **Proposed in ticket 01 and accepted by the maintainer on 2026-11-16,** flipped in ticket 02:
  API-measure-reads-harness-transcripts, and API-command-line-with-measure, which supersedes
  API-command-line-keyed-findings to give `nuc measure` its output and exit codes. The draft of G-8
  stands in the hand-off of ticket 01 for the maintainer to land.
