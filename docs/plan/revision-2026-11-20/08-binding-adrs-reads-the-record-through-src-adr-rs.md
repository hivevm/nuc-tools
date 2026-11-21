# Ticket: binding-adrs reads the record through src/adr.rs

## What to build

The spec of `sensors-under-nuc-sense` said that the sensors reuse the rules' reading of the record.
binding-adrs reuses `adr::section` and `adr::rule_path_bytes`, and keeps other readings of its own:

- the glob of an ADR file: `binding_adrs::is_adr_name` and the regex of
  `binding_adrs::listed_adrs`, beside `adr::is_adr_name`. review-binding keeps a third as the
  pathspecs of `review_binding::adr_pathspecs`;
- `field_value`, beside `after_last_label` of `src/rules/docs_adr_index.rs`;
- `key` and `id`, beside `adr::key_of`, and `basename`, the last part of a path, which `key`
  uses;
- the status test of `Text::in_force`, beside `adr::status_emoji`;
- `unfenced_section`, the reading of a section before the fences, beside `adr::section`;
- `binding_rows`, which reads the index's *Binding* table outside `src/adr.rs`.

The evidence is the same as for ticket 04 of this revision. Review found this class twice in the
same work, a decoder in ticket 06 and an interner in ticket 05 of `sensors-under-nuc-sense`.

Move each reading that a sensor and a rule share to `src/adr.rs`, and the readings of the record
that only binding-adrs makes, `binding_rows` among them, beside them. Where a release reads the
record otherwise, the function there takes the release. The release constants `binding_adrs::NAMED`
and `review_binding::NAMED` stay: each names the release at which that sensor's bash changed.

## Acceptance criteria

- [ ] Given `src/sense/`, when it is searched for a regex or a pathspec of ADR file names, then
  binding-adrs and review-binding call the one function of `src/adr.rs` that `adr::is_adr_name`
  uses, or one beside it there.
- [ ] Given `src/sense/binding_adrs.rs`, when it is searched for `field_value`, `fn key`,
  `fn basename`, `fn id`, `unfenced_section`, and `binding_rows`, then none is defined there, and
  `Text::in_force` tests no status emoji itself but calls `adr::status_emoji` or a function beside
  it.
- [ ] Given the parity rows of `tests/parity/sensors.tsv` and the rules' parity, when the harness
  runs, then both pass unchanged.

## Read first

- Touches: `src/adr.rs`, `src/sense/binding_adrs.rs`, `src/sense/review_binding.rs`,
  `src/rules/docs_adr_index.rs`
- ARCH-rules-outcome-hooks-answer; API-sensor-cores-ask-the-shell; the convention "A port copies
  the bash, quirks included" in `docs/CONVENTIONS.md`

## Blocked by

- None (can start immediately)

## Mode

with the human: it changes the parsers of ADR files, a protected part of DEV-one-agent-one-ticket.
