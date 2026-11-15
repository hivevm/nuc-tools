# Ticket: The rules of the record read the new identifiers

## What to build

On a tree of v0.9.0 or later, the rules that read ADR identifiers check as the bash of v0.9.1:

- `docs/adr-index`: rows matched by identifier and linking `<id>.md`, a `NUC-` row refused, the
  `NUC-` files left out of the unlisted check;
- `docs/adr-refs`: identifiers matched as whole words by the category list, code spans skipped;
- `docs/adr-link-targets`: a link whose text holds an identifier points at `<id>.md`, for links
  that resolve into the record;
- `docs/links`: the placeholder skipped is `<CAT>`, not `NNNN`;
- `docs/adr-supersessions`: identifiers matched to a word boundary, no project ADR superseding a
  `NUC-` ADR, and each `NUC-x §n` of an `Extends` or `Narrows` field a numbered clause of that
  ADR's Decision.

## Acceptance criteria

- [ ] Given the cases of the copied tables for these five rules, as v0.9.1 decides them, when the
      runner runs, then they agree and leave the pending list; a case that v0.10.0 added stays.
- [ ] Given a tree on v0.8.1, when `nuc check` runs, then these rules report what they report
      today, and the parity harness agrees.

## Read first

- Touches: `src/rules/docs_adr_index.rs`, `src/rules/docs_adr_refs.rs`,
  `src/rules/docs_adr_link_targets.rs`, `src/rules/docs_links.rs`,
  `src/rules/docs_adr_supersessions.rs`, `src/adr.rs`, `src/markdown.rs`,
  `tests/parity/releases.tsv`
- The template's functions of these rules and `scripts/lib/adr.sh`, diffed v0.8.1 to v0.9.1.
- TEST-copies-keep-old-identifiers, for messages of old releases.
- Seam: the table runner; the parity harness.

## Blocked by

- docs/plan/port-v0-9-to-v0-11/03-adrs-named-by-category-and-slug.md

## Mode

with the human: parsers of Markdown and ADR files, protected under DEV-one-agent-one-ticket.
