# API-command-line-with-sense: `nuc` reads `nuc.toml` as a TOML subset, writes one finding per line as `<file>[:<line>]: <rule ID>: <message> [<rule ID>:<reason>]`, `nuc measure` its measures ending in `tokens <n>`, `nuc sense` a sensor's report, and exits 0, 1, or 2, and `nuc hook` also 3 or 4

- **Status:** 🟡 proposed
- **Date:** 2026-11-19
- **Deciders:** Markus Brigl
- **Applies to:** the keys and the syntax of `nuc.toml` and how `src/config.rs` reads them, the text output of every `nuc` command, `nuc measure`'s and `nuc sense`'s among them, the reason every finding carries and the list of keys kept beside its test, and the exit codes
- **Supersedes:** [API-command-line-with-measure](API-command-line-with-measure.md)

## Context

`nuc` runs in the hooks and the CI of every derived project. Three things it shares with them
outlive any release. The first is the `nuc.toml` the template release ships
([API-version-coupling-hook-defaults](API-version-coupling-hook-defaults.md), which leaves its
format out of scope). The second is the line a finding is printed as, which editors, scripts, and
the parity harness read ([TEST-parity-harness](TEST-parity-harness.md)). The third is the exit codes
the hooks and CI branch on. G-6 asks a finding to name a stable rule ID and `file:line`. G-5 and Q-1
ask every run that could not decide to fail closed with a message naming how to install `nuc`. Q-6
asks that an update of `nuc` never turn a green project red. Q-3 counts every dependency against the
supply chain; `nuc` adds a crate only as
[OPS-crates-pass-deny-and-audit](OPS-crates-pass-deny-and-audit.md) allows, and reads `nuc.toml`
without one. A newer template release may add a key that an older
`nuc` does not know; that `nuc` then says it is too old and does not guess. The JSON and SARIF of
G-7 stand beside the text ([API-output-formats-carry-reason](API-output-formats-carry-reason.md)). A
hook whose bash runs the project's checks tells its shim that they run now. The standard output is
taken by findings, and a hook's refusal guards a ref, not a file
([API-hook-contract-keyed-shims](API-hook-contract-keyed-shims.md)).

From template release v0.9.1 on, every finding of the bash checks ends in a key,
`[<rule>:<reason>]`: the rule and one word for the cause, such as `[docs/adr-index:unlisted]`. The
template's fixture tables expect that key and the data a line names, never its wording, so a test
or a reader tells two causes of one rule apart without parsing a message that may change in any
release. A rule of `nuc` reports several causes under one rule ID, and its findings told apart
only by their wording. `nuc` ports the releases that print the key.

From template release v0.10.0 each hand-off states its tokens in a Measures line, which only the
harness knows. `nuc measure` reads them from the harness's transcripts
([API-measure-reads-harness-transcripts](API-measure-reads-harness-transcripts.md)). It reports no
finding: what it writes is a figure to paste, so the standard output, kept for findings alone, has
no room for it.

[API-git-only-sensors-under-sense](API-git-only-sensors-under-sense.md) ports the template's sensors
that need only git as `nuc sense <name>`. A sensor reports no finding either: it prints a report in
prose, which a session reads and the template's check run shows by its first line. The standard
output, kept for findings and measures, has no room for it.

## Decision

We will hold every command of `nuc` to one contract for `nuc.toml`, the standard output, and the
exit codes, as the clauses below decide.

- **`nuc.toml`:** comments, blank lines, and two keys: `minimum`, a string `"X.Y.Z"`, and
  `rules`, an array of strings that may span lines and end in a comma. It is read without a TOML
  crate. `minimum` is read first: above the running version, `nuc` refuses as too old before any
  other key is judged. A key it does not know, a key set twice, or any other syntax refuses the
  run as unreadable; neither is ignored.
- **Output:** each finding is one line `<file>[:<line>]: <rule ID>: <message> [<rule ID>:<reason>]`
  on the standard output, and nothing else stands there.
  - The location of a finding of `nuc hook` is the ref the hook guards, such as `refs/heads/main`,
    where it guards no file.
  - The summary, every refusal, and its message stand on the standard error. The message of a
    refusal a newer `nuc` resolves names how to install or update it.
  - The line is omitted where the rule names none, as for a missing file.
- **`nuc measure`:** it writes its measures on the standard output and nothing else there. The last
  line is `tokens <n>`, the part of the Measures line to paste. A breakdown or a draft it is asked
  for stands before that line. Its refusals go to the standard error, as every refusal does.
- **`nuc sense`:** it writes the sensor's report on the standard output and nothing else there,
  byte for byte as [API-git-only-sensors-under-sense](API-git-only-sensors-under-sense.md) decides.
  Its refusals go to the standard error. Each refusal of its own ends in a key
  `[sense/<name>:<reason>]`, and a name it does not know ends in `[sense/unknown:name]`.
- **The key:** every finding, of `nuc check` and of `nuc hook` alike, carries a reason, one word
  of lower-case ASCII letters, and its line ends in a space and `[<rule ID>:<reason>]`, the key.
  - Where the bash of template release v0.11.0 names a key for the same finding, the reason is
    that key's; elsewhere `nuc` names its own, as for a rule that release no longer has, a cause
    the bash keys under another rule, and every `hook/` rule.
  - The key is shown on a tree of every release, those whose bash prints none included.
  - A key is as stable as a rule ID (G-6). A reason is added with the cause it names and is never
    renamed, dropped, or given to another cause; a release that would do so supersedes this ADR.
- **Exit codes:** 0 without a finding, 1 with findings, and 2 when it cannot decide, for every
  command; every cause that keeps `nuc` from
  deciding exits 2, one code for all of them. A panic exits 101, non-zero as well. `nuc hook`
  exits 3 and 4 where
  [API-hook-contract-keyed-shims](API-hook-contract-keyed-shims.md) says, and no other command
  does; its `--record` exits 0 or 2. `nuc measure` exits 0 where it printed its measures and 2
  where it cannot, never 1, since it reports no finding. `nuc sense` exits 0 where it printed the
  report, 1 where a `--require` fails, and 2 where it cannot decide.

**Out of scope:** the JSON and SARIF formats of G-7; options a rule may take; the wording of the
messages, which may change in any release; the output of `nuc explain`, which is prose and takes a
rule ID only, not a key.

## Alternatives considered

- **No clause for `nuc sense`** — what the contract decided before it, accepted on 2026-11-16 with
  every other clause of this one. Its standard output held findings and measures alone, so a
  sensor's report had nowhere to write it, and this ADR replaced that clause.
- **The exit codes of each bash sensor** — strict parity, but some sensors exit 1 where they cannot
  read their input. `nuc sense` would then answer 1 for two causes, and a gate could not tell a
  failed `--require` from an unreadable tree.
- **No clause for `nuc measure`** — what the contract decided before it, accepted on 2026-11-15
  with every other clause of this one. Its standard output held findings alone, so a
  command that reports a figure had nowhere to write it, and this ADR replaced that clause.
- **The figure on the standard error** — the standard output would stay the findings' alone, but a
  figure meant to be pasted or redirected belongs where a shell pipes it.
- **The line without a key** — what the contract decided before the key, accepted on 2026-11-06
  with every other clause of this one. Once the bash keys every finding, `nuc` cannot be
  held to the template's fixture tables by key, and a reader who switches from the bash loses the
  word that names the cause, so this ADR replaced that clause.
- **The reason alone at the end, `[<reason>]`** — no rule ID twice on a line, but its end differs
  from the bash's, so a reader or a test that looks for a line ending in `[<key>]` needs two forms.
- **The key in place of the rule ID, `<file>[:<line>]: <rule ID>:<reason>: <message>`** —
  compact, but the field changes its meaning, and every reader of the old form breaks.
- **The reason free to change, like the wording** — less to keep, but a reason is what a reader
  and a test branch on in place of the wording, and it would break with a release as the wording
  may.
- **`nuc explain` on a key** — copied from the output as it stands, but the reasons have no
  explanation of their own to give, and the rule's reason is what `nuc explain` gives.
- **The codes 0, 1, and 2 for every command, `nuc hook` included, and a file as every location**
  — the first contract of the command line, accepted on 2026-10-31. It left a hook no way to tell
  its shim that the checks run now, and gave a refusal that guards a ref no location. The hook
  contract needed both, so the contract that followed replaced it and kept every other clause.
- **Ignoring an unknown key** — forward compatible without a version bump, but a key a newer
  template adds would be dropped silently by an older `nuc`, against Q-1.
- **Refusing an unknown key before reading `minimum`** — the simplest parser, but an older `nuc`
  would call a newer `nuc.toml` unreadable where the cause is its own age.
- **The `toml` crate** — full TOML, but it adds the crate and its dependencies to what
  `cargo deny` and `cargo audit` have to hold (Q-3), for two keys; a richer value supersedes this
  ADR.
- **A verdict for the shim on the standard output, or a second call** — no new code, but the
  standard output is the findings', and git hands a hook its standard input once; the hook
  contract weighs both.
- **One exit code per cause** — callers could branch on the cause, but every new cause would
  widen the contract, and the hooks and CI only ask whether to stop.
- **JSON as the only output** — the line form is what a terminal, an editor's `file:line` jump,
  and the bash checks' own users read; JSON and SARIF stand beside it under `--format`.

## Sources / Prior art

- TOML v1.0.0 — <https://toml.io/en/v1.0.0>: the subset is valid TOML.
- The exit status of `diff`, POSIX: 0 when no differences were found, 1 when differences were
  found and output, above 1 when an error occurred; `nuc` reads a finding as a difference —
  <https://pubs.opengroup.org/onlinepubs/9799919799/utilities/diff.html>.
- GNU Coding Standards, *Formatting Error Messages*, the form `file:line: message` that editors
  parse — <https://www.gnu.org/prep/standards/html_node/Errors.html>.
- The template's `scripts/check-docs.sh` at v0.9.1 and v0.11.0, whose header maps each key to its
  check, and its `scripts/fixtures/check-docs-record.txt`, whose header says what a case expects of
  a line.

## Consequences

- Positive: every run that could not decide fails closed with a cause (Q-1, G-5). An older `nuc`
  meeting a newer template says it is too old. The parity harness and editors read one stable line
  form (G-6, Q-2); a finding names its cause by a word that stays, in `nuc` as in the bash, so the
  rules can be tested against the template's tables. `nuc measure` writes a figure to paste where
  a shell pipes it, and `nuc sense` a report a session reads, in the same contract of exit codes.
- Negative / trade-offs: the rule ID stands twice on every line. Every reader of the old line,
  the parity harness and the tests among them, moves to the key in the change that adds it, and a
  script of a project that matched the end of a line sees a longer one. The reasons `nuc` names
  where the bash has none are words of its own to keep. One command has a code the others lack
  and a location that is no file, and a caller of `nuc hook` refuses every code it does not name. A
  template release that adds a key raises `minimum` with it. Callers cannot tell the causes of
  exit 2 apart without reading the message.
- Follow-ups: none.

## Enforcement

Tests cited with `Verifies: API-command-line-with-sense` run `nuc` on fixture repositories and check
each clause. `tests/contract.rs` checks that a `nuc.toml` with an unknown key and a higher `minimum`
refuses as too old; that an unknown key, a key set twice, and another syntax refuse as unreadable,
with exit 2 and the install hint; that a finding is the line form, its key included, on the standard
output and the summary is on the standard error; and that the exit codes are 0, 1, and 2 for a clean
tree, a finding, and a refusal, and that no command but `nuc hook` exits 3 or 4. It checks that `nuc
measure` ends its standard output in `tokens <n>` and exits 0, and exits 2 on a transcript it cannot
read. It checks that `nuc sense` with a name it does not know exits 2 and ends its refusal in
`[sense/unknown:name]`, and that a failed `--require` exits 1. It holds every key `nuc` can print to
the list kept beside it, one `<rule ID>:<reason>` per line, as it holds the rule IDs, and fails on a
key the code names that the list lacks, a key of the list the code no longer names, and a reason
that is not one word of lower-case letters. `tests/hooks.rs` checks that `nuc hook` exits 3 or 4
only where a shim runs a check, and that a refusal of `nuc hook` names the ref it guards as its
location and ends in its key.
