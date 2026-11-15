# Ticket: The checks of v0.10.0

## What to build

On a tree of v0.10.0 or later, the rules check as the bash of v0.10.0:

- `docs/adr-identifiers` names its findings by its own key and accepts `docs/adr-numbering` as an
  older name of a rule key;
- `docs/adr-refs` reports a rule key that the *Applies to* of an ADR in force names for
  `scripts/check-docs.sh` and that the script's header list lacks, old keys included;
- `docs/template-release` reports a change that moves the release line, against the merge base
  with `origin/main` or else `main`, while a `proposed` `NUC-` ADR stands; the shell reads the
  merge base;
- `docs/template-identity` reports the issue template for template findings still present after
  the setup;
- `docs/opens-no-issue`, a new rule, while `TEMPLATE-SETUP.md` exists, reports a script of
  `scripts/`, `.githooks/`, `.github/`, or `.agents/`, outside `scripts/fixtures/`, that opens an
  issue by the CLI, the API, GraphQL, HTTP, or a script, as the bash's five reasons.

v0.10.0 joins the releases at which a ported check changed.

## Acceptance criteria

- [ ] Given the cases of the copied tables for these rules, when the runner runs, then they agree,
      and the pending list holds only the `shell/` cases.
- [ ] Given a branch that moves the release line while a `proposed` `NUC-` ADR stands, when
      `nuc check` runs on a tree of v0.10.0, then `docs/template-release` reports it; on v0.9.1 it
      does not.
- [ ] Given `docs/opens-no-issue`, when the contract test runs, then the ID is listed and ships
      off.

## Open

- The case 'no README to carry the Template release line' of `tests/tables/check-docs-record.txt`
  expects `docs/template-release:absent`, but `nuc` refuses a tree without a README before any rule
  runs (API-version-coupling-hook-defaults), so it cannot agree. TEST-cases-from-template-tables
  knows no status for a case an ADR in force decides otherwise; it stands as pending on this ticket
  until the human settles it, by a superseding ADR or otherwise.
- So does the case 'a release line that says 'not forked'', for the same reason in another form:
  its README names v0.8.1, which `nuc` checks as the bash of v0.8.1, which has no
  `docs/base-blobs`, while the table expects the check of v0.11.0. The case 'a fork without a
  list, the README naming the release it forked' agrees by chance for the same reason: its README
  names v0.8.1, where nuc has no `docs/base-blobs` to pass, so no table case decides the forked
  path of v0.9.0 on; a unit test does.

## Notes

- From ticket 08: `docs/adr-identifiers` words its `numbered` finding as v0.9.1 does, "map it to
  '<CAT>-<slug>' with .agents/skills/normalize/remap.sh, as the normalize skill's migration says";
  v0.10.0 rewords it, "migrate the record with the normalize skill of a v0.9 release, which maps
  it to '<CAT>-<slug>'", and renames the keys of the rule from `docs/adr-numbering:` to
  `docs/adr-identifiers:`.

## Read first

- Touches: `src/rules/docs_adr_refs.rs`, `src/rules/docs_template_release.rs`,
  `src/rules/docs_template_identity.rs`, the module of `docs/adr-identifiers`, a new rule module,
  `src/rules/mod.rs`, `src/shell/mod.rs`, `src/shell/git.rs`, `tests/contract.rule-ids`,
  `tests/parity/releases.tsv`, `tests/git_cli.neutralized`
- The template's `scripts/check-docs.sh`, diffed v0.9.1 to v0.10.0, and the v0.10.0 tag message.
- Seam: the table runner.

## Blocked by

- docs/plan/port-v0-9-to-v0-11/08-parity-on-v0-9.md

## Mode

with the human: parsers of repository content and a new git command, protected under
DEV-one-agent-one-ticket.
