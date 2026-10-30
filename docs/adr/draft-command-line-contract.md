# ADR-draft-command-line-contract: `nuc` reads `nuc.toml` as a TOML subset, writes one finding per line as `<file>[:<line>]: <rule ID>: <message>`, and exits 0, 1, or 2

- **Status:** 🟡 proposed
- **Date:** 2026-10-31
- **Deciders:** Markus Brigl
- **Applies to:** the keys and the syntax of `nuc.toml` and how `src/config.rs` reads them, the text output of every `nuc` command, and its exit codes

## Context

`nuc` runs in the hooks and the CI of every derived project, and three things it shares with them
outlive any release: the `nuc.toml` the template release ships
([ADR-draft-version-coupling](draft-version-coupling.md), which leaves its format out of scope),
the lines a finding is printed as, which editors, scripts, and the parity harness read
([ADR-draft-parity-harness](draft-parity-harness.md)), and the exit codes the hooks and CI branch
on. G-6 asks a finding to name its rule ID and `file:line`; G-5 and Q-1 ask every run that could
not decide to fail closed with a message naming how to install `nuc`; Q-6 asks that an update of
`nuc` never turn a green project red; Q-3 counts every dependency against the supply chain. A
newer template release may add a key that an older `nuc` does not know, and that `nuc` must then
say it is too old rather than guess. G-7 adds JSON and SARIF later.

## Decision

We will have `nuc` read `nuc.toml` as a strict subset of TOML holding `minimum` and `rules`,
write each finding as one line `<file>[:<line>]: <rule ID>: <message>` on the standard output,
and exit 0 without a finding, 1 with findings, and 2 when it could not decide.

- **`nuc.toml`:** comments, blank lines, and two keys: `minimum`, a string `"X.Y.Z"`, and
  `rules`, an array of strings that may span lines and end in a comma. It is read without a TOML
  crate. `minimum` is read first: above the running version, `nuc` refuses as too old before any
  other key is judged. A key it does not know, a key set twice, or any other syntax refuses the
  run as unreadable; neither is ignored.
- **Output:** findings on the standard output and nothing else there; the summary, every refusal,
  and its message on the standard error, the message of a refusal a newer `nuc` could resolve
  naming how to install or update it. The line is omitted where the rule names none, as for a
  missing file.
- **Exit codes:** 0, 1, and 2 as above, for every command; every cause that keeps `nuc` from
  deciding exits 2, one code for all of them. A panic exits 101, non-zero as well.

**Out of scope:** the JSON and SARIF formats of G-7; options a rule may take; the wording of the
messages, which may change in any release; the output of `nuc explain`, which is prose.

## Alternatives considered

- **Ignoring an unknown key** — forward compatible without a version bump, but a key a newer
  template adds would be dropped silently by an older `nuc`, against Q-1.
- **Refusing an unknown key before reading `minimum`** — the simplest parser, but an older `nuc`
  would call a newer `nuc.toml` unreadable where the cause is its own age.
- **The `toml` crate** — full TOML, but it adds the crate and its dependencies to what
  `cargo deny` and `cargo audit` have to hold (Q-3), for two keys; a richer value supersedes this
  ADR.
- **One exit code per cause** — callers could branch on the cause, but every new cause would
  widen the contract, and the hooks and CI only ask whether to stop.
- **JSON as the only output** — what G-7 adds; the line form is what a terminal, an editor's
  `file:line` jump, and the bash checks' own users read today.

## Sources / Prior art

- TOML v1.0.0 — <https://toml.io/en/v1.0.0>: the subset is valid TOML.
- The exit status of `diff`, POSIX: 0 when no differences were found, 1 when differences were
  found and output, above 1 when an error occurred; `nuc` reads a finding as a difference —
  <https://pubs.opengroup.org/onlinepubs/9799919799/utilities/diff.html>.
- GNU Coding Standards, *Formatting Error Messages*, the form `file:line: message` that editors
  parse —
  <https://www.gnu.org/prep/standards/html_node/Errors.html>.

## Consequences

- Positive: every run that could not decide fails closed with a cause (Q-1, G-5); an older `nuc`
  meeting a newer template says it is too old; the parity harness and editors read one stable
  line form (G-6, Q-2); no dependency is added (Q-3).
- Negative / trade-offs: a template release that adds a key has to raise `minimum` with it;
  callers cannot tell the causes of exit 2 apart without reading the message.
- Follow-ups: the JSON and SARIF formats of G-7.

## Enforcement

Tests cited with `Verifies: ADR-draft-command-line-contract` run `nuc` on fixture repositories
and check each clause: a `nuc.toml` with an unknown key and a higher `minimum` refuses as too old;
an unknown key, a key set twice, and another syntax refuse as unreadable, with exit 2 and the
install hint; a finding is the line form on the standard output and the summary is on the
standard error; and the exit codes are 0, 1, and 2 for a clean tree, a finding, and a refusal.
