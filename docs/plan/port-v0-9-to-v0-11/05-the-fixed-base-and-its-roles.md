# Ticket: The fixed base and the roles of the record

## What to build

On a tree of v0.9.0 or later:

- `docs/base-blobs`, a new rule, compares the blob of each `NUC-` ADR and of `AGENTS.md` with the
  list in `docs/adr/NUC.blobs`, as the bash of v0.9.1: a list missing while `NUC-` files exist,
  unless the release line reads `vX.Y.Z, forked`; a listed file gone; a file unlisted; a blob that
  differs; a blob that cannot be computed;
- `docs/adr-header` reads the heading as the first `# ` line and the fields `Extends`, `Narrows`,
  and `Role`;
- `docs/template-identity` holds that each of the roles harness and toolchain is filled by exactly
  one accepted ADR with that `Role:` line.

The blob is computed as git computes it, through the git the shell spawns.

## Acceptance criteria

- [ ] Given the cases of the copied tables for `docs/base-blobs`, `docs/adr-header`, and the roles
      of `docs/template-identity`, as v0.9.1 decides them, when the runner runs, then they agree
      and leave the pending list.
- [ ] Given a `NUC-` file changed by one byte, when `nuc check` runs, then `docs/base-blobs`
      reports it with the key the bash names.
- [ ] Given a tree on v0.8.1, when `nuc check` runs, then nothing of this changes its findings.
- [ ] Given `docs/base-blobs`, when the contract test runs, then the ID is listed and ships off.

## Read first

- Touches: a new rule module, `src/rules/docs_adr_header.rs`,
  `src/rules/docs_template_identity.rs`, `src/adr.rs`, `src/shell/mod.rs`, `src/shell/git.rs`,
  `src/rules/mod.rs`, `tests/contract.rule-ids`, `tests/parity/releases.tsv`,
  `tests/git_cli.neutralized`
- The template's `check_base_blobs`, `check_adr_headers`, and `check_template_identity` at v0.9.1,
  and docs/adr/NUC-base-is-fixed.md.
- Seam: the table runner.

## Blocked by

- docs/plan/port-v0-9-to-v0-11/03-adrs-named-by-category-and-slug.md

## Mode

with the human: parsers of ADR files and a new git command, protected under
DEV-one-agent-one-ticket.
