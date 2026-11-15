# Ticket: Section references and skill pointers as v0.9

## What to build

On a tree of v0.9.0 or later:

- `docs/section-refs` takes code spans and clause citations, such as `<ID> §4, §5` and `§2–§5`,
  out of a line before it reads a `§`, in Markdown and in the other files it reads;
- `docs/skill-pointers` holds the front matter of each skill: it opens and closes with `---`, its
  `name` equals the directory, and it has a `description`.

## Acceptance criteria

- [ ] Given the cases of the copied tables for both rules, when the runner runs, then they agree
      and leave the pending list.
- [ ] Given a tree on v0.8.1, when `nuc check` runs, then both rules report what they report today.

## Read first

- Touches: `src/rules/docs_section_refs.rs`, `src/rules/docs_skill_pointers.rs`,
  `src/markdown.rs`, `tests/parity/releases.tsv`
- The template's `check_section_refs` and `check_skill_pointers`, diffed v0.8.1 to v0.9.1.
- Seam: the table runner.

## Blocked by

- docs/plan/port-v0-9-to-v0-11/03-adrs-named-by-category-and-slug.md

## Mode

with the human: parsers of Markdown and of the front matter, protected under
DEV-one-agent-one-ticket.
