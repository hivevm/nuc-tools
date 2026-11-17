# API-measure-reads-harness-transcripts: `nuc measure` reads a harness's transcripts from one directory outside the repository, read-only, through a pure adapter per harness, and prints numbers and names only

- **Status:** 🟢 accepted
- **Date:** 2026-11-16
- **Deciders:** Markus Brigl
- **Applies to:** the `nuc measure` command and its source, the adapter of each harness it reads, the one function of `src/shell/` that reads outside the repository, the structural test that holds it, and the tests of `nuc measure`

## Context

From template release v0.10.0 each hand-off states what the work cost in the line
`Measures: tokens <n> | checks <n> s | review rounds <n>`. The design revision reads it, and so
does a template finding (NUC-design-is-revised, NUC-template-learns-from-projects). Only the
harness knows the tokens, so every hand-off of this repository writes `tokens -`. Issue #81 asks
`nuc` to read them.

The forces:

- **The data lies outside the repository.** Claude Code keeps one transcript per session as JSON
  Lines in its configuration directory, under a directory named for the working tree. It keeps one
  transcript per subagent beside it. Every other command of `nuc` reads only inside the repository
  and its git directory, each path resolved through one function that refuses a path outside.
- **The format is the harness's.** It is undocumented and changes with the harness's releases. One
  request of the model spans several lines, each repeating its usage. In this repository's
  sessions, 3749 lines carry 1858 requests, so a naive sum counts about twice.
- **The transcripts are private.** They hold prompts, code, tool inputs, and tool outputs. A figure
  is pasted into a pull request, and a finding goes to another repository
  (NUC-template-learns-from-projects §2).
- **The criteria in force.**
  - Q-1 asks a run that cannot decide to fail closed, not to pass.
  - Q-7 forbids running what `nuc` reads, and Q-8 writing outside the repository.
  - Q-10 forbids disclosing a token, and Q-11 the network in checks and hooks.
  - ARCH-rules-outcome-hooks-answer keeps the decisions pure and the effects in one shell.
  - OPS-crates-pass-deny-and-audit allows a crate that widens what `nuc` reads.
- **One harness today.** The template names no harness. A project on another harness gets nothing
  from a reader of Claude Code alone, but a second adapter has no user yet.

The maintainer gave the decisions of the spec `nuc-measure` on 2026-11-16, and those of this ADR in
the same session.

## Decision

We will have `nuc measure` read a harness's transcripts from one directory outside the repository,
read-only, through a pure adapter per harness. It fails closed on what it cannot read, and prints
numbers and names only.

- **The directory.** By default it is Claude Code's directory for this working tree. That is under
  the configuration directory the harness's own variable names, or under its default in the home
  directory. An option names another directory.
- **What is read.** Only regular files are read: the transcripts in that directory and the
  transcripts of the subagents under each session's directory. A symbolic link, or a file that is
  not regular, stops the command. Nothing outside that directory is read, and nothing is written
  anywhere. One function of the shell reads outside the repository, and only for `nuc measure`.
- **The adapter.** For each harness, a pure function of the library turns the bytes of its
  transcripts into requests. A request carries its identifier, its usage, its branch, and its time.
  It also carries its session, its subagent, and its tool calls with the sizes of their outputs.
  - The harness is chosen by an option. Claude Code is the default and the only adapter for now.
  - The counting, the selection, and the output are pure functions of those requests, and know no
    harness.
  - The transcripts are parsed with a JSON crate that passes `cargo deny` and `cargo audit`.
- **The figure.** `tokens <n>` is the sum of the input, cache read, cache written, and output tokens
  of every selected request. A request counts once, however many lines repeat its usage.
- **The selection.** The requests made on the current branch count. Options add branches and a time
  range. No git history is read to find the branches of another.
- **Fail closed.**
  - A line that does not parse, or a request without the usage the format gives it, stops the
    command with exit 2. The message names the file and the line.
  - No transcript for the selection stops it too. It never prints `tokens 0` for data it did not
    find.
  - A field the adapter does not read is ignored.
- **Numbers and names only.** The output carries token counts, sizes, session identifiers, subagent
  types and descriptions, tool names, and the release of the README. It carries no prompt, tool
  input, tool output, path a tool named, or other text of a transcript. A draft of a template
  finding carries no subagent description, branch, session identifier, or path either.
- **Nothing sent.** `nuc measure` uses no network and sends nothing. The human posts the figure and
  any finding (NUC-template-learns-from-projects §4).

**Out of scope:** the form of the command's output and its exit codes, which
API-command-line-with-measure decides. The summary across the projects of a machine, a harness
other than Claude Code, and prices.

## Latitude

None.

## Alternatives considered

- **A script of this repository beside `sensor-work-measures.sh`.** No change to `nuc` or its
  specification. But it would run only here, and a derived project, whose hand-offs ask for the
  same line, gets nothing.
- **A second binary.** It isolates the read outside the repository from `nuc check` and `nuc hook`.
  But it doubles the release tooling, the attestations, and the Action, for one command.
- **A trait per harness that also finds its files.** It is open to the next harness, but with one
  adapter it is indirection without a second caller. The catalogue's entry on ports and adapters
  says it does not fit a small CLI.
- **Claude Code read directly, with no adapter.** It is the simplest. But the counting would learn
  the harness's field names, and a second harness would be a rewrite.
- **Following symbolic links.** It is kinder to a home directory that links the harness's directory
  elsewhere. But the read would leave the one directory where a link points out of it, and the
  option names the real directory instead.
- **Skipping a line that does not parse.** A figure would still come out when the harness changes
  its format. But it would come out low and silently, against Q-1, and a hand-off cannot tell.
- **A JSON reader of `nuc`'s own.** No crate is added. But a parser of foreign input, written once
  for one command, is more to keep and to harden than a crate that `cargo deny` and `cargo audit`
  hold.
- **Finding a `feat/` branch's tickets in git.** A `feat/` branch integrates its tickets by rebase
  and fast-forward, so no ticket branch's name stays in git. Each ticket is measured at its own
  hand-off, and the `feat/` pull request sums the lines, as its template asks.

## Sources / Prior art

- The transcripts of Claude Code in this repository's sessions, read on 2026-11-16. Each line is a
  JSON object; an assistant line carries `requestId`, `gitBranch`, `timestamp`, `sessionId`, and
  `message.usage` with `input_tokens`, `cache_read_input_tokens`, `cache_creation_input_tokens`,
  and `output_tokens`. Subagents are in `<session>/subagents/agent-*.jsonl` with a `.meta.json`.
- Anthropic, *Messages API*, the usage object of a response, whose four token counts the
  transcripts copy.
- JSON Lines — <https://jsonlines.org/>.
- `.github/PULL_REQUEST_TEMPLATE.md`: the Measures line and what its tokens count.

## Consequences

- Positive:
  - A hand-off states its tokens, and the design revision can weigh them.
  - A template finding can name its cost without carrying the project.
  - The counting is tested without a harness, on requests the tests build.
- Negative / trade-offs:
  - `nuc` reads outside the repository for the first time. The structural test holds that read to
    one function and one command.
  - The format is the harness's and will change. Each change stops `nuc measure` until an adapter
    follows it.
  - A JSON crate joins the supply chain.
  - A `feat/` pull request still sums its tickets' lines by hand.
- Follow-ups:
  - A summary across the projects of a machine.
  - An adapter for another harness, and Claude Code's OpenTelemetry export.

## Enforcement

The tests of `nuc measure` cite this ADR with `Verifies:` and run the command on fixture
transcripts in a directory of the test's own:

- a request over several lines counts once;
- a subagent counts with its session;
- the branch and the time range select;
- a line that does not parse, a request without usage, a symbolic link, and no transcript each
  exit 2 with their cause;
- a canary string in every text field of the fixtures appears in no output.

The structural test `tests/functional_core.rs` fails where a function of `src/shell/` other than
the one named for `nuc measure` reads a path outside the repository.
